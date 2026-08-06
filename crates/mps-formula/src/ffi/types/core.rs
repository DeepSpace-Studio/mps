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

// ===========================================================================
// PHYSICS_EXPANSION_PLAN C1 — config structs for new ForceLaw variants.
//
// Each of these is the C-ABI shape that `world_set_<name>_law` accepts from
// external callers (c, JNI, FFM).  They mirror the Rust private struct
// inside `mps-core/src/rapier/interaction.rs`; the mps-core FFI wrapper
// copies fields into the private law and registers it.
// ===========================================================================

/// Configure the solar-wind dynamic-pressure force law.
///
/// `proton_density` in protons/m³, `v_sw_mps` in m/s (world-frame bulk
/// speed along `wind_direction`), `effective_area_m2` is the body's apparent
/// disc area presented to the wind.  Push is along `wind_direction`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SolarWindPressureLaw {
    pub proton_density: f64,
    pub v_sw_mps: f64,
    pub wind_direction: Vec3,
    pub effective_area_m2: f64,
    pub enabled: Bool,
}

/// Configure the Chandrasekhar dynamical-friction force law.
///
/// `background_density_kg_m3` is the uniform ρ_bg; `coulomb_log` is ln Λ
/// (typical ~10).  When enabled, every dynamic Rapier body moving through
/// the background is decelerated opposite to its velocity.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DynamicalFrictionLaw {
    pub background_density_kg_m3: f64,
    pub coulomb_log: f64,
    pub enabled: Bool,
}

/// Configure the MOND-corrected gravity force law.
///
/// Callers supply `newtonian_a` (m/s²) — the Newtonian acceleration
/// magnitude toward the dominant attractor — along with `mond_a_zero`
/// (Milgrom scale, typical 1.2e-10 m/s²) and `direction` (world-frame
/// unit-vector toward the attractor).  When `a_N < a_0` the field is
/// boosted to `sqrt(a_N · a_0)`; otherwise the Newtonian value is used.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MonDGravityLaw {
    pub newtonian_a: f64,
    pub mond_a_zero: f64,
    pub direction: Vec3,
    pub enabled: Bool,
}

/// Configure the Eddington-limited radiation-pressure force law.
///
/// `mass_kg` is the accretor's mass (kg); `opacity` is the opacity κ in
/// m²/kg (electron-scattering for H ≈ 0.034); `source_position` is the
/// world-frame location of the luminous accretor; `effective_area_m2` is
/// each Rapier body's apparent optical cross-section; `enabled` toggles
/// the law.  When enabled, every dynamic Rapier body is pushed outward
/// from `source_position` with force `(L_Edd / (c·4π·r²)) · A_eff` where
/// `L_Edd = 4π G M c / κ` (see
/// `mps_formula::high_energy_astro::eddington_limited_luminosity`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EddingtonRadiationPressureLaw {
    /// Accretor mass [kg].
    pub mass_kg: f64,
    /// Opacity κ [m²/kg].
    pub opacity: f64,
    /// World-frame accretor position.
    pub source_position: Vec3,
    /// Body apparent optical cross-section area [m²].
    pub effective_area_m2: f64,
    pub enabled: Bool,
}

/// Configure the X-ray disc bolometric irradiation force law.
///
/// `k_t_eff_kev` is the inner-edge effective temperature `kT_eff` in keV;
/// `r_in_km` the inner disc radius in km; `spectral_hardening` f_col;
/// `source_position` the world-frame X-ray source location;
/// `effective_area_m2` each Rapier body's apparent optical cross-section;
/// `enabled` toggles the law.  When enabled, every dynamic Rapier body is
/// pushed outward from `source_position` with force
/// `(L_X / (c·4π·r²)) · A_eff` where
/// `L_X = L_SUN · xray_disc_bolometric_luminosity(kT, r_in, f_col)` (see
/// `mps_formula::high_energy_astro::xray_disc_bolometric_luminosity`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct XrayIrradiationLaw {
    /// Inner-edge effective temperature `kT_eff` [keV].
    pub k_t_eff_kev: f64,
    /// Inner disc radius [km].
    pub r_in_km: f64,
    /// Spectral hardening factor `f_col` (~1.7 for BH discs).
    pub spectral_hardening: f64,
    /// World-frame X-ray source position.
    pub source_position: Vec3,
    /// Body apparent optical cross-section area [m²].
    pub effective_area_m2: f64,
    pub enabled: Bool,
}

/// Configure the pulsar magnetic-dipole torque law.
///
/// `moment_of_inertia` [kg·m²], `ns_radius_m` [m], `period_ms` [ms],
/// `period_derivative` [s/s] describe the pulsar (used to compute its
/// surface B-field via `pulsar_surface_b_field`).  `pulsar_position` is its
/// world-frame location; `spin_axis` is the unit vector along its magnetic
/// (≈ rotation) axis; `body_dipole_moment` is the Rapier body's magnetic
/// dipole moment μ [A·m²] as a vector (direction = dipole axis,
/// magnitude = |μ|); `enabled` toggles the law.  When enabled, every
/// dynamic Rapier body at distance `r` from the pulsar experiences torque
/// `τ = μ × B(r)` with `B(r) = B_surf · (R_ns / r)³`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PulsarMagneticDipoleLaw {
    /// Pulsar moment of inertia [kg·m²].
    pub moment_of_inertia: f64,
    /// Neutron-star radius [m] (canonical 1e4 m = 10 km).
    pub ns_radius_m: f64,
    /// Spin period P [ms].
    pub period_ms: f64,
    /// Spin-down rate Ṗ [s/s].
    pub period_derivative: f64,
    /// World-frame pulsar position.
    pub pulsar_position: Vec3,
    /// Unit vector along the pulsar magnetic (≈ rotation) axis.
    pub spin_axis: Vec3,
    /// Body magnetic dipole moment μ [A·m²] — direction = dipole axis,
    /// magnitude = |μ|.
    pub body_dipole_moment: Vec3,
    pub enabled: Bool,
}

/// Configure the Jeans-escape drag force law.
///
/// `n_exo` [m⁻³], `temperature` [K], `escape_parameter` λ (dimensionless),
/// `mass_kg` molecule mass [kg] describe the exobase; `escape_direction` the
/// unit vector along the escape direction (radially outward); `effective_area_m2`
/// each Rapier body's apparent cross-section; `enabled` toggles the law.
/// When enabled, every dynamic Rapier body is pushed along `escape_direction`
/// with `F = (Φ · m · v_thermal) · A_eff`, where
/// `Φ = mps_formula::heliophysics::jeans_escape_flux(n_exo, T, λ, m)` and
/// `v_thermal = √(2 k_B T / m)`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JeansEscapeLaw {
    /// Exobase number density `n_exo` [m⁻³].
    pub n_exo: f64,
    /// Exobase temperature `T` [K].
    pub temperature: f64,
    /// Jeans escape parameter λ (dimensionless).
    pub escape_parameter: f64,
    /// Mass of the escaping molecule `m` [kg].
    pub mass_kg: f64,
    /// World-frame unit vector along the escape direction.
    pub escape_direction: Vec3,
    /// Body apparent cross-section area [m²].
    pub effective_area_m2: f64,
    pub enabled: Bool,
}
