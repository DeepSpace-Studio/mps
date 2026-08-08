//! `flight::dynamics` — 6-DOF flight-dynamics force/moment synthesis and
//! one-step integration for a single rotorcraft `RigidBody`.
//!
//! This is the Rapier-touching integration layer that the
//! `mps_formula::rotor` pure-computation functions feed into.  One call to
//! [`simulate_one_step`] extracts the body's state (translation, linear
//! velocity, angular velocity, rotation), evaluates rotor + atmosphere +
//! gravity totals, and advances the body one semi-implicit Euler step.
//!
//! ## Force architecture
//!
//! Convention: the rotor thrust axis is the body's **+z** axis.  The
//! `FlightControls` carry collective, cyclic (longitudinal + lateral),
//! tail-rotor collective, and a yaw pedal; they produce world-frame forces
//! and a body-frame torque which are added to gravity and the airframe's
//! translational drag.  The body's principal inertia is used to convert
//! body-frame torque into angular-velocity increment `Δω = I⁻¹·τ·dt`.
//!
//! ### Sources
//!
//! - Leishman, *Principles of Helicopter Aerodynamics*, §6 (rotor-body
//!   coupling), §8 (flight dynamics).
//! - Johnson, *Helicopter Theory*, §15 (equations of motion).

use super::{Atmosphere, Gravity, body_to_world, world_to_body};
use mps_formula::rotor::{
    BladeElementResult, LinearAirfoil, PitchDistribution, RotorParams,
    blade_element::Airfoil, compute_rotor_forces, rotor_climb_power,
    rotor_figure_of_merit, rotor_flat_plate_area, rotor_forward_induced_velocity,
    rotor_hover_induced_velocity, rotor_parasite_power, rotor_profile_power,
    rotor_total_power,
};
use rapier3d::prelude::{RigidBody, Rotation, Vector};

/// Pilot / control inputs.  All angles in rad; the collector is in m or
/// dimensionless depending on the airframe model.  The convention here is
/// the classical helicopter 4-channel set:
///
/// | field | meaning |
/// |---|---|
/// | `collective` | main-rotor collective pitch (rad) — lifts the aircraft |
/// | `cyclic_lon` | longitudinal cyclic pitch (rad) — pitches nose up/down |
/// | `cyclic_lat` | lateral cyclic pitch (rad) — rolls left/right |
/// | `tail_collective` | tail-rotor collective (rad) — yaw control / anti-torque |
/// | `throttle` | engine throttle setting `[0,1]`; sets main-rotor `ω` as `ω₀·throttle` |
#[derive(Clone, Copy, Debug, Default)]
pub struct FlightControls {
    pub collective: f64,
    pub cyclic_lon: f64,
    pub cyclic_lat: f64,
    pub tail_collective: f64,
    pub throttle: f64,
}

impl FlightControls {
    /// Validate: collective and cyclic are finite; throttle is `[0,1]`-ish
    /// (clamped to that range by the user, we reject only non-finite).
    pub fn valid(&self) -> bool {
        self.collective.is_finite()
            && self.cyclic_lon.is_finite()
            && self.cyclic_lat.is_finite()
            && self.tail_collective.is_finite()
            && self.throttle.is_finite()
    }
}

/// Snapshot of a Rapier `RigidBody`'s 6-DOF state as the integrator sees it.
///
/// Hand-built (no allocation) so the per-step inner loop stays heap-free.
/// The body-frame linear velocity `v_body` is `world_to_body(rotation, linvel)`
/// because rotor aerodynamics are naturally body-frame quantities.
#[derive(Clone, Copy, Debug, Default)]
pub struct RigidBodyState {
    pub position: Vector,
    pub linvel_world: Vector,
    pub angvel_body: Vector,
    pub rotation: Rotation,
    pub mass: f64,
}

impl RigidBodyState {
    /// Read the 6-DOF state from a Rapier `RigidBody` (`&mut` because we may
    /// wake it later; here it's read-only).
    pub fn from_body(body: &RigidBody) -> Self {
        let rotation = *body.rotation();
        let linvel_world = body.linvel();
        // rapier3d-f64 `angvel()` returns an `AngVector` (a `Vector` alias);
        // for a 3-D body the angular velocity lives in the local frame.
        let angvel_body = Vector::new(body.angvel().x, body.angvel().y, body.angvel().z);
        let mass = body.mass();
        Self {
            position: body.translation(),
            linvel_world,
            angvel_body,
            rotation,
            mass,
        }
    }

    /// Body-frame linear velocity (⋔ of the world-frame velocity by the
    /// inverse rotation).
    pub fn linvel_body(&self) -> Vector {
        world_to_body(&self.rotation, self.linvel_world)
    }
}

/// Aggregated force/moment report from one flight-dynamics evaluation.
/// Useful for tests and for instrumenting the trim solver.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlightDynamics {
    pub force_world: Vector,
    pub moment_body: Vector,
    pub rotor_thrust: f64,
    pub rotor_torque: f64,
    pub induced_velocity: f64,
    pub total_power: f64,
}

