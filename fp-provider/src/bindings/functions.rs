use super::r#async::*;
use super::support::*;
use super::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_gen_log(message: FatPtr);

    fn __fp_gen_make_request(request: FatPtr) -> FatPtr;

    fn __fp_gen_now() -> FatPtr;

    fn __fp_gen_random(len: u32) -> FatPtr;

    pub fn __fp_host_resolve_async_value(async_value_ptr: FatPtr, result_ptr: FatPtr);
}

/// Logs a message to the (development) console.
pub fn log(message: String) {
    let message = export_value_to_host(&message);
    unsafe { __fp_gen_log(message); }
}

/// Performs an HTTP request.
pub async fn make_request(request: Request) -> Result<Response, RequestError> {
    let request = export_value_to_host(&request);
    unsafe {
        let ret = __fp_gen_make_request(request);
        let result_ptr = HostFuture::new(ret).await;
        import_value_from_host(result_ptr)
    }
}

/// Returns the current timestamp.
pub fn now() -> Timestamp {
    unsafe {
        let ret = __fp_gen_now();
        import_value_from_host(ret)
    }
}

/// Generates random bytes.
pub fn random(len: u32) -> Vec<u8> {
    unsafe {
        let ret = __fp_gen_random(len);
        import_value_from_host(ret)
    }
}

#[macro_export]
macro_rules! fp_export {
    (async fn fetch_instant$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_fetch_instant(query: __fp_macro::FatPtr, opts: __fp_macro::FatPtr) -> __fp_macro::FatPtr {
            use __fp_macro::*;
            let len = std::mem::size_of::<AsyncValue>() as u32;
            let ptr = malloc(len);
            let fat_ptr = to_fat_ptr(ptr, len);
            let ptr = ptr as *mut AsyncValue;

            Task::spawn(Box::pin(async move {
                let query = unsafe { import_value_from_host::<String>(query) };
                let opts = unsafe { import_value_from_host::<QueryInstantOptions>(opts) };
                let ret = fetch_instant(query, opts).await;
                unsafe {
                    let result_ptr = export_value_to_host::<Result<Vec<Instant>, FetchError>>(&ret);
                    __fp_host_resolve_async_value(fat_ptr, result_ptr);
                }
            }));

            fat_ptr
        }

        async fn fetch_instant$args -> $ret $body
    };

    (async fn fetch_series$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_fetch_series(query: __fp_macro::FatPtr, opts: __fp_macro::FatPtr) -> __fp_macro::FatPtr {
            use __fp_macro::*;
            let len = std::mem::size_of::<AsyncValue>() as u32;
            let ptr = malloc(len);
            let fat_ptr = to_fat_ptr(ptr, len);
            let ptr = ptr as *mut AsyncValue;

            Task::spawn(Box::pin(async move {
                let query = unsafe { import_value_from_host::<String>(query) };
                let opts = unsafe { import_value_from_host::<QuerySeriesOptions>(opts) };
                let ret = fetch_series(query, opts).await;
                unsafe {
                    let result_ptr = export_value_to_host::<Result<Vec<Series>, FetchError>>(&ret);
                    __fp_host_resolve_async_value(fat_ptr, result_ptr);
                }
            }));

            fat_ptr
        }

        async fn fetch_series$args -> $ret $body
    };
}
