use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, RigidBodyHandleRaw, TrajectoryEnvironment, TrajectoryForceReport,
    TrajectoryGlideEnvironment, TrajectoryGlideReport, TrajectoryGlideState, TrajectoryState,
    WorldHandle, unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

/// Estimate the aerodynamic/gravity forces acting on a trajectory state.
///
/// # Safety
///
/// `out_report`, when non-null, must be valid for a single
/// `TrajectoryForceReport` write.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_estimate_forces(
    state: TrajectoryState,
    env: TrajectoryEnvironment,
    out_report: *mut TrajectoryForceReport,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(report) = mps_formula::trajectory::compute_forces(&state, &env) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid trajectory force parameters");
            return Bool::FALSE;
        };

        if let Some(out_report) = unsafe { out_report.as_mut() } {
            *out_report = report;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Advance a trajectory state by one integration step.
///
/// # Safety
///
/// `out_state` and `out_report`, when non-null, must each be valid for a
/// single write of `TrajectoryState` / `TrajectoryForceReport`.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_integrate_step(
    state: TrajectoryState,
    env: TrajectoryEnvironment,
    dt: f64,
    out_state: *mut TrajectoryState,
    out_report: *mut TrajectoryForceReport,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some((next_state, report)) = mps_formula::trajectory::integrate_step(&state, &env, dt)
        else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid trajectory integration parameters",
            );
            return Bool::FALSE;
        };

        if let Some(out_state) = unsafe { out_state.as_mut() } {
            *out_state = next_state;
        }
        if let Some(out_report) = unsafe { out_report.as_mut() } {
            *out_report = report;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Apply trajectory forces to a rigid body in the world.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_report`, when non-null,
/// must be valid for a single `TrajectoryForceReport` write.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_apply_forces_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    env: TrajectoryEnvironment,
    wake_up: Bool,
    out_report: *mut TrajectoryForceReport,
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

        let state = TrajectoryState {
            position: vec3_from_rapier(body.translation()),
            velocity: vec3_from_rapier(body.linvel()),
        };
        let Some(report) = mps_formula::trajectory::compute_forces(&state, &env) else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid trajectory body force parameters",
            );
            return Bool::FALSE;
        };

        body.add_force(vec3_to_rapier(report.total_force), wake_up.0 != 0);
        if let Some(out_report) = unsafe { out_report.as_mut() } {
            *out_report = report;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Flag-returning variant of `trajectory_apply_forces_to_body`.
///
/// # Safety
///
/// Same pointer contract as `trajectory_apply_forces_to_body`.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_apply_forces_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    env: TrajectoryEnvironment,
    wake_up: Bool,
    out_report: *mut TrajectoryForceReport,
) -> u8 {
    ffi_guard(0, || {
        trajectory_apply_forces_to_body(world, body_handle, env, wake_up, out_report).0
    })
}

/// Estimate the glide forces acting on a gliding trajectory state.
///
/// # Safety
///
/// `out_report`, when non-null, must be valid for a single
/// `TrajectoryGlideReport` write.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_glide_estimate(
    state: TrajectoryGlideState,
    env: TrajectoryGlideEnvironment,
    out_report: *mut TrajectoryGlideReport,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(report) = mps_formula::trajectory::compute_glide_report(&state, &env) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid trajectory glide parameters");
            return Bool::FALSE;
        };

        if let Some(out_report) = unsafe { out_report.as_mut() } {
            *out_report = report;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Advance a gliding trajectory state by one integration step.
///
/// # Safety
///
/// `out_state` and `out_report`, when non-null, must each be valid for a
/// single write of `TrajectoryGlideState` / `TrajectoryGlideReport`.
#[unsafe(no_mangle)]
pub extern "C" fn trajectory_glide_integrate_step(
    state: TrajectoryGlideState,
    env: TrajectoryGlideEnvironment,
    dt: f64,
    out_state: *mut TrajectoryGlideState,
    out_report: *mut TrajectoryGlideReport,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some((next_state, report)) =
            mps_formula::trajectory::integrate_glide_step(&state, &env, dt)
        else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid trajectory glide integration parameters",
            );
            return Bool::FALSE;
        };

        if let Some(out_state) = unsafe { out_state.as_mut() } {
            *out_state = next_state;
        }
        if let Some(out_report) = unsafe { out_report.as_mut() } {
            *out_report = report;
        }
        clear_error();
        Bool::TRUE
    })
}