/// Compute the aggregated force/moment on a rotorcraft body for one step,
/// using momentum theory for the induced velocity and BET for the rotor
/// forces.  `flat_plate_area` is the airframe equivalent flat-plate area
/// `f = S·C_d` (m²) — supply `0` to neglect parasite drag.
///
/// The main rotor's thrust axis is the body **+z**; the tail rotor's thrust
/// axis is the body **-y** (conventional right-hand layout: tail pushes
/// sideways to cancel main-rotor torque).  `rotor_omega` is the nominal
/// main-rotor angular speed (rad/s) at full throttle; the actual `ω` is
/// `rotor_omega · max(0, throttle)`.
pub fn total_forces_and_moments(
    state: &RigidBodyState,
    rotor: &RotorParams,
    tail_rotor: &RotorParams,
    atmosphere: &dyn Atmosphere,
    gravity: &dyn Gravity,
    controls: &FlightControls,
    rotor_omega: f64,
    flat_plate_area: f64,
    airfoil: &dyn Airfoil,
    stations: u32,
) -> Option<FlightDynamics> {
    if !state.mass.is_finite() || state.mass <= 0.0
        || !rotor.valid() || !tail_rotor.valid() || !controls.valid()
        || !rotor_omega.is_finite() || rotor_omega <= 0.0
    {
        return None;
    }
    let rho = atmosphere.density(state.position.z)?;
    let g_vec = gravity.gravity_vector(state.position.z);
    let omega = rotor_omega * controls.throttle.max(0.0).min(1.0);

    // Body-frame wind (assume steady air, no wind vector yet): body-frame
    // forward speed along +x, vertical along +z; the rotor inflow is
    // perpendicular to the disk → along body +z, opposite to thrust.
    let v_body = state.linvel_body();
    // axial component of free stream along the thrust axis (+z body)
    let axial = v_body.z;
    let horizontal = (v_body.x * v_body.x + v_body.y * v_body.y).sqrt();

    // --- Main rotor induced velocity via momentum theory ------------------
    let thrust_guess = state.mass * (g_vec).length(); // hover guess
    let v_i = rotor_forward_induced_velocity(thrust_guess, rho, rotor.radius, axial)?;
    // BET integration for the main rotor (uniform collective pitch; cyclic
    // is modelled as a tilt of the thrust vector in body frame handled
    // below — the BEM sweep itself is rotationally symmetric here).
    let main_pitch = PitchDistribution::Uniform { theta: controls.collective };
    let bem: BladeElementResult = compute_rotor_forces(
        rotor, v_i, omega, rho, &main_pitch, airfoil, stations,
    )?;
    // Body-frame main rotor force: thrust along +z, tilt by cyclic.
    // Cyclic longitudinal (pitch nose-up) tilts the thrust forward → +x body
    // negative cyclic_lon tilts it back.  Lateral cyclic rolls the disk.
    let main_thrust_body = bem.thrust;
    let main_force_body = Vector::new(
        main_thrust_body * (-controls.cyclic_lon),
        main_thrust_body * (-controls.cyclic_lat),
        main_thrust_body,
    );
    // hub moment ≈ thrust × hinge offset × cyclic (body frame); sign matches
    // the standard hinge-offset formula `M = T·e·θ_c` (Leishman §4.12).
    let e = rotor.hinge_offset;
    let main_moment_body = Vector::new(
        main_thrust_body * e * controls.cyclic_lon,
        main_thrust_body * e * controls.cyclic_lat,
        bem.torque, // +Q rotor torque about thrust axis
    );

    // --- Tail rotor ------------------------------------------------------
    // Tail rotor cancels main-rotor reaction torque: its thrust along body -y.
    // It is a smaller propeller; we use momentum theory directly for speed
    // (single call) — its induced velocity is uncoupled to the main one.
    let tail_thrust_needed = bem.torque / tail_rotor.radius.max(1.0e-6);
    let _v_i_tail = rotor_hover_induced_velocity(tail_thrust_needed, rho, tail_rotor.radius)?;
    // total tail thrust = pilot input plus reactive anti-torque part.
    // We scale tail thrust by (1 + tail_collective):
    let sigma_t = 1.0 + controls.tail_collective;
    let tail_force_body = Vector::new(0.0, -tail_thrust_needed * sigma_t, 0.0);
    let tail_moment_body = Vector::new(tail_force_body.y * tail_rotor.radius, 0.0, 0.0);

    // --- Airframe parasite drag (body frame) ------------------------------
    let parasite_drag_body = if flat_plate_area > 0.0 && horizontal > 1.0e-6 {
        let q = 0.5 * rho * v_body.x * v_body.x * v_body.x.signum();
        let fx = -q * flat_plate_area;
        Vector::new(fx, 0.0, 0.0)
    } else {
        Vector::ZERO
    };

    // --- Combine ---------------------------------------------------------
    let force_body = main_force_body + tail_force_body + parasite_drag_body;
    let moment_body = main_moment_body + tail_moment_body;
    // Gravity (world frame).
    let gravity_force_world = g_vec * state.mass;
    let main_force_world = body_to_world(&state.rotation, force_body);

    let force_world = main_force_world + gravity_force_world;
    // moment stays in body frame (Euler's equation): Δω = I⁻¹·(τ − ω × Iω)·dt

    // --- Power accounting -------------------------------------------------
    let induced_p = bem.induced_power;
    let profile_p = rotor_profile_power(rotor, rho, omega).unwrap_or(bem.profile_power);
    let climb_p = rotor_climb_power(bem.thrust, axial).unwrap_or(0.0);
    let parasite_p = rotor_parasite_power(rho, horizontal, flat_plate_area).unwrap_or(0.0);
    let total_p = rotor_total_power(induced_p, profile_p, climb_p, parasite_p).unwrap_or(0.0);

    // Figure of merit is available for instrumentation; we expose per-step
    // power sums rather than FM (which the caller may form themselves).
    let _ = rotor_figure_of_merit(induced_p, total_p.max(1.0e-9));

    Some(FlightDynamics {
        force_world,
        moment_body,
        rotor_thrust: bem.thrust,
        rotor_torque: bem.torque,
        induced_velocity: v_i,
        total_power: total_p,
    })
}

