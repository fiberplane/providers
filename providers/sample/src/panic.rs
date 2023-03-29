//! This module provides a helper function that
//! sets up the panic handler so that unrecoverable
//! errors from the provider show better messages.

use fiberplane_pdk::prelude::log;
use std::panic;

pub fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| {
            log(format!("Sample provider panic: {info}"))
        }));
    });
}
