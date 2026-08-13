//! Shared-memory physics arena — spaceflight C-ABI submodule split.
//!
//! Originally a single 2610-line `spaceflight.rs`.  Split into per-domain
//! submodules per OPTIMIZATION.md §3; this `mod.rs` concentrates the shared
//! zero-overhead numeric helpers + constants each per-domain file needs.
//!
//! All `use` statements below are `pub(crate)` so per-domain submodules can
//! write `use super::*;` and immediately see every imported type/helper
//! (without each file restating a 30-entry import list — that would be worse
//! for maintenance than the original monolith).  The `_` trick avoids
//! unused-import warnings: the name is re-exported (so `super::*` picks it
//! up) but it's *not* added to *this* module's namespace, so it cannot
//! trigger an unused warning here.
//!
//! ABI invariant: every `extern "C" fn space_*` keeps its
//! `#[unsafe(no_mangle)]` name, signature and behaviour unchanged — no
//! `ABI_VERSION` bump is needed (function names and signatures unchanged).

// Re-exported so submodules see them via `use super::*;`.  Using a single
// `X` re-export keeps *this* module's namespace lean (no unused-import
// warnings even when only one submodule actually references a given type).
pub(crate) use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
pub(crate) use crate::rapier::ffi::{
    AirlockDepressurization, AtomicOxygenErosion, BangOffBangProfile, BatteryEquivalentCircuit,
    Bool, ChemicalReactionRate, CmgExchange, CmgRobustInverse, Co2MassBalance,
    CollisionProbability, ContactForceModel, CwDerivative, CwState, DhTransform,
    FlexibleModeDerivative, FluidLoopHeatTransfer, FriisLink, GnssObservation,
    HallThrusterPerformance, HohmannTransfer, LeastSquaresAttitude, ManipulatorDynamics,
    MassProperties, OrbitalElements, Quat, QuaternionDerivative, RadarMeasurement, RadiatorPower,
    RigidBodyEulerDerivative, RigidBodyHandleRaw, ScalarKalman, Sgp4SecularRates,
    SloshPendulumDerivative, SolarPanelPower, StateVector, ThermalBalance, VariationalState, Vec3,
    WorldHandle, unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};
pub(crate) use rapier3d::prelude::Vector;
pub(crate) use std::f64::consts::{PI, TAU};

// Numeric constants reused across submodules.
pub(crate) const EPS: f64 = 1.0e-12;
pub(crate) const SIGMA: f64 = 5.670_374_419e-8;
pub(crate) const SPEED_OF_LIGHT: f64 = 299_792_458.0;

pub(crate) fn finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

pub(crate) fn write_out<T: Copy>(out: *mut T, value: T) -> Bool {
    let Some(out) = (unsafe { out.as_mut() }) else {
        set_error(ERR_INVALID_ARGUMENT, "output pointer is null");
        return Bool::FALSE;
    };
    *out = value;
    clear_error();
    Bool::TRUE
}

pub(crate) fn write_optional_out<T: Copy>(out: *mut T, value: T) {
    if let Some(out) = unsafe { out.as_mut() } {
        *out = value;
    }
}

pub(crate) fn invalid_nan(message: &str) -> f64 {
    set_error(ERR_INVALID_ARGUMENT, message);
    f64::NAN
}

pub(crate) fn cross(a: Vector, b: Vector) -> Vector {
    Vector::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

pub(crate) fn clamp_unit(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

// Per-domain submodules — see OPTIMIZATION.md §3 for the layout rationale.
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
