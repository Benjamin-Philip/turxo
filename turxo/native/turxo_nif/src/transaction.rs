use rustler::{Env, Resource, ResourceArc};
use turso::Connection;
// use crate::connection::ConnectionResource;
use std::sync::Arc;
use turso::transaction::Transaction;

pub struct TransactionResource {
    pub conn: Connection,
    // pub conn_arc: Arc<Connection>,
    pub tx: Option<Transaction<'static>>,
}

#[rustler::resource_impl]
impl Resource for TransactionResource {
    fn destructor(self, _env: Env<'_>) {}
}
