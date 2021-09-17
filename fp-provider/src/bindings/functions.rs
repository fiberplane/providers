use super::r#async::*;
use super::support::*;
use super::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_gen_log(message: FatPtr);

    fn __fp_gen_make_request(request: FatPtr) -> FatPtr;

    fn __fp_gen_now() -> FatPtr;

    fn __fp_gen_random(len: u32) -> FatPtr;

    fn __fp_host_resolve_async_value(async_value_ptr: FatPtr);
}

/// Logs a message to the (development) console.
pub fn log(message: String) {
    let message = export_value_to_host(&message);
    unsafe {
        __fp_gen_log(message);
    }
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

#[doc(hidden)]
pub unsafe fn _fp_host_resolve_async_value(async_value_ptr: FatPtr) {
    __fp_host_resolve_async_value(async_value_ptr)
}

#[macro_export]
macro_rules! fp_export {
    (async fn fetch_instant($($param:ident: $ty:ty),*) -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_fetch_instant(query: _FP_FatPtr, opts: _FP_FatPtr) -> _FP_FatPtr {
            let len = std::mem::size_of::<_FP_AsyncValue>() as u32;
            let ptr = _fp_malloc(len);
            let fat_ptr = _fp_to_fat_ptr(ptr, len);
            let ptr = ptr as *mut _FP_AsyncValue;

            _FP_Task::spawn(Box::pin(async move {
                let query = unsafe { _fp_import_value_from_host::<String>(query) };
                let opts = unsafe { _fp_import_value_from_host::<QueryInstantOptions>(opts) };
                let ret = fetch_instant(query, opts).await;
                unsafe {
                    let (result_ptr, result_len) =
                        _fp_from_fat_ptr(_fp_export_value_to_host::<Result<Vec<Instant>, FetchError>>(&ret));
                    (*ptr).ptr = result_ptr as u32;
                    (*ptr).len = result_len;
                    (*ptr).status = 1;
                    _fp_host_resolve_async_value(fat_ptr);
                }
            }));

            fat_ptr
        }

        async fn fetch_instant($($param: $ty),*) -> $ret $body
    };

    (async fn fetch_series($($param:ident: $ty:ty),*) -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_fetch_series(query: _FP_FatPtr, opts: _FP_FatPtr) -> _FP_FatPtr {
            let len = std::mem::size_of::<_FP_AsyncValue>() as u32;
            let ptr = _fp_malloc(len);
            let fat_ptr = _fp_to_fat_ptr(ptr, len);
            let ptr = ptr as *mut _FP_AsyncValue;

            _FP_Task::spawn(Box::pin(async move {
                let query = unsafe { _fp_import_value_from_host::<String>(query) };
                let opts = unsafe { _fp_import_value_from_host::<QuerySeriesOptions>(opts) };
                let ret = fetch_series(query, opts).await;
                unsafe {
                    let (result_ptr, result_len) =
                        _fp_from_fat_ptr(_fp_export_value_to_host::<Result<Vec<Series>, FetchError>>(&ret));
                    (*ptr).ptr = result_ptr as u32;
                    (*ptr).len = result_len;
                    (*ptr).status = 1;
                    _fp_host_resolve_async_value(fat_ptr);
                }
            }));

            fat_ptr
        }

        async fn fetch_series($($param: $ty),*) -> $ret $body
    };
}
