//! Thermodynamics C ABI — thin `#[unsafe(no_mangle)]` wrappers around the pure
//! `mps_formula::thermodynamics` helpers (ideal gas law, polytropic processes).
//!
//! Scalar results reuse [`crate::rapier::ffi::ffi_scalar`] (null `out` →
//! `Bool::FALSE`, `None` result → `Bool::FALSE`, otherwise writes and returns
//! `Bool::TRUE`).
//!
//! Note: this module wraps the *gas-state* formulas. Thermal-conduction /
//! radiation / FEM-diffusion C ABI already live in `mps_formula`'s thermal FFI.
//!
//! Rust module name is `thermo`; the exported C symbols are prefixed
//! `thermodynamics_`.
//!
//! NOTE: functions are written out explicitly (no macro) because cbindgen does
//! not expand declarative macros, so macro-generated `pub extern "C" fn`
//! items are silently omitted from `rigid_body.h`.

use crate::rapier::ffi::{Bool, ffi_scalar};
use mps_formula::thermodynamics::*;

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_pressure(
    volume: f64,
    moles: f64,
    temperature: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || ideal_gas_pressure(volume, moles, temperature))
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_volume(
    pressure: f64,
    moles: f64,
    temperature: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || ideal_gas_volume(pressure, moles, temperature))
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_temperature(
    pressure: f64,
    volume: f64,
    moles: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || ideal_gas_temperature(pressure, volume, moles))
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_polytropic_pressure(
    p1: f64,
    v1: f64,
    v2: f64,
    gamma: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || polytropic_pressure(p1, v1, v2, gamma))
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_polytropic_work(
    p1: f64,
    v1: f64,
    p2: f64,
    v2: f64,
    gamma: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || polytropic_work(p1, v1, p2, v2, gamma))
}
