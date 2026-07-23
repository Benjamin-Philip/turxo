use crate::connection::ConnectionResource;
use crate::utils;
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
    utils::spawn_task_with_result(env, async move {
        let result = Builder::new_local(&db_path).build().await;

        match result {
            Ok(db) => Ok(ResourceArc::new(DatabaseResource { db })),
            Err(e) => Err(e.to_string()),
        }
    })
}

#[rustler::nif]
fn db_connect<'a>(env: Env<'a>, db_resource: ResourceArc<DatabaseResource>) -> Reference<'a> {
    utils::spawn_task_with_result(env, async move {
        let result = db_resource.db.connect();

        match result {
            Ok(conn) => Ok(ResourceArc::new(ConnectionResource { conn })),
            Err(e) => Err(e.to_string()),
        }
    })
}
