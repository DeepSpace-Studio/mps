//! `spaceflight::rotation` submodule — attitude determination & control (quaternion derivative, Euler, CMG, TRIAD, EKF, least-squares, gravity gradient torque, magnetic torquer, solar array PD torque)
//!
//! Split out of the original 2040-line `spaceflight.rs` per OPTIMIZATION.md §N8.
//! See [`super`] for the shared helpers (`finite`, `clamp_unit`,
//! `stumpff_functions`) and numeric constants (`EPS`, `SIGMA`,
//! `SPEED_OF_LIGHT`, `PI`, `TAU`).
//!
//! All public functions keep their `pub fn` names and signatures
//! unchanged; the crate-level `pub use` in `super::mod` keeps the
//! downstream `mps-core::rapier::spaceflight::*` path stable.

use super::*;

pub fn cmg_exchange(
    gimbal_axis: Vec3,
    wheel_momentum: Vec3,
    gimbal_rate: f64,
) -> Option<CmgExchange> {
    if !vec3_finite(gimbal_axis) || !vec3_finite(wheel_momentum) || !gimbal_rate.is_finite() {
        return None;
    }
    let axis = vec3_to_rapier(gimbal_axis).try_normalize()?;
    let h_dot = (axis * gimbal_rate).cross(vec3_to_rapier(wheel_momentum));
    Some(CmgExchange {
        body_torque: vec3_from_rapier(-h_dot),
        wheel_momentum_dot: vec3_from_rapier(h_dot),
    })
}

pub fn cmg_robust_pseudoinverse_diag(
    jacobian_diag: Vec3,
    desired_torque: Vec3,
    damping: f64,
) -> Option<CmgRobustInverse> {
    if !vec3_finite(jacobian_diag)
        || !vec3_finite(desired_torque)
        || !damping.is_finite()
        || damping < 0.0
    {
        return None;
    }
    let solve = |j: f64, t: f64| j * t / (j * j + damping * damping);
    Some(CmgRobustInverse {
        gimbal_rates: Vec3 {
            x: solve(jacobian_diag.x, desired_torque.x),
            y: solve(jacobian_diag.y, desired_torque.y),
            z: solve(jacobian_diag.z, desired_torque.z),
        },
        damping,
    })
}

pub fn ekf_gain_scalar(
    covariance: f64,
    measurement_jacobian: f64,
    measurement_noise: f64,
) -> Option<f64> {
    if !finite(&[covariance, measurement_jacobian, measurement_noise])
        || covariance < 0.0
        || measurement_noise < 0.0
    {
        return None;
    }
    let innovation_covariance =
        measurement_jacobian * covariance * measurement_jacobian + measurement_noise;
    if innovation_covariance <= EPS {
        return None;
    }
    Some(covariance * measurement_jacobian / innovation_covariance)
}

pub fn ekf_predict_scalar(
    state: f64,
    covariance: f64,
    nonlinear_delta: f64,
    jacobian: f64,
    process_noise: f64,
) -> Option<ScalarKalman> {
    if !finite(&[state, covariance, nonlinear_delta, jacobian, process_noise])
        || covariance < 0.0
        || process_noise < 0.0
    {
        return None;
    }
    Some(ScalarKalman {
        value: state + nonlinear_delta,
        covariance: jacobian * covariance * jacobian + process_noise,
    })
}

pub fn ekf_update_scalar(
    predicted_state: f64,
    predicted_covariance: f64,
    measurement: f64,
    predicted_measurement: f64,
    kalman_gain: f64,
    measurement_jacobian: f64,
) -> Option<ScalarKalman> {
    if !finite(&[
        predicted_state,
        predicted_covariance,
        measurement,
        predicted_measurement,
        kalman_gain,
        measurement_jacobian,
    ]) || predicted_covariance < 0.0
    {
        return None;
    }
    Some(ScalarKalman {
        value: predicted_state + kalman_gain * (measurement - predicted_measurement),
        covariance: (1.0 - kalman_gain * measurement_jacobian) * predicted_covariance,
    })
}

pub fn gravity_gradient_torque(position: Vec3, inertia_diag: Vec3, mu: f64) -> Option<Vec3> {
    if !vec3_finite(position) || !vec3_finite(inertia_diag) || !mu.is_finite() || mu <= 0.0 {
        return None;
    }
    let r = vec3_to_rapier(position);
    let rn = r.length();
    if rn <= EPS {
        return None;
    }
    let n = r / rn;
    let in_vec = rapier3d::prelude::Vector::new(
        inertia_diag.x * n.x,
        inertia_diag.y * n.y,
        inertia_diag.z * n.z,
    );
    Some(vec3_from_rapier(
        n.cross(in_vec) * (3.0 * mu / (rn * rn.sqrt())),
    ))
}

