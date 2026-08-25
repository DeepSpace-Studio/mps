//! `spaceflight::dynamics` submodule — relative motion & guidance (Clohessy-Wiltshire, dh-transform, manipulator, flexible, slosh, docking, mass props, bang-off-bang, variational)
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// Computes the first (base) joint angle of a planar arm from the wrist position.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_arm_first_joint_inverse(wrist_x: f64, wrist_y: f64) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[wrist_x, wrist_y]) || (wrist_x.abs() <= EPS && wrist_y.abs() <= EPS) {
            return invalid_nan("invalid first joint inverse parameters");
        }
        clear_error();
        wrist_y.atan2(wrist_x)
    })
}

/// Computes the third joint angle of a planar arm via the law of cosines.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_arm_third_joint_angle(
    planar_radius: f64,
    vertical_offset: f64,
    link2: f64,
    link3: f64,
    elbow_up: Bool,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[planar_radius, vertical_offset, link2, link3]) || link2 <= 0.0 || link3 <= 0.0
        {
            return invalid_nan("invalid third joint inverse parameters");
        }
        let c3 = (planar_radius * planar_radius + vertical_offset * vertical_offset
            - link2 * link2
            - link3 * link3)
            / (2.0 * link2 * link3);
        if !(-1.0..=1.0).contains(&c3) {
            return invalid_nan("third joint target is unreachable");
        }
        clear_error();
        let s3 = (1.0 - c3 * c3).sqrt() * if elbow_up.0 != 0 { 1.0 } else { -1.0 };
        s3.atan2(c3)
    })
}

/// # Safety
/// `out_command` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_artificial_potential_guidance(
    position: Vec3,
    target: Vec3,
    obstacle: Vec3,
    attractive_gain: f64,
    repulsive_gain: f64,
    influence_radius: f64,
    out_command: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(position)
            || !vec3_finite(target)
            || !vec3_finite(obstacle)
            || !finite(&[attractive_gain, repulsive_gain, influence_radius])
            || influence_radius <= 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid artificial potential guidance parameters",
            );
            return Bool::FALSE;
        }
        let p = vec3_to_rapier(position);
        let attractive = (vec3_to_rapier(target) - p) * attractive_gain;
        let away = p - vec3_to_rapier(obstacle);
        let d = away.length();
        let repulsive = if d > EPS && d < influence_radius {
            away / d * repulsive_gain * (1.0 / d - 1.0 / influence_radius) / (d * d)
        } else {
            Vector::ZERO
        };
        write_out(out_command, vec3_from_rapier(attractive + repulsive))
    })
}

/// # Safety
/// `out_profile` must be null or point to a valid, writable `BangOffBangProfile`.
#[unsafe(no_mangle)]
pub extern "C" fn space_bang_off_bang_profile(
    angle: f64,
    max_acceleration: f64,
    max_rate: f64,
    out_profile: *mut BangOffBangProfile,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[angle, max_acceleration, max_rate])
            || max_acceleration <= 0.0
            || max_rate <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid bang-off-bang parameters");
            return Bool::FALSE;
        }
        let theta = angle.abs();
        let triangular_angle = max_rate * max_rate / max_acceleration;
        let (coast, total, switch_angle) = if theta <= triangular_angle {
            let t = (theta / max_acceleration).sqrt();
            (0.0, 2.0 * t, 0.5 * theta)
        } else {
            let accel_time = max_rate / max_acceleration;
            let coast = (theta - triangular_angle) / max_rate;
            (coast, 2.0 * accel_time + coast, 0.5 * triangular_angle)
        };
        write_out(
            out_profile,
            BangOffBangProfile {
                coast_time: coast,
                total_time: total,
                switch_angle,
            },
        )
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `CwDerivative`.
#[unsafe(no_mangle)]
pub extern "C" fn space_cw_derivative(
    state: CwState,
    mean_motion: f64,
    out_derivative: *mut CwDerivative,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(state.position) || !vec3_finite(state.velocity) || !mean_motion.is_finite()
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid Clohessy-Wiltshire parameters",
            );
            return Bool::FALSE;
        }
        let n = mean_motion;
        let r = state.position;
        let v = state.velocity;
        write_out(
            out_derivative,
            CwDerivative {
                velocity: v,
                acceleration: Vec3 {
                    x: 3.0 * n * n * r.x + 2.0 * n * v.y,
                    y: -2.0 * n * v.x,
                    z: -n * n * r.z,
                },
            },
        )
    })
}

