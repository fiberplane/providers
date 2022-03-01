use crate::types::*;

#[fp_bindgen_support::fp_export_signature]
pub async fn invoke(request: ProviderRequest, config: rmpv::Value) -> ProviderResponse;
