use rustler::{Env, OwnedEnv, Reference, Resource, ResourceArc};
use std::sync::OnceLock;

use tokio::runtime::Runtime;

use turso::{Builder, Connection, Database};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME
        .get()
        .expect("RUNTIME not initialized (NIF load failed?)")
}

// Resources

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

// NIFs

#[rustler::nif]
fn build_db<'a>(env: Env<'a>, path: String) -> Reference<'a> {
    let erl_ref = env.make_ref();
    let pid = env.pid();
    
    let mut owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));
    
    runtime().spawn(async move {
        let result = Builder::new_local(&path).build().await;

        let ret = match result {
            Ok(db) => Ok(ResourceArc::new(DatabaseResource{db})),
            Err(e) => Err(e.to_string())
        };

        let _ = owned_env.send_and_clear(&pid, |_env| {
            (owned_ref, ret)
        });
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
