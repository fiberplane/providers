use super::types::*;
use crate::{
    support::{
        assign_async_value, create_future_value, export_to_guest, import_from_guest,
        resolve_async_value, FatPtr, InvocationError, ModuleFuture, FUTURE_STATUS_READY,
    },
    Runtime, RuntimeInstanceData,
};
use wasmer::{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv};

impl Runtime {
    /// Fetches a data instant based on the given query and options.
    pub async fn fetch_instant(&self, query: String, opts: QueryInstantOptions) -> Result<Result<Vec<Instant>, FetchError>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let query = export_to_guest(&env, &query);
        let opts = export_to_guest(&env, &opts);

        let function = instance
            .exports
            .get_function("fetch_instant")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[query.into(), opts.into()])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        Ok(ModuleFuture::new(env.clone(), async_ptr).await)
    }

    /// Fetches a series of data based on the given query and options.
    pub async fn fetch_series(&self, query: String, opts: QuerySeriesOptions) -> Result<Result<Vec<Series>, FetchError>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let query = export_to_guest(&env, &query);
        let opts = export_to_guest(&env, &opts);

        let function = instance
            .exports
            .get_function("fetch_series")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[query.into(), opts.into()])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        Ok(ModuleFuture::new(env.clone(), async_ptr).await)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_log" => Function::new_native_with_env(store, env.clone(), _log),
            "__fp_gen_make_request" => Function::new_native_with_env(store, env.clone(), _make_request),
            "__fp_gen_now" => Function::new_native_with_env(store, env.clone(), _now),
            "__fp_gen_random" => Function::new_native_with_env(store, env.clone(), _random),
        }
    }
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);

    super::log(message);
}

pub fn _make_request(env: &RuntimeInstanceData, request: FatPtr) -> FatPtr {
    let request = import_from_guest::<Request>(env, request);

    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let ptr = export_to_guest(&env, &super::make_request(request).await);
        assign_async_value(&env, async_ptr, FUTURE_STATUS_READY, ptr);

        unsafe {
            env.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr)
                .expect("Runtime error: Cannot resolve async value");
        }
    });

    async_ptr
}

pub fn _now(env: &RuntimeInstanceData) -> FatPtr {
    export_to_guest(env, &super::now())
}

pub fn _random(env: &RuntimeInstanceData, len: u32) -> FatPtr {
    export_to_guest(env, &super::random(len))
}
