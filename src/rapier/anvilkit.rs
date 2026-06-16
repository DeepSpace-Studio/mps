use anvilkit::core::math::Transform;
use anvilkit::ecs::physics as ak_physics;
use anvilkit::ecs::prelude::*;
use hashbrown::HashMap;
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyType};

use crate::rapier::aerodynamics;
use crate::rapier::fluid;
use crate::rapier::ffi::{
    AeroForceReport, AeroSurface, Bool, ColliderHandleRaw, Quat, RigidBodyHandleRaw, ShapeDesc,
    FluidForceReport, FluidVolume, Vec3, WorldHandle,
    pack_collider_handle, pack_rigid_body_handle, quat_finite, unpack_rigid_body_handle,
    vec3_finite,
};

pub(crate) struct AnvilKitAppState {
    app: anvilkit::ecs::app::App,
    entity_to_body: HashMap<Entity, RigidBodyHandleRaw>,
    entity_to_collider: HashMap<Entity, ColliderHandleRaw>,
}

#[derive(Component, Clone, Copy)]
struct BodyLink {
    handle: RigidBodyHandleRaw,
}

#[derive(Component, Clone, Copy)]
struct ColliderLink {
    handle: ColliderHandleRaw,
}

#[derive(Component, Clone, Copy)]
struct PendingCollider {
    shape: ShapeDesc,
}

fn transform_from_parts(translation: Vec3, rotation: Quat) -> Transform {
    Transform::from_xyz(
        translation.x as f32,
        translation.y as f32,
        translation.z as f32,
    )
    .with_rotation(anvilkit::core::Quat::from_xyzw(
        rotation.i as f32,
        rotation.j as f32,
        rotation.k as f32,
        rotation.w as f32,
    ))
}

fn vec3_from_glam(value: anvilkit::core::Vec3) -> Vec3 {
    Vec3 {
        x: value.x as f64,
        y: value.y as f64,
        z: value.z as f64,
    }
}

fn quat_from_glam(value: anvilkit::core::Quat) -> Quat {
    Quat {
        i: value.x as f64,
        j: value.y as f64,
        k: value.z as f64,
        w: value.w as f64,
    }
}

fn body_type_from_raw(status: u32) -> RigidBodyType {
    match status {
        0 => RigidBodyType::Dynamic,
        2 => RigidBodyType::KinematicPositionBased,
        3 => RigidBodyType::KinematicVelocityBased,
        _ => RigidBodyType::Fixed,
    }
}

fn ak_body_type_from_raw(status: u32) -> ak_physics::RigidBodyType {
    match status {
        0 => ak_physics::RigidBodyType::Dynamic,
        2 | 3 => ak_physics::RigidBodyType::Kinematic,
        _ => ak_physics::RigidBodyType::Fixed,
    }
}

fn shape_builder(shape: ShapeDesc) -> Option<ColliderBuilder> {
    if !crate::rapier::ffi::shape_desc_valid(shape) {
        return None;
    }
    Some(ColliderBuilder::new(crate::rapier::ffi::shape_from_desc(shape)))
}

impl AnvilKitAppState {
    fn new() -> Self {
        let mut app = anvilkit::ecs::app::App::new();
        app.add_plugins(anvilkit::ecs::plugin::AnvilKitEcsPlugin);
        Self {
            app,
            entity_to_body: HashMap::new(),
            entity_to_collider: HashMap::new(),
        }
    }

    fn entity_from_bits(&self, entity_bits: u64) -> Option<Entity> {
        Entity::try_from_bits(entity_bits)
            .ok()
            .filter(|entity| self.app.world.entities().contains(*entity))
    }

    fn spawn_body(&mut self, translation: Vec3, rotation: Quat, status: u32) -> u64 {
        if !vec3_finite(translation) || !quat_finite(rotation) {
            return 0;
        }
        let entity = self
            .app
            .world
            .spawn((
                transform_from_parts(translation, rotation),
                ak_physics::RigidBody::new(ak_body_type_from_raw(status)),
            ))
            .id();
        entity.to_bits()
    }

