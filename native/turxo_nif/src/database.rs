use crate::connection::ConnectionResource;
use crate::utils::{runtime, send_result, setup_async_env};
use rustler::{Env, Reference, Resource, ResourceArc};
use turso::{Builder, Database};

// Database and Connection Setup

struct DatabaseResource {
    db: Database,
}

#[rustler::resource_impl]
impl Resource for DatabaseResource {
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
