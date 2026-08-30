//! `rotor::blade_element` — blade-element integration of rotor thrust & torque.
//!
//! Blade-element theory (BET) slices each blade into radial stations `r..r+dr`
//! and treats every station as a 2-D airfoil in a local flow whose magnitude
//! is `√((Ωr)² + v_i²)`.  The lift and drag on each element are projected
//! onto the thrust and torque axes and summed (numerical quadrature by the
//! trapezoid rule over `N` stations).
//!
//! ## Local geometry
//!
//! `inflow angle φ = θ − arctan(v_i / (Ω r))` (the angle between the rotor
//! disk plane and the local relative wind).  Aerodynamic angle of attack
//! `α = θ − φ = arctan(v_i / (Ωr))` — i.e. the **pitch above inflow**.
//! Lift `dL = ½ ρ V_tot² c C_l(α) dr`, drag `dD = ½ ρ V_tot² c C_d(α) dr`.
//! Resolving onto the thrust / torque axes:
//!
//! `dT = dL cos φ − dD sin φ`,  `dQ = (dL sin φ + dD cos φ) · r`.
//!
//! The twist distribution `θ(r)` is supplied by the caller; a linearly
//! twisted blade is `θ(r) = θ_0 + (r/R)·(θ_1 − θ_0)`.
//!
//! ### Sources
//!
//! - Leishman §3 (blade-element theory + combined BET/momentum).
//! - Johnson §5-1..5-3.
//!
//! No wake model is bundled here — the inflow `v_i` is an input.  Combine
//! [`blade_element::compute_rotor_forces`] with the [`momentum`] entry
//! points (or [`vortex`]) for a self-consistent solution.

use super::*;
use crate::error::{ERR_INVALID_ARGUMENT, set_error};

/// A 2-D airfoil's lift and drag as a function of section angle of attack.
///
/// The trait is the seam along which an airfoil database (NACA tables, XFOIL
/// polar files, ...). is plugged into BET.  Pure-formula crate ships one
/// analytical implementation, [`LinearAirfoil`] (thin-airfoil `C_l = a₀·α`,
/// `C_d = C_d0 + k·C_l²` bounded by a stall angle), and downstream callers
/// may supply their own tables.
pub trait Airfoil {
    /// Lift coefficient at section angle of attack `alpha` (rad).
    fn cl(&self, alpha: f64) -> f64;
    /// Drag coefficient at section angle of attack `alpha` (rad).
    fn cd(&self, alpha: f64) -> f64;
}

/// Analytical thin-airfoil polar: `C_l = a₀·(α − α_zl)` (clipped above a
/// stall angle of attack), `C_d = C_d0 + k·C_l²`.
///
/// Construct with [`LinearAirfoil::from_rotor`] to reuse a [`RotorParams`]'s
/// `lift_slope`, `zero_lift_alpha`, `profile_cd0`, `cd_k`.  Stall is treated
/// as a hard clamp at `±stall_alpha` (post-stall lift held at the
/// stall-station value) — a coarse but well-defined first-order model.
#[derive(Clone, Copy, Debug)]
pub struct LinearAirfoil {
    pub lift_slope: f64,
    pub zero_lift_alpha: f64,
    pub cd0: f64,
    pub cd_k: f64,
    pub stall_alpha: f64,
}

impl LinearAirfoil {
    /// Build from a [`RotorParams`]; defaults the stall angle to 12° (`0.21`
    /// rad) — typical of a cambered rotor section.
    pub fn from_rotor(r: &RotorParams) -> Self {
        Self {
            lift_slope: r.lift_slope,
            zero_lift_alpha: r.zero_lift_alpha,
            cd0: r.profile_cd0,
            cd_k: r.cd_k,
            stall_alpha: 0.21,
        }
    }
}

impl Airfoil for LinearAirfoil {
    fn cl(&self, alpha: f64) -> f64 {
        let a = alpha - self.zero_lift_alpha;
        let a = clamp(a, -self.stall_alpha, self.stall_alpha);
        self.lift_slope * a
    }
    fn cd(&self, alpha: f64) -> f64 {
        let cl = self.cl(alpha);
        self.cd0 + self.cd_k * cl * cl
    }
}

/// Pitch-angle distribution `θ(r/R)` along the blade, sampled at every
/// quadrature station.  The caller is free to supply any twist law; the
/// [`PitchDistribution::uniform`] and [`PitchDistribution::linear_twist`]
/// constructors cover the two common cases.
#[derive(Clone, Debug)]
pub enum PitchDistribution {
    /// Constant collective pitch `θ` across the blade.
    Uniform { theta: f64 },
    /// Linear twist from `theta_root` at the hub to `theta_tip` at `r = R`.
    Linear { theta_root: f64, theta_tip: f64 },
    /// Caller-supplied per-station pitch (one entry per quadrature station,
    /// in the same order `BladeElementResult` iterates). Otherwise returned
    /// as `None` on length mismatch.
    Sampled(Vec<f64>),
}

