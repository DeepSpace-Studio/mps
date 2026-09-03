//! `spaceflight::dynamics` submodule — relative motion & guidance (Clohessy-Wiltshire, dh-transform, manipulator, flexible, slosh, docking, mass props, bang-off-bang, variational, contact force, structural freq, artificial potential)
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

pub fn arm_first_joint_inverse(wrist_x: f64, wrist_y: f64) -> Option<f64> {
    if !finite(&[wrist_x, wrist_y]) || (wrist_x.abs() <= EPS && wrist_y.abs() <= EPS) {
        return None;
    }
    Some(wrist_y.atan2(wrist_x))
}

pub fn arm_third_joint_angle(
    planar_radius: f64,
    vertical_offset: f64,
    link2: f64,
    link3: f64,
    elbow_up: bool,
) -> Option<f64> {
    if !finite(&[planar_radius, vertical_offset, link2, link3]) || link2 <= 0.0 || link3 <= 0.0 {
        return None;
    }
    let c3 = (planar_radius * planar_radius + vertical_offset * vertical_offset
        - link2 * link2
        - link3 * link3)
        / (2.0 * link2 * link3);
    if !(-1.0..=1.0).contains(&c3) {
        return None;
    }
    let s3 = (1.0 - c3 * c3).sqrt() * if elbow_up { 1.0 } else { -1.0 };
    Some(s3.atan2(c3))
}

pub fn artificial_potential_guidance(
    position: Vec3,
    target: Vec3,
    obstacle: Vec3,
    attractive_gain: f64,
    repulsive_gain: f64,
    influence_radius: f64,
) -> Option<Vec3> {
    if !vec3_finite(position)
        || !vec3_finite(target)
        || !vec3_finite(obstacle)
        || !finite(&[attractive_gain, repulsive_gain, influence_radius])
        || influence_radius <= 0.0
    {
        return None;
    }
    let p = vec3_to_rapier(position);
    let attractive = (vec3_to_rapier(target) - p) * attractive_gain;
    let away = p - vec3_to_rapier(obstacle);
    let d = away.length();
    let repulsive = if d > EPS && d < influence_radius {
        away / d * repulsive_gain * (1.0 / d - 1.0 / influence_radius) / (d * d)
    } else {
        nalgebra::Vector3::<f64>::zeros()
    };
    Some(vec3_from_rapier(attractive + repulsive))
}

pub fn bang_off_bang_profile(
    angle: f64,
    max_acceleration: f64,
    max_rate: f64,
) -> Option<BangOffBangProfile> {
    if !finite(&[angle, max_acceleration, max_rate]) || max_acceleration <= 0.0 || max_rate <= 0.0 {
        return None;
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
    Some(BangOffBangProfile {
        coast_time: coast,
        total_time: total,
        switch_angle,
    })
}

pub fn contact_force_hunt_crossley(
    penetration: f64,
    penetration_rate: f64,
    stiffness: f64,
    damping: f64,
    exponent: f64,
) -> Option<ContactForceModel> {
    if !finite(&[penetration, penetration_rate, stiffness, damping, exponent])
        || stiffness < 0.0
        || damping < 0.0
        || exponent <= 0.0
    {
        return None;
    }
    let depth = penetration.max(0.0);
    let normal = stiffness * depth.powf(exponent);
    let damping_force = damping * depth.powf(exponent) * penetration_rate.max(0.0);
    Some(ContactForceModel {
        normal_force: normal,
        damping_force,
        total_force: normal + damping_force,
    })
}

pub fn cw_derivative(state: CwState, mean_motion: f64) -> Option<CwDerivative> {
    if !vec3_finite(state.position) || !vec3_finite(state.velocity) || !mean_motion.is_finite() {
        return None;
    }
    let n = mean_motion;
    let r = state.position;
    let v = state.velocity;
    Some(CwDerivative {
        velocity: v,
        acceleration: Vec3 {
            x: 3.0 * n * n * r.x + 2.0 * n * v.y,
            y: -2.0 * n * v.x,
            z: -n * n * r.z,
        },
    })
}

pub fn dh_transform(theta: f64, d: f64, a: f64, alpha: f64) -> Option<DhTransform> {
    if !finite(&[theta, d, a, alpha]) {
        return None;
    }
    let (st, ct) = theta.sin_cos();
    let (sa, ca) = alpha.sin_cos();
    Some(DhTransform {
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
    })
}

pub fn docking_buffer_energy(
    relative_speed: f64,
    reduced_mass: f64,
    stroke: f64,
    efficiency: f64,
) -> Option<f64> {
    if !finite(&[relative_speed, reduced_mass, stroke, efficiency])
        || reduced_mass < 0.0
        || stroke <= 0.0
        || efficiency <= 0.0
    {
        return None;
    }
    Some(0.5 * reduced_mass * relative_speed * relative_speed / efficiency)
}

pub fn docking_glideslope_command(
    range: f64,
    desired_slope: f64,
    closing_speed_limit: f64,
) -> Option<f64> {
    if !finite(&[range, desired_slope, closing_speed_limit]) || closing_speed_limit < 0.0 {
        return None;
    }
    Some((-desired_slope * range).clamp(-closing_speed_limit, closing_speed_limit))
}

pub fn flexible_mode_derivative(
    displacement: f64,
    velocity: f64,
    natural_frequency: f64,
    damping_ratio: f64,
    modal_force: f64,
    modal_mass: f64,
) -> Option<FlexibleModeDerivative> {
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
        return None;
    }
    Some(FlexibleModeDerivative {
        displacement_dot: velocity,
        velocity_dot: modal_force / modal_mass
            - 2.0 * damping_ratio * natural_frequency * velocity
            - natural_frequency * natural_frequency * displacement,
    })
}