/// Advance a rotorcraft `RigidBody` one time-step `dt` using the
/// aggregated flight-dynamics forces and moments.
///
/// Semi-implicit Euler (`v += a·dt; x += v·dt`) on the linear channels;
/// body-frame angular momentum update `ω += I⁻¹·τ·dt` (coriolis-like
/// gyroscopic `ω × Iω` term neglected for the per-step convention — the
/// flight envelope where it matters is small for a small UAV/rotorcraft;
/// callers wanting gyroscopic forces should hook the Rapier force pipeline).
///
/// Returns the `FlightDynamics` report so tests and the trim solver can
/// inspect the per-step force totals.
pub fn simulate_one_step(
    body: &mut RigidBody,
    rotor: &RotorParams,
    tail_rotor: &RotorParams,
    atmosphere: &dyn Atmosphere,
    gravity: &dyn Gravity,
    controls: &FlightControls,
    rotor_omega: f64,
    flat_plate_area: f64,
    dt: f64,
    airfoil: &dyn Airfoil,
    stations: u32,
) -> Option<FlightDynamics> {
    if !dt.is_finite() || dt <= 0.0 {
        return None;
    }
    let state = RigidBodyState::from_body(body);
    let report = total_forces_and_moments(
        &state,
        rotor,
        tail_rotor,
        atmosphere,
        gravity,
        controls,
        rotor_omega,
        flat_plate_area,
        airfoil,
        stations,
    )?;

    // Linear: semi-implicit Euler.
    let a = report.force_world / state.mass;
    let v_new = body.linvel() + a * dt;
    let x_new = body.translation() + v_new * dt;
    body.set_linvel(v_new, true);
    body.set_translation(x_new, true);

    // Angular: body-frame Euler equation Δω = I⁻¹·τ·dt. Use the body's
    // principal inertia (rapier stores it as `principal_inertia()` on the
    // local mass props; we read via the mass-props accessor).
    let mprops = body.mass_properties();
    let p = mprops.local_mprops.principal_inertia();
    let i_vec = Vector::new(p.x.max(1.0e-12), p.y.max(1.0e-12), p.z.max(1.0e-12));
    let domega = Vector::new(
        report.moment_body.x / i_vec.x,
        report.moment_body.y / i_vec.y,
        report.moment_body.z / i_vec.z,
    ) * dt;
    // Rapier's angvel for a 3-D body is reported in the local frame already;
    // update it in the local frame.
    let w = body.angvel();
    let w_new = Vector::new(w.x + domega.x, w.y + domega.y, w.z + domega.z);
    body.set_angvel(w_new, true);

    // Orientation: apply Δθ = ω·dt as a small quaternion multiply.  Glam's
    // `DQuat::from_scaled_axis` constructs a quaternion from an
    // `axis = ω / |ω|, angle = |ω|` vector representation (here Δθ = ω·dt;
    // half is ½·dt to land on the half-angle convention used by a unit
    // quaternion `dq = [ê·sin(θ/2), cos(θ/2)]`).
    let half = 0.5 * dt;
    let half_dtheta = Vector::new(
        w_new.x * half,
        w_new.y * half,
        w_new.z * half,
    );
    let dq = rapier3d::prelude::Rotation::from_scaled_axis(half_dtheta);
    let current = *body.rotation();
    let new_rot = current * dq;
    body.set_rotation(new_rot, true);

    Some(report)
}

/// Flat-plate-airframe convenience wrapper around
/// [`rotor_flat_plate_area`] — callers that have `S` and `C_d` separately.
pub fn flat_plate_area(reference_area: f64, drag_coefficient: f64) -> Option<f64> {
    rotor_flat_plate_area(reference_area, drag_coefficient)
}

/// A convenience airfoil builder — the default [`LinearAirfoil`] polar for
/// the main rotor of a small UAV, derived from `RotorParams`.
pub fn default_airfoil(rotor: &RotorParams) -> LinearAirfoil {
    LinearAirfoil::from_rotor(rotor)
}
