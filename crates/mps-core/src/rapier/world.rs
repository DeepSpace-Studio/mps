use rapier3d::prelude::{
    ActiveHooks, BroadPhaseBvh, CCDSolver, ColliderSet, ImpulseJointSet, IntegrationParameters,
    IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodySet, Vector,
};
use std::sync::Arc;

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard,
    set_error,
};
use crate::rapier::ffi::{
    Bool, MAX_OUTPUT_CAPACITY, Quat, RigidBodyHandleRaw, Vec3, WorldHandle,
    force_law_type_from_u32, isometry_from_parts, pack_rigid_body_handle, quat_finite,
    quat_from_rapier, unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};
use crate::rapier::forces::{BodyForceLog, ForceFacade, ForceRegistry};

const MAX_STEP_SECONDS: f64 = 1.0;

/// Preallocated working storage reused each frame to avoid per-step heap allocations.
pub(crate) struct FrameWorkBuffers {
    /// Per-body force log: indexed by handle index for O(1) access without hashing.
    /// Index = RigidBodyHandle::into_raw_parts().0 (arena index portion).
    /// Auto-expands when new bodies are inserted beyond current capacity.
    pub(crate) body_log: Vec<Option<BodyForceLog>>,
    /// Scratch buffer for Coulomb friction pairs (avoid per-frame Vec::new()).
    pub(crate) friction_work: Vec<(
        rapier3d::prelude::RigidBodyHandle,
        rapier3d::prelude::RigidBodyHandle,
        Vector,
    )>,
    /// Scratch buffer for legacy external force computation.
    pub(crate) pending_forces: smallvec::SmallVec<[crate::rapier::events::PendingForce; 128]>,
    /// Scratch buffer for arena command → handle mapping.
    pub(crate) arena_idx_map: Vec<Option<rapier3d::prelude::RigidBodyHandle>>,
}

impl Default for FrameWorkBuffers {
    fn default() -> Self {
        Self {
            body_log: Vec::with_capacity(256),
            friction_work: Vec::with_capacity(512),
            pending_forces: smallvec::SmallVec::new(),
            arena_idx_map: Vec::with_capacity(256),
        }
    }
}

pub struct PhysicsWorld {
    pub(crate) pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) islands: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) hooks: crate::rapier::events::CallbackPhysicsHooks,
    pub(crate) events: Arc<crate::rapier::events::CollectingEventHandler>,
    pub(crate) force_registry: ForceRegistry,
    pub(crate) shared_arena: Option<Box<crate::rapier::shared_arena::SharedPhysicsArena>>,
    /// Persistent per-frame work buffers — cleared and reused each `world_step`.
    pub(crate) buffers: FrameWorkBuffers,
}

impl PhysicsWorld {
    pub(crate) fn new(gravity: Vec3) -> Self {
        let integration_parameters = IntegrationParameters {
            dt: 1.0 / 60.0,
            num_solver_iterations: 4,
            max_ccd_substeps: 4,
            ..IntegrationParameters::default()
        };

        let events = Arc::new(crate::rapier::events::CollectingEventHandler::default());
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: vec3_to_rapier(gravity),
            integration_parameters,
            islands: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            hooks: crate::rapier::events::CallbackPhysicsHooks::new(events.clone()),
            events,
            force_registry: ForceRegistry::new(),
            shared_arena: None,
            buffers: FrameWorkBuffers::default(),
        }
    }
}

/// Create a new physics world.  Non-finite gravity components fall back to zero.
///
/// The returned pointer is owned by Rust; release it with `world_destroy`.
#[unsafe(no_mangle)]
pub extern "C" fn world_create(gravity: Vec3) -> *mut WorldHandle {
    ffi_guard(std::ptr::null_mut(), || {
        let gravity = if vec3_finite(gravity) {
            gravity
        } else {
            Vec3::default()
        };

        Box::into_raw(Box::new(WorldHandle {
            inner: PhysicsWorld::new(gravity),
        }))
    })
}

/// Destroy a physics world created by `world_create`.  Null is a no-op.
///
/// # Safety
/// `world` must be a pointer returned by `world_create` (or null) and must not
/// be used again after this call.
#[unsafe(no_mangle)]
pub extern "C" fn world_destroy(world: *mut WorldHandle) {
    ffi_guard((), || {
        if world.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(world));
        }
    })
}

