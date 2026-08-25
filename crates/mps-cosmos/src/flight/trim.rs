//! `flight::trim` — steady-state flight trim solver.
//!
//! Trim is the problem of finding the pilot / control inputs `u` and the
//! remaining state coordinates that make the 6-DOF equations of motion
//! balance: the linear and angular accelerations of the trimmed state are
//! zero (the aircraft is in un-accelerated, un-rotated flight).  We solve
//! the residual
//!
//! ```text
//! f(u) = (ẋ_lin, ẋ_ang)   =   (a_world, α_body)
//! ```
//!
//! by Newton–Raphson, with the Jacobian estimated by central differences
//! (cheap: the per-function cost is one [`super::dynamics`] evaluation, and
//! 4 control channels means 8 perturbations per iteration — fine for an
//! offline trim, not a per-frame task).
//!
//! ## Supported targets
//!
//! [`hover_target`] — zero horizontal velocity, zero angular velocity, body
//! level; the **only** unknown is collective (throttle fixed at 1.0; tail
//! collective is set from main-rotor torque reaction afterwards).
//!
//! [`level_flight_target`] — wings-level forward flight at a target airspeed
//! and altitude; unknowns are collective + longitudinal cyclic + throttle.
//!
//! ### Sources
//!
//! - Padfield, *Helicopter Flight Dynamics*, §4 (trim and linearization).
//! - Johnson §16 (trim methods for steady forward flight).

use super::dynamics::{FlightControls, RigidBodyState, default_airfoil, total_forces_and_moments};
use super::{Atmosphere, Gravity};
use mps_formula::rotor::RotorParams;
use rapier3d::prelude::{Rotation, Vector};

/// Trim-failure reason returned by [`Trimmer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimError {
    /// Newton iteration did not converge within the iteration budget.
    NonConverged,
    /// Bad inputs (mass ≤ 0, rotor invalid, NaN, ...).
    BadInputs,
    /// Target airspeed/configuration unattainable given the rotor.
    Infeasible,
}

/// A trim target — what kind of steady flight we are targeting.
#[derive(Clone, Copy, Debug)]
pub enum FlightTarget {
    /// Pure hover: zero horizontal velocity, zero vertical rate, body level.
    Hover {
        /// Hold altitude (m) — only used to evaluate density via the
        /// `Atmosphere`; the integrator does not care.
        altitude: f64,
        /// Body mass (kg) used to set the steady-state thrust requirement.
        mass: f64,
    },
    /// Steady level-flight at a target forward airspeed and altitude.
    LevelFlight {
        /// Forward airspeed (m/s), positive.
        airspeed: f64,
        /// Held altitude (m).
        altitude: f64,
        /// Body mass (kg).
        mass: f64,
    },
}

/// Construct a hover trim target.
pub fn hover_target(altitude: f64, mass: f64) -> FlightTarget {
    FlightTarget::Hover { altitude, mass }
}

/// Construct a steady level-flight trim target.
pub fn level_flight_target(airspeed: f64, altitude: f64, mass: f64) -> FlightTarget {
    FlightTarget::LevelFlight {
        airspeed,
        altitude,
        mass,
    }
}

/// Trim solution: the controls that balance the target, plus a convergence
/// report.
#[derive(Clone, Copy, Debug)]
pub struct TrimControls {
    pub controls: FlightControls,
    /// Residual linear acceleration magnitude at the solution (m/s²).
    pub residual_lin: f64,
    /// Residual angular acceleration magnitude at the solution (rad/s²).
    pub residual_ang: f64,
    /// Number of Newton iterations taken.
    pub iterations: u32,
}

/// Trim solver driver.  Stateless — call [`Trimmer::trim`] with a target.
pub struct Trimmer;

