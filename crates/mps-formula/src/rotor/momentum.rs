//! `rotor::momentum` — actuator-disk momentum theory for a lifting rotor.
//!
//! Pure momentum theory models the rotor as a thin actuator disk that
//! imparts a uniform induced velocity `v_i` to the air passing through it.
//! Hover and climb have closed-form relations; forward flight requires
//! iteration because the induced velocity appears on both sides of the
//! inflow equation (the classic "μ" coupling).
//!
//! ## Hover / axial climb
//!
//! `T = 2 ρ A v_h²` → `v_h = √(T / (2ρA))` ; climb aircraft (rate `V_c`)
//! adds `v_i` to the slipstream: `T = 2 ρ A (V_c + v_i) v_i`, giving
//! `v_i = -V_c/2 + √( (V_c/2)² + v_h² )`.
//!
//! ## Forward flight
//!
//! With axial flight speed `V_a` (component along the thrust axis) the
//! standard Glauert inflow relation is the perturbation form
//! `v_i = v_h² / √(V_a² + v_i²)`  (rearranged from
//! `T = 2ρA v_i √(V_a² + v_i²)`), which we solve by fixed-point iteration
//! with `v_h` as the initial guess.
//!
//! Induced power `P_i = T v_i`; ideal (Momentum-theory) hover power
//! `P_ideal = T v_h`.  Figure of merit `FM = P_ideal / P_actual` is a
//! dimensionless rotor efficiency (typical helicopters 0.65–0.80).
//!
//! ### Sources
//!
//! - Leishman, *Principles of Helicopter Aerodynamics*, §2 (momentum
//!   theory), §5 (forward-flight inflow).
//! - Johnson, *Helicopter Theory*, §4-2.2.
//!
//! All units SI.  Returns `None` and calls `set_error(ERR_INVALID_ARGUMENT)`
//! on any bad input (negative density/radius, NaN thrust, ...).

use super::*;
use crate::error::{ERR_INVALID_ARGUMENT, set_error};

/// Induced velocity in hover `v_h = √(T / (2ρA))` with `A = π R²`.
///
/// Returns `0` when `thrust = 0` (a windmilling or free-wheeling rotor with
/// no thrust produces no induced flow).  Invalid inputs (`rho <= 0`,
/// `radius <= 0`, negative / NaN thrust) return `None`.
pub fn rotor_hover_induced_velocity(thrust: f64, rho: f64, radius: f64) -> Option<f64> {
    if !momentum_inputs_ok(thrust, rho, radius) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_hover_induced_velocity: thrust<0 / rho<=0 / radius<=0 / NaN",
        );
        return None;
    }
    if thrust == 0.0 {
        return Some(0.0);
    }
    let area = PI * radius * radius;
    Some((thrust / (2.0 * rho * area)).sqrt())
}

/// Induced power in hover `P_i = T · v_h` (the ideal, loss-free lower bound).
///
/// For `thrust = 0` returns `0`.  `v_h` may be supplied directly (e.g. from
/// [`rotor_hover_induced_velocity`]) or recomputed inside by passing the
/// `(thrust, rho, radius)` triple.
pub fn rotor_hover_power(thrust: f64, induced_velocity: f64) -> Option<f64> {
    if !finite_non_negative(thrust) || !finite_non_negative(induced_velocity) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_hover_power: thrust or v_h negative / NaN",
        );
        return None;
    }
    Some(thrust * induced_velocity)
}

/// Induced velocity in axial climb / descent.
///
/// Solves `T = 2ρA (V_c + v_i) v_i` for `v_i`, returning the **physical**
/// (positive, real) root
/// `v_i = -V_c/2 + √( (V_c/2)² + v_h² )`.
///
/// The vortex-ring state (`V_c < 0`, rate of descent comparable to `v_h`)
/// has no well-defined momentum-theory answer — this function still returns
/// the real root of the quadratic when one exists (which it does for any
/// finite `V_c`), but the user is responsible for knowing that momentum
/// theory is physically invalid there; see Leishman §2.8.
pub fn rotor_climb_induced_velocity(
    thrust: f64,
    rho: f64,
    radius: f64,
    climb_rate: f64,
) -> Option<f64> {
    if !momentum_inputs_ok(thrust, rho, radius) || !finite(climb_rate) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_climb_induced_velocity: bad inputs",
        );
        return None;
    }
    let v_h = rotor_hover_induced_velocity(thrust, rho, radius)?;
    if v_h == 0.0 {
        return Some(0.0);
    }
    let half = climb_rate * 0.5;
    let disc = half * half + v_h * v_h;
    Some(-half + disc.sqrt())
}