/// Advance the simulation by `delta_seconds` (clamped to (0, 1]).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` and not yet destroyed.
#[unsafe(no_mangle)]
pub extern "C" fn world_step(world: *mut WorldHandle, delta_seconds: f64) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 || delta_seconds > MAX_STEP_SECONDS {
            return;
        }

        world.inner.integration_parameters.dt = delta_seconds;

        // --- Arena: drain Java commands before applying forces ---
        // Java writes forces/set-poses/impulses via shared memory, Rust reads them here.
        if let Some(ref arena) = world.inner.shared_arena {
            let commands = arena.drain_commands();
            if !commands.is_empty() {
                // Use persistent cached index map (P3 fix: avoid per-frame Vec rebuild)
                let idx = &mut world.inner.buffers.arena_idx_map;
                idx.clear();
                for (h, _) in world.inner.bodies.iter() {
                    idx.push(Some(h));
                }
                for (cmd_type, body_idx, a0, a1, a2) in commands {
                    if let Some(Some(h)) = idx.get(body_idx as usize)
                        && let Some(body) = world.inner.bodies.get_mut(*h)
                    {
                        match cmd_type {
                            0 => {
                                // AddForce
                                body.add_force(rapier3d::prelude::Vector::new(a0, a1, a2), true);
                            }
                            1 => {
                                // AddTorque
                                body.add_torque(rapier3d::prelude::Vector::new(a0, a1, a2), true);
                            }
                            2 => {
                                // SetPose
                                // a0..a2 = position, rest packed into user_data via cmd encoding
                                let pos = rapier3d::prelude::Pose::from_parts(
                                    rapier3d::prelude::Vector::new(a0, a1, a2),
                                    *body.rotation(),
                                );
                                body.set_position(pos, true);
                            }
                            3 => {
                                // SetVelocity
                                body.set_linvel(rapier3d::prelude::Vector::new(a0, a1, a2), true);
                            }
                            4 => {
                                // ApplyImpulse
                                body.apply_impulse(
                                    rapier3d::prelude::Vector::new(a0, a1, a2),
                                    true,
                                );
                            }
                            5 => {
                                // ApplyTorqueImpulse
                                body.apply_torque_impulse(
                                    rapier3d::prelude::Vector::new(a0, a1, a2),
                                    true,
                                );
                            }
                            6 => {
                                // WakeUp
                                body.wake_up(true);
                            }
                            7 => {
                                // Sleep
                                body.sleep();
                            }
                            8 => {
                                // SetRotation — a0..a2 = rotation vector (axis-angle)
                                let axis_angle = rapier3d::prelude::Vector::new(a0, a1, a2);
                                let angle = axis_angle.length();
                                if angle > 1e-12 {
                                    let unit_axis = axis_angle / angle;
                                    body.set_rotation(
                                        rapier3d::prelude::Rotation::from_axis_angle(
                                            unit_axis, angle,
                                        ),
                                        true,
                                    );
                                }
                            }
                            9 => {
                                // SetGravityScale — a0 = scale
                                body.set_gravity_scale(a0, true);
                            }
                            10 => {
                                // SetLinearDamping — a0 = damping
                                body.set_linear_damping(a0);
                            }
                            11 => {
                                // SetAngularDamping — a0 = damping
                                body.set_angular_damping(a0);
                            }
                            12 => {
                                // AddForceAtPoint — a0..a2 = force, need point from next cmd or use COM
                                body.add_force(rapier3d::prelude::Vector::new(a0, a1, a2), true);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // --- Coulomb hook setup ---
        let custom = world.inner.events.custom_physics();
        let coulomb_active = custom
            .coulomb_friction
            .is_some_and(|law| law.enabled.0 != 0);

        if coulomb_active {
            let hook_bit = ActiveHooks::MODIFY_SOLVER_CONTACTS;
            for (_, collider) in world.inner.colliders.iter_mut() {
                let current = collider.active_hooks();
                if !current.contains(hook_bit) {
                    collider.set_active_hooks(current | hook_bit);
                }
            }
        }

        // --- Force facade: the single entry-point for all force application ---
        // O1 fix: reuse persistent body_log (Vec-indexed by handle) instead of HashMap.
        // Take ownership of the buffers, use them, then put them back.
        let mut body_log = std::mem::take(&mut world.inner.buffers.body_log);
        let mut pending_forces = std::mem::take(&mut world.inner.buffers.pending_forces);
        let mut friction_work = std::mem::take(&mut world.inner.buffers.friction_work);
        let mut facade = ForceFacade::new(
            &mut world.inner.bodies,
            &mut world.inner.colliders,
            &world.inner.narrow_phase,
            &mut body_log,
            &mut pending_forces,
            &mut friction_work,
        );

        // 1. Registered ForceLaw list (from new system)
        world.inner.force_registry.apply_all(&mut facade);

        // 2. Backward-compat: old unregistered external-force law setter path
        //   Work around borrowck by copying body handles/positions, then replaying forces through facade.
        crate::rapier::events::apply_custom_external_forces_with_facade(&custom, &mut facade);

        // 3. Backward-compat: old unregistered body-interaction path
        //   Same approach: compute forces first (immutable reads), then replay.
        crate::rapier::interaction::apply_body_interactions_with_facade(
            &world.inner.force_registry,
            &custom,
            &mut facade,
        );

        // 4. Drain the facade frame-log into a report and write it to events
        let force_report = facade.drain_report();
        // P1+P5 fix: put drained buffers back for next frame reuse
        let empty_log = std::mem::take(facade.body_log);
        world.inner.buffers.body_log = empty_log;
        world.inner.buffers.pending_forces = std::mem::take(facade.pending_forces);
        world.inner.buffers.friction_work = std::mem::take(facade.friction_work);
        if force_report
            .contributions
            .values()
            .any(|c| c.body_count > 0)
        {
            world
                .inner
                .events
                .set_last_custom_physics_report(force_report.to_legacy_report());
        }

        world.inner.pipeline.step(
            world.inner.gravity,
            &world.inner.integration_parameters,
            &mut world.inner.islands,
            &mut world.inner.broad_phase,
            &mut world.inner.narrow_phase,
            &mut world.inner.bodies,
            &mut world.inner.colliders,
            &mut world.inner.impulse_joints,
            &mut world.inner.multibody_joints,
            &mut world.inner.ccd_solver,
            &world.inner.hooks,
            &*world.inner.events,
        );

        // 5. Flush shared arena body/collider state → Java zero-JNI read
        if let Some(ref arena) = world.inner.shared_arena {
            arena.flush_all_bodies(&world.inner.bodies);
            arena.flush_all_colliders(&world.inner.colliders);
            arena.flush_integration_params(
                world.inner.integration_parameters.dt,
                world.inner.integration_parameters.num_solver_iterations as u32,
                world.inner.integration_parameters.max_ccd_substeps as u32,
                &world.inner.gravity,
            );
            let legacy = &force_report.to_legacy_report();
            arena.flush_force_report(
                force_report.max_reynolds_number,
                &legacy.total_external_force,
                &legacy.total_drag_force,
                legacy.drag_body_count,
                legacy.external_force_body_count,
            );
            // Per-type breakdown (zero-JNI for Java to inspect)
            arena.flush_force_breakdown(&force_report);
            arena.flush_events_from_handler(&world.inner.events);
        }
    })
}

/// Set integration parameters (dt, solver iterations, CCD substeps).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` and not yet destroyed.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_integration_parameters(
    world: *mut WorldHandle,
    dt: f64,
    solver_iterations: u32,
    ccd_substeps: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return crate::rapier::ffi::Bool::FALSE;
        };
        if !dt.is_finite()
            || dt <= 0.0
            || dt > MAX_STEP_SECONDS
            || solver_iterations == 0
            || solver_iterations > 255
            || ccd_substeps > 255
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid integration parameters");
            return crate::rapier::ffi::Bool::FALSE;
        }

        world.inner.integration_parameters.dt = dt;
        world.inner.integration_parameters.num_solver_iterations = solver_iterations as usize;
        world.inner.integration_parameters.max_ccd_substeps = ccd_substeps as usize;
        clear_error();
        crate::rapier::ffi::Bool::TRUE
    })
}

