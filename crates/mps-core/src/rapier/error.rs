//! Thread-local last-error slot shared with `mps-formula`.
//!
//! The slot itself lives in `mps_formula::error` (the lowest-level crate), so
//! errors written by formula code and by the Rapier FFI layer land in the
//! same place and are both visible to Java through the `last_error_*` ABI
//! below. This module only re-exports the shared implementation and adds the
//! exported accessors plus the `ffi_guard` panic barrier.

use std::os::raw::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub use mps_formula::error::{
    ERR_CAPACITY, ERR_INTERNAL, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
    ERR_UNSUPPORTED, clear_error, set_error,
};

/// Run `f`, converting any panic into `ERR_INTERNAL` and `default`.
///
/// Every `extern "C"` FFI entry point wraps its body in this guard so a Rust
/// panic can never unwind across the FFI boundary (which would abort the host
/// JVM). `default` is the return value a Java caller already treats as the
/// failure sentinel for that function (0 / 0.0 / false / null / `Default`).
pub fn ffi_guard<R>(default: R, f: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(_) => {
            set_error(ERR_INTERNAL, "internal panic");
            default
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn last_error_code() -> u32 {
    mps_formula::error::error_code()
}

#[unsafe(no_mangle)]
pub extern "C" fn last_error_message() -> *const c_char {
    mps_formula::error::error_message()
}

#[unsafe(no_mangle)]
pub extern "C" fn last_error_clear() {
    clear_error();
}
