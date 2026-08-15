//! Acoustics C ABI — thin `#[unsafe(no_mangle)]` wrappers around the pure
//! `mps_formula::acoustics` scalar helpers (spreading loss, absorption, RT60,
//! impedance, transmission/mass-law, Helmholtz resonance, Doppler, barrier
//! attenuation, sonar figure-of-merit).
//!
//! Scalar results reuse [`crate::rapier::ffi::ffi_scalar`] (null `out` →
//! `Bool::FALSE`, `None` result → `Bool::FALSE`, otherwise writes and returns
//! `Bool::TRUE`). `doppler_shift` takes a `Bool` (C `uint8_t`) `approach` flag.
//!
//! No `WorldHandle` / Rapier state is touched — these are pure calculators.
//! The wave/modal/structural `acoustic_*` FFI already live in
//! `mps_formula::acoustics` and are not re-wrapped here.
//!
//! Rust module name is `acoustics_ffi`; the exported C symbols are prefixed
//! `acoustics_`.
//!
//! NOTE: functions are written out explicitly (no macro) because cbindgen does
//! not expand declarative macros, so macro-generated `pub extern "C" fn`
//! items are silently omitted from `rigid_body.h`.

use crate::rapier::ffi::{Bool, ffi_scalar};
use mps_formula::acoustics::*;

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_spherical_spreading_loss(range: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || spherical_spreading_loss(range))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_cylindrical_spreading_loss(range: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || cylindrical_spreading_loss(range))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_thorp_absorption(frequency_khz: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || thorp_absorption(frequency_khz))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_sabine_rt60(
    volume: f64,
    surface_area: f64,
    mean_absorption: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || sabine_rt60(volume, surface_area, mean_absorption))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_eyring_rt60(
    volume: f64,
    surface_area: f64,
    mean_absorption: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || eyring_rt60(volume, surface_area, mean_absorption))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_acoustic_impedance(
    density: f64,
    sound_speed: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || acoustic_impedance(density, sound_speed))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_transmission_coefficient(z1: f64, z2: f64, out: *mut f64) -> Bool {
    ffi_scalar(out, || transmission_coefficient(z1, z2))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_mass_law_tl(
    frequency: f64,
    surface_density: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || mass_law_tl(frequency, surface_density))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_helmholtz_resonance_frequency(
    sound_speed: f64,
    neck_area: f64,
    cavity_volume: f64,
    neck_length: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        helmholtz_resonance_frequency(sound_speed, neck_area, cavity_volume, neck_length)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_doppler_shift(
    source_frequency: f64,
    sound_speed: f64,
    receiver_velocity: f64,
    source_velocity: f64,
    approach: Bool,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        doppler_shift(
            source_frequency,
            sound_speed,
            receiver_velocity,
            source_velocity,
            approach.0 != 0,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_maekawa_barrier_attenuation(
    fresnel_number: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || maekawa_barrier_attenuation(fresnel_number))
}

#[unsafe(no_mangle)]
pub extern "C" fn acoustics_active_sonar_echo_level(
    source_level: f64,
    transmission_loss: f64,
    target_strength: f64,
    noise_level: f64,
    directivity_index: f64,
    detection_threshold: f64,
    out: *mut f64,
) -> Bool {
    ffi_scalar(out, || {
        active_sonar_echo_level(
            source_level,
            transmission_loss,
            target_strength,
            noise_level,
            directivity_index,
            detection_threshold,
        )
    })
}