/// Read integration parameters into `out_values` (dt, iterations, CCD substeps).
///
/// # Safety
/// `world` must be a valid world pointer (or null); `out_values` must point to
/// writable memory for at least `capacity` f64 values.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_integration_parameters(
    world: *const WorldHandle,
    out_values: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_values.is_null() {
            set_error(ERR_NULL_POINTER, "integration parameter output is null");
            return 0;
        }
        if capacity < 3 {
            set_error(
                ERR_CAPACITY,
                "integration parameter output capacity must be at least 3",
            );
            return 0;
        }

        let out = unsafe { std::slice::from_raw_parts_mut(out_values, capacity as usize) };
        out[0] = world.inner.integration_parameters.dt;
        out[1] = world.inner.integration_parameters.num_solver_iterations as f64;
        out[2] = world.inner.integration_parameters.max_ccd_substeps as f64;
        clear_error();
        3
    })
}

/// Set the world gravity vector.  Non-finite input is ignored.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` and not yet destroyed.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_gravity(world: *mut WorldHandle, gravity: Vec3) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        if !vec3_finite(gravity) {
            return;
        }

        world.inner.gravity = vec3_to_rapier(gravity);
    })
}

/// Get the world gravity vector.
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_gravity(world: *const WorldHandle) -> Vec3 {
    ffi_guard(Vec3::default(), || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return Vec3::default();
        };

        crate::rapier::ffi::vec3_from_rapier(world.inner.gravity)
    })
}

