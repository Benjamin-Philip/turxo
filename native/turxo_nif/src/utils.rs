use rustler::{Encoder, Env, LocalPid, OwnedEnv, Reference};
use std::future::Future;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn runtime() -> &'static Runtime {
    RUNTIME
        .get()
        .expect("RUNTIME not initialized (NIF load failed?)")
}

pub fn setup_async_env<'a, 'b>(env: Env<'a>) -> (Reference<'a>, LocalPid, OwnedEnv, Reference<'b>) {
    let erl_ref = env.make_ref();
    let pid = env.pid();

    let owned_env = OwnedEnv::new();
    let owned_ref = owned_env.run(|env| erl_ref.in_env(env));

    (erl_ref, pid, owned_env, owned_ref)
}

pub fn send_result<T: Encoder>(
    result: Result<T, String>,
    pid: LocalPid,
    mut owned_env: OwnedEnv,
    owned_ref: Reference<'_>,
) {
    let _ = owned_env.send_and_clear(&pid, |_env| (owned_ref, result));
}

pub fn spawn_task_with_result<'a, F, T>(env: Env<'a>, future: F) -> Reference<'a>
where
    F: Future<Output = Result<T, String>> + Send + 'static,
    T: Encoder + Send + 'static,
{
    let (erl_ref, pid, owned_env, owned_ref) = setup_async_env(env);

    runtime().spawn(async move {
        let result = future.await;
        send_result(result, pid, owned_env, owned_ref);
    });

    erl_ref
}
