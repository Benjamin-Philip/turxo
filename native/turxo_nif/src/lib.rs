mod connection;
mod database;
mod statement;
mod utils;

use utils::RUNTIME;

use rustler::Env;
use tokio::runtime::Runtime;

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
