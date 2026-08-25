//! Nuclear physics C ABI — thin `#[unsafe(no_mangle)]` wrappers around the
//! pure `mps_formula::nuclear` scalar helpers (radioactive decay, semi-empirical
//! mass formula, reaction Q-values, fusion/fission energies, reactor physics,
//! attenuation).
//!
//! Scalar results reuse [`crate::rapier::ffi::ffi_scalar`] (null `out` →
//! `Bool::FALSE`, `None` result → `Bool::FALSE`, otherwise writes and returns
//! `Bool::TRUE`). The constant (`f64`, never-failing) helpers wrap their value
//! in `Some(..)` so `ffi_scalar` can emit it uniformly.
//!
//! No `WorldHandle` / Rapier state is touched — these are pure calculators.
//!
//! Rust module name is `nucphys`; the exported C symbols are prefixed
//! `nuclear_`.
//!
//! NOTE: functions are written out explicitly (no macro) because cbindgen does
//! not expand declarative macros, so macro-generated `pub extern "C" fn`
//! items are silently omitted from `rigid_body.h`.

use crate::rapier::ffi::{Bool, ffi_scalar};
// 公式入口走学科窗口 `mps_formula::disciplines::nuclear`,
// 不直接 `use mps_formula::nuclear::*` 而绕过学科目录。
// disciplines::nuclear re-export 包含 enrico_fermi + ernest_rutherford 全部
// 核物理公式,实现裸函数调用接口形状不变。
use mps_formula::disciplines::nuclear::*;

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_decay_constant(half_life: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || decay_constant(half_life))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_remaining_nuclei(
    initial: f64,
    decay_constant: f64,
    time: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || remaining_nuclei(initial, decay_constant, time))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_activity(decay_constant: f64, nuclei: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || activity(decay_constant, nuclei))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_half_life(decay_constant: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || half_life(decay_constant))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_mean_lifetime(decay_constant: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || mean_lifetime(decay_constant))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_bethe_weizsaecker_binding_energy(
    mass_number: f64,
    atomic_number: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        bethe_weizsaecker_binding_energy(mass_number, atomic_number)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_binding_energy_per_nucleon(
    mass_number: f64,
    atomic_number: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        binding_energy_per_nucleon(mass_number, atomic_number)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_reaction_q_value(
    initial_mass_u: f64,
    final_mass_u: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || reaction_q_value(initial_mass_u, final_mass_u))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_dt_fusion_energy(out: *mut f64) -> Bool {
    ffi_scalar(out, || Some(dt_fusion_energy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_dd_fusion_branch1_energy(out: *mut f64) -> Bool {
    ffi_scalar(out, || Some(dd_fusion_branch1_energy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_dd_fusion_branch2_energy(out: *mut f64) -> Bool {
    ffi_scalar(out, || Some(dd_fusion_branch2_energy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_u235_fission_energy(out: *mut f64) -> Bool {
    ffi_scalar(out, || Some(u235_fission_energy()))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_four_factor_formula(
    eta: f64,
    epsilon: f64,
    p: f64,
    f: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || four_factor_formula(eta, epsilon, p, f))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_reaction_rate(
    macroscopic_cross_section: f64,
    neutron_flux: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        reaction_rate(macroscopic_cross_section, neutron_flux)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_atomic_mass_approx(
    mass_number: f64,
    binding_energy_mev: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || atomic_mass_approx(mass_number, binding_energy_mev))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_specific_activity(
    decay_constant: f64,
    mass_number: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || specific_activity(decay_constant, mass_number))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_half_value_layer(linear_attenuation: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || half_value_layer(linear_attenuation))
}

#[unsafe(no_mangle)]
pub extern "C" fn nuclear_dt_fusion_q_value(out: *mut f64) -> Bool {
    ffi_scalar(out, || Some(dt_fusion_q_value()))
}
