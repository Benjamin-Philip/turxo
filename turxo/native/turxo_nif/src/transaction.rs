use rustler::{Env, Resource};
use turso::transaction::Transaction;

pub struct TransactionResource {
    pub tx: Transaction<'static>,
}

#[rustler::resource_impl]
impl Resource for TransactionResource {
    fn destructor(self, _env: Env<'_>) {}
}
