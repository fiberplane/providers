use super::types::*;
use crate::errors::InvocationError;
use crate::{
    support::{
        create_future_value, export_to_guest, export_to_guest_raw, import_from_guest,
        resolve_async_value, FatPtr, ModuleRawFuture,
    },
    Runtime, RuntimeInstanceData,
};
use wasmer::{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv};

impl Runtime {
    pub async fn invoke(&self, request: ProviderRequest, config: Config) -> Result<ProviderResponse, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let request = export_to_guest(&env, &request);
        let config = export_to_guest(&env, &config);

        let function = instance
            .exports
            .get_function("__fp_gen_invoke")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[request.into(), config.into()])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        let raw_result = ModuleRawFuture::new(env.clone(), async_ptr).await;
        Ok(rmp_serde::from_slice(&raw_result).unwrap())
    }

    pub async fn invoke_raw(&self, request: Vec<u8>, config: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let request = export_to_guest_raw(&env, request);
        let config = export_to_guest_raw(&env, config);

        let function = instance
            .exports
            .get_function("__fp_gen_invoke")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[request.into(), config.into()])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        Ok(ModuleRawFuture::new(env.clone(), async_ptr).await)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_log" => Function::new_native_with_env(store, env.clone(), _log),
            "__fp_gen_make_http_request" => Function::new_native_with_env(store, env.clone(), _make_http_request),
            "__fp_gen_now" => Function::new_native_with_env(store, env.clone(), _now),
            "__fp_gen_random" => Function::new_native_with_env(store, env.clone(), _random),
        }
    }
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);

    super::log(message);
}

pub fn _make_http_request(env: &RuntimeInstanceData, request: FatPtr) -> FatPtr {
    let request = import_from_guest::<HttpRequest>(env, request);

    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result_ptr = export_to_guest(&env, &super::make_http_request(request).await);

        unsafe {
            env.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr, result_ptr)
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
