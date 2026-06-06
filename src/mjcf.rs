use std::slice;

use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::prelude::{
    ColliderBuilder, ColliderHandle, FixedJointBuilder, ImpulseJointHandle, PrismaticJointBuilder,
    RevoluteJointBuilder, RigidBodyBuilder, RigidBodyHandle, SharedShape, SphericalJointBuilder,
};
use rapier3d_mjcf::mjcf_rs::body::{Geom, GeomType, Joint, JointType};
use rapier3d_mjcf::mjcf_rs::glam::{DQuat, DVec3};
use rapier3d_mjcf::mjcf_rs::model::Model;
use rapier3d_mjcf::mjcf_rs::{self, Pose as MjcfPose};

use crate::ffi::{
    Bool, ColliderHandleRaw, ImpulseJointHandleRaw, MjcfImportOptions, MjcfImportResult,
    RigidBodyHandleRaw, WorldHandle, pack_collider_handle, pack_impulse_joint_handle,
    pack_rigid_body_handle,
};

const MJCF_IMPORT_OK: u32 = 0;
const MJCF_IMPORT_NULL_WORLD: u32 = 1;
const MJCF_IMPORT_NULL_INPUT: u32 = 2;
const MJCF_IMPORT_INVALID_UTF8: u32 = 3;
const MJCF_IMPORT_PARSE_ERROR: u32 = 4;

#[derive(Clone, Copy)]
struct ImportOptions {
    make_roots_fixed: bool,
    scale: f64,
    density: f64,
    friction: f64,
    restitution: f64,
}

struct OutputBuffers {
    body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
    collider_handles: *mut ColliderHandleRaw,
    collider_capacity: u32,
    joint_handles: *mut ImpulseJointHandleRaw,
    joint_capacity: u32,
}

impl From<MjcfImportOptions> for ImportOptions {
    fn from(value: MjcfImportOptions) -> Self {
        let defaults = MjcfImportOptions::default();
        Self {
            make_roots_fixed: value.make_roots_fixed.as_bool(),
            scale: if value.scale.is_finite() && value.scale > 0.0 {
                value.scale
            } else {
                defaults.scale
            },
            density: if value.density.is_finite() {
                value.density
            } else {
                defaults.density
            },
            friction: if value.friction.is_finite() {
                value.friction
            } else {
                defaults.friction
            },
            restitution: if value.restitution.is_finite() {
                value.restitution
            } else {
                defaults.restitution
            },
        }
    }
}

fn status_result(status: u32) -> MjcfImportResult {
    MjcfImportResult {
        status,
        ..Default::default()
    }
}

fn v3(value: DVec3, scale: f64) -> Vector {
    Vector::new(value.x * scale, value.y * scale, value.z * scale)
}

fn rot(value: DQuat) -> Rotation {
    Rotation::from_xyzw(value.x, value.y, value.z, value.w)
}

fn pose_from_mjcf(value: MjcfPose, scale: f64) -> Pose {
    Pose::from_parts(v3(value.translation, scale), rot(value.rotation))
}

fn vector_from_array(value: [f64; 3], scale: f64) -> Vector {
    Vector::new(value[0] * scale, value[1] * scale, value[2] * scale)
}

fn shape_from_geom(geom: &Geom, scale: f64) -> Option<SharedShape> {
    match geom.type_ {
        GeomType::Sphere => Some(SharedShape::ball(geom.size[0] * scale)),
        GeomType::Capsule => Some(SharedShape::capsule_z(
            geom.size[1] * scale,
            geom.size[0] * scale,
        )),
        GeomType::Cylinder => Some(SharedShape::cylinder(
            geom.size[1] * scale,
            geom.size[0] * scale,
        )),
        GeomType::Box => Some(SharedShape::cuboid(
            geom.size[0] * scale,
            geom.size[1] * scale,
            geom.size[2] * scale,
        )),
        GeomType::Ellipsoid => Some(SharedShape::ball(
            geom.size[0].max(geom.size[1]).max(geom.size[2]) * scale,
        )),
        GeomType::Plane | GeomType::Hfield | GeomType::Mesh | GeomType::Sdf => None,
    }
}

fn write_packed_handles<T, U>(
    handles: &[T],
    out_handles: *mut U,
    capacity: u32,
    pack: impl Fn(T) -> U,
) where
    T: Copy,
{
    if out_handles.is_null() || capacity == 0 {
        return;
    }

    let out = unsafe { slice::from_raw_parts_mut(out_handles, capacity as usize) };
    for (dst, src) in out.iter_mut().zip(handles.iter().copied()) {
        *dst = pack(src);
    }
}

fn insert_collider(
    world: &mut WorldHandle,
    body: RigidBodyHandle,
    geom: &Geom,
    options: ImportOptions,
) -> Option<ColliderHandle> {
    let shape = shape_from_geom(geom, options.scale)?;
    let collider = ColliderBuilder::new(shape)
        .position(pose_from_mjcf(geom.pose, options.scale))
        .density(geom.density.unwrap_or(options.density))
        .friction(geom.friction[0].max(options.friction))
        .restitution(options.restitution)
        .build();
    Some(
        world
            .inner
            .colliders
            .insert_with_parent(collider, body, &mut world.inner.bodies),
    )
}

