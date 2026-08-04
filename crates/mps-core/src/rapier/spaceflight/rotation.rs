//! `spaceflight::rotation` submodule — attitude determination & control (quaternion derivative, Euler, CMG, TRIAD, EKF, least-squares, magnetic torquers, surface charging)
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_exchange` must be null or point to a valid, writable `CmgExchange`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_cmg_torque_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    gimbal_axis: Vec3,
    wheel_momentum: Vec3,
    gimbal_rate: f64,
    wake_up: Bool,
    out_exchange: *mut CmgExchange,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let mut exchange = CmgExchange::default();
        if space_cmg_exchange(gimbal_axis, wheel_momentum, gimbal_rate, &mut exchange)
            == Bool::FALSE
        {
            return Bool::FALSE;
        }
        body.add_torque(vec3_to_rapier(exchange.body_torque), wake_up.0 != 0);
        write_optional_out(out_exchange, exchange);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_exchange` must be null or point to a valid, writable `CmgExchange`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_cmg_torque_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    gimbal_axis: Vec3,
    wheel_momentum: Vec3,
    gimbal_rate: f64,
    wake_up: Bool,
    out_exchange: *mut CmgExchange,
) -> u8 {
    ffi_guard(0, || {
        space_apply_cmg_torque_to_body(
            world,
            body_handle,
            gimbal_axis,
            wheel_momentum,
            gimbal_rate,
            wake_up,
            out_exchange,
        )
        .0
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_dipole` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_magnetic_torquer_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    commanded_torque: Vec3,
    magnetic_field: Vec3,
    max_dipole: f64,
    wake_up: Bool,
    out_dipole: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let mut dipole = Vec3::default();
        if space_magnetic_torquer_dipole(commanded_torque, magnetic_field, max_dipole, &mut dipole)
            == Bool::FALSE
        {
            return Bool::FALSE;
        }
        let torque = cross(vec3_to_rapier(dipole), vec3_to_rapier(magnetic_field));
        body.add_torque(torque, wake_up.0 != 0);
        write_optional_out(out_dipole, dipole);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_dipole` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_magnetic_torquer_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    commanded_torque: Vec3,
    magnetic_field: Vec3,
    max_dipole: f64,
    wake_up: Bool,
    out_dipole: *mut Vec3,
) -> u8 {
    ffi_guard(0, || {
        space_apply_magnetic_torquer_to_body(
            world,
            body_handle,
            commanded_torque,
            magnetic_field,
            max_dipole,
            wake_up,
            out_dipole,
        )
        .0
    })
}

/// # Safety
/// `out_exchange` must be null or point to a valid, writable `CmgExchange`.
#[unsafe(no_mangle)]
pub extern "C" fn space_cmg_exchange(
    gimbal_axis: Vec3,
    wheel_momentum: Vec3,
    gimbal_rate: f64,
    out_exchange: *mut CmgExchange,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(gimbal_axis) || !vec3_finite(wheel_momentum) || !gimbal_rate.is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "invalid CMG parameters");
            return Bool::FALSE;
        }
        let Some(axis) = vec3_to_rapier(gimbal_axis).try_normalize() else {
            set_error(ERR_INVALID_ARGUMENT, "CMG gimbal axis is zero");
            return Bool::FALSE;
        };
        let h_dot = cross(axis * gimbal_rate, vec3_to_rapier(wheel_momentum));
        write_out(
            out_exchange,
            CmgExchange {
                body_torque: vec3_from_rapier(-h_dot),
                wheel_momentum_dot: vec3_from_rapier(h_dot),
            },
        )
    })
}

/// # Safety
/// `out_inverse` must be null or point to a valid, writable `CmgRobustInverse`.
#[unsafe(no_mangle)]
pub extern "C" fn space_cmg_robust_pseudoinverse_diag(
    jacobian_diag: Vec3,
    desired_torque: Vec3,
    damping: f64,
    out_inverse: *mut CmgRobustInverse,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(jacobian_diag)
            || !vec3_finite(desired_torque)
            || !damping.is_finite()
            || damping < 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid CMG robust inverse parameters",
            );
            return Bool::FALSE;
        }
        let solve = |j: f64, t: f64| j * t / (j * j + damping * damping);
        write_out(
            out_inverse,
            CmgRobustInverse {
                gimbal_rates: Vec3 {
                    x: solve(jacobian_diag.x, desired_torque.x),
                    y: solve(jacobian_diag.y, desired_torque.y),
                    z: solve(jacobian_diag.z, desired_torque.z),
                },
                damping,
            },
        )
    })
}

