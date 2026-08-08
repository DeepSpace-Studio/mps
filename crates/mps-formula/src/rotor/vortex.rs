//! `rotor::vortex` — vortex-wake induced velocity via the Biot–Savart law.
//!
//! A finite-length vortex filament of strength `Γ` carrying from
//! `r_a` to `r_b` induces a velocity at field point `r_p`
//!
//! ```text
//! v = Γ / (4π) ·  ∫ (dl × (r_p − r')) / |r_p − r'|³
//! ```
//!
//! which has the closed form for a straight segment
//!
//! ```text
//! v = Γ / (4π) · (r1 × r2) · (r1·r̂1 − r2·r̂2) / (|r1 × r2|²)
//! ```
//!
//! with `r1 = r_a − r_p`, `r2 = r_b − r_p`, `r̂ᵢ = rᵢ / |rᵢ|`.  The
//! sign of the cross product carries the induced-velocity direction.
//!
//! Tip-vortex circulation approximation: `Γ ≈ T / (2 ρ R V_tip)` (uniform
//! bound-circulation rotor).
//!
//! ### Sources
//!
//! - Leishman §10 (vortex wake methods).
//! - Milne-Thomson, *Theoretical Aerodynamics*, §9 (Biot–Savart segment).
//!
//! All units SI; angles rad; return `None` + `set_error` on degenerate
//! geometry (`|r1×r2|` collapses — filaments seen end-on, or endpoints
//! coincident with the field point).

use super::*;
use crate::error::{ERR_INVALID_ARGUMENT, set_error};

/// Induced velocity at `point` from a straight vortex filament of strength
/// `circulation` running from `a` to `b`.
///
/// `None` when the geometry is degenerate (endpoint coincides with the field
/// point, or segment seen end-on — `|r1 × r2|² ≈ 0`).
pub fn rotor_vortex_segment_induced_velocity(
    circulation: f64,
    a: Vec3,
    b: Vec3,
    point: Vec3,
) -> Option<Vec3> {
    if !finite(circulation) || !vec3_finite(a) || !vec3_finite(b) || !vec3_finite(point) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_vortex_segment_induced_velocity: NaN inputs",
        );
        return None;
    }
    let pa = vec3_to_rapier(a) - vec3_to_rapier(point);
    let pb = vec3_to_rapier(b) - vec3_to_rapier(point);
    let cross = pa.cross(pb);
    let cross_sq = cross.length_squared();
    if cross_sq < 1.0e-30 {
        // segment end-on or zero-length — singular
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_vortex_segment_induced_velocity: degenerate segment",
        );
        return None;
    }
    let pa_len = pa.length();
    let pb_len = pb.length();
    if pa_len < 1.0e-30 || pb_len < 1.0e-30 {
        return Some(Vec3::default());
    }
    // Biot–Savart for a straight segment:
    //   v = Γ/(4π) · (r₁ × r₂) / |r₁ × r₂|² · (r̂₁ − r̂₂)·t̂
    // where t̂ is the unit tangent from a to b.  The factor
    // `(r̂₁ − r̂₂)·t̂ = cos θ₁ − cos θ₂` (Leishman fig 10.6 form).
    let ab = vec3_to_rapier(b) - vec3_to_rapier(a);
    let ab_len = ab.length();
    if ab_len < 1.0e-30 {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_vortex_segment_induced_velocity: zero-length segment",
        );
        return None;
    }
    let t_hat = ab / ab_len;
    let r1_hat = pa / pa_len;
    let r2_hat = pb / pb_len;
    let factor = (r1_hat - r2_hat).dot(t_hat);
    let v = cross * (circulation / (4.0 * PI) * factor / cross_sq);
    Some(vec3_from_rapier(v))
}

/// Uniform-bound-circulation tip-vortex strength approximation
/// `Γ ≈ T / (2 ρ R V_tip)`.
///
/// `None` when `ρ`, `R`, or `V_tip` are not positive-finite; `T = 0` returns
/// `Γ = 0` (zero-thrust rotor → zero circulation).
pub fn rotor_tip_circulation(thrust: f64, rho: f64, radius: f64, tip_speed: f64) -> Option<f64> {
    let rho_ok = finite_positive(rho);
    let r_ok = finite_positive(radius);
    let v_ok = finite_positive(tip_speed);
    let t_ok = finite_non_negative(thrust);
    if !t_ok || !rho_ok || !r_ok || !v_ok {
        set_error(
            ERR_INVALID_ARGUMENT,
            "rotor_tip_circulation: bad (T, ρ, R, V_tip)",
        );
        return None;
    }
    if thrust == 0.0 {
        return Some(0.0);
    }
    Some(thrust / (2.0 * rho * radius * tip_speed))
}

/// Sum the induced velocity at `point` from a finite wake represented as a
/// list of straight segments.  Each segment carries the same circulation
/// `Γ`.  Degenerate segments (singular geometry) are skipped rather than
/// aborting — a wake discretization routinely has near-degenerate pieces.
pub fn rotor_wake_induced_velocity(
    circulation: f64,
    segments: &[(Vec3, Vec3)],
    point: Vec3,
) -> Vec3 {
    let mut acc = rapier3d::prelude::Vector::ZERO;
    if !finite(circulation) || !vec3_finite(point) {
        return Vec3::default();
    }
    for (a, b) in segments {
        if let Some(v) = rotor_vortex_segment_induced_velocity(circulation, *a, *b, point) {
            acc += vec3_to_rapier(v);
        }
    }
    vec3_from_rapier(acc)
}
