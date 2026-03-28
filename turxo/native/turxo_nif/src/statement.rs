use crate::utils::{runtime, send_result, setup_async_env};
use rustler::{Atom, Env, NifUntaggedEnum, OwnedEnv, Reference, Resource, ResourceArc};
use tokio::sync::Mutex;
use turso::{Error as TursoError, IntoValue, Statement, Value as TursoValue};

pub struct StatementResource(pub Mutex<Statement>);

#[rustler::resource_impl]
impl Resource for StatementResource {
    fn destructor(self, _env: Env<'_>) {}
}

// Params and Value handling

#[derive(NifUntaggedEnum, Debug, Clone)]
pub enum Params {
    Positional(Vec<Value>),
    Named(Vec<(Atom, Value)>),
}

pub fn params_atom_to_key(
    owned_env: &OwnedEnv,
    params: Vec<(Atom, Value)>,
) -> Vec<(String, Value)> {
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

#[derive(NifUntaggedEnum, Debug, Clone)]
pub enum Value {
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
    pub fn new(value: TursoValue) -> Value {
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

// NIFs

#[rustler::nif]
fn stmt_execute<'a>(
    env: Env<'a>,
    stmt_resource: ResourceArc<StatementResource>,
    params: Params,
) -> Reference<'a> {
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let mut stmt = stmt_resource.0.try_lock().unwrap();

        let result = match params {
            Params::Positional(p) => stmt.execute(p).await,
            Params::Named(n) => stmt.execute(params_atom_to_key(&owned_env, n)).await,
        };

        let result = result.map_err(|e| e.to_string());
        send_result::<u64>(result, pid, owned_env, owned_ref);
    });

    erl_ref
}