/// Induced velocity in forward flight via fixed-point iteration of the
/// Glauert inflow equation `v_i = v_h² / √(V_a² + v_i²)`.
///
/// `axial_speed` is the component of the free-stream velocity **along the
/// rotor thrust axis** (positive = climb / headwind into the disk).  At
/// `V_a = 0` the iteration collapses to `v_h` and is short-circuited.
/// Convergence is `|Δv_i| < 1e-8 · v_h` or 200 iterations, whichever comes
/// first; on non-convergence the last iterate is returned (no `None`), since
/// the equation is a contraction for `V_a > 0` and the residual is informative.
pub fn rotor_forward_induced_velocity(
    thrust: f64,
    rho: f64,
    radius: f64,
    axial_speed: f64,
) -> Option<f64> {
    if !momentum_inputs_ok(thrust, rho, radius) || !finite(axial_speed) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_forward_induced_velocity: bad inputs",
        );
        return None;
    }
    let v_h = rotor_hover_induced_velocity(thrust, rho, radius)?;
    if v_h == 0.0 {
        return Some(0.0);
    }
    if axial_speed.abs() < EPS * v_h.max(EPS) {
        return Some(v_h);
    }
    let va2 = axial_speed * axial_speed;
    let mut v = v_h; // initial guess
    let tol = 1.0e-8 * v_h;
    for _ in 0..200 {
        let denom = (va2 + v * v).sqrt();
        if denom < EPS {
            break;
        }
        let next = v_h * v_h / denom;
        if (next - v).abs() <= tol {
            return Some(next);
        }
        v = next;
    }
    Some(v)
}

/// Figure of merit `FM = P_ideal / P_actual = T v_h / P_actual`.
///
/// A dimensionless rotor efficiency bounded above by `1` (the ideal
/// actuator disk).  Real rotors run 0.65–0.80 in hover.  `P_actual <= 0`
/// or `P_actual` non-finite is rejected.
pub fn rotor_figure_of_merit(ideal_power: f64, actual_power: f64) -> Option<f64> {
    if !finite_non_negative(ideal_power) || !finite_positive(actual_power) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_figure_of_merit: ideal<0 / actual<=0 / NaN",
        );
        return None;
    }
    Some(ideal_power / actual_power)
}

/// Tip-speed `V_tip = ω R` (m/s), the blade-tip tangential speed.
///
/// Useful as the reference speed for advance ratio `μ = V_∞ / V_tip` and for
/// the Mach-number check (`V_tip + a_slant` < local sonic speed).
pub fn rotor_tip_speed(omega: f64, radius: f64) -> Option<f64> {
    if !finite_non_negative(omega) || !finite_positive(radius) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_tip_speed: omega<0 / radius<=0 / NaN",
        );
        return None;
    }
    Some(omega * radius)
}

/// Advance ratio `μ = V_∞ / (ω R)` — dimensionless forward-flight speed.
///
/// ` μ > 0.4 ` enters the high-`μ` regime where reverse-flow, compressibility,
/// and unsteady effects become dominant; the caller is responsible for
/// rejecting such operating points, this function only computes the ratio.
pub fn rotor_advance_ratio(forward_speed: f64, omega: f64, radius: f64) -> Option<f64> {
    if !finite_non_negative(forward_speed) || !finite_positive(omega) || !finite_positive(radius) {
        set_error(ERR_INVALID_ARGUMENT, "rotor_advance_ratio: bad inputs");
        return None;
    }
    Some(forward_speed / (omega * radius))
}