/// # Safety
/// `out_transform` must be null or point to a valid, writable `DhTransform`.
#[unsafe(no_mangle)]
pub extern "C" fn space_dh_transform(
    theta: f64,
    d: f64,
    a: f64,
    alpha: f64,
    out_transform: *mut DhTransform,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[theta, d, a, alpha]) {
            set_error(ERR_INVALID_ARGUMENT, "invalid D-H parameters");
            return Bool::FALSE;
        }
        let (st, ct) = theta.sin_cos();
        let (sa, ca) = alpha.sin_cos();
        write_out(
            out_transform,
            DhTransform {
                m00: ct,
                m01: -st * ca,
                m02: st * sa,
                m03: a * ct,
                m10: st,
                m11: ct * ca,
                m12: -ct * sa,
                m13: a * st,
                m20: 0.0,
                m21: sa,
                m22: ca,
                m23: d,
                m30: 0.0,
                m31: 0.0,
                m32: 0.0,
                m33: 1.0,
            },
        )
    })
}

/// Computes the kinetic energy a docking buffer must absorb, scaled by its efficiency.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_docking_buffer_energy(
    relative_speed: f64,
    reduced_mass: f64,
    stroke: f64,
    efficiency: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[relative_speed, reduced_mass, stroke, efficiency])
            || reduced_mass < 0.0
            || stroke <= 0.0
            || efficiency <= 0.0
        {
            return invalid_nan("invalid docking buffer parameters");
        }
        clear_error();
        0.5 * reduced_mass * relative_speed * relative_speed / efficiency
    })
}

/// Computes a clamped closing-speed command for a docking glideslope.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_docking_glideslope_command(
    range: f64,
    desired_slope: f64,
    closing_speed_limit: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[range, desired_slope, closing_speed_limit]) || closing_speed_limit < 0.0 {
            return invalid_nan("invalid docking glideslope parameters");
        }
        clear_error();
        (-desired_slope * range).clamp(-closing_speed_limit, closing_speed_limit)
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `FlexibleModeDerivative`.
#[unsafe(no_mangle)]
pub extern "C" fn space_flexible_mode_derivative(
    displacement: f64,
    velocity: f64,
    natural_frequency: f64,
    damping_ratio: f64,
    modal_force: f64,
    modal_mass: f64,
    out_derivative: *mut FlexibleModeDerivative,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[
            displacement,
            velocity,
            natural_frequency,
            damping_ratio,
            modal_force,
            modal_mass,
        ]) || natural_frequency < 0.0
            || damping_ratio < 0.0
            || modal_mass <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid flexible mode parameters");
            return Bool::FALSE;
        }
        write_out(
            out_derivative,
            FlexibleModeDerivative {
                displacement_dot: velocity,
                velocity_dot: modal_force / modal_mass
                    - 2.0 * damping_ratio * natural_frequency * velocity
                    - natural_frequency * natural_frequency * displacement,
            },
        )
    })
}

/// # Safety
/// `out_dynamics` must be null or point to a valid, writable `ManipulatorDynamics`.
#[unsafe(no_mangle)]
pub extern "C" fn space_manipulator_dynamics_diag(
    mass_matrix_diag: Vec3,
    joint_acceleration: Vec3,
    coriolis: Vec3,
    gravity: Vec3,
    out_dynamics: *mut ManipulatorDynamics,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(mass_matrix_diag)
            || !vec3_finite(joint_acceleration)
            || !vec3_finite(coriolis)
            || !vec3_finite(gravity)
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid manipulator dynamics parameters",
            );
            return Bool::FALSE;
        }
        write_out(
            out_dynamics,
            ManipulatorDynamics {
                torque: Vec3 {
                    x: mass_matrix_diag.x * joint_acceleration.x + coriolis.x + gravity.x,
                    y: mass_matrix_diag.y * joint_acceleration.y + coriolis.y + gravity.y,
                    z: mass_matrix_diag.z * joint_acceleration.z + coriolis.z + gravity.z,
                },
            },
        )
    })
}

