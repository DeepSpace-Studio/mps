//! Rotor aerodynamics — momentum theory, blade-element theory, vortex wake,
//! and performance synthesis for rotary-wing aircraft (helicopter rotors,
//! propellers, ducted fans).
//!
//! Pure computation only — no access to `WorldHandle`, `RigidBody`, or Rapier
//! state.  Mirrors the layout used by [`crate::spaceflight`] (per-domain
//! submodules + a `mod.rs` that re-exports shared numeric helpers and the
//! `pub use <domain>::*;` globs keeping `mps_formula::rotor::<name>` paths
//! stable).
//!
//! ## Naming
//!
//! Public `pub fn` names carry the `rotor_` prefix so the crate-level glob
//! re-export does not collide with the existing `aerodynamics::*` surface
//! (`compute_surface_force`, `estimate_surface_force`, ...).  Downstream
//! callers (`mps-core::rapier::rotor::*`, `mps-cosmos::flight::*`,
//! `mps-test::rapier::rotor::*`) reach every function through the
//! [`super`] glob without qualifier.
//!
//! ## Units
//!
//! All inputs and outputs are SI unless noted: density `ρ` in kg/m³, lengths in
//! m, velocities in m/s, angles in rad, forces in N, torques in N·m, power in
//! W, angular speed `ω` in rad/s.  The rotor radius `R` is the blade-tip
//! radius (disk area `A = π R²`).

// Re-exported so submodules see them via `use super::*;`.  `pub(crate) use`
// makes them visible to every submodule that writes `use super::*;`.
pub(crate) use std::f64::consts::PI;

pub(crate) use crate::ffi::{Vec3, vec3_finite, vec3_from_rapier, vec3_to_rapier};
pub(crate) use crate::math::{clamp, finite, finite_non_negative, finite_positive};

/// Disk-area relative tolerance used by iterative forward-flight solvers
/// before declaring non-convergence (sub-metre / sub-m/s class).
pub(crate) const EPS: f64 = 1.0e-12;

// ---------------------------------------------------------------------------

/// Physical and geometric description of a single rotor.
///
/// Carried by the BEM / momentum / vortex entry points as a plain value
/// (no lifetime, no Rapier handles).  All fields are SI:
///
/// | field | meaning |
/// |---|---|
/// | `radius` | blade-tip radius `R` (m) |
/// | `n_blades` | number of blades `N` |
/// | `chord` | blade chord `c` (m); taper is not modelled — use the mean |
/// | `hinge_offset` | flapping-hinge offset from hub centre (m); `0` for a teetering / articulated rotor with offset hinge ignored |
/// | `lift_slope` | 2-D lift-curve slope `a₀ = dC_L/dα` (per rad); a thin flat plate is `2π` |
/// | `zero_lift_alpha` | blade pitch angle at which the section produces zero lift (rad) |
/// | `profile_cd0` | zero-lift profile drag coefficient `C_d0` used by the simple-drag polar in [`blade_element`] |
/// | `cd_k` | induced-drag-polar coefficient (`C_d = C_d0 + k·C_L²`) |
#[derive(Clone, Copy, Debug)]
pub struct RotorParams {
    pub radius: f64,
    pub n_blades: u32,
    pub chord: f64,
    pub hinge_offset: f64,
    pub lift_slope: f64,
    pub zero_lift_alpha: f64,
    pub profile_cd0: f64,
    pub cd_k: f64,
}

impl RotorParams {
    /// Disk area `A = π R²` (m²) — `None` when `radius` is not finite/positive.
    #[inline]
    pub fn disk_area(&self) -> Option<f64> {
        if !finite_positive(self.radius) {
            return None;
        }
        Some(PI * self.radius * self.radius)
    }

    /// Solidity `σ = N c / (π R)` (dimensionless).  `None` on bad geometry.
    #[inline]
    pub fn solidity(&self) -> Option<f64> {
        if !finite_positive(self.radius) || self.n_blades == 0 {
            return None;
        }
        let nc = (self.n_blades as f64) * self.chord;
        if !finite_positive(nc) {
            return None;
        }
        Some(nc / (PI * self.radius))
    }

    /// True when every numeric field is finite and the geometry is positive.
    /// Used by the entry points before `set_error`.
    pub fn valid(&self) -> bool {
        finite_positive(self.radius)
            && self.n_blades > 0
            && finite_positive(self.chord)
            && finite_non_negative(self.hinge_offset)
            && finite_positive(self.lift_slope)
            && finite_non_negative(self.profile_cd0)
            && finite_non_negative(self.cd_k)
            && finite(self.zero_lift_alpha)
    }
}

/// Pitfall: `RotorParams` is intentionally NOT `#[repr(C)]`.  It lives in the
/// pure-formula crate and must never cross the C ABI; the C boundary is
/// owned by `mps-core`/`mps-cosmos` wrappers that read these fields by name.
/// Validate the scalar pre-conditions common to every momentum-theory entry
/// point: density positive, radius positive, finite non-negative thrust.
/// Returns `false` on any bad input; the *caller* is responsible for
/// `set_error`.  Kept here so the `rotor_*` momentum fns share one verdict.
#[inline]
pub(crate) fn momentum_inputs_ok(thrust: f64, rho: f64, radius: f64) -> bool {
    finite_non_negative(thrust) && finite_positive(rho) && finite_positive(radius)
}

// ---------------------------------------------------------------------------
// Per-domain submodules.
// ---------------------------------------------------------------------------

pub mod blade_element;
pub mod momentum;
pub mod performance;
pub mod vortex;

pub use blade_element::*;
pub use momentum::*;
pub use performance::*;
pub use vortex::*;
