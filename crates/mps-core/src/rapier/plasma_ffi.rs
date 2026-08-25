//! Plasma physics C ABI — thin `#[unsafe(no_mangle)]` wrappers around the pure
//! `mps_formula::plasma` scalar helpers (plasma beta, gyrofrequency, Larmor
//! radius, mirror ratio, loss-cone angle).
//!
//! Scalar results reuse [`crate::rapier::ffi::ffi_scalar`] (null `out` →
//! `Bool::FALSE`, `None` result → `Bool::FALSE`, otherwise writes and returns
//! `Bool::TRUE`). The multi-valued helpers (`mhd_wave_speeds`, `safety_factor`,
//! `landau_damping_rate`) return tuples/structs and are not wrapped here
//! (the existing `plasma_*` FFI already cover the struct-based surface).
//!
//! No `WorldHandle` / Rapier state is touched — these are pure calculators.
//!
//! Rust module name is `plasma_ffi`; the exported C symbols are prefixed
//! `plasma_`.
//!
//! NOTE: functions are written out explicitly (no macro) because cbindgen does
//! not expand declarative macros, so macro-generated `pub extern "C" fn`
//! items are silently omitted from `rigid_body.h`.

use crate::rapier::ffi::{Bool, ffi_scalar};
use mps_formula::plasma::*;

#[unsafe(no_mangle)]
pub extern "C" fn plasma_beta(
    density: f64,
    temperature: f64,
    magnetic_field: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        mps_formula::plasma::plasma_beta(density, temperature, magnetic_field)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn plasma_gyrofrequency(
    charge: f64,
    magnetic_field: f64,
    mass: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || gyrofrequency(charge, magnetic_field, mass))
}

#[unsafe(no_mangle)]
pub extern "C" fn plasma_larmor_radius(
    mass: f64,
    perpendicular_velocity: f64,
    charge: f64,
    magnetic_field: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        larmor_radius(mass, perpendicular_velocity, charge, magnetic_field)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn plasma_mirror_ratio(max_field: f64, min_field: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || mirror_ratio(max_field, min_field))
}

#[unsafe(no_mangle)]
pub extern "C" fn plasma_mirror_loss_cone_angle(
    max_field: f64,
    min_field: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || mirror_loss_cone_angle(max_field, min_field))
}
