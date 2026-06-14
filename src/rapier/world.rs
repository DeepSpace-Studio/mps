use rapier3d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodySet, Vector,
};

use crate::rapier::ffi::{
    MAX_OUTPUT_CAPACITY, RigidBodyHandleRaw, Vec3, WorldHandle, pack_rigid_body_handle,
    quat_from_rapier, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

const MAX_STEP_SECONDS: f64 = 1.0;

pub(crate) struct PhysicsWorld {
    pub(crate) pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) islands: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) hooks: crate::rapier::events::CallbackPhysicsHooks,
    pub(crate) events: crate::rapier::events::CollectingEventHandler,
}

impl PhysicsWorld {
    pub(crate) fn new(gravity: Vec3) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / 60.0;
        integration_parameters.num_solver_iterations = 4;
        integration_parameters.max_ccd_substeps = 4;

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
            hooks: crate::rapier::events::CallbackPhysicsHooks::default(),
            events: crate::rapier::events::CollectingEventHandler::default(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn world_create(gravity: Vec3) -> *mut WorldHandle {
    let gravity = if vec3_finite(gravity) {
        gravity
    } else {
        Vec3::default()
    };

    Box::into_raw(Box::new(WorldHandle {
        inner: PhysicsWorld::new(gravity),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn world_destroy(world: *mut WorldHandle) {
    if world.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(world));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn world_step(world: *mut WorldHandle, delta_seconds: f64) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return;
    };
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 || delta_seconds > MAX_STEP_SECONDS {
        return;
    }

    world.inner.integration_parameters.dt = delta_seconds;
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
        &world.inner.events,
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn world_set_gravity(world: *mut WorldHandle, gravity: Vec3) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return;
    };
    if !vec3_finite(gravity) {
        return;
    }

    world.inner.gravity = vec3_to_rapier(gravity);
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_gravity(world: *const WorldHandle) -> Vec3 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return Vec3::default();
    };

    crate::rapier::ffi::vec3_from_rapier(world.inner.gravity)
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_rigid_body_set_size(world: *const WorldHandle) -> i32 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return -1;
    };

    world.inner.bodies.len() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_collider_set_size(world: *const WorldHandle) -> i32 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return -1;
    };

    world.inner.colliders.len() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_gravity_out(world: *const WorldHandle, out_gravity: *mut Vec3) {
    let Some(out_gravity) = (unsafe { out_gravity.as_mut() }) else {
        return;
    };

    *out_gravity = world_get_gravity(world);
}

#[unsafe(no_mangle)]
pub extern "C" fn world_dynamic_body_snapshot_count(world: *const WorldHandle) -> u32 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return 0;
    };

    world
        .inner
        .bodies
        .iter()
        .filter(|(_, body)| body.is_dynamic())
        .count() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn world_dynamic_body_snapshot(
    world: *const WorldHandle,
    out_handles: *mut RigidBodyHandleRaw,
    out_values: *mut f64,
    capacity: u32,
) -> u32 {
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
}
