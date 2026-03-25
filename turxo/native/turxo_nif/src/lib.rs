use rustler::{Atom, Env, NifUntaggedEnum, OwnedEnv, Reference, Resource, ResourceArc};
use std::sync::OnceLock;

use tokio::runtime::Runtime;

use turso::{Builder, Connection, Database, Error as TursoError, IntoValue, Value as TursoValue};

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
fn build_db<'a>(env: Env<'a>, db_path: String) -> Reference<'a> {
    let erl_ref = env.make_ref();
    let pid = env.pid();

    let mut owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));

    runtime().spawn(async move {
        let result = Builder::new_local(&db_path).build().await;

        let ret = match result {
            Ok(db) => Ok(ResourceArc::new(DatabaseResource { db })),
            Err(e) => Err(e.to_string()),
        };

        let _ = owned_env.send_and_clear(&pid, |_env| (owned_ref, ret));
    });

    erl_ref
}

#[rustler::nif]
fn connect_db<'a>(env: Env<'a>, db_resource: ResourceArc<DatabaseResource>) -> Reference<'a> {
    let erl_ref = env.make_ref();
    let pid = env.pid();

    let mut owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));

    runtime().spawn(async move {
        let result = db_resource.db.connect();

        let ret = match result {
            Ok(conn) => Ok(ResourceArc::new(ConnectionResource { conn })),
            Err(e) => Err(e.to_string()),
        };

        let _ = owned_env.send_and_clear(&pid, |_env| (owned_ref, ret));
    });

    erl_ref
}

#[derive(NifUntaggedEnum, Debug, Clone)]
enum Params {
    Positional(Vec<Value>),
    Named(Vec<(Atom, Value)>),
}

#[derive(NifUntaggedEnum, Debug, Clone)]
enum Value {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl IntoValue for Value {
    fn into_value(self) -> Result<TursoValue, TursoError> {
        match self {
            Value::Integer(i) => Ok(TursoValue::Integer(i)),
            Value::Real(f) => Ok(TursoValue::Real(f)),
            Value::Text(s) => Ok(TursoValue::Text(s)),
            Value::Blob(b) => Ok(TursoValue::Blob(b)),
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
    let erl_ref = env.make_ref();
    let pid = env.pid();

    let mut owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));

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

        let ret = match result {
            Ok(result) => Ok(result),
            Err(e) => Err(e.to_string()),
        };

        let _ = owned_env.send_and_clear(&pid, |_env| (owned_ref, ret));
    });

    erl_ref
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