/// Number of rigid bodies in the world (-1 on null world).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_rigid_body_set_size(world: *const WorldHandle) -> i32 {
    ffi_guard(-1, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return -1;
        };

        world.inner.bodies.len() as i32
    })
}

/// Number of colliders in the world (-1 on null world).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_collider_set_size(world: *const WorldHandle) -> i32 {
    ffi_guard(-1, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return -1;
        };

        world.inner.colliders.len() as i32
    })
}

/// Write the world gravity into `out_gravity`.
///
/// # Safety
/// `out_gravity` must point to a writable `Vec3` (or be null); `world` must be
/// a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_gravity_out(world: *const WorldHandle, out_gravity: *mut Vec3) {
    ffi_guard((), || {
        let Some(out_gravity) = (unsafe { out_gravity.as_mut() }) else {
            return;
        };

        *out_gravity = world_get_gravity(world);
    })
}

/// Count of dynamic bodies (for sizing a `world_dynamic_body_snapshot` call).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_dynamic_body_snapshot_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };

        world
            .inner
            .bodies
            .iter()
            .filter(|(_, body)| body.is_dynamic())
            .count() as u32
    })
}

/// Snapshot dynamic body handles + poses (7 f64 per body: pos3 + quat4).
///
/// # Safety
/// `world` must be a valid world pointer (or null); `out_handles` must point to
/// writable memory for `capacity` handles and `out_values` for `capacity * 7`
/// f64 values.
#[unsafe(no_mangle)]
pub extern "C" fn world_dynamic_body_snapshot(
    world: *const WorldHandle,
    out_handles: *mut RigidBodyHandleRaw,
    out_values: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        if out_handles.is_null()
            || out_values.is_null()
            || capacity == 0
            || capacity > MAX_OUTPUT_CAPACITY
        {
            return 0;
        }

        let capacity = capacity as usize;
        let Some(value_capacity) = capacity.checked_mul(7) else {
            return 0;
        };
        let handles = unsafe { std::slice::from_raw_parts_mut(out_handles, capacity) };
        let values = unsafe { std::slice::from_raw_parts_mut(out_values, value_capacity) };
        let mut written = 0usize;

        for (handle, body) in world.inner.bodies.iter() {
            if !body.is_dynamic() {
                continue;
            }
            if written >= capacity {
                break;
            }

            let translation = vec3_from_rapier(body.translation());
            let rotation = quat_from_rapier(*body.rotation());
            handles[written] = pack_rigid_body_handle(handle);
            let offset = written * 7;
            values[offset] = translation.x;
            values[offset + 1] = translation.y;
            values[offset + 2] = translation.z;
            values[offset + 3] = rotation.i;
            values[offset + 4] = rotation.j;
            values[offset + 5] = rotation.k;
            values[offset + 6] = rotation.w;
            written += 1;
        }

        written as u32
    })
}