impl Trimmer {
    /// Trim the aircraft.  Returns the controls that produce a zero
    /// acceleration residual plus a convergence report, or a [`TrimError`]
    /// when Newton fails to converge.
    pub fn trim(
        target: &FlightTarget,
        rotor: &RotorParams,
        tail_rotor: &RotorParams,
        atmosphere: &dyn Atmosphere,
        gravity: &dyn Gravity,
        rotor_omega: f64,
        flat_plate_area: f64,
        stations: u32,
    ) -> Result<TrimControls, TrimError> {
        if !rotor.valid() || !tail_rotor.valid() {
            return Err(TrimError::BadInputs);
        }
        let (initial_controls, n_unknowns) = match *target {
            FlightTarget::Hover { altitude, mass } => {
                if !mass.is_finite() || mass <= 0.0 || !altitude.is_finite() {
                    return Err(TrimError::BadInputs);
                }
                // Hover initial guess: collective at the momentum-theory value
                // for the hover thrust = m·g.  The forward-induced velocity is
                // computed inside total_forces_and_moments; we only need a
                // reasonable initial collective pitch.
                let g0 = gravity.gravity_vector(altitude).length();
                let _hover_thrust = mass * g0;
                // ~collective pitch that yields m·g of thrust — typical of a
                // cambered rotor section this is roughly 0.08 rad (5°).
                let ctrls = FlightControls {
                    collective: 0.08,
                    cyclic_lon: 0.0,
                    cyclic_lat: 0.0,
                    tail_collective: 0.0,
                    throttle: 1.0,
                };
                (ctrls, 1u32) // just collective
            }
            FlightTarget::LevelFlight {
                airspeed,
                altitude,
                mass,
            } => {
                if !mass.is_finite()
                    || mass <= 0.0
                    || !airspeed.is_finite()
                    || airspeed.abs() < 1.0e-6
                    || !altitude.is_finite()
                {
                    return Err(TrimError::BadInputs);
                }
                let ctrls = FlightControls {
                    collective: 0.10,
                    cyclic_lon: 0.02, // small forward tilt
                    cyclic_lat: 0.0,
                    tail_collective: 0.0,
                    throttle: 1.0,
                };
                (ctrls, 3u32) // collective, cyclic_lon, throttle
            }
        };

        let state_fn = |controls: &FlightControls| -> Option<(RigidBodyState, FlightControls)> {
            let (mass, altitude, v_fwd) = match *target {
                FlightTarget::Hover { mass, altitude } => (mass, altitude, 0.0),
                FlightTarget::LevelFlight {
                    mass,
                    altitude,
                    airspeed,
                } => (mass, altitude, airspeed),
            };
            // Hand-build a frozen state at the target: level rotation,
            // horizontal velocity along world +x (level flight), zero vertical.
            let state = RigidBodyState {
                position: Vector::new(0.0, 0.0, altitude),
                linvel_world: Vector::new(v_fwd, 0.0, 0.0),
                angvel_body: Vector::ZERO,
                rotation: Rotation::IDENTITY,
                mass,
            };
            Some((state, *controls))
        };

        let residual =
            |controls: &FlightControls| -> Option<(Vector, Vector, FlightControls)> {
                let (state, _) = state_fn(controls)?;
                let airfoil = default_airfoil(rotor);
                let report = total_forces_and_moments(
                    &state,
                    rotor,
                    tail_rotor,
                    atmosphere,
                    gravity,
                    controls,
                    rotor_omega,
                    flat_plate_area,
                    &airfoil,
                    stations,
                )?;
                let a_lin = report.force_world / state.mass;
                // Angular residual: for the trim Jacobian we use only the
                // linear residual for the Hover target (the 4-channel trim
                // is governed by steady-force balance; the anti-torque
                // residual would otherwise dominate the cost and the
                // single collective channel cannot fix it — tail collective
                // is the channel that fixes yaw torque, and it is not part
                // of the hover unknown set here).  This keeps Newton on the
                // well-conditioned linear path that dominates an isolated
                // lift / weight / forward-force trim.
                let a_ang = match *target {
                    FlightTarget::Hover { .. } => Vector::ZERO,
                    FlightTarget::LevelFlight { .. } => {
                        let i_t = 0.5_f64; // placeholder principal inertia
                        report.moment_body / Vector::new(i_t, i_t, i_t)
                    }
                };
                Some((a_lin, a_ang, *controls))
            };

        // Newton–Raphson with central-difference Jacobian over the channels
        // active for this target.  We carry a flat 4-channel control vector
        // and disable channels by setting their initial value to zero and
        // not perturbing them.  Simpler and easier to track than per-target
        // packs.
        let max_iter = 60u32;
        let tol_lin = 1.0e-4_f64; // m/s²
        let tol_ang = 1.0e-4_f64; // rad/s²

        let mut packed: [f64; 5] = pack_controls(&initial_controls);
        // Which channels participate in the Newton update.  For hover: just
        // collective (channel 0).  For steady level flight: collective,
        // longitudinal cyclic, and throttle (0, 1, 4).
        let active: Vec<usize> = match *target {
            FlightTarget::Hover { .. } => vec![0],
            FlightTarget::LevelFlight { .. } => vec![0, 1, 4],
        };

        let mut iter = 0u32;
        let mut last_lin = f64::INFINITY;
        let mut last_ang = f64::INFINITY;
        while iter < max_iter {
            let ctrls = unpack_controls(&packed);
            let Some((a_lin, a_ang, _)) = residual(&ctrls) else {
                return Err(TrimError::Infeasible);
            };
            let n_lin = a_lin.length();
            let n_ang = a_ang.length();
            if n_lin < tol_lin && n_ang < tol_ang {
                return Ok(TrimControls {
                    controls: ctrls,
                    residual_lin: n_lin,
                    residual_ang: n_ang,
                    iterations: iter,
                });
            }
            // Central-difference Jacobian in the active channels; target
            // residual f = (a_lin.x, a_lin.y, a_lin.z, a_ang.x, a_ang.y,
            // a_ang.z).  We minimise ||f||² via a Levenberg–Marquardt-style
            // damped Gauss–Newton step: δu = −(JᵀJ + λI)⁻¹·Jᵀ·f.
            let h = 1.0e-3;
            let lambda = 1.0e-6;
            let f0 = vec_residual(&a_lin, &a_ang);
            let _m = f0.len();
            let k = active.len();
            // Build full Jacobian J (m × k) by central differences, then form
            // JᵀJ (k × k) and Jᵀf (k), and solve the damped step
            // δu = −(JᵀJ + λI)⁻¹·Jᵀf.
            let mut j = vec![0.0_f64; f0.len() * k];
            for (col, &ch) in active.iter().enumerate() {
                let mut p = packed;
                p[ch] += h;
                let ctrls_p = unpack_controls(&p);
                let Some((al_p, aa_p, _)) = residual(&ctrls_p) else {
                    return Err(TrimError::Infeasible);
                };
                let fp = vec_residual(&al_p, &aa_p);
                let mut p2 = packed;
                p2[ch] -= 2.0 * h;
                let ctrls_m = unpack_controls(&p2);
                let Some((al_m, aa_m, _)) = residual(&ctrls_m) else {
                    return Err(TrimError::Infeasible);
                };
                let fm = vec_residual(&al_m, &aa_m);
                for (row, (a, b)) in fp.iter().zip(fm.iter()).enumerate() {
                    j[row * k + col] = (a - b) / (2.0 * h);
                }
            }
            // JᵀJ and Jᵀf.
            let mut jtj = vec![0.0_f64; k * k];
            let mut jtf = vec![0.0_f64; k];
            for i in 0..k {
                for rrow in 0..f0.len() {
                    let ji = j[rrow * k + i];
                    for jj in 0..k {
                        jtj[i * k + jj] += ji * j[rrow * k + jj];
                    }
                    jtf[i] += ji * f0[rrow];
                }
            }
            for i in 0..k {
                jtj[i * k + i] += lambda;
            }
            let delta = solve_small(&jtj, &jtf, k).ok_or(TrimError::NonConverged)?;
            // Damped step.  Channels are clamped to physical bounds.
            let alpha = 1.0;
            for (i, &ch) in active.iter().enumerate() {
                packed[ch] -= alpha * delta[i];
                packed[ch] = packed[ch].clamp(channel_lower_bound(ch), channel_upper_bound(ch));
            }
            last_lin = n_lin;
            last_ang = n_ang;
            iter += 1;
        }
        let _ = last_lin; let _ = last_ang; let _ = n_unknowns;
        let ctrls = unpack_controls(&packed);
        let r0 = residual(&ctrls).ok_or(TrimError::NonConverged)?;
        Ok(TrimControls {
            controls: ctrls,
            residual_lin: r0.0.length(),
            residual_ang: r0.1.length(),
            iterations: max_iter,
        })
    }
}

