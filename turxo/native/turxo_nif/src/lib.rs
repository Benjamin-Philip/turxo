use rustler::{Atom, Env, LocalPid, NifUntaggedEnum, OwnedEnv, Reference, Resource, ResourceArc};
use std::sync::OnceLock;

use tokio::runtime::Runtime;

use turso::{
    Builder, Connection, Database, Error as TursoError, IntoValue, Rows, Value as TursoValue,
};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME
        .get()
        .expect("RUNTIME not initialized (NIF load failed?)")
}

// Database and Connection Setup

struct DatabaseResource {
    db: Database,
}

struct ConnectionResource {
    conn: Connection,
}

#[rustler::resource_impl]
impl Resource for DatabaseResource {
    fn destructor(self, _env: Env<'_>) {}
}

#[rustler::resource_impl]
impl Resource for ConnectionResource {
    fn destructor(self, _env: Env<'_>) {}
}

#[rustler::nif]
fn db_open<'a>(env: Env<'a>, db_path: String) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let result = Builder::new_local(&db_path).build().await;

        let result = match result {
            Ok(db) => Ok(ResourceArc::new(DatabaseResource { db })),
            Err(e) => Err(e.to_string()),
        };

        send_result(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

#[rustler::nif]
fn db_connect<'a>(env: Env<'a>, db_resource: ResourceArc<DatabaseResource>) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let result = db_resource.db.connect();

        let result = match result {
            Ok(conn) => Ok(ResourceArc::new(ConnectionResource { conn })),
            Err(e) => Err(e.to_string()),
        };

        send_result(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

// Connection Execution

#[derive(NifUntaggedEnum, Debug, Clone)]
enum Params {
    Positional(Vec<Value>),
    Named(Vec<(Atom, Value)>),
}

#[derive(NifUntaggedEnum, Debug, Clone)]
enum Value {
    Null(Atom),
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl IntoValue for Value {
    fn into_value(self) -> Result<TursoValue, TursoError> {
        match self {
            Value::Null(_i) => Ok(TursoValue::Null),
            Value::Integer(i) => Ok(TursoValue::Integer(i)),
            Value::Real(f) => Ok(TursoValue::Real(f)),
            Value::Text(s) => Ok(TursoValue::Text(s)),
            Value::Blob(b) => Ok(TursoValue::Blob(b)),
        }
    }
}

impl Value {
    fn new(value: TursoValue) -> Value {
        match value {
            TursoValue::Null => Value::Null(rustler::types::atom::nil()),
            TursoValue::Integer(i) => Value::Integer(i),
            TursoValue::Real(f) => Value::Real(f),
            TursoValue::Text(s) => Value::Text(s),
            TursoValue::Blob(b) => Value::Blob(b),
        }
    }
}

impl From<Value> for TursoValue {
    fn from(value: Value) -> TursoValue {
        value.into_value().unwrap()
    }
}

fn params_atom_to_key(owned_env: &OwnedEnv, params: Vec<(Atom, Value)>) -> Vec<(String, Value)> {
    owned_env.run(|env| {
        params
            .into_iter()
            .map(|(key, val)| {
                (
                    ":".to_owned() + &key.to_term(env).atom_to_string().unwrap(),
                    val,
                )
            })
            .collect::<Vec<_>>()
    })
}

#[rustler::nif]
fn conn_execute<'a>(
    env: Env<'a>,
    conn_resource: ResourceArc<ConnectionResource>,
    sql: String,
    params: Params,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let result = match params {
            Params::Positional(p) => conn_resource.conn.execute(sql, p).await,
            Params::Named(n) => {
                conn_resource
                    .conn
                    .execute(sql, params_atom_to_key(&owned_env, n))
                    .await
            }
        };

        let result = result.map_err(|e| e.to_string());
        send_result::<u64>(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

// Connection Queries

#[rustler::nif]
fn conn_query<'a>(
    env: Env<'a>,
    conn_resource: ResourceArc<ConnectionResource>,
    sql: String,
    params: Params,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let rows = match params {
            Params::Positional(p) => conn_resource.conn.query(sql, p).await,
            Params::Named(n) => {
                conn_resource
                    .conn
                    .query(sql, params_atom_to_key(&owned_env, n))
                    .await
            }
        };

        let result = match rows {
            Ok(rows) => decode_rows(rows).await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };

        send_result(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

async fn decode_rows(mut rows: Rows) -> Result<Vec<Vec<Value>>, TursoError> {
    let count = rows.column_count();
    let mut decoded = Vec::new();

    while let Some(row) = rows.next().await? {
        let mut decoded_row: Vec<Value> = Vec::new();

        for idx in 0..count {
            decoded_row.push(Value::new(row.get_value(idx).unwrap()));
        }

        decoded.push(decoded_row);
    }

    Ok(decoded)
}

// Helpers

fn setup_async_env<'a, 'b>(env: Env<'a>) -> (Reference<'a>, LocalPid, OwnedEnv, Reference<'b>) {
    let erl_ref = env.make_ref();
    let pid = env.pid();

    let owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));

    (erl_ref, pid, owned_env, owned_ref)
}

fn send_result<T: rustler::Encoder>(
    result: Result<T, String>,
    pid: LocalPid,
    mut owned_env: OwnedEnv,
    owned_ref: Reference<'_>,
) {
    let _ = owned_env.send_and_clear(&pid, |_env| (owned_ref, result));
}

fn on_load(_env: Env, _info: rustler::Term) -> bool {
    match Runtime::new() {
        Ok(rt) => {
            let _ = RUNTIME.set(rt);
            true
        }
        Err(_) => false,
    }
}

rustler::init!("Elixir.Turxo.NIF", load = on_load);
