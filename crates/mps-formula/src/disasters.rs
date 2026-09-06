//! Natural-disaster field models — tropical cyclone (typhoon / hurricane),
//! tornado and hailstorm. Pure math: every function maps parameters and a
//! sample point to a wind velocity / pressure deficit / impact impulse
//! without touching any Rapier or world state.
//!
//! ## Wind field models
//!
//! * **Tropical cyclone** — Rankine-like vortex: tangential wind grows
//!   linearly inside the radius of maximum wind `R_m` and decays as
//!   `(R_m/r)^0.6` outside; adds a radial inflow fraction (boundary-layer
//!   convergence), the storm translation velocity, and a power-law height
//!   profile `v(z) = v₁₀ (z/z₁₀)^α` above the reference height. The same
//!   model covers both regional names — 台风 (typhoon, NW Pacific) and
//!   hurricane (Atlantic/NE Pacific) — via the `spin` sign (+1
//!   counterclockwise for the northern hemisphere, −1 clockwise).
//! * **Tornado** — Rankine combined vortex on a vertical axis with a much
//!   tighter core, a core-concentrated updraft with a sinusoidal vertical
//!   profile, linear wind fade above the funnel top, and a cyclostrophic
//!   pressure deficit `Δp = ½ ρ v²`.
//! * **Hail** — spherical ice stone ballistics: quadratic-drag terminal
//!   velocity, closed-form fall speed after a given drop height, kinetic
//!   energy and the perfectly-inelastic capture impulse.
//!
//! The world-facing C ABI that applies these fields as forces lives in
//! `mps-core::rapier::disasters` (`disaster_*` functions).

use crate::ffi::Vec3;
use crate::math::{Vector3f64, clamp};

/// Sea-level air density at 15 °C (kg/m³), ISA.
pub const AIR_DENSITY_SEA_LEVEL: f64 = 1.225;
/// Dense bulk ice density (kg/m³).
pub const ICE_DENSITY: f64 = 917.0;
/// Standard gravity (m/s²).
pub const GRAVITY: f64 = 9.80665;

// ---------------------------------------------------------------------------
// Tropical cyclone — typhoon / hurricane
// ---------------------------------------------------------------------------

/// Parameters of the tropical-cyclone wind field.
#[derive(Clone, Copy, Debug)]
pub struct CycloneParams {
    /// Storm centre (sea level; sampling only uses the horizontal offset and
    /// the height above `center.y`).
    pub center: Vec3,
    /// Storm translation velocity (m/s), added uniformly to the wind field.
    pub translation: Vec3,
    /// Maximum sustained wind speed at `radius_max_wind` (m/s).
    pub max_wind: f64,
    /// Radius of maximum wind `R_m` (m).
    pub radius_max_wind: f64,
    /// Rotation sense: +1 counterclockwise (northern hemisphere), −1 clockwise.
    pub spin: f64,
    /// Radial inflow fraction of the tangential wind (0…0.3 typical), driving
    /// boundary-layer convergence towards the eye.
    pub inflow_fraction: f64,
    /// Power-law exponent α of the boundary-layer height profile (~0.11 over
    /// open sea, ~0.2 over rough terrain).
    pub height_exponent: f64,
    /// Reference height for `max_wind` (m; standard anemometer height is 10).
    pub reference_height: f64,
}

impl Default for CycloneParams {
    fn default() -> Self {
        Self {
            center: Vec3::default(),
            translation: Vec3::default(),
            max_wind: 40.0,
            radius_max_wind: 30_000.0,
            spin: 1.0,
            inflow_fraction: 0.15,
            height_exponent: 0.11,
            reference_height: 10.0,
        }
    }
}

/// Tangential wind speed of the Rankine-like cyclone profile at horizontal
/// distance `r` from the centre.
#[inline]
pub fn cyclone_tangential_speed(max_wind: f64, radius_max_wind: f64, r: f64) -> f64 {
    let rm = radius_max_wind.max(1.0e-9);
    if r <= rm {
        max_wind * (r / rm)
    } else {
        max_wind * (rm / r).powf(0.6)
    }
}