/// Computes the scalar Kalman gain.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_ekf_gain_scalar(
    covariance: f64,
    measurement_jacobian: f64,
    measurement_noise: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[covariance, measurement_jacobian, measurement_noise])
            || covariance < 0.0
            || measurement_noise < 0.0
        {
            return invalid_nan("invalid EKF gain parameters");
        }
        let innovation_covariance =
            measurement_jacobian * covariance * measurement_jacobian + measurement_noise;
        if innovation_covariance <= EPS {
            return invalid_nan("invalid EKF innovation covariance");
        }
        clear_error();
        covariance * measurement_jacobian / innovation_covariance
    })
}

/// # Safety
/// `out_prediction` must be null or point to a valid, writable `ScalarKalman`.
#[unsafe(no_mangle)]
pub extern "C" fn space_ekf_predict_scalar(
    state: f64,
    covariance: f64,
    nonlinear_delta: f64,
    jacobian: f64,
    process_noise: f64,
    out_prediction: *mut ScalarKalman,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[state, covariance, nonlinear_delta, jacobian, process_noise])
            || covariance < 0.0
            || process_noise < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid EKF prediction parameters");
            return Bool::FALSE;
        }
        write_out(
            out_prediction,
            ScalarKalman {
                value: state + nonlinear_delta,
                covariance: jacobian * covariance * jacobian + process_noise,
            },
        )
    })
}

/// # Safety
/// `out_update` must be null or point to a valid, writable `ScalarKalman`.
#[unsafe(no_mangle)]
pub extern "C" fn space_ekf_update_scalar(
    predicted_state: f64,
    predicted_covariance: f64,
    measurement: f64,
    predicted_measurement: f64,
    kalman_gain: f64,
    measurement_jacobian: f64,
    out_update: *mut ScalarKalman,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[
            predicted_state,
            predicted_covariance,
            measurement,
            predicted_measurement,
            kalman_gain,
            measurement_jacobian,
        ]) || predicted_covariance < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid EKF update parameters");
            return Bool::FALSE;
        }
        write_out(
            out_update,
            ScalarKalman {
                value: predicted_state + kalman_gain * (measurement - predicted_measurement),
                covariance: (1.0 - kalman_gain * measurement_jacobian) * predicted_covariance,
            },
        )
    })
}

/// # Safety
/// `out_attitude` must be null or point to a valid, writable `LeastSquaresAttitude`.
#[unsafe(no_mangle)]
pub extern "C" fn space_least_squares_attitude_two_vector(
    body_primary: Vec3,
    body_secondary: Vec3,
    reference_primary: Vec3,
    reference_secondary: Vec3,
    out_attitude: *mut LeastSquaresAttitude,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let mut quat = Quat::default();
        if space_triad_attitude(
            body_primary,
            body_secondary,
            reference_primary,
            reference_secondary,
            &mut quat,
        ) != Bool::TRUE
        {
            return Bool::FALSE;
        }
        write_out(
            out_attitude,
            LeastSquaresAttitude {
                attitude: quat,
                rms_error: 0.0,
            },
        )
    })
}

/// # Safety
/// `out_dipole` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_magnetic_torquer_dipole(
    commanded_torque: Vec3,
    magnetic_field: Vec3,
    max_dipole: f64,
    out_dipole: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(commanded_torque)
            || !vec3_finite(magnetic_field)
            || !max_dipole.is_finite()
            || max_dipole < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid magnetic torquer parameters");
            return Bool::FALSE;
        }
        let b = vec3_to_rapier(magnetic_field);
        let b2 = b.length_squared();
        if b2 <= EPS {
            set_error(ERR_INVALID_ARGUMENT, "magnetic field is zero");
            return Bool::FALSE;
        }
        let mut m = cross(b, vec3_to_rapier(commanded_torque)) / b2;
        let mn = m.length();
        if mn > max_dipole && mn > EPS {
            m *= max_dipole / mn;
        }
        write_out(out_dipole, vec3_from_rapier(m))
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `QuaternionDerivative`.
#[unsafe(no_mangle)]
pub extern "C" fn space_quaternion_derivative(
    attitude: Quat,
    angular_velocity: Vec3,
    out_derivative: *mut QuaternionDerivative,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[attitude.i, attitude.j, attitude.k, attitude.w])
            || !vec3_finite(angular_velocity)
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid quaternion kinematics parameters",
            );
            return Bool::FALSE;
        }
        let wx = angular_velocity.x;
        let wy = angular_velocity.y;
        let wz = angular_velocity.z;
        write_out(
            out_derivative,
            QuaternionDerivative {
                i_dot: 0.5 * (attitude.w * wx + attitude.j * wz - attitude.k * wy),
                j_dot: 0.5 * (attitude.w * wy + attitude.k * wx - attitude.i * wz),
                k_dot: 0.5 * (attitude.w * wz + attitude.i * wy - attitude.j * wx),
                w_dot: -0.5 * (attitude.i * wx + attitude.j * wy + attitude.k * wz),
            },
        )
    })
}

