//! Thermodynamics C ABI — thin `#[unsafe(no_mangle)]` wrappers around the pure
//! `mps_formula::thermodynamics` helpers (ideal gas law, polytropic processes).
//!
//! Every function returns `Bool` (success) and writes its `Option<f64>` result
//! into a caller-provided `*mut f64` output slot.
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

use crate::rapier::ffi::Bool;
use mps_formula::thermodynamics::*;

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_pressure(
    volume: f64,
    moles: f64,
    temperature: f64,
    out: *mut f64,
) -> Bool {
    let Some(v) = ideal_gas_pressure(volume, moles, temperature) else {
        return Bool::FALSE;
    };
    if out.is_null() {
        return Bool::FALSE;
    }
    unsafe { *out = v };
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_volume(
    pressure: f64,
    moles: f64,
    temperature: f64,
    out: *mut f64,
) -> Bool {
    let Some(v) = ideal_gas_volume(pressure, moles, temperature) else {
        return Bool::FALSE;
    };
    if out.is_null() {
        return Bool::FALSE;
    }
    unsafe { *out = v };
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_ideal_gas_temperature(
    pressure: f64,
    volume: f64,
    moles: f64,
    out: *mut f64,
) -> Bool {
    let Some(v) = ideal_gas_temperature(pressure, volume, moles) else {
        return Bool::FALSE;
    };
    if out.is_null() {
        return Bool::FALSE;
    }
    unsafe { *out = v };
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermodynamics_polytropic_pressure(
    p1: f64,
    v1: f64,
    v2: f64,
    gamma: f64,
    out: *mut f64,
) -> Bool {
    let Some(v) = polytropic_pressure(p1, v1, v2, gamma) else {
        return Bool::FALSE;
    };
    if out.is_null() {
        return Bool::FALSE;
    }
    unsafe { *out = v };
    Bool::TRUE
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
    let Some(v) = polytropic_work(p1, v1, p2, v2, gamma) else {
        return Bool::FALSE;
    };
    if out.is_null() {
        return Bool::FALSE;
    }
    unsafe { *out = v };
    Bool::TRUE
}
