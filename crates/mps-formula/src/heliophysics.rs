//! Heliophysics — solar wind, magnetosphere, ionosphere, space weather.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W2).
//! Pure-computation module: no `WorldHandle`, no `RigidBody`, no Rapier
//! state.  All public functions return `Option<f64>` (or tuples thereof)
//! with `None` on invalid inputs and a prior `set_error` for callers that
//! need the error code.

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::{finite, finite_positive};

/// 1 AU in metres (IAU 2012 Resolution B2 value).
const METRES_PER_AU: f64 = 1.495978707e11;
/// Solar equatorial rotation rate (sidereal) [rad/s].
/// 25.05 d at the equator; Carrington rotation ≈ 25.38 d is synodic.
const OMEGA_SUN: f64 = 2.0 * core::f64::consts::PI / (25.05 * 86_400.0);

/// Parker spiral magnetic-field azimuthal angle (radians) at heliocentric
/// distance `r` (AU) for a solar-wind speed `v_sw` (km/s) and the Sun's
/// sidereal rotation rate `omega_sun` (rad/s).
///
/// Panels/panels drawing of the spiral: `tan(ψ) = -ω · r / v_sw`.  Returns
/// the angle in the half-open `[-π/2, 0]` range because `r, v_sw, ω > 0`.
/// For a default Carrington/sidereal value use [`solar_wind_parker_angle_au`].
pub fn solar_wind_parker_spiral_angle(radius_au: f64, v_sw: f64, omega_sun: f64) -> Option<f64> {
    if !finite_positive(radius_au) || !finite_positive(v_sw) || !finite_positive(omega_sun) {
        set_error(ERR_INVALID_ARGUMENT, "bad Parker spiral args");
        return None;
    }
    let r_m = radius_au * METRES_PER_AU;
    let v_si = v_sw * 1000.0;
    Some((-omega_sun * r_m / v_si).atan())
}

/// Convenience wrapper for [`solar_wind_parker_spiral_angle`] using the
/// default sidereal solar rotation rate ([OMEGA_SUN]).
pub fn solar_wind_parker_angle_au(radius_au: f64, v_sw: f64) -> Option<f64> {
    solar_wind_parker_spiral_angle(radius_au, v_sw, OMEGA_SUN)
}

/// Dynamic pressure of the solar wind: `P = ρ · v²` where the mass density
/// `rho` is given in protons/m³ (= ρ / m_p) and `v_sw` in km/s.
/// Returns pressure in nPa (1 nPa = 1e-9 Pa).
pub fn solar_wind_dynamic_pressure(proton_density: f64, v_sw: f64) -> Option<f64> {
    const M_PROTON: f64 = 1.6726219e-27; // kg
    if !finite_positive(proton_density) || !finite_positive(v_sw) {
        set_error(ERR_INVALID_ARGUMENT, "bad solar wind pressure args");
        return None;
    }
    let rho = proton_density * M_PROTON;
    let v_si = v_sw * 1000.0;
    let p_pa = rho * v_si * v_si;
    Some(p_pa * 1.0e9) // → nPa
}

/// Annales Geophysicae Burton equation (1975) — Dst index time-rate in nT/h:
/// `dDst/dt = -Dst/τ + Q`, with `tau_hours` the ring-current decay timescale
/// (typical 7-8 h during storm main phase) and `q_nT_per_hour` the source
/// rate driven by magnetopause current integration.  Returns the instantaneous
/// dDst/dt.
pub fn dst_index_rate(dst_n_t: f64, tau_hours: f64, q_n_t_per_hour: f64) -> Option<f64> {
    if !finite(dst_n_t) || !finite_positive(tau_hours) || !finite(q_n_t_per_hour) {
        set_error(ERR_INVALID_ARGUMENT, "bad Burton equation args");
        return None;
    }
    Some(-dst_n_t / tau_hours + q_n_t_per_hour)
}

/// Jeans escape flux [molecules m^-2 s^-1] from an exosphere; uses the
/// `Φ = n_exo · v_thermal · (1 + λ) · exp(-λ)` form where λ is the
/// escape parameter `G · M · m / (k · T · r)`.
///
/// Inputs:
/// - `n_exo` [m^-3] exobase number density
/// - `temperature` [K]  exobase temperature
/// - `escape_parameter` dimensionless λ
/// - `mass_kg` [kg] escaping molecule mass
pub fn jeans_escape_flux(
    n_exo: f64,
    temperature: f64,
    escape_parameter: f64,
    mass_kg: f64,
) -> Option<f64> {
    if !finite_positive(n_exo)
        || !finite_positive(temperature)
        || !finite(escape_parameter)
        || escape_parameter < 0.0
        || !finite_positive(mass_kg)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Jeans escape args");
        return None;
    }
    const BOLTZMANN: f64 = 1.380649e-23;
    let v_thermal = (2.0 * BOLTZMANN * temperature / mass_kg).sqrt();
    Some(n_exo * v_thermal * (1.0 + escape_parameter) * (-escape_parameter).exp())
}
