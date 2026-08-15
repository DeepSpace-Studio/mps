pub mod convert;
pub mod types;

pub use convert::*;
pub use types::*;

/// Common body for the *pure calculator* FFI wrappers.
///
/// Given a `out` pointer (which may be null) and a thunk that computes the
/// `Option<f64>` result of a `mps_formula` helper, this writes the value into
/// `out` on `Some` and returns `Bool::TRUE`, or returns `Bool::FALSE` when the
/// pointer is null or the computation yields `None` (invalid input).
///
/// Behaviour is identical to the hand-written expansion previously duplicated
/// across every scalar FFI function (see `matmech.rs` / `thermo.rs`):
/// null `out` → `FALSE` without writing; `None` → `FALSE` without writing.
pub(crate) fn ffi_scalar<F>(out: *mut f64, f: F) -> Bool
where
    F: FnOnce() -> Option<f64>,
{
    if out.is_null() {
        return Bool::FALSE;
    }
    match f() {
        Some(v) => {
            unsafe { *out = v };
            Bool::TRUE
        }
        None => Bool::FALSE,
    }
}