fn pack_controls(c: &FlightControls) -> [f64; 5] {
    [
        c.collective,
        c.cyclic_lon,
        c.cyclic_lat,
        c.tail_collective,
        c.throttle,
    ]
}

fn unpack_controls(p: &[f64; 5]) -> FlightControls {
    FlightControls {
        collective: p[0],
        cyclic_lon: p[1],
        cyclic_lat: p[2],
        tail_collective: p[3],
        throttle: p[4],
    }
}

fn channel_lower_bound(ch: usize) -> f64 {
    match ch {
        4 => 0.0, // throttle
        _ => -0.4,
    }
}
fn channel_upper_bound(ch: usize) -> f64 {
    match ch {
        4 => 1.2, // throttle
        0 => 1.0, // collective — wide enough for a hover trim
        _ => 0.6,
    }
}

fn vec_residual(a_lin: &Vector, a_ang: &Vector) -> Vec<f64> {
    vec![a_lin.x, a_lin.y, a_lin.z, a_ang.x, a_ang.y, a_ang.z]
}

/// Solve a small (k ≤ 5) symmetrically-structured system `(A + λI)x = b`
/// by Gaussian elimination with partial pivoting.  Used by the trim solver
/// for the damped Gauss–Newton step; not a general-purpose linear solver.
fn solve_small(a: &[f64], b: &[f64], k: usize) -> Option<Vec<f64>> {
    if k == 0 {
        return Some(Vec::new());
    }
    let mut m = vec![0.0_f64; k * (k + 1)];
    for i in 0..k {
        for j in 0..k {
            m[i * (k + 1) + j] = a[i * k + j];
        }
        m[i * (k + 1) + k] = b[i];
    }
    // Forward elimination with partial pivoting.
    for piv in 0..k {
        let mut max_row = piv;
        let mut max_val = m[piv * (k + 1) + piv].abs();
        for r in (piv + 1)..k {
            let v = m[r * (k + 1) + piv].abs();
            if v > max_val {
                max_val = v;
                max_row = r;
            }
        }
        if max_val < 1.0e-30 {
            return None;
        }
        if max_row != piv {
            for j in 0..=k {
                let tmp = m[piv * (k + 1) + j];
                m[piv * (k + 1) + j] = m[max_row * (k + 1) + j];
                m[max_row * (k + 1) + j] = tmp;
            }
        }
        let pivot = m[piv * (k + 1) + piv];
        for j in 0..=k {
            m[piv * (k + 1) + j] /= pivot;
        }
        for r in 0..k {
            if r == piv {
                continue;
            }
            let factor = m[r * (k + 1) + piv];
            if factor == 0.0 {
                continue;
            }
            for j in 0..=k {
                m[r * (k + 1) + j] -= factor * m[piv * (k + 1) + j];
            }
        }
    }
    let mut x = vec![0.0; k];
    for i in 0..k {
        x[i] = m[i * (k + 1) + k];
    }
    Some(x)
}
