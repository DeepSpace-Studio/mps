use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::parry::query::ShapeCastOptions;
use rapier3d::parry::shape::SharedShape;
use rapier3d::prelude::{
    ActiveEvents, ActiveHooks, ColliderHandle, Group,
    ImpulseJointHandle as RapierImpulseJointHandle, InteractionGroups, InteractionTestMode,
    JointAxis, QueryFilter, QueryFilterFlags, RigidBodyHandle,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Quat {
    pub i: f64,
    pub j: f64,
    pub k: f64,
    pub w: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Bool(pub u8);

impl Bool {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(1);
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyStatus {
    Dynamic = 0,
    Fixed = 1,
    KinematicPositionBased = 2,
    KinematicVelocityBased = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeType {
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoxelColliderMode {
    Auto = 0,
    Cuboids = 1,
    GreedyCuboids = 2,
    SurfaceMesh = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelColliderOptions {
    pub mode: u32,
    pub dynamic_body: Bool,
    pub small_voxel_limit: u32,
    pub mesh_voxel_limit: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelBuildStats {
    pub cell_count: u32,
    pub solid_count: u32,
    pub selected_mode: u32,
    pub estimated_parts: u32,
    pub estimated_vertices: u32,
    pub estimated_triangles: u32,
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
}

impl Default for VoxelColliderOptions {
    fn default() -> Self {
        Self {
            mode: VoxelColliderMode::Auto as u32,
            dynamic_body: Bool::FALSE,
            small_voxel_limit: 128,
            mesh_voxel_limit: 20_000,
        }
    }
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::Ball
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeDesc {
    pub shape_type: u32,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InteractionGroupsDesc {
    pub memberships: u32,
    pub filter: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct QueryFilterDesc {
    pub flags: u32,
    pub groups: InteractionGroupsDesc,
    pub use_groups: Bool,
    pub exclude_collider: ColliderHandleRaw,
    pub use_exclude_collider: Bool,
    pub exclude_rigid_body: RigidBodyHandleRaw,
    pub use_exclude_rigid_body: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeCastOptionsDesc {
    pub max_time_of_impact: f64,
    pub target_distance: f64,
    pub stop_at_penetration: Bool,
    pub compute_impact_geometry_on_penetration: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PointProjection {
    pub point: Vec3,
    pub is_inside: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RayHit {
    pub collider: ColliderHandleRaw,
    pub time_of_impact: f64,
    pub normal: Vec3,
    pub feature: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeCastHit {
    pub collider: ColliderHandleRaw,
    pub time_of_impact: f64,
    pub witness1: Vec3,
    pub witness2: Vec3,
    pub normal1: Vec3,
    pub normal2: Vec3,
    pub status: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AabbDesc {
    pub mins: Vec3,
    pub maxs: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Obb {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub rotation: Quat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Capsule {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Ssv {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Ellipsoid {
    pub center: Vec3,
    pub radii: Vec3,
    pub rotation: Quat,
    pub segments: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Prism {
    pub center: Vec3,
    pub radius: f64,
    pub half_height: f64,
    pub sides: u32,
    pub rotation: Quat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Cylinder {
    pub center: Vec3,
    pub radius: f64,
    pub half_height: f64,
    pub rotation: Quat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SphericalShell {
    pub center: Vec3,
    pub inner_radius: f64,
    pub outer_radius: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeuralActivation {
    Relu = 0,
    Tanh = 1,
    Sin = 2,
    Linear = 3,
}

impl Default for NeuralActivation {
    fn default() -> Self {
        Self::Relu
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NeuralBoundsDesc {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub rotation: Quat,
    pub sample_resolution: u32,
    pub hidden_width: u32,
    pub hidden_layers: u32,
    pub activation: u32,
    pub output_scale: f64,
    pub padding: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KdopPreset {
    K6 = 6,
    K14 = 14,
    K18 = 18,
    K26 = 26,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EffectiveCharacterMovement {
    pub translation: Vec3,
    pub grounded: Bool,
    pub is_sliding_down_slope: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CharacterCollision {
    pub collider: ColliderHandleRaw,
    pub character_translation: Vec3,
    pub translation_applied: Vec3,
    pub translation_remaining: Vec3,
    pub world_witness1: Vec3,
    pub world_witness2: Vec3,
    pub normal1: Vec3,
    pub normal2: Vec3,
    pub time_of_impact: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CollisionEventRecord {
    pub started: Bool,
    pub collider1: ColliderHandleRaw,
    pub collider2: ColliderHandleRaw,
    pub sensor: Bool,
    pub removed: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ContactForceEventRecord {
    pub collider1: ColliderHandleRaw,
    pub collider2: ColliderHandleRaw,
    pub total_force: Vec3,
    pub total_force_magnitude: f64,
    pub max_force_direction: Vec3,
    pub max_force_magnitude: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AeroSurface {
    pub point: Vec3,
    pub normal: Vec3,
    pub area: f64,
    pub drag_coefficient: f64,
    pub lift_coefficient: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AeroForceReport {
    pub total_force: Vec3,
    pub total_torque: Vec3,
    pub surface_count: u32,
    pub active_surface_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FluidVolume {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub density: f64,
    pub linear_drag: f64,
    pub quadratic_drag: f64,
    pub angular_drag: f64,
    pub flow_velocity: Vec3,
    pub gravity: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FluidForceReport {
    pub buoyancy_force: Vec3,
    pub drag_force: Vec3,
    pub angular_damping_torque: Vec3,
    pub total_force: Vec3,
    pub total_torque: Vec3,
    pub submerged_fraction: f64,
    pub displaced_volume: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryState {
    pub position: Vec3,
    pub velocity: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryEnvironment {
    pub gravity: Vec3,
    pub flow_velocity: Vec3,
    pub mass: f64,
    pub reference_area: f64,
    pub density: f64,
    pub drag_coefficient: f64,
    pub lift_coefficient: f64,
    pub lift_direction: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TrajectoryForceReport {
    pub gravity_force: Vec3,
    pub drag_force: Vec3,
    pub lift_force: Vec3,
    pub total_force: Vec3,
    pub acceleration: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JointAxisDesc {
    LinX = 0,
    LinY = 1,
    LinZ = 2,
    AngX = 3,
    AngY = 4,
    AngZ = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JointTypeDesc {
    Fixed = 0,
    Revolute = 1,
    Prismatic = 2,
    Rope = 3,
    Spring = 4,
    Spherical = 5,
}

pub struct WorldHandle {
    pub(crate) inner: crate::rapier::world::PhysicsWorld,
}

pub struct AnvilKitAppHandle {
    pub(crate) inner: crate::rapier::anvilkit::AnvilKitAppState,
}

pub struct RigidBodyBuilderHandle {
    pub(crate) inner: rapier3d::prelude::RigidBodyBuilder,
}

pub struct ColliderBuilderHandle {
    pub(crate) inner: rapier3d::prelude::ColliderBuilder,
}

pub struct JointBuilderHandle {
    pub(crate) inner: crate::rapier::joints::JointBuilderKind,
}

pub struct CharacterControllerHandle {
    pub(crate) inner: crate::rapier::controller::CharacterControllerState,
}

pub struct RTreeHandle {
    pub(crate) inner: crate::rapier::rtree::RTreeIndex,
}

pub struct CRbTreeHandle {
    pub(crate) inner: crate::rapier::crbtree::CRbTreeIndex,
}

pub type RigidBodyHandleRaw = u64;
pub type ColliderHandleRaw = u64;
pub type ImpulseJointHandleRaw = u64;

pub(crate) const MAX_OUTPUT_CAPACITY: u32 = 1_000_000;
pub(crate) const MAX_TREE_ENTRIES: usize = 1_000_000;

const INVALID_HANDLE_RAW: u64 = u64::MAX;

fn pack_handle_parts(id: u32, generation: u32) -> u64 {
    (((generation as u64) << 32) | (id as u64)).wrapping_add(1)
}

fn unpack_handle_parts(handle: u64) -> (u32, u32) {
    let raw = handle.checked_sub(1).unwrap_or(INVALID_HANDLE_RAW);
    ((raw & 0xffff_ffff) as u32, (raw >> 32) as u32)
}

pub(crate) fn vec3_to_rapier(value: Vec3) -> Vector {
    Vector::new(value.x, value.y, value.z)
}

pub(crate) fn vec3_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

pub(crate) fn vec3_from_rapier(value: Vector) -> Vec3 {
    Vec3 {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

pub(crate) fn quat_to_rapier(value: Quat) -> Rotation {
    Rotation::from_xyzw(value.i, value.j, value.k, value.w)
}

pub(crate) fn quat_finite(value: Quat) -> bool {
    value.i.is_finite() && value.j.is_finite() && value.k.is_finite() && value.w.is_finite()
}

pub(crate) fn quat_from_rapier(value: Rotation) -> Quat {
    Quat {
        i: value.x,
        j: value.y,
        k: value.z,
        w: value.w,
    }
}

pub(crate) fn isometry_from_parts(translation: Vec3, rotation: Quat) -> Pose {
    Pose::from_parts(vec3_to_rapier(translation), quat_to_rapier(rotation))
}

pub(crate) fn pack_rigid_body_handle(handle: RigidBodyHandle) -> RigidBodyHandleRaw {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_rigid_body_handle(handle: RigidBodyHandleRaw) -> RigidBodyHandle {
    let (id, generation) = unpack_handle_parts(handle);
    RigidBodyHandle::from_raw_parts(id, generation)
}

pub(crate) fn pack_collider_handle(handle: ColliderHandle) -> ColliderHandleRaw {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_collider_handle(handle: ColliderHandleRaw) -> ColliderHandle {
    let (id, generation) = unpack_handle_parts(handle);
    ColliderHandle::from_raw_parts(id, generation)
}

pub(crate) fn pack_impulse_joint_handle(handle: RapierImpulseJointHandle) -> ImpulseJointHandleRaw {
    let (id, generation) = handle.into_raw_parts();
    pack_handle_parts(id, generation)
}

pub(crate) fn unpack_impulse_joint_handle(
    handle: ImpulseJointHandleRaw,
) -> RapierImpulseJointHandle {
    let (id, generation) = unpack_handle_parts(handle);
    RapierImpulseJointHandle::from_raw_parts(id, generation)
}

pub(crate) fn body_status_to_rapier(status: BodyStatus) -> rapier3d::prelude::RigidBodyType {
    match status {
        BodyStatus::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
        BodyStatus::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
        BodyStatus::KinematicPositionBased => {
            rapier3d::prelude::RigidBodyType::KinematicPositionBased
        }
        BodyStatus::KinematicVelocityBased => {
            rapier3d::prelude::RigidBodyType::KinematicVelocityBased
        }
    }
}

pub(crate) fn body_status_from_raw(value: u32) -> BodyStatus {
    match value {
        0 => BodyStatus::Dynamic,
        1 => BodyStatus::Fixed,
        2 => BodyStatus::KinematicPositionBased,
        3 => BodyStatus::KinematicVelocityBased,
        _ => BodyStatus::Fixed,
    }
}

pub(crate) fn body_status_from_rapier(status: rapier3d::prelude::RigidBodyType) -> BodyStatus {
    match status {
        rapier3d::prelude::RigidBodyType::Dynamic => BodyStatus::Dynamic,
        rapier3d::prelude::RigidBodyType::Fixed => BodyStatus::Fixed,
        rapier3d::prelude::RigidBodyType::KinematicPositionBased => {
            BodyStatus::KinematicPositionBased
        }
        rapier3d::prelude::RigidBodyType::KinematicVelocityBased => {
            BodyStatus::KinematicVelocityBased
        }
    }
}

pub(crate) fn body_status_to_raw(status: BodyStatus) -> u32 {
    status as u32
}

pub(crate) fn shape_type_from_raw(value: u32) -> ShapeType {
    match value {
        1 => ShapeType::Cuboid,
        2 => ShapeType::CapsuleY,
        3 => ShapeType::CapsuleX,
        4 => ShapeType::CapsuleZ,
        5 => ShapeType::Cylinder,
        6 => ShapeType::RoundCylinder,
        7 => ShapeType::Cone,
        8 => ShapeType::RoundCone,
        9 => ShapeType::RoundCuboid,
        _ => ShapeType::Ball,
    }
}

pub(crate) fn shape_from_desc(desc: ShapeDesc) -> SharedShape {
    match shape_type_from_raw(desc.shape_type) {
        ShapeType::Ball => SharedShape::ball(desc.a),
        ShapeType::Cuboid => SharedShape::cuboid(desc.a, desc.b, desc.c),
        ShapeType::CapsuleY => SharedShape::capsule_y(desc.a, desc.b),
        ShapeType::CapsuleX => SharedShape::capsule_x(desc.a, desc.b),
        ShapeType::CapsuleZ => SharedShape::capsule_z(desc.a, desc.b),
        ShapeType::Cylinder => SharedShape::cylinder(desc.a, desc.b),
        ShapeType::RoundCylinder => SharedShape::round_cylinder(desc.a, desc.b, desc.c),
        ShapeType::Cone => SharedShape::cone(desc.a, desc.b),
        ShapeType::RoundCone => SharedShape::round_cone(desc.a, desc.b, desc.c),
        ShapeType::RoundCuboid => SharedShape::round_cuboid(desc.a, desc.b, desc.c, desc.d),
    }
}

pub(crate) fn shape_desc_valid(desc: ShapeDesc) -> bool {
    if !desc.a.is_finite() || !desc.b.is_finite() || !desc.c.is_finite() || !desc.d.is_finite() {
        return false;
    }

    match shape_type_from_raw(desc.shape_type) {
        ShapeType::Ball => desc.a > 0.0,
        ShapeType::Cuboid => desc.a > 0.0 && desc.b > 0.0 && desc.c > 0.0,
        ShapeType::CapsuleY | ShapeType::CapsuleX | ShapeType::CapsuleZ => {
            desc.a > 0.0 && desc.b > 0.0
        }
        ShapeType::Cylinder | ShapeType::Cone => desc.a > 0.0 && desc.b > 0.0,
        ShapeType::RoundCylinder | ShapeType::RoundCone => {
            desc.a > 0.0 && desc.b > 0.0 && desc.c >= 0.0
        }
        ShapeType::RoundCuboid => desc.a > 0.0 && desc.b > 0.0 && desc.c > 0.0 && desc.d >= 0.0,
    }
}

pub(crate) fn voxel_collider_mode_from_raw(value: u32) -> VoxelColliderMode {
    match value {
        1 => VoxelColliderMode::Cuboids,
        2 => VoxelColliderMode::GreedyCuboids,
        3 => VoxelColliderMode::SurfaceMesh,
        _ => VoxelColliderMode::Auto,
    }
}

pub(crate) fn neural_activation_from_raw(value: u32) -> NeuralActivation {
    match value {
        1 => NeuralActivation::Tanh,
        2 => NeuralActivation::Sin,
        3 => NeuralActivation::Linear,
        _ => NeuralActivation::Relu,
    }
}

pub(crate) fn kdop_preset_from_raw(value: u32) -> KdopPreset {
    match value {
        14 => KdopPreset::K14,
        18 => KdopPreset::K18,
        26 => KdopPreset::K26,
        _ => KdopPreset::K6,
    }
}

pub(crate) fn joint_type_from_raw(value: u32) -> JointTypeDesc {
    match value {
        1 => JointTypeDesc::Revolute,
        2 => JointTypeDesc::Prismatic,
        3 => JointTypeDesc::Rope,
        4 => JointTypeDesc::Spring,
        5 => JointTypeDesc::Spherical,
        _ => JointTypeDesc::Fixed,
    }
}

pub(crate) fn interaction_groups_to_rapier(groups: InteractionGroupsDesc) -> InteractionGroups {
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

pub(crate) fn query_filter_from_desc(desc: QueryFilterDesc) -> QueryFilter<'static> {
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

pub(crate) fn shape_cast_options_to_rapier(options: ShapeCastOptionsDesc) -> ShapeCastOptions {
    ShapeCastOptions {
        max_time_of_impact: options.max_time_of_impact,
        target_distance: options.target_distance,
        stop_at_penetration: options.stop_at_penetration.0 != 0,
        compute_impact_geometry_on_penetration: options.compute_impact_geometry_on_penetration.0
            != 0,
    }
}

pub(crate) fn joint_axis_to_rapier(axis: JointAxisDesc) -> JointAxis {
    match axis {
        JointAxisDesc::LinX => JointAxis::LinX,
        JointAxisDesc::LinY => JointAxis::LinY,
        JointAxisDesc::LinZ => JointAxis::LinZ,
        JointAxisDesc::AngX => JointAxis::AngX,
        JointAxisDesc::AngY => JointAxis::AngY,
        JointAxisDesc::AngZ => JointAxis::AngZ,
    }
}

pub(crate) fn joint_axis_from_raw(value: u32) -> JointAxisDesc {
    match value {
        1 => JointAxisDesc::LinY,
        2 => JointAxisDesc::LinZ,
        3 => JointAxisDesc::AngX,
        4 => JointAxisDesc::AngY,
        5 => JointAxisDesc::AngZ,
        _ => JointAxisDesc::LinX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_arena_handles_reserve_zero_for_null() {
        let body = RigidBodyHandle::from_raw_parts(0, 0);
        let collider = ColliderHandle::from_raw_parts(0, 0);
        let joint = RapierImpulseJointHandle::from_raw_parts(0, 0);

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