/// Count of all bodies (for sizing a `world_body_snapshot` call).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_body_snapshot_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };

        world.inner.bodies.len().min(u32::MAX as usize) as u32
    })
}

/// Snapshot all body handles + poses + velocities (13 f64 per body:
/// pos3 + quat4 + linvel3 + angvel3).
///
/// # Safety
/// `world` must be a valid world pointer (or null); `out_handles` must point to
/// writable memory for `capacity` handles and `out_values` for `capacity * 13`
/// f64 values.
#[unsafe(no_mangle)]
pub extern "C" fn world_body_snapshot(
    world: *const WorldHandle,
    out_handles: *mut RigidBodyHandleRaw,
    out_values: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_handles.is_null()
            || out_values.is_null()
            || capacity == 0
            || capacity > MAX_OUTPUT_CAPACITY
        {
            set_error(ERR_CAPACITY, "invalid body snapshot output");
            return 0;
        }

        let capacity = capacity as usize;
        let Some(value_capacity) = capacity.checked_mul(13) else {
            set_error(ERR_CAPACITY, "body snapshot output capacity overflow");
            return 0;
        };
        let handles = unsafe { std::slice::from_raw_parts_mut(out_handles, capacity) };
        let values = unsafe { std::slice::from_raw_parts_mut(out_values, value_capacity) };
        let mut written = 0usize;

        for (handle, body) in world.inner.bodies.iter() {
            if written >= capacity {
                break;
            }

            let translation = vec3_from_rapier(body.translation());
            let rotation = quat_from_rapier(*body.rotation());
            let linvel = vec3_from_rapier(body.linvel());
            let angvel = vec3_from_rapier(body.angvel());
            handles[written] = pack_rigid_body_handle(handle);
            let offset = written * 13;
            values[offset] = translation.x;
            values[offset + 1] = translation.y;
            values[offset + 2] = translation.z;
            values[offset + 3] = rotation.i;
            values[offset + 4] = rotation.j;
            values[offset + 5] = rotation.k;
            values[offset + 6] = rotation.w;
            values[offset + 7] = linvel.x;
            values[offset + 8] = linvel.y;
            values[offset + 9] = linvel.z;
            values[offset + 10] = angvel.x;
            values[offset + 11] = angvel.y;
            values[offset + 12] = angvel.z;
            written += 1;
        }

        clear_error();
        written as u32
    })
}

/// Batch-update body poses (7 f64 per body: pos3 + quat4).  Returns the number
/// of bodies actually updated.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`; `handles` and
/// `values` must point to readable arrays of `count` handles and `count * 7`
/// f64 values respectively.
#[unsafe(no_mangle)]
pub extern "C" fn world_update_body_poses(
    world: *mut WorldHandle,
    handles: *const RigidBodyHandleRaw,
    values: *const f64,
    count: u32,
    wake_up: crate::rapier::ffi::Bool,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if handles.is_null() || values.is_null() || count == 0 || count > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid body pose input");
            return 0;
        }

        let count = count as usize;
        let Some(value_count) = count.checked_mul(7) else {
            set_error(ERR_CAPACITY, "body pose input capacity overflow");
            return 0;
        };
        let handles = unsafe { std::slice::from_raw_parts(handles, count) };
        let values = unsafe { std::slice::from_raw_parts(values, value_count) };
        let mut updated = 0u32;

        for (index, handle) in handles.iter().enumerate() {
            let offset = index * 7;
            let translation = Vec3 {
                x: values[offset],
                y: values[offset + 1],
                z: values[offset + 2],
            };
            let rotation = Quat {
                i: values[offset + 3],
                j: values[offset + 4],
                k: values[offset + 5],
                w: values[offset + 6],
            };
            if !vec3_finite(translation) || !quat_finite(rotation) {
                continue;
            }
            if let Some(body) = world
                .inner
                .bodies
                .get_mut(unpack_rigid_body_handle(*handle))
            {
                body.set_position(isometry_from_parts(translation, rotation), wake_up.0 != 0);
                updated += 1;
            }
        }

        if updated == 0 {
            set_error(ERR_NOT_FOUND, "no body poses were updated");
        } else {
            clear_error();
        }
        updated
    })
}

