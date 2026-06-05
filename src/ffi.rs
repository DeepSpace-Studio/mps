use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::parry::query::ShapeCastOptions;
use rapier3d::parry::shape::SharedShape;
use rapier3d::prelude::{
    ActiveEvents, ActiveHooks, ColliderHandle, Group, ImpulseJointHandle, InteractionGroups,
    InteractionTestMode, JointAxis, QueryFilter, QueryFilterFlags, RigidBodyHandle,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcQuat {
    pub i: f64,
    pub j: f64,
    pub k: f64,
    pub w: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RcBool(pub u8);

impl RcBool {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(1);
}

impl From<bool> for RcBool {
    fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RcBodyStatus {
    Dynamic = 0,
    Fixed = 1,
    KinematicPositionBased = 2,
    KinematicVelocityBased = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RcShapeType {
    Ball = 0,
    Cuboid = 1,
    CapsuleY = 2,
    CapsuleX = 3,
    CapsuleZ = 4,
    Cylinder = 5,
    RoundCylinder = 6,
    Cone = 7,
    RoundCone = 8,
    RoundCuboid = 9,
}

impl Default for RcShapeType {
    fn default() -> Self {
        Self::Ball
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcShapeDesc {
    pub shape_type: RcShapeType,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcInteractionGroups {
    pub memberships: u32,
    pub filter: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcQueryFilterDesc {
    pub flags: u32,
    pub groups: RcInteractionGroups,
    pub use_groups: RcBool,
    pub exclude_collider: RcColliderHandle,
    pub use_exclude_collider: RcBool,
    pub exclude_rigid_body: RcRigidBodyHandle,
    pub use_exclude_rigid_body: RcBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcShapeCastOptions {
    pub max_time_of_impact: f64,
    pub target_distance: f64,
    pub stop_at_penetration: RcBool,
    pub compute_impact_geometry_on_penetration: RcBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcPointProjection {
    pub point: RcVec3,
    pub is_inside: RcBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcRayHit {
    pub collider: RcColliderHandle,
    pub time_of_impact: f64,
    pub normal: RcVec3,
    pub feature: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcShapeCastHit {
    pub collider: RcColliderHandle,
    pub time_of_impact: f64,
    pub witness1: RcVec3,
    pub witness2: RcVec3,
    pub normal1: RcVec3,
    pub normal2: RcVec3,
    pub status: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcAabb {
    pub mins: RcVec3,
    pub maxs: RcVec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcEffectiveCharacterMovement {
    pub translation: RcVec3,
    pub grounded: RcBool,
    pub is_sliding_down_slope: RcBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcCharacterCollision {
    pub collider: RcColliderHandle,
    pub character_translation: RcVec3,
    pub translation_applied: RcVec3,
    pub translation_remaining: RcVec3,
    pub world_witness1: RcVec3,
    pub world_witness2: RcVec3,
    pub normal1: RcVec3,
    pub normal2: RcVec3,
    pub time_of_impact: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcCollisionEventRecord {
    pub started: RcBool,
    pub collider1: RcColliderHandle,
    pub collider2: RcColliderHandle,
    pub sensor: RcBool,
    pub removed: RcBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RcContactForceEventRecord {
    pub collider1: RcColliderHandle,
    pub collider2: RcColliderHandle,
    pub total_force: RcVec3,
    pub total_force_magnitude: f64,
    pub max_force_direction: RcVec3,
    pub max_force_magnitude: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RcJointAxis {
    LinX = 0,
    LinY = 1,
    LinZ = 2,
    AngX = 3,
    AngY = 4,
    AngZ = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RcJointType {
    Fixed = 0,
    Revolute = 1,
    Prismatic = 2,
    Rope = 3,
    Spring = 4,
    Spherical = 5,
}

pub struct RcWorldHandle {
    pub(crate) inner: crate::world::PhysicsWorld,
}

pub struct RcRigidBodyBuilderHandle {
    pub(crate) inner: rapier3d::prelude::RigidBodyBuilder,
}

pub struct RcColliderBuilderHandle {
    pub(crate) inner: rapier3d::prelude::ColliderBuilder,
}

pub struct RcJointBuilderHandle {
    pub(crate) inner: crate::joints::JointBuilderKind,
}

pub struct RcCharacterControllerHandle {
    pub(crate) inner: crate::controller::CharacterControllerState,
}

pub type RcRigidBodyHandle = u64;
pub type RcColliderHandle = u64;
pub type RcImpulseJointHandle = u64;

const INVALID_HANDLE_RAW: u64 = u64::MAX;

fn pack_handle_parts(id: u32, generation: u32) -> u64 {
    (((generation as u64) << 32) | (id as u64)).wrapping_add(1)
}

fn unpack_handle_parts(handle: u64) -> (u32, u32) {
    let raw = handle.checked_sub(1).unwrap_or(INVALID_HANDLE_RAW);
    ((raw & 0xffff_ffff) as u32, (raw >> 32) as u32)
}

pub(crate) fn vec3_to_rapier(value: RcVec3) -> Vector {
    Vector::new(value.x, value.y, value.z)
}

pub(crate) fn vec3_from_rapier(value: Vector) -> RcVec3 {
    RcVec3 {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

pub(crate) fn quat_to_rapier(value: RcQuat) -> Rotation {
    Rotation::from_xyzw(value.i, value.j, value.k, value.w)
}

pub(crate) fn quat_from_rapier(value: Rotation) -> RcQuat {
    RcQuat {
        i: value.x,
        j: value.y,
        k: value.z,
        w: value.w,
    }
}

pub(crate) fn isometry_from_parts(translation: RcVec3, rotation: RcQuat) -> Pose {
    Pose::from_parts(vec3_to_rapier(translation), quat_to_rapier(rotation))
}

pub(crate) fn pack_rigid_body_handle(handle: RigidBodyHandle) -> RcRigidBodyHandle {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_rigid_body_handle(handle: RcRigidBodyHandle) -> RigidBodyHandle {
    let (id, generation) = unpack_handle_parts(handle);
    RigidBodyHandle::from_raw_parts(id, generation)
}

pub(crate) fn pack_collider_handle(handle: ColliderHandle) -> RcColliderHandle {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_collider_handle(handle: RcColliderHandle) -> ColliderHandle {
    let (id, generation) = unpack_handle_parts(handle);
    ColliderHandle::from_raw_parts(id, generation)
}

pub(crate) fn pack_impulse_joint_handle(handle: ImpulseJointHandle) -> RcImpulseJointHandle {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_impulse_joint_handle(handle: RcImpulseJointHandle) -> ImpulseJointHandle {
    let (id, generation) = unpack_handle_parts(handle);
    ImpulseJointHandle::from_raw_parts(id, generation)
}

pub(crate) fn body_status_to_rapier(status: RcBodyStatus) -> rapier3d::prelude::RigidBodyType {
    match status {
        RcBodyStatus::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
        RcBodyStatus::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
        RcBodyStatus::KinematicPositionBased => {
            rapier3d::prelude::RigidBodyType::KinematicPositionBased
        }
        RcBodyStatus::KinematicVelocityBased => {
            rapier3d::prelude::RigidBodyType::KinematicVelocityBased
        }
    }
}

pub(crate) fn body_status_from_rapier(status: rapier3d::prelude::RigidBodyType) -> RcBodyStatus {
    match status {
        rapier3d::prelude::RigidBodyType::Dynamic => RcBodyStatus::Dynamic,
        rapier3d::prelude::RigidBodyType::Fixed => RcBodyStatus::Fixed,
        rapier3d::prelude::RigidBodyType::KinematicPositionBased => {
            RcBodyStatus::KinematicPositionBased
        }
        rapier3d::prelude::RigidBodyType::KinematicVelocityBased => {
            RcBodyStatus::KinematicVelocityBased
        }
    }
}

pub(crate) fn shape_from_desc(desc: RcShapeDesc) -> SharedShape {
    match desc.shape_type {
        RcShapeType::Ball => SharedShape::ball(desc.a),
        RcShapeType::Cuboid => SharedShape::cuboid(desc.a, desc.b, desc.c),
        RcShapeType::CapsuleY => SharedShape::capsule_y(desc.a, desc.b),
        RcShapeType::CapsuleX => SharedShape::capsule_x(desc.a, desc.b),
        RcShapeType::CapsuleZ => SharedShape::capsule_z(desc.a, desc.b),
        RcShapeType::Cylinder => SharedShape::cylinder(desc.a, desc.b),
        RcShapeType::RoundCylinder => SharedShape::round_cylinder(desc.a, desc.b, desc.c),
        RcShapeType::Cone => SharedShape::cone(desc.a, desc.b),
        RcShapeType::RoundCone => SharedShape::round_cone(desc.a, desc.b, desc.c),
        RcShapeType::RoundCuboid => SharedShape::round_cuboid(desc.a, desc.b, desc.c, desc.d),
    }
}

pub(crate) fn interaction_groups_to_rapier(groups: RcInteractionGroups) -> InteractionGroups {
    InteractionGroups::new(
        Group::from_bits_truncate(groups.memberships),
        Group::from_bits_truncate(groups.filter),
        InteractionTestMode::And,
    )
}

pub(crate) fn active_events_from_bits(bits: u32) -> ActiveEvents {
    ActiveEvents::from_bits_truncate(bits)
}

pub(crate) fn active_hooks_from_bits(bits: u32) -> ActiveHooks {
    ActiveHooks::from_bits_truncate(bits)
}

pub(crate) fn query_filter_from_desc(desc: RcQueryFilterDesc) -> QueryFilter<'static> {
    let mut filter = QueryFilter::from(QueryFilterFlags::from_bits_truncate(desc.flags));

    if desc.use_groups.0 != 0 {
        filter = filter.groups(interaction_groups_to_rapier(desc.groups));
    }
    if desc.use_exclude_collider.0 != 0 {
        filter = filter.exclude_collider(unpack_collider_handle(desc.exclude_collider));
    }
    if desc.use_exclude_rigid_body.0 != 0 {
        filter = filter.exclude_rigid_body(unpack_rigid_body_handle(desc.exclude_rigid_body));
    }

    filter
}

pub(crate) fn shape_cast_options_to_rapier(options: RcShapeCastOptions) -> ShapeCastOptions {
    ShapeCastOptions {
        max_time_of_impact: options.max_time_of_impact,
        target_distance: options.target_distance,
        stop_at_penetration: options.stop_at_penetration.0 != 0,
        compute_impact_geometry_on_penetration: options.compute_impact_geometry_on_penetration.0
            != 0,
    }
}

pub(crate) fn joint_axis_to_rapier(axis: RcJointAxis) -> JointAxis {
    match axis {
        RcJointAxis::LinX => JointAxis::LinX,
        RcJointAxis::LinY => JointAxis::LinY,
        RcJointAxis::LinZ => JointAxis::LinZ,
        RcJointAxis::AngX => JointAxis::AngX,
        RcJointAxis::AngY => JointAxis::AngY,
        RcJointAxis::AngZ => JointAxis::AngZ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_arena_handles_reserve_zero_for_null() {
        let body = RigidBodyHandle::from_raw_parts(0, 0);
        let collider = ColliderHandle::from_raw_parts(0, 0);
        let joint = ImpulseJointHandle::from_raw_parts(0, 0);

        assert_ne!(pack_rigid_body_handle(body), 0);
        assert_ne!(pack_collider_handle(collider), 0);
        assert_ne!(pack_impulse_joint_handle(joint), 0);

        assert_eq!(
            unpack_rigid_body_handle(pack_rigid_body_handle(body)).into_raw_parts(),
            (0, 0)
        );
        assert_eq!(
            unpack_collider_handle(pack_collider_handle(collider)).into_raw_parts(),
            (0, 0)
        );
        assert_eq!(
            unpack_impulse_joint_handle(pack_impulse_joint_handle(joint)).into_raw_parts(),
            (0, 0)
        );
    }
}
