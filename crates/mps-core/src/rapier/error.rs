//! Thread-local last-error slot shared with `mps-formula`.
//!
//! The slot itself lives in `mps_formula::error` (the lowest-level crate), so
//! errors written by formula code and by the Rapier FFI layer land in the
//! same place and are both visible to Java through the `last_error_*` ABI
//! below. This module only re-exports the shared implementation and adds the
//! exported accessors plus the `ffi_guard` panic barrier.

use std::os::raw::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub use mps_formula::error::{clear_error, set_error};

// The error codes are re-declared as local `pub const`s (instead of `pub use`
// re-exports) so cbindgen — which only parses this crate, not dependencies —
// emits them as `#define`s in the generated C header. The values must be
// literals (cbindgen cannot evaluate cross-crate paths); the compile-time
// assertions below pin them to the canonical values in `mps_formula::error`.
pub const ERR_OK: u32 = 0;
pub const ERR_NULL_POINTER: u32 = 1;
pub const ERR_INVALID_ARGUMENT: u32 = 2;
pub const ERR_NOT_FOUND: u32 = 3;
pub const ERR_CAPACITY: u32 = 4;
pub const ERR_UNSUPPORTED: u32 = 5;
pub const ERR_INTERNAL: u32 = 6;

const _: () = {
    assert!(ERR_OK == mps_formula::error::ERR_OK);
    assert!(ERR_NULL_POINTER == mps_formula::error::ERR_NULL_POINTER);
    assert!(ERR_INVALID_ARGUMENT == mps_formula::error::ERR_INVALID_ARGUMENT);
    assert!(ERR_NOT_FOUND == mps_formula::error::ERR_NOT_FOUND);
    assert!(ERR_CAPACITY == mps_formula::error::ERR_CAPACITY);
    assert!(ERR_UNSUPPORTED == mps_formula::error::ERR_UNSUPPORTED);
    assert!(ERR_INTERNAL == mps_formula::error::ERR_INTERNAL);
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

/// Current thread's last error code (`ERR_OK` when no error).
///
/// # Safety
///
/// No pointer parameters; safe to call from any thread. The error slot is
/// thread-local, so the result reflects only errors reported on the calling
/// thread.
#[unsafe(no_mangle)]
pub extern "C" fn last_error_code() -> u32 {
    ffi_guard(ERR_OK, mps_formula::error::error_code)
}

/// Current thread's last error message ("ok" when no error).
///
/// The returned pointer is borrowed from a thread-local slot owned by Rust;
/// it is invalidated by the next error-reporting call on the same thread and
/// must not be freed or stored.
///
/// # Safety
///
/// No pointer parameters; safe to call from any thread. The returned pointer
/// is borrowed from a thread-local slot owned by Rust (no ownership transfer):
/// it remains valid only until the next error-reporting call on the same
/// thread and must not be freed by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn last_error_message() -> *const c_char {
    ffi_guard(std::ptr::null(), mps_formula::error::error_message)
}

/// Reset the current thread's error slot to `ERR_OK` / "ok".
///
/// # Safety
///
/// No pointer parameters; safe to call from any thread. Only the calling
/// thread's error slot is affected.
#[unsafe(no_mangle)]
pub extern "C" fn last_error_clear() {
    ffi_guard((), clear_error);
}

/// Static name of an error code ("ERR_OK", "ERR_NULL_POINTER", ...).
///
/// Unknown codes yield "ERR_UNKNOWN". The returned pointer refers to a
/// string with `'static` lifetime owned by Rust; it must not be freed.
///
/// # Safety
///
/// No pointer parameters; safe to call from any thread with any `code` value
/// (unknown codes return "ERR_UNKNOWN"). The returned pointer refers to a
/// `'static` string owned by Rust (no ownership transfer) and must not be
/// freed by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn error_code_name(code: u32) -> *const c_char {
    ffi_guard(std::ptr::null(), || match code {
        ERR_OK => c"ERR_OK".as_ptr(),
        ERR_NULL_POINTER => c"ERR_NULL_POINTER".as_ptr(),
        ERR_INVALID_ARGUMENT => c"ERR_INVALID_ARGUMENT".as_ptr(),
        ERR_NOT_FOUND => c"ERR_NOT_FOUND".as_ptr(),
        ERR_CAPACITY => c"ERR_CAPACITY".as_ptr(),
        ERR_UNSUPPORTED => c"ERR_UNSUPPORTED".as_ptr(),
        ERR_INTERNAL => c"ERR_INTERNAL".as_ptr(),
        _ => c"ERR_UNKNOWN".as_ptr(),
    })
}