fn insert_joint(
    world: &mut WorldHandle,
    parent: RigidBodyHandle,
    child: RigidBodyHandle,
    joint: Option<&Joint>,
    options: ImportOptions,
) -> Option<ImpulseJointHandle> {
    match joint.map(|j| j.type_) {
        None => Some(world.inner.impulse_joints.insert(
            parent,
            child,
            FixedJointBuilder::new().build(),
            true,
        )),
        Some(JointType::Hinge) => {
            let joint = joint?;
            let axis = vector_from_array(joint.axis, 1.0).normalize_or_zero();
            let axis = if axis == Vector::ZERO {
                Vector::X
            } else {
                axis
            };
            let mut builder = RevoluteJointBuilder::new(axis)
                .local_anchor1(vector_from_array(joint.pos, options.scale))
                .local_anchor2(Vector::ZERO);
            if let Some(range) = joint.range {
                builder = builder.limits(range);
            }
            Some(
                world
                    .inner
                    .impulse_joints
                    .insert(parent, child, builder.build(), true),
            )
        }
        Some(JointType::Slide) => {
            let joint = joint?;
            let axis = vector_from_array(joint.axis, 1.0).normalize_or_zero();
            let axis = if axis == Vector::ZERO {
                Vector::X
            } else {
                axis
            };
            let mut builder = PrismaticJointBuilder::new(axis)
                .local_anchor1(vector_from_array(joint.pos, options.scale))
                .local_anchor2(Vector::ZERO);
            if let Some(range) = joint.range {
                builder = builder.limits([range[0] * options.scale, range[1] * options.scale]);
            }
            Some(
                world
                    .inner
                    .impulse_joints
                    .insert(parent, child, builder.build(), true),
            )
        }
        Some(JointType::Ball) => Some(world.inner.impulse_joints.insert(
            parent,
            child,
            SphericalJointBuilder::new().build(),
            true,
        )),
        Some(JointType::Free) => None,
    }
}