/// Height-profile multiplier `clamp(z / z_ref, 0.2, 100)^α`. Wind never drops
/// below 20 % of the reference value at the surface, and the profile is
/// capped 100 reference heights up.
#[inline]
pub fn boundary_layer_height_factor(z: f64, reference_height: f64, exponent: f64) -> f64 {
    (z / reference_height.max(1.0e-9))
        .clamp(0.2, 100.0)
        .powf(exponent)
}

/// Wind velocity (m/s) of the tropical cyclone at `point`.
pub fn cyclone_wind_at(p: &CycloneParams, point: Vec3) -> Vec3 {
    let dx = point.x - p.center.x;
    let dz = point.z - p.center.z;
    let r = (dx * dx + dz * dz).sqrt();
    let v_t = cyclone_tangential_speed(p.max_wind, p.radius_max_wind, r);
    let height_factor = boundary_layer_height_factor(
        (point.y - p.center.y).max(0.0),
        p.reference_height,
        p.height_exponent,
    );

    let vortex = if r > 1.0e-9 {
        let ux = dx / r;
        let uz = dz / r;
        // Tangential (perpendicular to the radius) + radial inflow (towards eye).
        Vector3f64::new(
            p.spin * -uz * v_t - ux * v_t * p.inflow_fraction,
            0.0,
            p.spin * ux * v_t - uz * v_t * p.inflow_fraction,
        )
    } else {
        Vector3f64::zeros()
    };

    let vortex = vortex.scale(height_factor);
    Vec3 {
        x: vortex.x + p.translation.x,
        y: vortex.y + p.translation.y,
        z: vortex.z + p.translation.z,
    }
}

// ---------------------------------------------------------------------------
// Tornado
// ---------------------------------------------------------------------------

/// Parameters of the tornado wind field (vertical axis through `base`).
#[derive(Clone, Copy, Debug)]
pub struct TornadoParams {
    /// Funnel base point; the axis is vertical through `base.x`/`base.z`.
    pub base: Vec3,
    /// Funnel translation velocity (m/s), added uniformly to the wind field.
    pub translation: Vec3,
    /// Maximum tangential wind speed at the core edge (m/s).
    pub max_wind: f64,
    /// Core radius `R_c` of the Rankine vortex (m).
    pub core_radius: f64,
    /// Funnel top height above `base.y` (m); winds fade linearly to zero at
    /// twice this height.
    pub top_height: f64,
    /// Peak updraft as a fraction of `max_wind` (0…1).
    pub updraft_fraction: f64,
    /// Rotation sense: +1 counterclockwise (northern hemisphere), −1 clockwise.
    pub spin: f64,
}

impl Default for TornadoParams {
    fn default() -> Self {
        Self {
            base: Vec3::default(),
            translation: Vec3::default(),
            max_wind: 80.0,
            core_radius: 60.0,
            top_height: 1_500.0,
            updraft_fraction: 0.35,
            spin: 1.0,
        }
    }
}

/// Cyclostrophic pressure deficit (Pa) for wind speed `v` — the core pressure
/// drop of a tornado / cyclone in gradient-wind balance.
#[inline]
pub fn cyclostrophic_pressure_drop(v: f64, air_density: f64) -> f64 {
    0.5 * air_density * v * v
}

