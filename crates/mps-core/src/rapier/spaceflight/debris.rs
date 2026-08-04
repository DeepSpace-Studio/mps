//! `spaceflight::debris` submodule — SGP4 secular rates & debris collision probability
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `out_probability` must be null or point to a valid, writable `CollisionProbability`.
#[unsafe(no_mangle)]
pub extern "C" fn space_debris_collision_probability(
    miss_distance: f64,
    combined_radius: f64,
    sigma_radial: f64,
    sigma_intrack: f64,
    out_probability: *mut CollisionProbability,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[miss_distance, combined_radius, sigma_radial, sigma_intrack])
            || combined_radius < 0.0
            || sigma_radial <= 0.0
            || sigma_intrack <= 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid debris collision probability parameters",
            );
            return Bool::FALSE;
        }
        let sigma = (sigma_radial * sigma_intrack).sqrt();
        let probability = (combined_radius * combined_radius
            / (2.0 * sigma_radial * sigma_intrack))
            * (-0.5 * miss_distance * miss_distance / (sigma * sigma)).exp();
        write_out(
            out_probability,
            CollisionProbability {
                probability: probability.clamp(0.0, 1.0),
                combined_sigma: sigma,
            },
        )
    })
}

/// # Safety
/// `out_rates` must be null or point to a valid, writable `Sgp4SecularRates`.
#[unsafe(no_mangle)]
pub extern "C" fn space_sgp4_j2_secular_rates(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    mean_motion: f64,
    equatorial_radius: f64,
    j2: f64,
    out_rates: *mut Sgp4SecularRates,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[
            semi_major_axis,
            eccentricity,
            inclination,
            mean_motion,
            equatorial_radius,
            j2,
        ]) || semi_major_axis <= 0.0
            || !(0.0..1.0).contains(&eccentricity)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid SGP4 secular parameters");
            return Bool::FALSE;
        }
        let p = semi_major_axis * (1.0 - eccentricity * eccentricity);
        let factor = 1.5 * j2 * mean_motion * (equatorial_radius / p).powi(2);
        write_out(
            out_rates,
            Sgp4SecularRates {
                mean_motion_dot: 0.0,
                raan_dot: -factor * inclination.cos(),
                argument_of_perigee_dot: 0.5 * factor * (5.0 * inclination.cos().powi(2) - 1.0),
            },
        )
    })
}
