use crate::statement::{params_atom_to_key, Params, StatementResource, Value};
use crate::transaction::TransactionResource;
use crate::utils::{runtime, send_result, setup_async_env};
use rustler::{Env, Reference, Resource, ResourceArc};
use tokio::sync::{Mutex, RwLock};
use turso::{Connection, Error as TursoError, Rows};

pub struct ConnectionResource(pub RwLock<Connection>);

#[rustler::resource_impl]
impl Resource for ConnectionResource {
    fn destructor(self, _env: Env<'_>) {}
}

// Connection Execution

#[rustler::nif]
fn conn_execute<'a>(
    env: Env<'a>,
    conn_resource: ResourceArc<ConnectionResource>,
    sql: String,
    params: Params,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let conn = conn_resource.0.try_read().unwrap();
        let result = match params {
            Params::Positional(p) => conn.execute(sql, p).await,
            Params::Named(n) => conn.execute(sql, params_atom_to_key(&owned_env, n)).await,
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
        let conn = conn_resource.0.try_read().unwrap();
        let rows = match params {
            Params::Positional(p) => conn.query(sql, p).await,
            Params::Named(n) => conn.query(sql, params_atom_to_key(&owned_env, n)).await,
        };

        let result = match rows {
            Ok(rows) => decode_rows(rows).await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };

        send_result(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

pub async fn decode_rows(mut rows: Rows) -> Result<Vec<Vec<Value>>, TursoError> {
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

// Prepare Statements

#[rustler::nif]
fn conn_prepare<'a>(
    env: Env<'a>,
    conn_resource: ResourceArc<ConnectionResource>,
    sql: String,
    cached: bool,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let conn = conn_resource.0.try_read().unwrap();

        let stmt = if cached {
            conn.prepare_cached(sql).await
        } else {
            conn.prepare(sql).await
        };

        let result = match stmt {
            Ok(stmt) => Ok(ResourceArc::new(StatementResource(Mutex::new(stmt)))),
            Err(e) => Err(e.to_string()),
        };

        send_result::<ResourceArc<StatementResource>>(result, pid, owned_env, owned_ref);
    });

    erl_ref
}

// Transaction

#[rustler::nif]
fn conn_transaction<'a>(
    env: Env<'a>,
    conn_resource: ResourceArc<ConnectionResource>,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let mut conn = conn_resource.0.try_write().unwrap();

        let tx_res = conn.transaction().await;

        let result = match tx_res {
            Ok(tx) => Ok(ResourceArc::new(TransactionResource { tx })),
            Err(e) => Err(e.to_string()),
        };

        send_result::<ResourceArc<TransactionResource>>(result, pid, owned_env, owned_ref);
    });

    erl_ref
}