/// Wind velocity (m/s) of the tornado at `point`.
pub fn tornado_wind_at(p: &TornadoParams, point: Vec3) -> Vec3 {
    let dx = point.x - p.base.x;
    let dz = point.z - p.base.z;
    let r = (dx * dx + dz * dz).sqrt();
    let rc = p.core_radius.max(1.0e-9);
    let height = (point.y - p.base.y).max(0.0);

    // Fade tangential wind linearly to zero between the funnel top and twice
    // the funnel top.
    let height_factor = if height <= p.top_height {
        1.0
    } else {
        clamp(1.0 - (height - p.top_height) / p.top_height, 0.0, 1.0)
    };
    if height_factor <= 0.0 {
        return p.translation;
    }

    // Rankine combined vortex.
    let v_t = p.max_wind * if r <= rc { r / rc } else { rc / r.max(1.0e-9) };

    let vortex = if r > 1.0e-9 {
        let ux = dx / r;
        let uz = dz / r;
        Vector3f64::new(p.spin * -uz * v_t, 0.0, p.spin * ux * v_t)
    } else {
        Vector3f64::zeros()
    };

    // Updraft concentrated in the core, peaking mid-funnel.
    let updraft = if height > 0.0 && height < p.top_height {
        let core_factor = if r <= rc { 1.0 } else { rc / r.max(1.0e-9) };
        p.updraft_fraction
            * p.max_wind
            * (std::f64::consts::PI * height / p.top_height).sin()
            * core_factor
    } else {
        0.0
    };

    let vortex = vortex
        .add(Vector3f64::new(0.0, updraft, 0.0))
        .scale(height_factor);
    Vec3 {
        x: vortex.x + p.translation.x,
        y: vortex.y + p.translation.y,
        z: vortex.z + p.translation.z,
    }
}

// ---------------------------------------------------------------------------
// Hail
// ---------------------------------------------------------------------------

/// Mass (kg) of a spherical hailstone of `radius` (m).
#[inline]
pub fn hail_mass(radius: f64, ice_density: f64) -> f64 {
    4.0 / 3.0 * std::f64::consts::PI * radius.powi(3) * ice_density
}

/// Quadratic-drag terminal velocity (m/s) of a spherical hailstone.
#[inline]
pub fn hail_terminal_velocity(
    radius: f64,
    ice_density: f64,
    air_density: f64,
    drag_coefficient: f64,
) -> f64 {
    // vt = sqrt(2 m g / (ρ_air · C_d · A)) = sqrt(8 ρ_ice g r / (3 ρ_air C_d))
    let denom = 3.0 * air_density * drag_coefficient;
    if denom <= 0.0 {
        return f64::INFINITY;
    }
    (8.0 * ice_density * GRAVITY * radius.max(0.0) / denom).sqrt()
}

/// Fall speed (m/s) reached after dropping `fall_height` (m) from rest under
/// quadratic drag: `v(h) = vt · sqrt(1 − exp(−2 g h / vt²))`.
pub fn hail_fall_speed(
    radius: f64,
    fall_height: f64,
    ice_density: f64,
    air_density: f64,
    drag_coefficient: f64,
) -> f64 {
    let vt = hail_terminal_velocity(radius, ice_density, air_density, drag_coefficient);
    if !vt.is_finite() || vt <= 0.0 {
        return vt;
    }
    let h = fall_height.max(0.0);
    vt * (1.0 - (-2.0 * GRAVITY * h / (vt * vt)).exp()).sqrt()
}

/// Kinetic energy (J) of a mass `m` (kg) moving at `speed` (m/s).
#[inline]
pub fn hail_kinetic_energy(mass: f64, speed: f64) -> f64 {
    0.5 * mass * speed * speed
}

/// Perfectly-inelastic capture impulse (N·s) when a hailstone of mass `m`
/// (kg) at `speed` (m/s) is stopped by a body.
#[inline]
pub fn hail_impulse(mass: f64, speed: f64) -> f64 {
    mass * speed
}

// ---------------------------------------------------------------------------
// Wind drag on a rigid body
// ---------------------------------------------------------------------------

/// Quadratic wind drag force (N) on a body from the relative wind
/// `relative_wind = wind − body_velocity`: `F = ½ ρ C_d A |v| v`, directed
/// along the relative wind (pushing the body with the wind).
pub fn wind_drag_force(
    relative_wind: Vec3,
    air_density: f64,
    drag_coefficient: f64,
    reference_area: f64,
) -> Vec3 {
    let v = Vector3f64::new(relative_wind.x, relative_wind.y, relative_wind.z);
    let speed = v.length();
    let scale = 0.5 * air_density * drag_coefficient * reference_area * speed;
    let f = v.scale(scale);
    Vec3 {
        x: f.x,
        y: f.y,
        z: f.z,
    }
}