impl PitchDistribution {
    /// Pitch at non-dimensional station `x = r/R ∈ [0, 1]`.  For `Sampled`
    /// the index is `round(x · (samples.len() - 1))`; out-of-range returns the
    /// endpoint.
    pub fn at(&self, x: f64) -> f64 {
        let x = clamp(x, 0.0, 1.0);
        match self {
            Self::Uniform { theta } => *theta,
            Self::Linear {
                theta_root,
                theta_tip,
            } => theta_root + x * (theta_tip - theta_root),
            Self::Sampled(v) => {
                let n = v.len();
                if n == 0 {
                    return 0.0;
                }
                let idx = (x * (n - 1) as f64).round() as usize;
                v[idx.min(n - 1)]
            }
        }
    }
}

/// BET integration result (per rotor, total over all blades).
#[derive(Clone, Copy, Debug, Default)]
pub struct BladeElementResult {
    /// Total thrust (N).
    pub thrust: f64,
    /// Total torque (N·m) — shaft torque needed to drive all blades.
    pub torque: f64,
    /// Induced power `P_i = T · v_i` (W), tracked so the caller can form a
    /// figure of merit without recomputing `v_i`.
    pub induced_power: f64,
    /// Profile power `P_0 = Q · Ω` (W), shaft power burnt in profile drag.
    pub profile_power: f64,
    /// Number of active (non-zero) quadrature stations — informative.
    pub stations: u32,
}

/// Trapezoid-rule BET integration of a single rotor's thrust and torque.
///
/// `inflow_velocity` is the uniform (momentum-theory) induced velocity `v_i`
/// perpendicular to the disk plane; `omega` is the rotor angular speed; the
/// quadrature uses `stations` evenly spaced sample points from the
/// root cut-out (`root_fraction · R`, default 0.15) to `R`.
///
/// Units: SI; returns `None` on bad geometry or non-finite airspeed.
pub fn compute_rotor_forces(
    rotor: &RotorParams,
    inflow_velocity: f64,
    omega: f64,
    rho: f64,
    pitch: &PitchDistribution,
    airfoil: &dyn Airfoil,
    stations: u32,
) -> Option<BladeElementResult> {
    if !rotor.valid()
        || !finite(inflow_velocity)
        || !finite_positive(omega)
        || !finite_positive(rho)
        || stations < 2
    {
        set_error(
            ERR_INVALID_ARGUMENT,
            "compute_rotor_forces: bad rotor / inflow / omega / rho / stations",
        );
        return None;
    }
    let r = rotor.radius;
    let root_frac = 0.15_f64;
    let r0 = root_frac * r;
    let dr = (r - r0) / (stations as f64 - 1.0);
    let n_b = rotor.n_blades as f64;

    let mut thrust = 0.0_f64;
    let mut torque = 0.0_f64;
    let mut active = 0u32;

    // Trapezoid rule: weight is ½ at the endpoints, 1 elsewhere.
    let mut x = r0;
    let mut i = 0u32;
    while i < stations {
        let omega_r = omega * x;
        // Avoid the r→0 singularity at the very hub (we start at root_frac
        // anyway, but guard omega_r very small).
        let v_tot = (omega_r * omega_r + inflow_velocity * inflow_velocity).sqrt();
        if v_tot < EPS {
            x += dr;
            i += 1;
            continue;
        }
        let phi = inflow_velocity.atan2(omega_r); // inflow angle
        let theta = pitch.at(if r > 0.0 { x / r } else { 0.0 });
        let alpha = theta - phi;
        let cl = airfoil.cl(alpha);
        let cd = airfoil.cd(alpha);
        let q_dyn = 0.5 * rho * v_tot * v_tot * rotor.chord * dr; // per blade per station
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        // per-blade station increments
        let d_t = q_dyn * (cl * cos_phi - cd * sin_phi);
        let d_q = q_dyn * (cl * sin_phi + cd * cos_phi) * x;
        let w = if i == 0 || i + 1 == stations {
            0.5
        } else {
            1.0
        };
        thrust += d_t * w * n_b;
        torque += d_q * w * n_b;
        if d_t.abs() > 0.0 || d_q.abs() > 0.0 {
            active += 1;
        }
        x += dr;
        i += 1;
    }

    Some(BladeElementResult {
        thrust,
        torque,
        induced_power: thrust * inflow_velocity.max(0.0),
        profile_power: torque * omega,
        stations: active,
    })
}
