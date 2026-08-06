//! Cosmology — FLRW evolution, ΛCDM distances, early-universe milestones.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W1).
//! All public functions return `Option<f64>` (or tuples thereof) with
//! `None` on invalid inputs and a prior `set_error` for callers that need
//! the error code.  No `extern "C" fn` lives here (this is the pure-formula
//! layer; C-ABI wrappers and FFI struct exposure stay in `mps-core`).

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::{finite, finite_positive};

/// Speed of light in vacuum [m/s].
const SPEED_OF_LIGHT: f64 = 299_792_458.0;
/// Parsec in metres (1 pc = 648000/π AU; 1 AU = 1.495978707e11 m).
const METRES_PER_PARSEC: f64 = 3.0856775814913673e16;
/// Metres per megaparsec.
const METRES_PER_MEGAPARSEC: f64 = 1.0e6 * METRES_PER_PARSEC;

/// Flat-ΛCDM line-of-sight comoving distance [Mpc] for small redshifts using
/// the z ≪ 1 Hubble approximation `D_C ≈ c · z / H0`.
///
/// - `hubble_constant` in km/s/Mpc (a.k.a. `H0`; typical 67.4 for Planck18,
///   70 for the classical "Hubble Key Project" value, 73 for SH0ES)
/// - `redshift` dimensionless and ≥ 0
///
/// Returns the comoving distance in Mpc.  Inputs are validated for
/// finiteness and positivity; failures set `ERR_INVALID_ARGUMENT` and return
/// `None`.
pub fn friedmann_hubble_distance(hubble_constant: f64, redshift: f64) -> Option<f64> {
    if !finite_positive(hubble_constant) || !finite(redshift) || redshift < 0.0 {
        set_error(ERR_INVALID_ARGUMENT, "bad cosmology arguments");
        return None;
    }
    // Convert H0 from km/s/Mpc → 1/s via dimensional analysis:
    //   H0 [km/s/Mpc] · 1000 [m/km] / (Mpc in metres) = H0 in 1/s.
    let h0_si = hubble_constant * 1000.0 / METRES_PER_MEGAPARSEC;
    // D_C = c · z / H0  (metres) → divide back by METRES_PER_MEGAPARSEC.
    let dist_m = SPEED_OF_LIGHT * redshift / h0_si;
    Some(dist_m / METRES_PER_MEGAPARSEC)
}

/// Luminosity distance `D_L = (1 + z) · D_C` under the flat-ΛCDM small-z
/// approximation.  Useful for converting apparent magnitude to absolute
/// magnitude at low redshift before the proper cosmological integral is
/// needed.  Inputs in the same units as [`friedmann_hubble_distance`].
pub fn luminosity_distance_hubble(hubble_constant: f64, redshift: f64) -> Option<f64> {
    let d_c = friedmann_hubble_distance(hubble_constant, redshift)?;
    Some((1.0 + redshift) * d_c)
}

/// Einstein-de Sitter (matter-only flat universe) cosmic age:
/// `t0 = 2 / (3 · H0)`.
/// `hubble_constant` in km/s/Mpc; returns age in gigayears (1 Gyr = 1e9 yr).
pub fn einstein_de_sitter_age(hubble_constant: f64) -> Option<f64> {
    if !finite_positive(hubble_constant) {
        set_error(ERR_INVALID_ARGUMENT, "bad H0 for Einstein-de Sitter age");
        return None;
    }
    let h0_si = hubble_constant * 1000.0 / METRES_PER_MEGAPARSEC;
    let age_s = 2.0 / (3.0 * h0_si);
    Some(age_s / 3.15576e16) // → Gyr (1 Gyr ≈ 3.15576e16 s)
}

/// Hubble flow recession velocity `v = H0 · D` for sub-luminal small-distance
/// regime (`D < c / H0`).  Inputs: `hubble_constant` [km/s/Mpc],
/// `distance_mpc` [Mpc]; returns `v` [km/s].
pub fn hubble_flow_velocity(hubble_constant: f64, distance_mpc: f64) -> Option<f64> {
    if !finite_positive(hubble_constant)
        || !finite(distance_mpc)
        || distance_mpc < 0.0
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Hubble flow args");
        return None;
    }
    Some(hubble_constant * distance_mpc)
}
