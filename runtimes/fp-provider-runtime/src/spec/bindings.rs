use super::types::*;
use fp_bindgen_support::{
    common::mem::FatPtr,
    host::{
        errors::{InvocationError, RuntimeError},
        mem::{
            deserialize_from_slice, export_to_guest, export_to_guest_raw, import_from_guest,
            import_from_guest_raw, serialize_to_vec,
        },
        r#async::{create_future_value, future::ModuleRawFuture, resolve_async_value},
        runtime::RuntimeInstanceData,
    },
};
use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};

pub struct Runtime {
    module: Module,
}

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        Ok(Self { module })
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer_compiler_cranelift::Cranelift::default();
        let engine = wasmer_engine_universal::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer_compiler_singlepass::Singlepass::default();
        let engine = wasmer_engine_universal::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    pub async fn invoke(
        &self,
        request: ProviderRequest,
        config: rmpv::Value,
    ) -> Result<ProviderResponse, InvocationError> {
        let request = serialize_to_vec(&request);
        let config = serialize_to_vec(&config);
        let result = self.invoke_raw(request, config);
        let result = result.await;
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub async fn invoke_raw(
        &self,
        request: Vec<u8>,
        config: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let request = export_to_guest_raw(&env, request);
        let config = export_to_guest_raw(&env, config);
        let function = instance
            .exports
            .get_native_function::<(FatPtr, FatPtr), FatPtr>("__fp_gen_invoke")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(request, config)?;
        let result = ModuleRawFuture::new(env.clone(), result).await;
        Ok(result)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
       "fp" => {
           "__fp_host_resolve_async_value" => Function :: new_native_with_env (store , env . clone () , resolve_async_value) ,
           "__fp_gen_log" => Function :: new_native_with_env (store , env . clone () , _log) ,
           "__fp_gen_make_http_request" => Function :: new_native_with_env (store , env . clone () , _make_http_request) ,
           "__fp_gen_now" => Function :: new_native_with_env (store , env . clone () , _now) ,
           "__fp_gen_random" => Function :: new_native_with_env (store , env . clone () , _random) ,
        }
    }
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);
    let result = super::log(message);
    ()
}

pub fn _make_http_request(env: &RuntimeInstanceData, request: FatPtr) -> FatPtr {
    let request = import_from_guest::<HttpRequest>(env, request);
    let result = super::make_http_request(request);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = result.await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}

pub fn _now(env: &RuntimeInstanceData) -> FatPtr {
    let result = super::now();
    export_to_guest(env, &result)
}

pub fn _random(env: &RuntimeInstanceData, len: u32) -> FatPtr {
    let result = super::random(len);
    export_to_guest(env, &result)
}