pub fn least_squares_attitude_two_vector(
    body_primary: Vec3,
    body_secondary: Vec3,
    reference_primary: Vec3,
    reference_secondary: Vec3,
) -> Option<LeastSquaresAttitude> {
    let quat = triad_attitude(
        body_primary,
        body_secondary,
        reference_primary,
        reference_secondary,
    )?;
    Some(LeastSquaresAttitude {
        attitude: quat,
        rms_error: 0.0,
    })
}

pub fn magnetic_torquer_dipole(
    commanded_torque: Vec3,
    magnetic_field: Vec3,
    max_dipole: f64,
) -> Option<Vec3> {
    if !vec3_finite(commanded_torque)
        || !vec3_finite(magnetic_field)
        || !max_dipole.is_finite()
        || max_dipole < 0.0
    {
        return None;
    }
    let b = vec3_to_rapier(magnetic_field);
    let b2 = b.length_squared();
    if b2 <= EPS {
        return None;
    }
    let mut m = b.cross(vec3_to_rapier(commanded_torque)) / b2;
    let mn = m.length();
    if mn > max_dipole && mn > EPS {
        m *= max_dipole / mn;
    }
    Some(vec3_from_rapier(m))
}

pub fn quaternion_derivative(
    attitude: Quat,
    angular_velocity: Vec3,
) -> Option<QuaternionDerivative> {
    if !finite(&[attitude.i, attitude.j, attitude.k, attitude.w]) || !vec3_finite(angular_velocity)
    {
        return None;
    }
    let wx = angular_velocity.x;
    let wy = angular_velocity.y;
    let wz = angular_velocity.z;
    Some(QuaternionDerivative {
        i_dot: 0.5 * (attitude.w * wx + attitude.j * wz - attitude.k * wy),
        j_dot: 0.5 * (attitude.w * wy + attitude.k * wx - attitude.i * wz),
        k_dot: 0.5 * (attitude.w * wz + attitude.i * wy - attitude.j * wx),
        w_dot: -0.5 * (attitude.i * wx + attitude.j * wy + attitude.k * wz),
    })
}

pub fn rigid_body_euler_derivative(
    inertia_diag: Vec3,
    angular_velocity: Vec3,
    torque: Vec3,
) -> Option<RigidBodyEulerDerivative> {
    if !vec3_finite(inertia_diag)
        || !vec3_finite(angular_velocity)
        || !vec3_finite(torque)
        || inertia_diag.x <= 0.0
        || inertia_diag.y <= 0.0
        || inertia_diag.z <= 0.0
    {
        return None;
    }
    let omega = vec3_to_rapier(angular_velocity);
    let h = rapier3d::prelude::Vector::new(
        inertia_diag.x * omega.x,
        inertia_diag.y * omega.y,
        inertia_diag.z * omega.z,
    );
    Some(RigidBodyEulerDerivative {
        angular_acceleration: Vec3 {
            x: (torque.x - (omega.y * h.z - omega.z * h.y)) / inertia_diag.x,
            y: (torque.y - (omega.z * h.x - omega.x * h.z)) / inertia_diag.y,
            z: (torque.z - (omega.x * h.y - omega.y * h.x)) / inertia_diag.z,
        },
    })
}

pub fn solar_array_pd_torque(angle_error: f64, rate_error: f64, kp: f64, kd: f64) -> Option<f64> {
    if !finite(&[angle_error, rate_error, kp, kd]) {
        return None;
    }
    Some(kp * angle_error + kd * rate_error)
}

pub fn triad_attitude(
    body_primary: Vec3,
    body_secondary: Vec3,
    reference_primary: Vec3,
    reference_secondary: Vec3,
) -> Option<Quat> {
    let make_basis = |a: Vec3,
                      b: Vec3|
     -> Option<(
        rapier3d::prelude::Vector,
        rapier3d::prelude::Vector,
        rapier3d::prelude::Vector,
    )> {
        let t1 = vec3_to_rapier(a).try_normalize()?;
        let t2 = t1.cross(vec3_to_rapier(b)).try_normalize()?;
        let t3 = t1.cross(t2);
        Some((t1, t2, t3))
    };
    let (bt1, bt2, bt3) = make_basis(body_primary, body_secondary)?;
    let (rt1, rt2, rt3) = make_basis(reference_primary, reference_secondary)?;
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
    Some(q)
}
