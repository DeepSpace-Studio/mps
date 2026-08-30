//! `rotor::performance` — combined rotor power accounting and efficiency.
//!
//! The total shaft power required by a rotor is
//!
//! ```text
//! P_total = P_i + P_0 + P_c + P_p
//! ```
//!
//! with:
//!
//! | term | meaning | source |
//! |---|---|---|
//! | `P_i` | induced power | [`crate::rotor::momentum::rotor_hover_power`] or `T·v_i` |
//! | `P_0 = σ C_d0 ρ A (ΩR)³ / 8` | profile power (blade profile drag) | closed-form strip-integral |
//! | `P_c = T V_c` | climb power (work done against gravity during climb) | kinematic |
//! | `P_p` | parasite power `½ ρ V_∞³ S_fus C_d_fus` | airframe drag |
//!
//! Profile power formula assumes a uniform blade profile drag coefficient
//! `C_d0` and an elliptical-section chord distribution (the classical
//! `P_0 = σ·C_d0·ρ·A·(ΩR)³/8`); for tapered / real blades use the
//! [`crate::rotor::blade_element`] integration result and feed its
//! `profile_power` field through [`rotor_total_power`] directly.
//!
//! ### Sources
//!
//! - Leishman §2, §5 (power accounting in hover and forward flight).
//! - Johnson §5-3.
//!
//! All SI; `None` + `set_error` on bad geometry.

use super::*;
use crate::error::{ERR_INVALID_ARGUMENT, set_error};

/// Closed-form profile power for a uniform-chord uniform-drag rotor:
/// `P_0 = σ·C_d0·ρ·A·(ΩR)³ / 8`.
///
/// `None` when the rotor geometry / density / angular speed are invalid.
pub fn rotor_profile_power(rotor: &RotorParams, rho: f64, omega: f64) -> Option<f64> {
    let sigma = rotor.solidity()?;
    let area = rotor.disk_area()?;
    if !finite_positive(rho) || !finite_positive(omega) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_profile_power: rho<=0 / omega<=0 / NaN",
        );
        return None;
    }
    let v_tip = omega * rotor.radius;
    Some(sigma * rotor.profile_cd0 * rho * area * v_tip * v_tip * v_tip / 8.0)
}

/// Total rotor power `P_total = induced + profile + climb + parasite`.
///
/// All four components are passed already-computed; the function does
/// non-negativity validation and summation.  This is the conventional
/// assembly point — its arguments come from the [`momentum`],
/// [`blade_element`]/[`super::performance`] and the airframe-drag module
/// (`crate::aerodynamics`).
pub fn rotor_total_power(
    induced_power: f64,
    profile_power: f64,
    climb_power: f64,
    parasite_power: f64,
) -> Option<f64> {
    let parts = [induced_power, profile_power, climb_power, parasite_power];
    if !parts.iter().all(|p| finite_non_negative(*p)) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_total_power: negative/NaN power term",
        );
        return None;
    }
    Some(parts.iter().sum())
}

/// Equivalent flat-plate area `f = S·C_d` for an airframe-drag power term
/// `P_p = ½ ρ V∞³ f`.  Returns the flat-plate area in m².
pub fn rotor_flat_plate_area(reference_area: f64, drag_coefficient: f64) -> Option<f64> {
    if !finite_positive(reference_area) || !finite_non_negative(drag_coefficient) {
        set_error(ERR_INVALID_ARGUMENT, "rotor_flat_plate_area: bad S / C_d");
        return None;
    }
    Some(reference_area * drag_coefficient)
}

/// Parasite power `P_p = ½ ρ V∞³ · f` (W) given a flat-plate area `f`
/// (m²) and free-stream speed `V∞` (m/s).
pub fn rotor_parasite_power(
    density: f64,
    free_stream_speed: f64,
    flat_plate_area: f64,
) -> Option<f64> {
    if !finite_positive(density)
        || !finite_non_negative(free_stream_speed)
        || !finite_non_negative(flat_plate_area)
    {
        set_error(ERR_INVALID_ARGUMENT, "rotor_parasite_power: bad ρ / V∞ / f");
        return None;
    }
    Some(0.5 * density * free_stream_speed.powi(3) * flat_plate_area)
}

/// Climb power `P_c = T · V_c` — the rate of work done lifting the aircraft
/// at vertical rate `V_c`.  `T` and `V_c` are signed (climb positive); a
/// descending aircraft contributes negative power.
pub fn rotor_climb_power(thrust: f64, climb_rate: f64) -> Option<f64> {
    if !finite_non_negative(thrust) || !finite(climb_rate) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_climb_power: bad thrust / climb rate",
        );
        return None;
    }
    Some(thrust * climb_rate)
}

/// Hover efficiency metric — the ratio of ideal induced power to the total
/// power required (the figure of merit when `P_actual = P_total`).  Identical
/// in formula to [`super::momentum::rotor_figure_of_merit`] but provided here
/// under the performance-accounting namespace for callers that route
/// everything through [`rotor_total_power`].
pub fn rotor_hover_efficiency(ideal_power: f64, actual_power: f64) -> Option<f64> {
    if !finite_non_negative(ideal_power) || !finite_positive(actual_power) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_hover_efficiency: ideal<0 / actual<=0 / NaN",
        );
        return None;
    }
    Some(ideal_power / actual_power)
}
