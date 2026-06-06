use std::collections::{HashMap, VecDeque};
use std::slice;

use rapier3d::glamx::EulerRot;
use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::prelude::{
    ColliderBuilder, ColliderHandle, FixedJointBuilder, GenericJoint, PrismaticJointBuilder,
    RevoluteJointBuilder, RigidBodyBuilder, RigidBodyHandle, SharedShape,
};
use rapier3d_urdf::urdf_rs::{self, Geometry, JointType, Pose as UrdfPose};

use crate::ffi::{
    Bool, ColliderHandleRaw, ImpulseJointHandleRaw, RigidBodyHandleRaw, UrdfImportOptions,
    UrdfImportResult, WorldHandle, pack_collider_handle, pack_impulse_joint_handle,
    pack_rigid_body_handle,
};

const URDF_IMPORT_OK: u32 = 0;
const URDF_IMPORT_NULL_WORLD: u32 = 1;
const URDF_IMPORT_NULL_INPUT: u32 = 2;
const URDF_IMPORT_INVALID_UTF8: u32 = 3;
const URDF_IMPORT_PARSE_ERROR: u32 = 4;

#[derive(Clone, Copy, Debug)]
struct ImportOptions {
    create_collision_colliders: bool,
    create_visual_colliders: bool,
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

impl From<UrdfImportOptions> for ImportOptions {
    fn from(value: UrdfImportOptions) -> Self {
        let defaults = UrdfImportOptions::default();
        Self {
            create_collision_colliders: value.create_collision_colliders.as_bool(),
            create_visual_colliders: value.create_visual_colliders.as_bool(),
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

fn status_result(status: u32) -> UrdfImportResult {
    UrdfImportResult {
        status,
        ..Default::default()
    }
}

fn pose_from_urdf(pose: &UrdfPose, scale: f64) -> Pose {
    Pose::from_parts(
        Vector::new(
            pose.xyz[0] * scale,
            pose.xyz[1] * scale,
            pose.xyz[2] * scale,
        ),
        Rotation::from_euler(EulerRot::XYZ, pose.rpy[0], pose.rpy[1], pose.rpy[2]),
    )
}

fn shape_from_geometry(geometry: &Geometry, scale: f64) -> Option<SharedShape> {
    match geometry {
        Geometry::Box { size } => Some(SharedShape::cuboid(
            size[0] * scale * 0.5,
            size[1] * scale * 0.5,
            size[2] * scale * 0.5,
        )),
        Geometry::Cylinder { radius, length } => {
            Some(SharedShape::cylinder(length * scale * 0.5, radius * scale))
        }
        Geometry::Capsule { radius, length } => {
            Some(SharedShape::capsule_z(length * scale * 0.5, radius * scale))
        }
        Geometry::Sphere { radius } => Some(SharedShape::ball(radius * scale)),
        Geometry::Mesh { .. } => None,
    }
}

fn add_colliders_for_link(
    world: &mut WorldHandle,
    body: RigidBodyHandle,
    link: &urdf_rs::Link,
    options: ImportOptions,
) -> (Vec<ColliderHandle>, u32) {
    let mut handles = Vec::new();
    let mut collider_count = 0u32;
    let mut skipped_mesh_count = 0u32;

    if options.create_collision_colliders {
        for collision in &link.collision {
            let Some(shape) = shape_from_geometry(&collision.geometry, options.scale) else {
                skipped_mesh_count += 1;
                continue;
            };
            let collider = ColliderBuilder::new(shape)
                .position(pose_from_urdf(&collision.origin, options.scale))
                .density(options.density)
                .friction(options.friction)
                .restitution(options.restitution)
                .build();
            let handle =
                world
                    .inner
                    .colliders
                    .insert_with_parent(collider, body, &mut world.inner.bodies);
            handles.push(handle);
            collider_count += 1;
        }
    }

    if options.create_visual_colliders {
        for visual in &link.visual {
            let Some(shape) = shape_from_geometry(&visual.geometry, options.scale) else {
                skipped_mesh_count += 1;
                continue;
            };
            let collider = ColliderBuilder::new(shape)
                .position(pose_from_urdf(&visual.origin, options.scale))
                .density(options.density)
                .friction(options.friction)
                .restitution(options.restitution)
                .build();
            let handle =
                world
                    .inner
                    .colliders
                    .insert_with_parent(collider, body, &mut world.inner.bodies);
            handles.push(handle);
            collider_count += 1;
        }
    }

    debug_assert_eq!(collider_count as usize, handles.len());
    (handles, skipped_mesh_count)
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

fn import_robot(
    world: &mut WorldHandle,
    robot: urdf_rs::Robot,
    options: ImportOptions,
    output: OutputBuffers,
) -> UrdfImportResult {
    let mut parent_joint_by_child = HashMap::new();
    let mut child_joints_by_parent: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, joint) in robot.joints.iter().enumerate() {
        parent_joint_by_child.insert(joint.child.link.as_str(), index);
        child_joints_by_parent
            .entry(joint.parent.link.as_str())
            .or_default()
            .push(index);
    }

    let mut link_by_name = HashMap::new();
    for (index, link) in robot.links.iter().enumerate() {
        link_by_name.insert(link.name.as_str(), index);
    }

    let mut poses = vec![Pose::IDENTITY; robot.links.len()];
    let mut roots = Vec::new();
    for (index, link) in robot.links.iter().enumerate() {
        if !parent_joint_by_child.contains_key(link.name.as_str()) {
            roots.push(index);
        }
    }

    let mut queue = VecDeque::new();
    for root in roots.iter().copied() {
        queue.push_back(root);
    }
    while let Some(parent_index) = queue.pop_front() {
        let parent_link = &robot.links[parent_index];
        let Some(child_joints) = child_joints_by_parent.get(parent_link.name.as_str()) else {
            continue;
        };
        for joint_index in child_joints.iter().copied() {
            let joint = &robot.joints[joint_index];
            let Some(&child_index) = link_by_name.get(joint.child.link.as_str()) else {
                continue;
            };
            poses[child_index] = poses[parent_index] * pose_from_urdf(&joint.origin, options.scale);
            queue.push_back(child_index);
        }
    }

    let mut handles = Vec::with_capacity(robot.links.len());
    let mut collider_handles = Vec::new();
    let mut collider_count = 0u32;
    let mut skipped_mesh_count = 0u32;

    for (index, link) in robot.links.iter().enumerate() {
        let is_root = roots.contains(&index);
        let builder = if is_root && options.make_roots_fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        };
        let body = builder.pose(poses[index]).build();
        let handle = world.inner.bodies.insert(body);
        let (link_collider_handles, link_skipped_mesh_count) =
            add_colliders_for_link(world, handle, link, options);
        collider_count += link_collider_handles.len() as u32;
        skipped_mesh_count += link_skipped_mesh_count;
        collider_handles.extend(link_collider_handles);
        handles.push(handle);
    }

    let mut joint_handles = Vec::new();
    let mut joint_count = 0u32;
    for joint in &robot.joints {
        let (Some(&parent_index), Some(&child_index)) = (
            link_by_name.get(joint.parent.link.as_str()),
            link_by_name.get(joint.child.link.as_str()),
        ) else {
            continue;
        };

        let axis = Vector::new(
            joint.axis.xyz[0] * options.scale,
            joint.axis.xyz[1] * options.scale,
            joint.axis.xyz[2] * options.scale,
        )
        .normalize_or_zero();
        let axis = if axis == Vector::ZERO {
            Vector::X
        } else {
            axis
        };
        let frame1 = pose_from_urdf(&joint.origin, options.scale);

        match joint.joint_type {
            JointType::Fixed => {
                let joint = FixedJointBuilder::new()
                    .local_frame1(frame1)
                    .local_frame2(Pose::IDENTITY)
                    .build();
                let handle = world.inner.impulse_joints.insert(
                    handles[parent_index],
                    handles[child_index],
                    joint,
                    true,
                );
                joint_handles.push(handle);
                joint_count += 1;
            }
            JointType::Continuous | JointType::Revolute => {
                let mut builder = RevoluteJointBuilder::new(axis)
                    .local_anchor1(frame1.translation)
                    .local_anchor2(Vector::ZERO);
                if joint.joint_type == JointType::Revolute {
                    builder = builder.limits([joint.limit.lower, joint.limit.upper]);
                }
                let handle = world.inner.impulse_joints.insert(
                    handles[parent_index],
                    handles[child_index],
                    builder.build(),
                    true,
                );
                joint_handles.push(handle);
                joint_count += 1;
            }
            JointType::Prismatic => {
                let joint = PrismaticJointBuilder::new(axis)
                    .local_anchor1(frame1.translation)
                    .local_anchor2(Vector::ZERO)
                    .limits([
                        joint.limit.lower * options.scale,
                        joint.limit.upper * options.scale,
                    ])
                    .build();
                let handle = world.inner.impulse_joints.insert(
                    handles[parent_index],
                    handles[child_index],
                    joint,
                    true,
                );
                joint_handles.push(handle);
                joint_count += 1;
            }
            JointType::Floating | JointType::Planar | JointType::Spherical => {
                let basis = GenericJoint::complete_ang_frame(axis);
                let joint = rapier3d::prelude::SphericalJointBuilder::new()
                    .local_frame1(frame1 * Pose::from_rotation(basis))
                    .local_frame2(Pose::from_rotation(basis))
                    .build();
                let handle = world.inner.impulse_joints.insert(
                    handles[parent_index],
                    handles[child_index],
                    joint,
                    true,
                );
                joint_handles.push(handle);
                joint_count += 1;
            }
        }
    }

    write_packed_handles(
        &handles,
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

    UrdfImportResult {
        status: URDF_IMPORT_OK,
        body_count: handles.len() as u32,
        collider_count,
        joint_count,
        skipped_mesh_count,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn urdf_import_options_default() -> UrdfImportOptions {
    UrdfImportOptions::default()
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_urdf_from_bytes(
    world: *mut WorldHandle,
    urdf_bytes: *const u8,
    urdf_len: u32,
    options: UrdfImportOptions,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
) -> UrdfImportResult {
    world_insert_urdf_from_bytes_ex(
        world,
        urdf_bytes,
        urdf_len,
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
pub extern "C" fn world_insert_urdf_from_bytes_ex(
    world: *mut WorldHandle,
    urdf_bytes: *const u8,
    urdf_len: u32,
    options: UrdfImportOptions,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
    out_collider_handles: *mut ColliderHandleRaw,
    collider_capacity: u32,
    out_joint_handles: *mut ImpulseJointHandleRaw,
    joint_capacity: u32,
) -> UrdfImportResult {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return status_result(URDF_IMPORT_NULL_WORLD);
    };
    if urdf_bytes.is_null() || urdf_len == 0 {
        return status_result(URDF_IMPORT_NULL_INPUT);
    }

    let bytes = unsafe { slice::from_raw_parts(urdf_bytes, urdf_len as usize) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return status_result(URDF_IMPORT_INVALID_UTF8);
    };
    let Ok(robot) = urdf_rs::read_from_string(text) else {
        return status_result(URDF_IMPORT_PARSE_ERROR);
    };

    import_robot(
        world,
        robot,
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
pub extern "C" fn world_insert_urdf_from_bytes_default(
    world: *mut WorldHandle,
    urdf_bytes: *const u8,
    urdf_len: u32,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
) -> UrdfImportResult {
    world_insert_urdf_from_bytes_ex(
        world,
        urdf_bytes,
        urdf_len,
        UrdfImportOptions::default(),
        out_body_handles,
        body_capacity,
        std::ptr::null_mut(),
        0,
        std::ptr::null_mut(),
        0,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_urdf_from_bytes_default_ex(
    world: *mut WorldHandle,
    urdf_bytes: *const u8,
    urdf_len: u32,
    out_body_handles: *mut RigidBodyHandleRaw,
    body_capacity: u32,
    out_collider_handles: *mut ColliderHandleRaw,
    collider_capacity: u32,
    out_joint_handles: *mut ImpulseJointHandleRaw,
    joint_capacity: u32,
) -> UrdfImportResult {
    world_insert_urdf_from_bytes_ex(
        world,
        urdf_bytes,
        urdf_len,
        UrdfImportOptions::default(),
        out_body_handles,
        body_capacity,
        out_collider_handles,
        collider_capacity,
        out_joint_handles,
        joint_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn world_insert_urdf_from_bytes_count(
    world: *mut WorldHandle,
    urdf_bytes: *const u8,
    urdf_len: u32,
) -> u32 {
    world_insert_urdf_from_bytes_default(world, urdf_bytes, urdf_len, std::ptr::null_mut(), 0)
        .body_count
}

#[unsafe(no_mangle)]
pub extern "C" fn urdf_import_status_ok(status: u32) -> Bool {
    (status == URDF_IMPORT_OK).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::Vec3;
    use crate::world::world_create;

    #[test]
    fn imports_primitive_urdf_into_world() {
        let urdf = br#"
<robot name="two_link">
  <link name="base">
    <collision>
      <geometry><box size="1 1 1"/></geometry>
    </collision>
  </link>
  <link name="tip">
    <collision>
      <origin xyz="0 0 0.5"/>
      <geometry><sphere radius="0.25"/></geometry>
    </collision>
  </link>
  <joint name="base_to_tip" type="fixed">
    <origin xyz="0 0 1"/>
    <parent link="base"/>
    <child link="tip"/>
  </joint>
</robot>
"#;
        let world = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        let mut body_handles = [0; 2];
        let mut collider_handles = [0; 2];
        let mut joint_handles = [0; 1];

        let result = world_insert_urdf_from_bytes_default_ex(
            world,
            urdf.as_ptr(),
            urdf.len() as u32,
            body_handles.as_mut_ptr(),
            body_handles.len() as u32,
            collider_handles.as_mut_ptr(),
            collider_handles.len() as u32,
            joint_handles.as_mut_ptr(),
            joint_handles.len() as u32,
        );

        assert_eq!(result.status, URDF_IMPORT_OK);
        assert_eq!(result.body_count, 2);
        assert_eq!(result.collider_count, 2);
        assert_eq!(result.joint_count, 1);
        assert_eq!(result.skipped_mesh_count, 0);
        assert_ne!(body_handles[0], 0);
        assert_ne!(body_handles[1], 0);
        assert_ne!(collider_handles[0], 0);
        assert_ne!(collider_handles[1], 0);
        assert_ne!(joint_handles[0], 0);

        crate::world::world_destroy(world);
    }
}