/// # Safety
/// `out_derivative` must be null or point to a valid, writable `RigidBodyEulerDerivative`.
#[unsafe(no_mangle)]
pub extern "C" fn space_rigid_body_euler_derivative(
    inertia_diag: Vec3,
    angular_velocity: Vec3,
    torque: Vec3,
    out_derivative: *mut RigidBodyEulerDerivative,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(inertia_diag)
            || !vec3_finite(angular_velocity)
            || !vec3_finite(torque)
            || inertia_diag.x <= 0.0
            || inertia_diag.y <= 0.0
            || inertia_diag.z <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid Euler rigid-body parameters");
            return Bool::FALSE;
        }
        let omega = vec3_to_rapier(angular_velocity);
        let h = Vector::new(
            inertia_diag.x * omega.x,
            inertia_diag.y * omega.y,
            inertia_diag.z * omega.z,
        );
        let accel = Vector::new(
            (torque.x - (omega.y * h.z - omega.z * h.y)) / inertia_diag.x,
            (torque.y - (omega.z * h.x - omega.x * h.z)) / inertia_diag.y,
            (torque.z - (omega.x * h.y - omega.y * h.x)) / inertia_diag.z,
        );
        write_out(
            out_derivative,
            RigidBodyEulerDerivative {
                angular_acceleration: vec3_from_rapier(accel),
            },
        )
    })
}

/// Computes the PD control torque for a solar array drive.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_solar_array_pd_torque(
    angle_error: f64,
    rate_error: f64,
    kp: f64,
    kd: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[angle_error, rate_error, kp, kd]) {
            return invalid_nan("invalid solar array PD parameters");
        }
        clear_error();
        kp * angle_error + kd * rate_error
    })
}

/// Computes the net spacecraft surface charging current balance.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_surface_charging_current_balance(
    photo_current: f64,
    secondary_current: f64,
    backscatter_current: f64,
    electron_current: f64,
    ion_current: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[
            photo_current,
            secondary_current,
            backscatter_current,
            electron_current,
            ion_current,
        ]) {
            return invalid_nan("invalid surface charging current parameters");
        }
        clear_error();
        photo_current + secondary_current + backscatter_current + ion_current - electron_current
    })
}

/// # Safety
/// `out_attitude` must be null or point to a valid, writable `Quat`.
#[unsafe(no_mangle)]
pub extern "C" fn space_triad_attitude(
    body_primary: Vec3,
    body_secondary: Vec3,
    reference_primary: Vec3,
    reference_secondary: Vec3,
    out_attitude: *mut Quat,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let make_basis = |a: Vec3, b: Vec3| -> Option<(Vector, Vector, Vector)> {
            let t1 = vec3_to_rapier(a).try_normalize()?;
            let t2 = cross(t1, vec3_to_rapier(b)).try_normalize()?;
            let t3 = cross(t1, t2);
            Some((t1, t2, t3))
        };
        let Some((bt1, bt2, bt3)) = make_basis(body_primary, body_secondary) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid TRIAD body vectors");
            return Bool::FALSE;
        };
        let Some((rt1, rt2, rt3)) = make_basis(reference_primary, reference_secondary) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid TRIAD reference vectors");
            return Bool::FALSE;
        };
        let m00 = bt1.x * rt1.x + bt2.x * rt2.x + bt3.x * rt3.x;
        let m01 = bt1.x * rt1.y + bt2.x * rt2.y + bt3.x * rt3.y;
        let m02 = bt1.x * rt1.z + bt2.x * rt2.z + bt3.x * rt3.z;
        let m10 = bt1.y * rt1.x + bt2.y * rt2.x + bt3.y * rt3.x;
        let m11 = bt1.y * rt1.y + bt2.y * rt2.y + bt3.y * rt3.y;
        let m12 = bt1.y * rt1.z + bt2.y * rt2.z + bt3.y * rt3.z;
        let m20 = bt1.z * rt1.x + bt2.z * rt2.x + bt3.z * rt3.x;
        let m21 = bt1.z * rt1.y + bt2.z * rt2.y + bt3.z * rt3.y;
        let m22 = bt1.z * rt1.z + bt2.z * rt2.z + bt3.z * rt3.z;
        let trace = m00 + m11 + m22;
        let q = if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Quat {
                w: 0.25 * s,
                i: (m21 - m12) / s,
                j: (m02 - m20) / s,
                k: (m10 - m01) / s,
            }
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Quat {
                w: (m21 - m12) / s,
                i: 0.25 * s,
                j: (m01 + m10) / s,
                k: (m02 + m20) / s,
            }
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Quat {
                w: (m02 - m20) / s,
                i: (m01 + m10) / s,
                j: 0.25 * s,
                k: (m12 + m21) / s,
            }
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Quat {
                w: (m10 - m01) / s,
                i: (m02 + m20) / s,
                j: (m12 + m21) / s,
                k: 0.25 * s,
            }
        };
        write_out(out_attitude, q)
    })
}