fn import_model(
    world: &mut WorldHandle,
    model: Model,
    options: ImportOptions,
    output: OutputBuffers,
) -> MjcfImportResult {
    let mut world_poses = vec![MjcfPose::IDENTITY; model.bodies.len()];
    for id in 1..model.bodies.len() {
        let entry = &model.bodies[id];
        let parent = entry.parent.unwrap_or(0);
        world_poses[id] = world_poses[parent] * entry.body.pose;
    }

    let mut body_handles_by_id: Vec<Option<RigidBodyHandle>> = vec![None; model.bodies.len()];
    let mut body_handles = Vec::new();
    let mut collider_handles = Vec::new();
    let mut skipped_geom_count = 0u32;

    for (id, entry) in model.bodies_iter() {
        let is_root = entry.parent.unwrap_or(0) == 0;
        let has_free_joint = entry
            .body
            .joints
            .iter()
            .any(|joint| joint.type_ == JointType::Free);
        let fixed = is_root && (options.make_roots_fixed || !has_free_joint);
        let builder = if fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        };
        let body = builder
            .pose(pose_from_mjcf(world_poses[id], options.scale))
            .build();
        let body_handle = world.inner.bodies.insert(body);
        body_handles_by_id[id] = Some(body_handle);
        body_handles.push(body_handle);

        for geom in &entry.body.geoms {
            if let Some(collider_handle) = insert_collider(world, body_handle, geom, options) {
                collider_handles.push(collider_handle);
            } else {
                skipped_geom_count += 1;
            }
        }
    }

    let mut joint_handles = Vec::new();
    for (id, entry) in model.bodies_iter() {
        let Some(child) = body_handles_by_id[id] else {
            continue;
        };
        let Some(parent_id) = entry.parent else {
            continue;
        };
        let Some(parent) = body_handles_by_id.get(parent_id).and_then(|handle| *handle) else {
            continue;
        };

        if entry.body.joints.is_empty() {
            if let Some(handle) = insert_joint(world, parent, child, None, options) {
                joint_handles.push(handle);
            }
        } else {
            for joint in &entry.body.joints {
                if let Some(handle) = insert_joint(world, parent, child, Some(joint), options) {
                    joint_handles.push(handle);
                }
            }
        }
    }

    write_packed_handles(
        &body_handles,
        output.body_handles,
        output.body_capacity,
        pack_rigid_body_handle,
    );
    write_packed_handles(
        &collider_handles,
        output.collider_handles,
        output.collider_capacity,
        pack_collider_handle,
    );
    write_packed_handles(
        &joint_handles,
        output.joint_handles,
        output.joint_capacity,
        pack_impulse_joint_handle,
    );

    MjcfImportResult {
        status: MJCF_IMPORT_OK,
        body_count: body_handles.len() as u32,
        collider_count: collider_handles.len() as u32,
        joint_count: joint_handles.len() as u32,
        skipped_geom_count,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mjcf_import_options_default() -> MjcfImportOptions {
    MjcfImportOptions::default()
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_mjcf_from_bytes_ex(
    world: *mut WorldHandle,
    mjcf_bytes: *const u8,
    mjcf_len: u32,
    options: MjcfImportOptions,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
    out_collider_handles: *mut ColliderHandleRaw,
    collider_capacity: u32,
    out_joint_handles: *mut ImpulseJointHandleRaw,
    joint_capacity: u32,
) -> MjcfImportResult {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return status_result(MJCF_IMPORT_NULL_WORLD);
    };
    if mjcf_bytes.is_null() || mjcf_len == 0 {
        return status_result(MJCF_IMPORT_NULL_INPUT);
    }

    let bytes = unsafe { slice::from_raw_parts(mjcf_bytes, mjcf_len as usize) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return status_result(MJCF_IMPORT_INVALID_UTF8);
    };
    let Ok(model) = mjcf_rs::Model::from_str(text, ".") else {
        return status_result(MJCF_IMPORT_PARSE_ERROR);
    };

    import_model(
        world,
        model,
        options.into(),
        OutputBuffers {
            body_handles: out_body_handles,
            body_capacity,
            collider_handles: out_collider_handles,
            collider_capacity,
            joint_handles: out_joint_handles,
            joint_capacity,
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_mjcf_from_bytes(
    world: *mut WorldHandle,
    mjcf_bytes: *const u8,
    mjcf_len: u32,
    options: MjcfImportOptions,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
) -> MjcfImportResult {
    world_insert_mjcf_from_bytes_ex(
        world,
        mjcf_bytes,
        mjcf_len,
        options,
        out_body_handles,
        body_capacity,
        std::ptr::null_mut(),
        0,
        std::ptr::null_mut(),
        0,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_mjcf_from_bytes_default_ex(
    world: *mut WorldHandle,
    mjcf_bytes: *const u8,
    mjcf_len: u32,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
    out_collider_handles: *mut ColliderHandleRaw,
    collider_capacity: u32,
    out_joint_handles: *mut ImpulseJointHandleRaw,
    joint_capacity: u32,
) -> MjcfImportResult {
    world_insert_mjcf_from_bytes_ex(
        world,
        mjcf_bytes,
        mjcf_len,
        MjcfImportOptions::default(),
        out_body_handles,
        body_capacity,
        out_collider_handles,
        collider_capacity,
        out_joint_handles,
        joint_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_mjcf_from_bytes_default(
    world: *mut WorldHandle,
    mjcf_bytes: *const u8,
    mjcf_len: u32,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
) -> MjcfImportResult {
    world_insert_mjcf_from_bytes_default_ex(
        world,
        mjcf_bytes,
        mjcf_len,
        out_body_handles,
        body_capacity,
        std::ptr::null_mut(),
        0,
        std::ptr::null_mut(),
        0,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_mjcf_from_bytes_count(
    world: *mut WorldHandle,
    mjcf_bytes: *const u8,
    mjcf_len: u32,
) -> u32 {
    world_insert_mjcf_from_bytes_default(world, mjcf_bytes, mjcf_len, std::ptr::null_mut(), 0)
        .body_count
}

#[unsafe(no_mangle)]
pub extern "C" fn mjcf_import_status_ok(status: u32) -> Bool {
    (status == MJCF_IMPORT_OK).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::Vec3;
    use crate::world::{world_create, world_destroy};

    #[test]
    fn imports_primitive_mjcf_into_world() {
        let mjcf = br#"
<mujoco model="simple">
  <worldbody>
    <body name="base" pos="0 0 0">
      <geom type="box" size="0.5 0.5 0.5"/>
      <body name="tip" pos="0 0 1">
        <joint type="hinge" axis="0 0 1"/>
        <geom type="sphere" size="0.25"/>
      </body>
    </body>
  </worldbody>
</mujoco>
"#;
        let world = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        let mut body_handles = [0; 2];
        let mut collider_handles = [0; 2];
        let mut joint_handles = [0; 1];

        let result = world_insert_mjcf_from_bytes_default_ex(
            world,
            mjcf.as_ptr(),
            mjcf.len() as u32,
            body_handles.as_mut_ptr(),
            body_handles.len() as u32,
            collider_handles.as_mut_ptr(),
            collider_handles.len() as u32,
            joint_handles.as_mut_ptr(),
            joint_handles.len() as u32,
        );

        assert_eq!(result.status, MJCF_IMPORT_OK);
        assert_eq!(result.body_count, 2);
        assert_eq!(result.collider_count, 2);
        assert_eq!(result.joint_count, 1);
        assert_eq!(result.skipped_geom_count, 0);
        assert_ne!(body_handles[0], 0);
        assert_ne!(body_handles[1], 0);
        assert_ne!(collider_handles[0], 0);
        assert_ne!(collider_handles[1], 0);
        assert_ne!(joint_handles[0], 0);

        world_destroy(world);
    }
}
