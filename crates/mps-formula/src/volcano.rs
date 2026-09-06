//! Volcanic-eruption field models — eruptive plume flow, thermal exposure
//! and lava-bomb ballistics. Pure math: every function maps parameters and a
//! sample point to a flow velocity / exposure temperature without touching
//! any Rapier or world state.
//!
//! ## Models
//!
//! * **Eruptive plume** — a widening Gaussian column above the vent: the
//!   column radius grows linearly with normalized height `q = h / H`, the
//!   core updraft decays linearly to zero at `q = 1.2`, and a mushroom-cap
//!   radial outflow surrounds `q ≈ 1`.
//! * **Thermal exposure** — the in-plume air temperature interpolates from
//!   the lava temperature at the vent core down to the ambient temperature
//!   with the same Gaussian/vertical decay; rigid bodies heat towards the
//!   local exposure temperature (Newton exchange) and, above their melt
//!   point, lose mass at a superheat-proportional rate.
//! * **Lava bombs** — ballistic ejecta clasts (spherical rock); impulse on
//!   impact is `m · v`.
//!
//! The world-facing C ABI that applies these fields to rigid bodies —
//! including progressive melting — lives in `mps-core::rapier::volcano`
//! (`volcano_*` functions).

use crate::ffi::Vec3;

/// Ambient air temperature (K, 20 °C).
pub const AMBIENT_TEMPERATURE: f64 = 293.15;
/// Basaltic lava temperature (K, ~1200 °C).
pub const LAVA_TEMPERATURE: f64 = 1473.15;
/// Solid rock density (kg/m³) for lava-bomb clasts.
pub const ROCK_DENSITY: f64 = 2700.0;
/// Reference lava-bomb clast radius (m) for the default ejecta impulse.
pub const BOMB_RADIUS: f64 = 0.15;
/// Reference lava-bomb launch speed (m/s) for the default ejecta impulse.
pub const BOMB_SPEED: f64 = 60.0;
/// Height (in normalized plume heights) at which the updraft has fully decayed.
pub const PLUME_DECAY_HEIGHT: f64 = 1.2;

/// Parameters of the eruptive plume (vertical axis through `vent`).
#[derive(Clone, Copy, Debug)]
pub struct PlumeParams {
    /// Vent position; the plume axis is vertical through `vent.x`/`vent.z`.
    pub vent: Vec3,
    /// Base column radius (m); widens linearly with normalized height.
    pub plume_radius: f64,
    /// Column height (m).
    pub plume_height: f64,
    /// Core vertical velocity at the vent (m/s).
    pub max_updraft: f64,
    /// Ejecta source temperature (K).
    pub lava_temperature: f64,
}

impl Default for PlumeParams {
    fn default() -> Self {
        Self {
            vent: Vec3::default(),
            plume_radius: 200.0,
            plume_height: 2000.0,
            max_updraft: 60.0,
            lava_temperature: LAVA_TEMPERATURE,
        }
    }
}

/// Column radius (m) at normalized height `q = h / plume_height` — the
/// plume widens linearly as it rises.
#[inline]
pub fn plume_column_radius(base_radius: f64, q: f64) -> f64 {
    base_radius.max(1.0e-9) * (1.0 + 2.0 * q.max(0.0))
}

/// Flow velocity (m/s) of the eruptive plume at `point`: a Gaussian-profile
/// updraft inside the widening column plus a mushroom-cap radial outflow
/// around the column top.
pub fn plume_flow_at(p: &PlumeParams, point: Vec3) -> Vec3 {
    let dx = point.x - p.vent.x;
    let dz = point.z - p.vent.z;
    let r = (dx * dx + dz * dz).sqrt();
    let h = (point.y - p.vent.y).max(0.0);
    let q = h / p.plume_height.max(1.0e-9);
    let radius = plume_column_radius(p.plume_radius, q);
    let core = (-(r * r) / (radius * radius)).exp();

    // Updraft decays linearly to zero at PLUME_DECAY_HEIGHT column heights.
    let updraft = p.max_updraft * (1.0 - q / PLUME_DECAY_HEIGHT).clamp(0.0, 1.0) * core;
    // Mushroom cap: radial outflow concentrated around q ≈ 1.
    let cap = p.max_updraft * 0.4 * ((q - 1.0) * (q - 1.0) / 0.08).exp() * core;

    if r > 1.0e-9 {
        let ux = dx / r;
        let uz = dz / r;
        Vec3 {
            x: ux * cap,
            y: updraft,
            z: uz * cap,
        }
    } else {
        Vec3 {
            x: 0.0,
            y: updraft,
            z: 0.0,
        }
    }
}

/// Exposure temperature (K) of the plume at `point` — the in-plume air
/// temperature a body exchanges heat with.
pub fn plume_temperature_at(p: &PlumeParams, point: Vec3, ambient: f64) -> f64 {
    let dx = point.x - p.vent.x;
    let dz = point.z - p.vent.z;
    let r = (dx * dx + dz * dz).sqrt();
    let h = (point.y - p.vent.y).max(0.0);
    let q = h / p.plume_height.max(1.0e-9);
    let radius = plume_column_radius(p.plume_radius, q);
    let core = (-(r * r) / (radius * radius)).exp();
    let vertical = (1.0 - q / PLUME_DECAY_HEIGHT).clamp(0.0, 1.0);
    ambient + (p.lava_temperature - ambient) * core * vertical
}

/// Newton-exchange temperature rate (K/s) of a body at exposure temperature
/// `exposure` (K): `dT/dt = k · (T_exposure − T_body)`. Positive when the
/// body heats up, negative when it cools.
#[inline]
pub fn body_heating_rate(exposure: f64, body_temp: f64, exchange_rate: f64) -> f64 {
    exchange_rate * (exposure - body_temp)
}

/// Fraction of remaining mass lost per second once the body temperature
/// exceeds `melt_point` (K); zero below the melt point, linear in superheat
/// above it: `rate · (T − T_melt) / T_melt`.
#[inline]
pub fn melt_mass_loss_rate(body_temp: f64, melt_point: f64, base_rate: f64) -> f64 {
    if body_temp <= melt_point || melt_point <= 0.0 {
        0.0
    } else {
        base_rate * (body_temp - melt_point) / melt_point
    }
}

/// Mass (kg) of a spherical lava-bomb clast of `radius` (m).
#[inline]
pub fn lava_bomb_mass(radius: f64, rock_density: f64) -> f64 {
    4.0 / 3.0 * std::f64::consts::PI * radius.powi(3) * rock_density
}

/// Impact impulse magnitude (N·s) of a lava bomb of mass `m` (kg) at
/// `speed` (m/s) — a perfectly inelastic capture.
#[inline]
pub fn lava_bomb_impulse(mass: f64, speed: f64) -> f64 {
    mass * speed
}