/// Batch-update body velocities (6 f64 per body: linvel3 + angvel3).  Returns
/// the number of bodies actually updated.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`; `handles` and
/// `values` must point to readable arrays of `count` handles and `count * 6`
/// f64 values respectively.
#[unsafe(no_mangle)]
pub extern "C" fn world_update_body_velocities(
    world: *mut WorldHandle,
    handles: *const RigidBodyHandleRaw,
    values: *const f64,
    count: u32,
    wake_up: crate::rapier::ffi::Bool,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if handles.is_null() || values.is_null() || count == 0 || count > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid body velocity input");
            return 0;
        }

        let count = count as usize;
        let Some(value_count) = count.checked_mul(6) else {
            set_error(ERR_CAPACITY, "body velocity input capacity overflow");
            return 0;
        };
        let handles = unsafe { std::slice::from_raw_parts(handles, count) };
        let values = unsafe { std::slice::from_raw_parts(values, value_count) };
        let mut updated = 0u32;

        for (index, handle) in handles.iter().enumerate() {
            let offset = index * 6;
            let linvel = Vec3 {
                x: values[offset],
                y: values[offset + 1],
                z: values[offset + 2],
            };
            let angvel = Vec3 {
                x: values[offset + 3],
                y: values[offset + 4],
                z: values[offset + 5],
            };
            if !vec3_finite(linvel) || !vec3_finite(angvel) {
                continue;
            }
            if let Some(body) = world
                .inner
                .bodies
                .get_mut(unpack_rigid_body_handle(*handle))
            {
                body.set_linvel(vec3_to_rapier(linvel), wake_up.0 != 0);
                body.set_angvel(vec3_to_rapier(angvel), wake_up.0 != 0);
                updated += 1;
            }
        }

        if updated == 0 {
            set_error(ERR_NOT_FOUND, "no body velocities were updated");
        } else {
            clear_error();
        }
        updated
    })
}

// ---------------------------------------------------------------------------
// Convenience: register celestial gravity as a ForceLaw
// ---------------------------------------------------------------------------

/// Convert a u32 tag to `CelestialBodyId`.  Returns `None` for out-of-range
/// values instead of relying on a range guard plus `transmute`.
fn celestial_body_id_from_u32(
    body_id: u32,
) -> Option<crate::rapier::celestial_data::CelestialBodyId> {
    use crate::rapier::celestial_data::CelestialBodyId;
    match body_id {
        0 => Some(CelestialBodyId::Sun),
        1 => Some(CelestialBodyId::Mercury),
        2 => Some(CelestialBodyId::Venus),
        3 => Some(CelestialBodyId::Earth),
        4 => Some(CelestialBodyId::Moon),
        5 => Some(CelestialBodyId::Mars),
        6 => Some(CelestialBodyId::Jupiter),
        7 => Some(CelestialBodyId::Saturn),
        8 => Some(CelestialBodyId::Uranus),
        9 => Some(CelestialBodyId::Neptune),
        _ => None,
    }
}

/// Register celestial body gravity as a ForceLaw in the world's registry.
///
/// `body_id` maps to `CelestialBodyId` (0=Sun, 3=Earth, 4=Moon, 5=Mars, etc.).
///
/// Returns handle (non-zero) on success, 0 on invalid body_id.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` and not yet destroyed.
#[unsafe(no_mangle)]
pub extern "C" fn world_register_celestial_gravity(
    world: *mut WorldHandle,
    body_id: u32,
    max_degree: u32,
) -> u64 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        let Some(id) = celestial_body_id_from_u32(body_id) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid celestial body ID");
            return 0;
        };
        let body = crate::rapier::celestial_data::get_celestial_body(id);
        let law = crate::rapier::interaction::CelestialGravityForceLaw {
            body,
            max_sh_degree: max_degree.min(body.max_degree),
            enabled: true,
        };

        // P8: single traversal to find + unregister all existing celestial gravity laws
        world
            .inner
            .force_registry
            .unregister_by_type(crate::rapier::forces::ForceLawType::CelestialGravity);

        clear_error();
        world.inner.force_registry.register(Box::new(law)).raw()
    })
}

// ---------------------------------------------------------------------------
// ForceRegistry FFI — generic access for advanced callers
// ---------------------------------------------------------------------------

/// Opaque handle for a force law registered in the world's ForceRegistry.
/// Maps to `ForceLawHandle` in Rust.
pub type ForceLawHandleRaw = u64;