/// # Safety
/// `out_properties` must be null or point to a valid, writable `MassProperties`.
#[unsafe(no_mangle)]
pub extern "C" fn space_mass_properties_two_body(
    mass1: f64,
    position1: Vec3,
    inertia1_diag: Vec3,
    mass2: f64,
    position2: Vec3,
    inertia2_diag: Vec3,
    out_properties: *mut MassProperties,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[mass1, mass2])
            || mass1 < 0.0
            || mass2 < 0.0
            || mass1 + mass2 <= 0.0
            || !vec3_finite(position1)
            || !vec3_finite(position2)
            || !vec3_finite(inertia1_diag)
            || !vec3_finite(inertia2_diag)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid mass properties parameters");
            return Bool::FALSE;
        }
        let p1 = vec3_to_rapier(position1);
        let p2 = vec3_to_rapier(position2);
        let total = mass1 + mass2;
        let com = (p1 * mass1 + p2 * mass2) / total;
        let parallel = |m: f64, p: Vector, i: Vec3| -> Vec3 {
            let d = p - com;
            Vec3 {
                x: i.x + m * (d.y * d.y + d.z * d.z),
                y: i.y + m * (d.x * d.x + d.z * d.z),
                z: i.z + m * (d.x * d.x + d.y * d.y),
            }
        };
        let i1 = parallel(mass1, p1, inertia1_diag);
        let i2 = parallel(mass2, p2, inertia2_diag);
        write_out(
            out_properties,
            MassProperties {
                center_of_mass: vec3_from_rapier(com),
                inertia_diag: Vec3 {
                    x: i1.x + i2.x,
                    y: i1.y + i2.y,
                    z: i1.z + i2.z,
                },
            },
        )
    })
}

/// Computes the absorbed radiation dose including a quality factor.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_radiation_absorbed_dose(
    energy_joules: f64,
    mass_kg: f64,
    quality_factor: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[energy_joules, mass_kg, quality_factor])
            || mass_kg <= 0.0
            || quality_factor < 0.0
        {
            return invalid_nan("invalid radiation dose parameters");
        }
        clear_error();
        energy_joules / mass_kg * quality_factor
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `SloshPendulumDerivative`.
#[unsafe(no_mangle)]
pub extern "C" fn space_slosh_pendulum_derivative(
    angle: f64,
    angular_rate: f64,
    length: f64,
    damping: f64,
    lateral_acceleration: f64,
    gravity: f64,
    out_derivative: *mut SloshPendulumDerivative,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[
            angle,
            angular_rate,
            length,
            damping,
            lateral_acceleration,
            gravity,
        ]) || length <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid slosh pendulum parameters");
            return Bool::FALSE;
        }
        write_out(
            out_derivative,
            SloshPendulumDerivative {
                angle_dot: angular_rate,
                angular_rate_dot: -(gravity / length) * angle.sin()
                    - damping * angular_rate
                    - lateral_acceleration / length,
            },
        )
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `VariationalState`.
#[unsafe(no_mangle)]
pub extern "C" fn space_variational_two_body(
    position: Vec3,
    velocity: Vec3,
    mu: f64,
    out_derivative: *mut VariationalState,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(position) || !vec3_finite(velocity) || !mu.is_finite() || mu <= 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid variational equation parameters",
            );
            return Bool::FALSE;
        }
        let r = vec3_to_rapier(position);
        let rn = r.length();
        if rn <= EPS {
            set_error(ERR_INVALID_ARGUMENT, "variational position is zero");
            return Bool::FALSE;
        }
        write_out(
            out_derivative,
            VariationalState {
                position_dot: velocity,
                // Compute mu/r³ as mu/(r² * |r|) to avoid powi(3) overflow
                velocity_dot: vec3_from_rapier(-r * (mu / (rn * rn.sqrt()))),
            },
        )
    })
}