    fn spawn_body_with_collider(
        &mut self,
        translation: Vec3,
        rotation: Quat,
        status: u32,
        shape: ShapeDesc,
    ) -> u64 {
        if shape_builder(shape).is_none() {
            return 0;
        }
        let entity_bits = self.spawn_body(translation, rotation, status);
        if entity_bits == 0 {
            return 0;
        }
        let Some(entity) = self.entity_from_bits(entity_bits) else {
            return 0;
        };
        self.app.world.entity_mut(entity).insert(PendingCollider { shape });
        entity_bits
    }

    fn set_transform(&mut self, entity_bits: u64, translation: Vec3, rotation: Quat) -> Bool {
        if !vec3_finite(translation) || !quat_finite(rotation) {
            return Bool::FALSE;
        }
        let Some(entity) = self.entity_from_bits(entity_bits) else {
            return Bool::FALSE;
        };
        let Some(mut transform) = self.app.world.get_mut::<Transform>(entity) else {
            return Bool::FALSE;
        };
        *transform = transform_from_parts(translation, rotation);
        Bool::TRUE
    }

    fn sync_to_world(&mut self, world: &mut WorldHandle) -> u32 {
        let entities: Vec<_> = self
            .app
            .world
            .query::<(
                Entity,
                &Transform,
                &ak_physics::RigidBody,
                Option<&PendingCollider>,
            )>()
            .iter(&self.app.world)
            .map(|(entity, transform, body, collider)| {
                (entity, *transform, body.body_type, collider.copied())
            })
            .collect();

        let mut synced = 0u32;
        for (entity, transform, body_type, pending_collider) in entities {
            let translation = vec3_from_glam(transform.translation);
            let rotation = quat_from_glam(transform.rotation);
            let body_handle = if let Some(handle) = self.entity_to_body.get(&entity).copied() {
                handle
            } else {
                let body = RigidBodyBuilder::new(body_type_from_raw(match body_type {
                    ak_physics::RigidBodyType::Dynamic => 0,
                    ak_physics::RigidBodyType::Fixed => 1,
                    ak_physics::RigidBodyType::Kinematic => 2,
                }))
                    .pose(crate::rapier::ffi::isometry_from_parts(translation, rotation))
                    .build();
                let packed = pack_rigid_body_handle(world.inner.bodies.insert(body));
                self.entity_to_body.insert(entity, packed);
                self.app.world.entity_mut(entity).insert(BodyLink { handle: packed });
                packed
            };

            if let Some(body) = world
                .inner
                .bodies
                .get_mut(unpack_rigid_body_handle(body_handle))
            {
                body.set_position(
                    crate::rapier::ffi::isometry_from_parts(translation, rotation),
                    false,
                );
                synced = synced.saturating_add(1);
            }

            if self.entity_to_collider.contains_key(&entity) {
                continue;
            }
            let Some(pending_collider) = pending_collider else {
                continue;
            };
            let Some(builder) = shape_builder(pending_collider.shape) else {
                continue;
            };
            let collider = builder.build();
            let handle = world.inner.colliders.insert_with_parent(
                collider,
                unpack_rigid_body_handle(body_handle),
                &mut world.inner.bodies,
            );
            let packed = pack_collider_handle(handle);
            self.entity_to_collider.insert(entity, packed);
            self.app.world.entity_mut(entity).insert(ColliderLink { handle: packed });
        }

        synced
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_create() -> *mut crate::rapier::ffi::AnvilKitAppHandle {
    Box::into_raw(Box::new(crate::rapier::ffi::AnvilKitAppHandle {
        inner: AnvilKitAppState::new(),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_destroy(app: *mut crate::rapier::ffi::AnvilKitAppHandle) {
    if app.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(app));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_update(app: *mut crate::rapier::ffi::AnvilKitAppHandle) {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return;
    };
    app.inner.app.update();
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_spawn_body(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    translation: Vec3,
    rotation: Quat,
    status: u32,
) -> u64 {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return 0;
    };
    app.inner.spawn_body(translation, rotation, status)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_spawn_body_with_collider(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    translation: Vec3,
    rotation: Quat,
    status: u32,
    shape: ShapeDesc,
) -> u64 {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return 0;
    };
    app.inner
        .spawn_body_with_collider(translation, rotation, status, shape)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_set_transform(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    entity_bits: u64,
    translation: Vec3,
    rotation: Quat,
) -> Bool {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return Bool::FALSE;
    };
    app.inner.set_transform(entity_bits, translation, rotation)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_sync_to_world(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    world: *mut WorldHandle,
) -> u32 {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return 0;
    };
    let Some(world) = (unsafe { world.as_mut() }) else {
        return 0;
    };
    app.inner.sync_to_world(world)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_entity_to_body(
    app: *const crate::rapier::ffi::AnvilKitAppHandle,
    entity_bits: u64,
) -> RigidBodyHandleRaw {
    let Some(app) = (unsafe { app.as_ref() }) else {
        return 0;
    };
    let Ok(entity) = Entity::try_from_bits(entity_bits) else {
        return 0;
    };
    app.inner.entity_to_body.get(&entity).copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_entity_to_collider(
    app: *const crate::rapier::ffi::AnvilKitAppHandle,
    entity_bits: u64,
) -> ColliderHandleRaw {
    let Some(app) = (unsafe { app.as_ref() }) else {
        return 0;
    };
    let Ok(entity) = Entity::try_from_bits(entity_bits) else {
        return 0;
    };
    app.inner
        .entity_to_collider
        .get(&entity)
        .copied()
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_apply_aero_surfaces(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    world: *mut WorldHandle,
    entity_bits: u64,
    wind_velocity: Vec3,
    air_density: f64,
    surfaces: *const AeroSurface,
    surface_count: u32,
    wake_up: Bool,
    out_report: *mut AeroForceReport,
) -> Bool {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return Bool::FALSE;
    };
    let Some(world) = (unsafe { world.as_mut() }) else {
        return Bool::FALSE;
    };
    let Ok(entity) = Entity::try_from_bits(entity_bits) else {
        return Bool::FALSE;
    };
    let Some(handle) = app.inner.entity_to_body.get(&entity).copied() else {
        return Bool::FALSE;
    };

    aerodynamics::aero_apply_surfaces(
        world,
        handle,
        wind_velocity,
        air_density,
        surfaces,
        surface_count,
        wake_up,
        out_report,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_apply_aero_voxel_grid(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    world: *mut WorldHandle,
    entity_bits: u64,
    wind_velocity: Vec3,
    air_density: f64,
    voxels: *const u8,
    size_x: u32,
    size_y: u32,
    size_z: u32,
    voxel_size: f64,
    local_origin: Vec3,
    drag_coefficient: f64,
    lift_coefficient: f64,
    wake_up: Bool,
    out_report: *mut AeroForceReport,
) -> Bool {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return Bool::FALSE;
    };
    let Some(world) = (unsafe { world.as_mut() }) else {
        return Bool::FALSE;
    };
    let Ok(entity) = Entity::try_from_bits(entity_bits) else {
        return Bool::FALSE;
    };
    let Some(handle) = app.inner.entity_to_body.get(&entity).copied() else {
        return Bool::FALSE;
    };

    aerodynamics::aero_apply_voxel_grid(
        world,
        handle,
        wind_velocity,
        air_density,
        voxels,
        size_x,
        size_y,
        size_z,
        voxel_size,
        local_origin,
        drag_coefficient,
        lift_coefficient,
        wake_up,
        out_report,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn anvilkit_app_apply_fluid_aabb_forces(
    app: *mut crate::rapier::ffi::AnvilKitAppHandle,
    world: *mut WorldHandle,
    entity_bits: u64,
    fluid_volume: FluidVolume,
    body_half_extents: Vec3,
    body_volume: f64,
    wake_up: Bool,
    out_report: *mut FluidForceReport,
) -> Bool {
    let Some(app) = (unsafe { app.as_mut() }) else {
        return Bool::FALSE;
    };
    let Some(world) = (unsafe { world.as_mut() }) else {
        return Bool::FALSE;
    };
    let Ok(entity) = Entity::try_from_bits(entity_bits) else {
        return Bool::FALSE;
    };
    let Some(handle) = app.inner.entity_to_body.get(&entity).copied() else {
        return Bool::FALSE;
    };

    fluid::fluid_apply_aabb_forces(
        world,
        handle,
        fluid_volume,
        body_half_extents,
        body_volume,
        wake_up,
        out_report,
    )
}