/// Number of force laws registered in the world's ForceRegistry.
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_force_registry_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        world.inner.force_registry.len() as u32
    })
}

/// Get count of registered force laws of a specific type.
/// `law_type` is the numeric discriminant of `ForceLawType`.
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_force_registry_typed_count(
    world: *const WorldHandle,
    law_type: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        let law_type = match force_law_type_from_u32(law_type) {
            Some(lt) => lt,
            None => return 0,
        };
        world.inner.force_registry.find_by_type(law_type).len() as u32
    })
}

// ---------------------------------------------------------------------------
// Tests

// ---------------------------------------------------------------------------
// Shared Arena FFI — zero-JNI physics data access
// ---------------------------------------------------------------------------

/// Create a shared-memory physics arena.
///
/// Returns the arena pointer as a u64 (suitable for `MemorySegment.ofAddress` in Java).
/// The arena persists for the lifetime of the world.
///
/// At most one arena may exist per world. Calling this again while an arena
/// is still live fails with `ERR_INVALID_ARGUMENT` and leaves the existing
/// arena untouched — call `world_destroy_shared_arena` first to recreate one.
///
/// WARNING (Java side): before calling `world_destroy_shared_arena`, the
/// `MemorySegment` mapping the arena must be released/unmapped; destroying
/// the arena frees the underlying memory, and any still-mapped Java segment
/// would become a use-after-free.
///
/// `max_bodies` — max concurrent bodies to mirror
/// `max_events` — max pending collision/contact events
/// `max_commands` — max pending commands (force/set pose etc.)
/// `out_address` — receives the arena base address
/// `out_size` — receives the total arena size in bytes (for Java MemorySegment mapping)
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`; `out_address`
/// and `out_size` may be null, otherwise each must point to a writable u64.
#[unsafe(no_mangle)]
pub extern "C" fn world_create_shared_arena(
    world: *mut WorldHandle,
    max_bodies: u32,
    max_colliders: u32,
    max_events: u32,
    max_commands: u32,
    out_address: *mut u64,
    out_size: *mut u64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if world.inner.shared_arena.is_some() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "shared arena already exists; destroy it before recreating",
            );
            return Bool::FALSE;
        }
        if max_bodies == 0 || max_colliders == 0 || max_events == 0 || max_commands == 0 {
            set_error(ERR_INVALID_ARGUMENT, "arena capacities must be >0");
            return Bool::FALSE;
        }

        let Some(arena) = crate::rapier::shared_arena::SharedPhysicsArena::new(
            max_bodies,
            max_colliders,
            max_events,
            max_commands,
        ) else {
            set_error(ERR_CAPACITY, "arena capacities exceed limits");
            return Bool::FALSE;
        };
        let addr = arena.address();
        let sz = arena.size() as u64;

        world.inner.shared_arena = Some(Box::new(arena));

        if let Some(p) = unsafe { out_address.as_mut() } {
            *p = addr;
        }
        if let Some(p) = unsafe { out_size.as_mut() } {
            *p = sz;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Destroy the shared arena (if any).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` (or null).  Any
/// Java `MemorySegment` mapping the arena must be released before this call.
#[unsafe(no_mangle)]
pub extern "C" fn world_destroy_shared_arena(world: *mut WorldHandle) {
    ffi_guard((), || {
        if let Some(world) = unsafe { world.as_mut() } {
            world.inner.shared_arena = None;
        }
    })
}

/// Get the arena address (returns 0 if no arena).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_shared_arena_address(world: *const WorldHandle) -> u64 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        world.inner.shared_arena.as_ref().map_or(0, |a| a.address())
    })
}

/// Get the arena size (returns 0 if no arena).
///
/// # Safety
/// `world` must be a valid world pointer (or null).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_shared_arena_size(world: *const WorldHandle) -> u64 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        world
            .inner
            .shared_arena
            .as_ref()
            .map_or(0, |a| a.size() as u64)
    })
}

/// Reset the event ring (Java calls this after draining events).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create` (or null) and not
/// yet destroyed.
#[unsafe(no_mangle)]
pub extern "C" fn world_reset_shared_arena_events(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        if let Some(ref arena) = world.inner.shared_arena {
            arena.reset_event_ring();
        }
    })
}