pub fn manipulator_dynamics_diag(
    mass_matrix_diag: Vec3,
    joint_acceleration: Vec3,
    coriolis: Vec3,
    gravity: Vec3,
) -> Option<ManipulatorDynamics> {
    if !vec3_finite(mass_matrix_diag)
        || !vec3_finite(joint_acceleration)
        || !vec3_finite(coriolis)
        || !vec3_finite(gravity)
    {
        return None;
    }
    Some(ManipulatorDynamics {
        torque: Vec3 {
            x: mass_matrix_diag.x * joint_acceleration.x + coriolis.x + gravity.x,
            y: mass_matrix_diag.y * joint_acceleration.y + coriolis.y + gravity.y,
            z: mass_matrix_diag.z * joint_acceleration.z + coriolis.z + gravity.z,
        },
    })
}

pub fn mass_properties_two_body(
    mass1: f64,
    position1: Vec3,
    inertia1_diag: Vec3,
    mass2: f64,
    position2: Vec3,
    inertia2_diag: Vec3,
) -> Option<MassProperties> {
    if !finite(&[mass1, mass2])
        || mass1 < 0.0
        || mass2 < 0.0
        || mass1 + mass2 <= 0.0
        || !vec3_finite(position1)
        || !vec3_finite(position2)
        || !vec3_finite(inertia1_diag)
        || !vec3_finite(inertia2_diag)
    {
        return None;
    }
    let p1 = vec3_to_rapier(position1);
    let p2 = vec3_to_rapier(position2);
    let total = mass1 + mass2;
    let com = (p1 * mass1 + p2 * mass2) / total;
    let parallel = |m: f64, p: nalgebra::Vector3<f64>, i: Vec3| -> Vec3 {
        let d = p - com;
        Vec3 {
            x: i.x + m * (d.y * d.y + d.z * d.z),
            y: i.y + m * (d.x * d.x + d.z * d.z),
            z: i.z + m * (d.x * d.x + d.y * d.y),
        }
    };
    let i1 = parallel(mass1, p1, inertia1_diag);
    let i2 = parallel(mass2, p2, inertia2_diag);
    Some(MassProperties {
        center_of_mass: vec3_from_rapier(com),
        inertia_diag: Vec3 {
            x: i1.x + i2.x,
            y: i1.y + i2.y,
            z: i1.z + i2.z,
        },
    })
}

pub fn slosh_pendulum_derivative(
    angle: f64,
    angular_rate: f64,
    length: f64,
    damping: f64,
    lateral_acceleration: f64,
    gravity: f64,
) -> Option<SloshPendulumDerivative> {
    if !finite(&[
        angle,
        angular_rate,
        length,
        damping,
        lateral_acceleration,
        gravity,
    ]) || length <= 0.0
    {
        return None;
    }
    Some(SloshPendulumDerivative {
        angle_dot: angular_rate,
        angular_rate_dot: -(gravity / length) * angle.sin()
            - damping * angular_rate
            - lateral_acceleration / length,
    })
}

pub fn structural_natural_frequency(stiffness: f64, mass: f64, mode_factor: f64) -> Option<f64> {
    if !finite(&[stiffness, mass, mode_factor]) || stiffness <= 0.0 || mass <= 0.0 {
        return None;
    }
    Some(mode_factor * (stiffness / mass).sqrt() / TAU)
}

pub fn variational_two_body(position: Vec3, velocity: Vec3, mu: f64) -> Option<VariationalState> {
    if !vec3_finite(position) || !vec3_finite(velocity) || !mu.is_finite() || mu <= 0.0 {
        return None;
    }
    let r = vec3_to_rapier(position);
    let rn = r.length();
    if rn <= EPS {
        return None;
    }
    Some(VariationalState {
        position_dot: velocity,
        velocity_dot: vec3_from_rapier(-r * (mu / (rn * rn.sqrt()))),
    })
}
