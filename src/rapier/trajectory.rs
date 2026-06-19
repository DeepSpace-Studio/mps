use rapier3d::prelude::Vector;

use crate::rapier::error::{ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, set_error};
use crate::rapier::ffi::{
    Bool, RigidBodyHandleRaw, TrajectoryEnvironment, TrajectoryForceReport, TrajectoryState, Vec3,
    WorldHandle, unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

const MAX_STEP_SECONDS: f64 = 10.0;

fn environment_valid(env: TrajectoryEnvironment) -> bool {
    vec3_finite(env.gravity)
        && vec3_finite(env.flow_velocity)
        && vec3_finite(env.lift_direction)
        && env.mass.is_finite()
        && env.mass > 0.0
        && env.reference_area.is_finite()
        && env.reference_area >= 0.0
        && env.density.is_finite()
        && env.density >= 0.0
        && env.drag_coefficient.is_finite()
        && env.drag_coefficient >= 0.0
        && env.lift_coefficient.is_finite()
        && env.lift_coefficient >= 0.0
}

fn state_valid(state: TrajectoryState) -> bool {
    vec3_finite(state.position) && vec3_finite(state.velocity)
}

fn compute_forces(state: TrajectoryState, env: TrajectoryEnvironment) -> Option<TrajectoryForceReport> {
    if !state_valid(state) || !environment_valid(env) {
        return None;
    }

    let gravity_force = vec3_to_rapier(env.gravity) * env.mass;
    let relative_flow = vec3_to_rapier(env.flow_velocity) - vec3_to_rapier(state.velocity);
    let speed_squared = relative_flow.length_squared();
    let mut drag_force = Vector::ZERO;
    let mut lift_force = Vector::ZERO;

    if speed_squared > 1.0e-18 && env.reference_area > 0.0 && env.density > 0.0 {
        let speed = speed_squared.sqrt();
        let flow_dir = relative_flow / speed;
        let dynamic_pressure = 0.5 * env.density * speed_squared;
        drag_force = flow_dir * (dynamic_pressure * env.reference_area * env.drag_coefficient);

        if env.lift_coefficient > 0.0 {
            let lift_dir = vec3_to_rapier(env.lift_direction)
                .try_normalize()
                .unwrap_or(Vector::ZERO);
            if lift_dir.length_squared() > 0.0 {
                lift_force = lift_dir * (dynamic_pressure * env.reference_area * env.lift_coefficient);
            }
        }
    }

    let total_force = gravity_force + drag_force + lift_force;
    let acceleration = total_force / env.mass;

    Some(TrajectoryForceReport {
        gravity_force: vec3_from_rapier(gravity_force),
        drag_force: vec3_from_rapier(drag_force),
        lift_force: vec3_from_rapier(lift_force),
        total_force: vec3_from_rapier(total_force),
        acceleration: vec3_from_rapier(acceleration),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn trajectory_estimate_forces(
    state: TrajectoryState,
    env: TrajectoryEnvironment,
    out_report: *mut TrajectoryForceReport,
) -> Bool {
    let Some(report) = compute_forces(state, env) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid trajectory force parameters");
        return Bool::FALSE;
    };

    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn trajectory_integrate_step(
    state: TrajectoryState,
    env: TrajectoryEnvironment,
    dt: f64,
    out_state: *mut TrajectoryState,
    out_report: *mut TrajectoryForceReport,
) -> Bool {
    if !dt.is_finite() || dt <= 0.0 || dt > MAX_STEP_SECONDS {
        set_error(ERR_INVALID_ARGUMENT, "invalid trajectory timestep");
        return Bool::FALSE;
    }
    let Some(report) = compute_forces(state, env) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid trajectory integration parameters");
        return Bool::FALSE;
    };

    let acceleration = vec3_to_rapier(report.acceleration);
    let velocity = vec3_to_rapier(state.velocity) + acceleration * dt;
    let position = vec3_to_rapier(state.position) + velocity * dt;

    if let Some(out_state) = unsafe { out_state.as_mut() } {
        *out_state = TrajectoryState {
            position: vec3_from_rapier(position),
            velocity: vec3_from_rapier(velocity),
        };
    }
    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn trajectory_apply_forces_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    env: TrajectoryEnvironment,
    wake_up: Bool,
    out_report: *mut TrajectoryForceReport,
) -> Bool {
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

    let state = TrajectoryState {
        position: vec3_from_rapier(body.translation()),
        velocity: vec3_from_rapier(body.linvel()),
    };
    let Some(report) = compute_forces(state, env) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid trajectory body force parameters");
        return Bool::FALSE;
    };

    body.add_force(vec3_to_rapier(report.total_force), wake_up.0 != 0);
    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn trajectory_apply_forces_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    env: TrajectoryEnvironment,
    wake_up: Bool,
    out_report: *mut TrajectoryForceReport,
) -> u8 {
    trajectory_apply_forces_to_body(world, body_handle, env, wake_up, out_report).0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> TrajectoryEnvironment {
        TrajectoryEnvironment {
            gravity: Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
            flow_velocity: Vec3::default(),
            mass: 2.0,
            reference_area: 0.1,
            density: 1.225,
            drag_coefficient: 0.5,
            lift_coefficient: 0.0,
            lift_direction: Vec3::default(),
        }
    }

    #[test]
    fn estimates_gravity_and_drag() {
        let mut report = TrajectoryForceReport::default();
        assert_eq!(
            trajectory_estimate_forces(
                TrajectoryState {
                    position: Vec3::default(),
                    velocity: Vec3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                env(),
                &mut report,
            ),
            Bool::TRUE
        );
        assert!(report.gravity_force.y < 0.0);
        assert!(report.drag_force.x < 0.0);
    }

    #[test]
    fn integrates_state_forward() {
        let mut out = TrajectoryState::default();
        assert_eq!(
            trajectory_integrate_step(
                TrajectoryState {
                    position: Vec3::default(),
                    velocity: Vec3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                env(),
                0.1,
                &mut out,
                std::ptr::null_mut(),
            ),
            Bool::TRUE
        );
        assert!(out.position.x > 0.0);
        assert!(out.velocity.y < 0.0);
    }
}
