//! Spaceflight engineering — orbital mechanics, attitude control, thermal, propulsion, and environment formulas.
//!
//! Pure computation only — no access to `WorldHandle`, `RigidBody`, or Rapier
//! state. Originally a single 2040-line `spaceflight.rs` file; split into eight
//! per-domain submodules per OPTIMIZATION.md §N8.  This `mod.rs` concentrates
//! the shared numeric helpers + constants the per-domain files need.
//!
//! All `pub(crate) use` / `pub(crate) const` / `pub(crate) fn` re-exports
//! below are visible to every per-domain submodule via `use super::*;` so the
//! per-domain files need not restate a 30-symbol import list (that would be
//! worse for maintenance than the original monolith).
//!
//! The original file had no `extern "C" fn` — this is the pure-formula
//! layer consumed by `mps-cosmos` (via `use mps_formula::spaceflight;`,
//! see `crates/mps-cosmos/src/orbit.rs` and `perturbation.rs`) and directly
//! by `mps-test` and `mps-web`.  The `pub use <domain>::*;` globs at the
//! bottom of this file keep both the crate-internal and the transitive
//! `mps_formula::spaceflight::<name>` paths stable.

// Re-exported so submodules see them via `use super::*;`.  Plain `use` would
// be private to this module; `pub(crate) use` makes them visible to every
// submodule that writes `use super::*;`.
pub(crate) use std::f64::consts::{PI, TAU};

pub(crate) use crate::ffi::{
    AirlockDepressurization, AtomicOxygenErosion, BangOffBangProfile, BatteryEquivalentCircuit,
    ChemicalReactionRate, CmgExchange, CmgRobustInverse, Co2MassBalance, CollisionProbability,
    ContactForceModel, CwDerivative, CwState, DhTransform, FlexibleModeDerivative,
    FluidLoopHeatTransfer, FriisLink, GnssObservation, HallThrusterPerformance, HohmannTransfer,
    LeastSquaresAttitude, ManipulatorDynamics, MassProperties, OrbitalElements, Quat,
    QuaternionDerivative, RadarMeasurement, RadiatorPower, RigidBodyEulerDerivative, ScalarKalman,
    Sgp4SecularRates, SloshPendulumDerivative, SolarPanelPower, StateVector, ThermalBalance,
    VariationalState, Vec3, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

// Numeric constants reused across submodules.
pub(crate) const EPS: f64 = 1.0e-12;
pub(crate) const SIGMA: f64 = 5.670_374_419e-8;
pub(crate) const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Returns `true` only if every element is finite (no NaN / infinity).
pub(crate) fn finite(values: &[f64]) -> bool {
    values.iter().all(|v| v.is_finite())
}

/// Clamps `value` into the `[−1.0, 1.0]` interval — used to keep square roots
/// of `1 − x²` numerically safe.
pub(crate) fn clamp_unit(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

/// Stumpff functions `c(z) = (1 − cos√z)/z` and `s(z) = (√z − sin√z)/(z√z)`,
/// with the `z → 0` limiting forms `c(0) = ½`, `s(0) = 1/6` used by the
/// universal-variable Lambert solver (see `spaceflight::kepler`).
pub(crate) fn stumpff_functions(z: f64) -> (f64, f64) {
    if z > EPS {
        let sz = z.sqrt();
        ((1.0 - sz.cos()) / z, (sz - sz.sin()) / (z * sz))
    } else if z < -EPS {
        let sz = (-z).sqrt();
        ((sz.cosh() - 1.0) / (-z), (sz.sinh() - sz) / ((-z) * sz))
    } else {
        (0.5, 1.0 / 6.0)
    }
}

// ---------------------------------------------------------------------------
// Per-domain submodules — see OPTIMIZATION.md §N8 for the layout rationale.
// ---------------------------------------------------------------------------

pub mod debris;
pub mod dynamics;
pub mod gnss;
pub mod kepler;
pub mod perturbation;
pub mod propulsion;
pub mod rotation;
pub mod thermal;

pub use debris::*;
pub use dynamics::*;
pub use gnss::*;
pub use kepler::*;
pub use perturbation::*;
pub use propulsion::*;
pub use rotation::*;
pub use thermal::*;
