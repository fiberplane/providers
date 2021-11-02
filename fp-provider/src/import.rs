use crate::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_gen_log(message: fp_bindgen_support::FatPtr);

    fn __fp_gen_make_request(request: fp_bindgen_support::FatPtr) -> fp_bindgen_support::FatPtr;

    fn __fp_gen_now() -> fp_bindgen_support::FatPtr;

    fn __fp_gen_random(len: u32) -> fp_bindgen_support::FatPtr;
}

/// Logs a message to the (development) console.
pub fn log(message: String) {
    let message = fp_bindgen_support::export_value_to_host(&message);
    unsafe { __fp_gen_log(message); }
}

/// Performs an HTTP request.
pub async fn make_request(request: Request) -> Result<Response, RequestError> {
    let request = fp_bindgen_support::export_value_to_host(&request);
    unsafe {
        let ret = __fp_gen_make_request(request);
        let result_ptr = fp_bindgen_support::HostFuture::new(ret).await;
        fp_bindgen_support::import_value_from_host(result_ptr)
    }
}

/// Returns the current timestamp.
pub fn now() -> Timestamp {
    unsafe {
        let ret = __fp_gen_now();
        fp_bindgen_support::import_value_from_host(ret)
    }
}

/// Generates random bytes.
pub fn random(len: u32) -> Vec<u8> {
    unsafe {
        let ret = __fp_gen_random(len);
        fp_bindgen_support::import_value_from_host(ret)
    }
}
