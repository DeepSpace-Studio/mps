pub type RigidBodyHandleRaw = u64;
pub type ColliderHandleRaw = u64;
pub type ImpulseJointHandleRaw = u64;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ShapeType {
    #[default]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum NeuralActivation {
    #[default]
    Relu = 0,
    Tanh = 1,
    Sin = 2,
    Linear = 3,
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
pub struct CoulombFrictionLaw {
    pub static_coefficient: f64,
    pub dynamic_coefficient: f64,
    pub velocity_threshold: f64,
    pub enabled: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AirDragLaw {
    pub fluid_velocity: Vec3,
    pub density: f64,
    pub dynamic_viscosity: f64,
    pub characteristic_length: f64,
    pub reference_area: f64,
    pub drag_coefficient: f64,
    pub reynolds_stokes_limit: f64,
    pub enabled: Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ExternalForceLaw {
    pub buoyancy_enabled: Bool,
    pub fluid_density: f64,
    pub displaced_volume: f64,
    pub buoyancy_gravity: Vec3,
    pub electromagnetic_enabled: Bool,
    pub charge: f64,
    pub electric_field: Vec3,
    pub magnetic_field: Vec3,
    pub elastic_enabled: Bool,
    pub spring_anchor: Vec3,
    pub spring_stiffness: f64,
    pub spring_damping: f64,
    pub gravity_enabled: Bool,
    pub gravity_source: Vec3,
    pub gravitational_parameter: f64,
    pub enabled: Bool,
}

/// Newtonian pairwise gravity configuration for body-body attraction.
///
/// When enabled, every dynamic body attracts every other dynamic body via
/// Newton's law:  F = G · m₁ · m₂ / r².
///
/// Set `gravitational_constant` to 6.67430e-11 for real-world gravity,
/// or a larger value for game-scale simulations.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NewtonGravityLaw {
    /// Gravitational constant (default: 6.67430e-11 N·m²/kg²).
    /// Use larger values for game-scale simulations.
    pub gravitational_constant: f64,
    /// Minimum distance to prevent division by zero (default: 0.01 m).
    pub min_distance: f64,
    /// Maximum distance for gravity to apply (0 = no limit).
    pub max_distance: f64,
    pub enabled: Bool,
}

impl Default for NewtonGravityLaw {
    fn default() -> Self {
        Self {
            gravitational_constant: 6.67430e-11,
            min_distance: 0.01,
            max_distance: 0.0,
            enabled: Bool::FALSE,
        }
    }
}
