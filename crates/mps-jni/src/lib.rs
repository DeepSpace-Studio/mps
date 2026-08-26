mod helper;

use crate::helper::jbytearray_to_array;
use ljni::JNIEnv;
use ljni::sys::{jbyte, jbyteArray, jclass, jdouble, jdoubleArray, jint, jlong, jstring};
#[cfg(feature = "anvilkit-bridge")]
use mps_core::rapier::anvilkit as ak;
#[cfg(feature = "anvilkit-bridge")]
use mps_core::rapier::ffi::AnvilKitAppHandle as AKH;
use mps_core::rapier::ffi::{
    AabbDesc, AeroForceReport, AeroSurface, AirDragLaw, Bool, CRbTreeHandle as CRTH, Capsule,
    CharacterCollision, CharacterControllerHandle as CCH, ColliderBuilderHandle as CBH,
    ColliderHandleRaw as CRaw, CollisionEventRecord as CER, ContactForceEventRecord,
    CoulombFrictionLaw, Cylinder, DynamicalFrictionLaw, EddingtonRadiationPressureLaw,
    EffectiveCharacterMovement, Ellipsoid, ExternalForceLaw, FluidForceReport, FluidVolume,
    FractureEnergyReport, FractureFragmentDesc, FractureMaterial, FractureModeReport,
    FractureReplaceReport, GriffithReport, HohmannTransfer, ImpulseJointHandleRaw as JRaw,
    InteractionGroupsDesc, JeansEscapeLaw, JointBuilderHandle as JBH, MinerDamageReport,
    MolecularForceLaw, MolecularPairReport, MolecularParticle, MonDGravityLaw, NeuralBoundsDesc,
    NewtonGravityLaw, Obb, PointProjection, Prism, PulsarMagneticDipoleLaw, Quat,
    QuaternionDerivative, QueryFilterDesc, RTreeHandle as RTH, RayHit,
    RigidBodyBuilderHandle as RBH, RigidBodyHandleRaw as RRaw, ScalarKalman, ShapeCastHit,
    ShapeCastOptionsDesc, ShapeDesc, SnCurveReport, SolarWindPressureLaw, Sphere, SphericalShell,
    Ssv, StressIntensityReport, TrajectoryEnvironment, TrajectoryForceReport, Vec3,
    VoxelBuildStats, VoxelColliderOptions, WorldHandle as WH, XrayIrradiationLaw,
};
use mps_core::rapier::{
    bounds as bo, collider as col, compat as com, controller as cc, crbtree as crt, dop,
    error as er, events as ev, fracture as fr, joints as jo, matmech as mm, molecular as mol,
    neural as neu, query as qu, rigid_body as rb, rtree as rt, soft_body as sb, spaceflight as sf,
    thermo as th, voxel as vx, world as wo,
};
use mps_core::rapier3d::prelude::{Collider as CB, RigidBody as RB};
use mps_ffm as abi;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn to_jlong<T>(value: *mut T) -> jlong {
    value as isize as jlong
}

fn to_jint(value: usize) -> jlong {
    value as jlong
}

fn m<T>(value: jlong) -> *mut T {
    value as isize as *mut T
}

fn cp<T>(value: jlong) -> *const T {
    value as isize as *const T
}

fn p<T>(value: jlong) -> *const T {
    value as isize as *const T
}

fn pm<T>(value: jlong) -> *mut T {
    value as isize as *mut T
}

fn jb(value: jint) -> Bool {
    Bool((value != 0) as u8)
}

fn u32_from_jint(value: jint) -> u32 {
    u32::try_from(value).unwrap_or(0)
}

fn v3(x: jdouble, y: jdouble, z: jdouble) -> Vec3 {
    Vec3 { x, y, z }
}

fn qt(i: jdouble, j: jdouble, k: jdouble, w: jdouble) -> Quat {
    Quat { i, j, k, w }
}

fn grp(memberships: jint, filter: jint) -> InteractionGroupsDesc {
    InteractionGroupsDesc {
        memberships: memberships as u32,
        filter: filter as u32,
    }
}

fn aa(
    min_x: jdouble,
    min_y: jdouble,
    min_z: jdouble,
    max_x: jdouble,
    max_y: jdouble,
    max_z: jdouble,
) -> AabbDesc {
    AabbDesc {
        mins: v3(min_x, min_y, min_z),
        maxs: v3(max_x, max_y, max_z),
    }
}

#[allow(clippy::too_many_arguments)]
fn qfilter(
    flags: jint,
    memberships: jint,
    filter: jint,
    use_groups: jint,
    exclude_collider: jlong,
    use_exclude_collider: jint,
    exclude_rigid_body: jlong,
    use_exclude_rigid_body: jint,
) -> QueryFilterDesc {
    QueryFilterDesc {
        flags: flags as u32,
        groups: grp(memberships, filter),
        use_groups: jb(use_groups),
        exclude_collider: exclude_collider as CRaw,
        use_exclude_collider: jb(use_exclude_collider),
        exclude_rigid_body: exclude_rigid_body as RRaw,
        use_exclude_rigid_body: jb(use_exclude_rigid_body),
    }
}

fn shape_type(value: jint) -> u32 {
    u32_from_jint(value)
}

fn body_status(value: jint) -> u32 {
    u32_from_jint(value)
}

fn joint_type(value: jint) -> u32 {
    u32_from_jint(value)
}

fn joint_axis(value: jint) -> u32 {
    u32_from_jint(value)
}

fn kdop_preset(value: jint) -> u32 {
    u32_from_jint(value)
}

fn neural_activation(value: jint) -> u32 {
    u32_from_jint(value)
}

fn voxel_mode(value: jint) -> u32 {
    u32_from_jint(value)
}

fn vec3_to_j_double_array(_env: JNIEnv, vec3: Vec3) -> jdoubleArray {
    let Ok(arr) = _env.new_double_array(3) else {
        return std::ptr::null_mut();
    };
    if _env
        .set_double_array_region(&arr, 0, &[vec3.x, vec3.y, vec3.z])
        .is_err()
    {
        return std::ptr::null_mut();
    }
    arr.as_raw()
}

fn quat_to_j_double_array(_env: JNIEnv, quat: Quat) -> jdoubleArray {
    let Ok(arr) = _env.new_double_array(4) else {
        return std::ptr::null_mut();
    };
    if _env
        .set_double_array_region(&arr, 0, &[quat.i, quat.j, quat.k, quat.w])
        .is_err()
    {
        return std::ptr::null_mut();
    }
    arr.as_raw()
}

fn sd(shape_type: jint, a: jdouble, b: jdouble, c: jdouble, d: jdouble) -> ShapeDesc {
    ShapeDesc {
        shape_type: self::shape_type(shape_type),
        a,
        b,
        c,
        d,
    }
}

/// Convert a (possibly non-unit) quaternion `q = (i, j, k, w)` into Rapier's
/// builder-rotation convention: an axis-angle encoded as a `Vec3` whose
/// direction is the rotation axis and whose magnitude is the rotation angle
/// in radians. This mirrors `Rotation3::scaled_axis_angle()`.
///
/// Returns `(0, 0, 0)` for the identity / near-identity case so that bizzare
/// callers passing `(0,0,0,1)` end up with no rotation rather than NaNs.
fn quat_to_axis_angle(q: Quat) -> Vec3 {
    // angle = 2 * acos(|w|); clamp w to [-1, 1] so acos never sees an OOB value.
    let w = q.w.clamp(-1.0, 1.0);
    let angle = 2.0 * w.acos();
    let s = (1.0 - w * w).sqrt();
    // s tiny ⇒ (near-)identity quaternion; pick zero rotation to stay finite.
    if s < 1e-12 {
        return Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
    }
    let k = angle / s;
    Vec3 {
        x: q.i * k,
        y: q.j * k,
        z: q.k * k,
    }
}

macro_rules! jni {
    (@ty long) => { jlong };
    (@ty boolean) => { jbyte };
    (@ty byte_array) => { jbyteArray };
    (@ty double) => { jdouble };
    (@ty int) => { jint };
    (@ty void) => { () };
    (@ty double_array) => { jdoubleArray };
    (@ty long_array) => { jlongArray };
    (@ty bool_array) => { jbooleanArray };
    (@ty String) => { jstring };
    (@default long) => { 0 };
    (@default boolean) => { 0 };
    (@default byte_array) => { std::ptr::null_mut() };
    (@default double) => { 0.0 };
    (@default int) => { 0 };
    (@default void) => { () };
    (@default double_array) => { std::ptr::null_mut() };
    (@default long_array) => { std::ptr::null_mut() };
    (@default bool_array) => { std::ptr::null_mut() };
    ($ret:ident $method:ident ( $($kind:ident $arg:ident),* ) $body:block) => {
        #[unsafe(export_name = concat!(
            "Java_org_polaris2023_mps_rapier_RapierNative_",
            stringify!($method)
        ))]
        #[allow(non_snake_case)]
        pub extern "system" fn $method(_env: JNIEnv, _class: jclass, $($arg: jni!(@ty $kind)),*) -> jni!(@ty $ret) {
            match catch_unwind(AssertUnwindSafe(|| $body)) {
                Ok(value) => value,
                Err(_) => {
                    er::set_error(er::ERR_INTERNAL, "internal panic");
                    jni!(@default $ret)
                }
            }
        }
    };
}

macro_rules! jni_e_c {
    // Delegate the shared @ty / @default table to `jni!`, only adding the two
    // extras (`env` / `class`) that `jni_e_c` needs but `jni!` does not provide.
    // Keeps the two macros' type tables in lockstep — see OPTIMIZATION.md §5.A.
    (@ty env) => { JNIEnv };
    (@ty class) => { jclass };
    (@ty $kind:ident) => { jni!(@ty $kind) };
    (@default $kind:ident) => { jni!(@default $kind) };
    ($ret:ident $method:ident ( $($kind:ident $arg:ident),* ) $body:block) => {
        #[unsafe(export_name = concat!(
            "Java_org_polaris2023_mps_rapier_RapierNative_",
            stringify!($method)
        ))]
        #[allow(non_snake_case)]
        pub extern "system" fn $method( $($arg: jni_e_c!(@ty $kind)),*) -> jni_e_c!(@ty $ret) {
            match catch_unwind(AssertUnwindSafe(|| $body)) {
                Ok(value) => value,
                Err(_) => {
                    er::set_error(er::ERR_INTERNAL, "internal panic");
                    jni_e_c!(@default $ret)
                }
            }
        }
    };
}

jni!(int abiVersion(){ abi::abi_version() as jint });
jni!(boolean abiSupportsFfm() { abi::abi_supports_ffm().0 as jbyte });
jni!(boolean abiSupportsJni() { abi::abi_supports_jni().0 as jbyte });
jni!(int abiLastErrorCode() { er::last_error_code() as jint });
jni!(void abiClearLastError() { er::last_error_clear(); });

#[unsafe(export_name = "Java_org_polaris2023_mps_rapier_RapierNative_abiLastErrorMessage")]
#[allow(non_snake_case)]
pub extern "system" fn abiLastErrorMessage(env: JNIEnv, _class: jclass) -> jstring {
    catch_unwind(AssertUnwindSafe(|| {
        let ptr = er::last_error_message();
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        let message = unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        env.new_string(message)
            .map(|value| value.as_raw())
            .unwrap_or(std::ptr::null_mut())
    }))
    .unwrap_or(std::ptr::null_mut())
}

//世界管理
jni!(long worldCreate(double gravity_x, double gravity_y, double gravity_z) { to_jlong(wo::world_create(v3(gravity_x, gravity_y, gravity_z))) });
jni!(void worldDestroy(long world) { wo::world_destroy(m::<WH>(world)); });
jni!(void worldStep(long world, double delta_seconds) { wo::world_step(m::<WH>(world), delta_seconds); });

jni!(void worldSetGravity(long world, double x, double y, double z) { wo::world_set_gravity(m::<WH>(world), v3(x, y, z)); });

jni_e_c!(double_array worldGetGravity(env _env, class _class, long world) { vec3_to_j_double_array(_env, wo::world_get_gravity(cp::<WH>(world))) });
jni!(void worldGetGravityOut(long world, long out_gravity) { wo::world_get_gravity_out(cp::<WH>(world), pm::<Vec3>(out_gravity)); });
jni!(int worldGetRigidBodySetSize(long world) { wo::world_get_rigid_body_set_size(cp::<WH>(world)) });
jni!(int worldGetColliderSetSize(long world) { wo::world_get_collider_set_size(cp::<WH>(world)) });

jni!(int worldDynamicBodySnapshotCount(long world) { wo::world_dynamic_body_snapshot_count(cp::<WH>(world)) as jint });
jni!(int worldDynamicBodySnapshot(long world, long out_handles, long out_values, int capacity) { wo::world_dynamic_body_snapshot(cp::<WH>(world),pm::<RRaw>(out_handles),pm::<f64>(out_values),u32_from_jint(capacity)) as jint });
jni!(boolean worldSetIntegrationParameters(long world, double dt, int solver_iterations, int ccd_substeps) { wo::world_set_integration_parameters(m::<WH>(world), dt, u32_from_jint(solver_iterations), u32_from_jint(ccd_substeps)).0 as jbyte });
jni!(int worldGetIntegrationParameters(long world, long out_values, int capacity) { wo::world_get_integration_parameters(cp::<WH>(world), pm::<f64>(out_values), u32_from_jint(capacity)) as jint });
jni!(int worldBodySnapshotCount(long world) { wo::world_body_snapshot_count(cp::<WH>(world)) as jint });
jni!(int worldBodySnapshot(long world, long out_handles, long out_values, int capacity) { wo::world_body_snapshot(cp::<WH>(world), pm::<RRaw>(out_handles), pm::<f64>(out_values), u32_from_jint(capacity)) as jint });
jni!(int worldUpdateBodyPoses(long world, long handles, long values, int count, int wake_up) { wo::world_update_body_poses(m::<WH>(world), p::<RRaw>(handles), p::<f64>(values), u32_from_jint(count), jb(wake_up)) as jint });
jni!(int worldUpdateBodyVelocities(long world, long handles, long values, int count, int wake_up) { wo::world_update_body_velocities(m::<WH>(world), p::<RRaw>(handles), p::<f64>(values), u32_from_jint(count), jb(wake_up)) as jint });

#[cfg(feature = "relative-force")]
jni!(boolean worldSetRelativeForceEnabled(long world, long handle, int enabled, double lx, double ly, double lz) { wo::world_set_relative_force_enabled(m::<WH>(world), handle as RRaw, jb(enabled), v3(lx, ly, lz)).0 as jbyte });
#[cfg(feature = "relative-force")]
jni!(boolean worldGetRelativeForceEnabled(long world, long handle) { wo::world_get_relative_force_enabled(cp::<WH>(world), handle as RRaw).0 as jbyte });
#[cfg(feature = "relative-force")]
jni_e_c!(double_array worldGetRelativeForceLocalPoint(env _env, class _class, long world, long handle) { vec3_to_j_double_array(_env, wo::world_get_relative_force_local_point(cp::<WH>(world), handle as RRaw)) });
#[cfg(feature = "relative-force")]
jni!(boolean worldSetRelativeForceLocalPoint(long world, long handle, double lx, double ly, double lz) { wo::world_set_relative_force_local_point(m::<WH>(world), handle as RRaw, v3(lx, ly, lz)).0 as jbyte });
#[cfg(feature = "relative-force")]
jni!(boolean worldRemoveRelativeForce(long world, long handle) { wo::world_remove_relative_force(m::<WH>(world), handle as RRaw).0 as jbyte });

//世界插入
jni!(long worldInsertRigidBody(long world, long memory_handle) { rb::world_insert_rigid_body(m::<WH>(world), m::<RB>(memory_handle)) as jlong });
jni!(boolean worldRemoveRigidBody(long world, long handle, int remove_attached_colliders) { rb::world_remove_rigid_body(m::<WH>(world), handle as RRaw, jb(remove_attached_colliders)).0 as jbyte });
jni!(long worldCopyRigidBody(long world, long handle) { rb::world_copy_rigid_body(m::<WH>(world), handle as RRaw) as jlong });
jni!(void rigidBodyDestroyRaw(long rigid_body) { rb::rigid_body_destroy_raw(m::<RB>(rigid_body)); });
jni!(long worldInsertCollider(long world, long memory_handle) { col::world_insert_collider(m::<WH>(world), m::<CB>(memory_handle)) as jlong });
jni!(long worldInsertColliderWithParent(long world, long memory_handle, long parent) { col::world_insert_collider_with_parent(m::<WH>(world), m::<CB>(memory_handle), parent as RRaw) as jlong });
jni!(boolean worldRemoveCollider(long world, long handle, int wake_up) { col::world_remove_collider(m::<WH>(world), handle as CRaw, jb(wake_up)).0 as jbyte });
jni!(long worldCopyCollider(long world, long handle)  { col::world_copy_collider(m::<WH>(world), handle as CRaw) as jlong });
jni!(void colliderDestroyRaw(long collider) { col::collider_destroy_raw(m::<CB>(collider)); });

jni!(long colliderBuilderCreate(int shape_type, double a, double b, double c) { to_jlong(col::collider_builder_create(self::shape_type(shape_type), v3(a, b, c))) });
jni!(long colliderBuilderCreateHalfSpace(double nx, double ny, double nz) { to_jlong(col::collider_builder_create_halfspace(v3(nx, ny, nz))) });
jni_e_c!(long colliderBuilderCreateHeightmap(env _env, class _class, long data, int data_x, int data_y, double scale_x, double scale_y, double scale_z) { to_jlong(col::collider_builder_create_heightmap(p::<f64>(data), u32_from_jint(data_x), u32_from_jint(data_y), Vec3 { x: scale_x, y: scale_y, z: scale_z })) });
jni!(long colliderBuilderCreateEx(int shape_type, double a, double b, double c, double d) { to_jlong(col::collider_builder_create_ex(sd(shape_type, a, b, c, d))) });
jni!(long colliderBuilderCreateSphere(double x, double y, double z, double radius) { to_jlong(col::collider_builder_create_sphere(Sphere { center: v3(x, y, z), radius })) });
jni!(long colliderBuilderCreateObb(double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw) { to_jlong(col::collider_builder_create_obb(Obb {center: v3(cx, cy, cz),half_extents: v3(hx, hy, hz),rotation: qt(qi, qj, qk, qw),})) });
jni!(long colliderBuilderCreateConvexHull(long points_xyz, int point_count) { to_jlong(col::collider_builder_create_convex_hull(p::<f64>(points_xyz), u32_from_jint(point_count))) });
jni!(long colliderBuilderCreatePointCloudBounds(long points_xyz, int point_count) { to_jlong(col::collider_builder_create_point_cloud_bounds(p::<f64>(points_xyz), u32_from_jint(point_count))) });
jni!(long colliderBuilderCreateDoubleBv(double a_min_x, double a_min_y, double a_min_z, double a_max_x, double a_max_y, double a_max_z, double b_min_x, double b_min_y, double b_min_z, double b_max_x, double b_max_y, double b_max_z) { to_jlong(col::collider_builder_create_double_bv(aa(a_min_x,a_min_y,a_min_z,a_max_x,a_max_y,a_max_z), aa(b_min_x,b_min_y,b_min_z,b_max_x,b_max_y,b_max_z))) });
jni!(long colliderBuilderCreateSkewedObb(double cx, double cy, double cz, double ax_x, double ax_y, double ax_z, double ay_x, double ay_y, double ay_z, double az_x, double az_y, double az_z) { to_jlong(col::collider_builder_create_skewed_obb(v3(cx,cy,cz), v3(ax_x,ax_y,ax_z), v3(ay_x,ay_y,ay_z), v3(az_x,az_y,az_z))) });
jni!(long colliderBuilderCreateDiscreteObb(long points_xyz, int point_count, int axis) { to_jlong(col::collider_builder_create_discrete_obb(p::<f64>(points_xyz), u32_from_jint(point_count), u32_from_jint(axis))) });
jni!(long colliderBuilderCreateFusedCollapsingBounds(long points_xyz, int point_count, double padding) { to_jlong(col::collider_builder_create_fused_collapsing_bounds(p::<f64>(points_xyz), u32_from_jint(point_count), padding)) });
jni!(long colliderBuilderCreateEdgeBvh(long vertices_xyz, int vertex_count, long edges, int edge_count, double radius) { to_jlong(col::collider_builder_create_edge_bvh(p::<f64>(vertices_xyz), u32_from_jint(vertex_count), p::<u32>(edges), u32_from_jint(edge_count), radius)) });
jni!(long colliderBuilderCreateMedialSpheres(long spheres_xyzw, int sphere_count) { to_jlong(col::collider_builder_create_medial_spheres(p::<f64>(spheres_xyzw), u32_from_jint(sphere_count))) });
jni!(long colliderBuilderCreateCapsule(double ax, double ay, double az, double bx, double by, double bz, double radius) { to_jlong(bo::collider_builder_create_capsule(Capsule { a: v3(ax, ay, az), b: v3(bx, by, bz), radius })) });
jni!(long colliderBuilderCreateSsv(double ax, double ay, double az, double bx, double by, double bz, double radius) { to_jlong(bo::collider_builder_create_ssv(Ssv { a: v3(ax, ay, az), b: v3(bx, by, bz), radius })) });
jni!(long colliderBuilderCreateEllipsoid(double cx, double cy, double cz, double rx, double ry, double rz, double qi, double qj, double qk, double qw, int segments) { to_jlong(bo::collider_builder_create_ellipsoid(Ellipsoid { center: v3(cx, cy, cz), radii: v3(rx, ry, rz), rotation: qt(qi, qj, qk, qw), segments: u32_from_jint(segments) })) });
jni!(long colliderBuilderCreatePrism(double cx, double cy, double cz, double radius, double half_height, int sides, double qi, double qj, double qk, double qw) { to_jlong(bo::collider_builder_create_prism(Prism { center: v3(cx, cy, cz), radius, half_height, sides: u32_from_jint(sides), rotation: qt(qi, qj, qk, qw) })) });
jni!(long colliderBuilderCreateCylinder(double cx, double cy, double cz, double radius, double half_height, double qi, double qj, double qk, double qw) { to_jlong(bo::collider_builder_create_cylinder(Cylinder { center: v3(cx, cy, cz), radius, half_height, rotation: qt(qi, qj, qk, qw) })) });
jni!(long colliderBuilderCreateSphericalShell(double cx, double cy, double cz, double inner_radius, double outer_radius) { to_jlong(bo::collider_builder_create_spherical_shell(SphericalShell { center: v3(cx, cy, cz), inner_radius, outer_radius })) });
jni!(long colliderBuilderCreateKdop(long points_xyz, int point_count, int preset) { to_jlong(dop::collider_builder_create_kdop(p::<f64>(points_xyz), u32_from_jint(point_count), kdop_preset(preset))) });
jni!(long colliderBuilderCreateFdh(long points_xyz, int point_count, long directions_xyz, int direction_count) { to_jlong(dop::collider_builder_create_fdh(p::<f64>(points_xyz), u32_from_jint(point_count), p::<f64>(directions_xyz), u32_from_jint(direction_count))) });
jni!(long colliderBuilderCreateNeuralBounds(double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, int sample_resolution, int hidden_width, int hidden_layers, int activation, double output_scale, double padding, long weights, int weight_count) { to_jlong(neu::collider_builder_create_neural_bounds(NeuralBoundsDesc { center: v3(cx,cy,cz), half_extents: v3(hx,hy,hz), rotation: qt(qi,qj,qk,qw), sample_resolution: u32_from_jint(sample_resolution), hidden_width: u32_from_jint(hidden_width), hidden_layers: u32_from_jint(hidden_layers), activation: neural_activation(activation), output_scale, padding,}, p::<f64>(weights), u32_from_jint(weight_count))) });
jni!(long colliderBuilderCreateVoxels(long voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) { to_jlong(vx::collider_builder_create_voxels(p::<u8>(voxels), u32_from_jint(size_x), u32_from_jint(size_y), u32_from_jint(size_z), voxel_size_x, voxel_size_y, voxel_size_z, v3(origin_x, origin_y, origin_z), VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) })) });
jni!(long colliderBuilderCreateVoxelsAuto(long voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int dynamic_body) { to_jlong(vx::collider_builder_create_voxels_auto(p::<u8>(voxels), u32_from_jint(size_x), u32_from_jint(size_y), u32_from_jint(size_z), voxel_size_x, voxel_size_y, voxel_size_z, v3(origin_x, origin_y, origin_z), jb(dynamic_body))) });
jni_e_c!(long colliderBuilderCreateVoxelBytes(env _env, class _class, byte_array voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) {
    let Some(values) = jbytearray_to_array(&_env, voxels) else {
        return 0;
    };
    to_jlong(vx::collider_builder_create_voxels(values.as_ptr(), u32_from_jint(size_x), u32_from_jint(size_y), u32_from_jint(size_z), voxel_size_x, voxel_size_y, voxel_size_z, v3(origin_x, origin_y, origin_z), VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) }))
});
jni_e_c!(long colliderBuilderCreateVoxelBytesAuto(env _env, class _class, byte_array voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int dynamic_body) {
    let Some(values) = jbytearray_to_array(&_env, voxels) else {
        return 0;
    };
    to_jlong(vx::collider_builder_create_voxels_auto(values.as_ptr(), u32_from_jint(size_x), u32_from_jint(size_y), u32_from_jint(size_z), voxel_size_x, voxel_size_y, voxel_size_z, v3(origin_x, origin_y, origin_z), jb(dynamic_body)))
});
jni!(long colliderBuilderCreateVoxelAabb(double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) {
    to_jlong(vx::collider_builder_create_voxel_aabb(
        aa(min_x, min_y, min_z, max_x, max_y, max_z),
        voxel_size_x, voxel_size_y, voxel_size_z,
        VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) }
    ))
});
jni!(long colliderBuilderCreateVoxelAabbAuto(double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, int dynamic_body) {
    to_jlong(vx::collider_builder_create_voxel_aabb_auto(
        aa(min_x, min_y, min_z, max_x, max_y, max_z),
        voxel_size_x, voxel_size_y, voxel_size_z,
        jb(dynamic_body)
    ))
});
jni!(long colliderBuilderCreateVoxelObb(double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) {
    to_jlong(vx::collider_builder_create_voxel_obb(
        Obb { center: v3(cx, cy, cz), half_extents: v3(hx, hy, hz), rotation: qt(qi, qj, qk, qw) },
        voxel_size_x, voxel_size_y, voxel_size_z,
        VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) }
    ))
});
jni!(long colliderBuilderCreateVoxelObbAuto(double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, double voxel_size_x, double voxel_size_y, double voxel_size_z, int dynamic_body) {
    to_jlong(vx::collider_builder_create_voxel_obb_auto(
        Obb { center: v3(cx, cy, cz), half_extents: v3(hx, hy, hz), rotation: qt(qi, qj, qk, qw) },
        voxel_size_x, voxel_size_y, voxel_size_z,
        jb(dynamic_body)
    ))
});
jni!(void voxelBuildStats(long voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit, long out_stats) {
    let stats = vx::voxel_build_stats(
        p::<u8>(voxels),
        u32_from_jint(size_x),
        u32_from_jint(size_y),
        u32_from_jint(size_z),
        voxel_size_x, voxel_size_y, voxel_size_z,
        v3(origin_x, origin_y, origin_z),
        VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) },
    );
    if let Some(out) = unsafe { pm::<VoxelBuildStats>(out_stats).as_mut() } { *out = stats; }
});
jni!(void voxelAabbBuildStats(double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit, long out_stats) {
    let stats = vx::voxel_aabb_build_stats(
        aa(min_x, min_y, min_z, max_x, max_y, max_z),
        voxel_size_x, voxel_size_y, voxel_size_z,
        VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) },
    );
    if let Some(out) = unsafe { pm::<VoxelBuildStats>(out_stats).as_mut() } { *out = stats; }
});
jni!(void voxelObbBuildStats(double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit, long out_stats) {
    let stats = vx::voxel_obb_build_stats(
        Obb { center: v3(cx, cy, cz), half_extents: v3(hx, hy, hz), rotation: qt(qi, qj, qk, qw) },
        voxel_size_x, voxel_size_y, voxel_size_z,
        VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: jb(dynamic_body), small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) },
    );
    if let Some(out) = unsafe { pm::<VoxelBuildStats>(out_stats).as_mut() } { *out = stats; }
});

jni!(void colliderBuilderSetTranslation(long builder, double x, double y, double z) { col::collider_builder_set_translation(m::<CBH>(builder), v3(x, y, z)); });
jni!(void colliderBuilderSetRotation(long builder, double qi, double qj, double qk, double qw) {
    // Rapier's builder-level `set_rotation` consumes an axis-angle `Vec3`,
    // but Java/ColliderBody callers pass a unit quaternion (i, j, k, w).
    // Convert quaternion → axis-angle (axis * angle) here so the existing
    // FFI `collider_builder_set_rotation(builder, Vec3)` can be reused and
    // Java keeps its (x, y, z, w) quaternion signature — see SKILL §FFI.
    col::collider_builder_set_rotation(m::<CBH>(builder), quat_to_axis_angle(qt(qi, qj, qk, qw)))
});
jni!(void colliderBuilderSetPose(long builder, double x, double y, double z, double qi, double qj, double qk, double qw) { col::collider_builder_set_pose(m::<CBH>(builder), v3(x, y, z), qt(qi, qj, qk, qw)); });
jni!(void colliderBuilderSetSensor(long builder, int sensor) { col::collider_builder_set_sensor(m::<CBH>(builder), jb(sensor)); });
jni!(void colliderBuilderSetFriction(long builder, double friction) { col::collider_builder_set_friction(m::<CBH>(builder), friction); });
jni!(void colliderBuilderSetRestitution(long builder, double restitution) { col::collider_builder_set_restitution(m::<CBH>(builder), restitution); });
jni!(void colliderBuilderSetDensity(long builder, double density) { col::collider_builder_set_density(m::<CBH>(builder), density); });
jni!(void colliderBuilderSetCollisionGroups(long builder, int memberships, int filter) { col::collider_builder_set_collision_groups(m::<CBH>(builder), grp(memberships, filter)); });
jni!(void colliderBuilderSetSolverGroups(long builder, int memberships, int filter) { col::collider_builder_set_solver_groups(m::<CBH>(builder), grp(memberships, filter)); });
jni!(void colliderBuilderSetActiveEvents(long builder, int bits) { col::collider_builder_set_active_events(m::<CBH>(builder), bits as u32); });
jni!(void colliderBuilderSetActiveHooks(long builder, int bits) { col::collider_builder_set_active_hooks(m::<CBH>(builder), bits as u32); });
jni!(void colliderBuilderSetContactForceEventThreshold(long builder, double threshold) { col::collider_builder_set_contact_force_event_threshold(m::<CBH>(builder), threshold); });

jni!(long colliderBuilderBuild(long builder) { to_jlong(col::collider_builder_build(m::<CBH>(builder))) });

// 就地体素编辑：对已插入的 voxel collider 几何原地更新，handle 不变。
// 单格翻转最贴 Minecraft 挖/放一格；批量覆盖用于 chunk 重载。
// 这两个函数要求 collider 是由 collider_builder_create_voxel* 创建的（world
// 内部保留了源网格），否则返回 false 并报 ERR_UNSUPPORTED。
jni!(boolean colliderVoxelEdit(long world, long handle, int x, int y, int z, int solid) {
    vx::collider_voxel_edit(m::<WH>(world), handle as CRaw, x as i64, y as i64, z as i64, solid).0 as jbyte
});
jni_e_c!(boolean colliderSetVoxels(env _env, class _class, long world, long handle, byte_array voxels, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) {
    let Some(values) = jbytearray_to_array(&_env, voxels) else {
        return 0;
    };
    vx::collider_set_voxels(
        m::<WH>(world),
        handle as CRaw,
        values.as_ptr(),
        u32_from_jint(size_x),
        u32_from_jint(size_y),
        u32_from_jint(size_z),
        voxel_size_x, voxel_size_y, voxel_size_z,
        v3(origin_x, origin_y, origin_z),
        shape_type(mode),
        dynamic_body,
        u32_from_jint(small_voxel_limit),
        u32_from_jint(mesh_voxel_limit),
    ).0 as jbyte
});

// 射线拾取：对单个 voxel collider 投射射线，反查命中的体素 (ix,iy,iz)。
// out_block 是调用方分配的 56 字节缓冲地址（VoxelCoord C 布局）。
jni!(boolean colliderVoxelRayPick(long world, long collider, double ox, double oy, double oz, double dx, double dy, double dz, double max_toi, int solid, long out_block) {
    vx::collider_voxel_ray_pick(
        m::<WH>(world),
        collider as CRaw,
        v3(ox, oy, oz),
        v3(dx, dy, dz),
        max_toi,
        jb(solid),
        out_block as *mut vx::VoxelCoord,
    ).0 as jbyte
});
jni!(boolean colliderVoxelCellAtPoint(long world, long collider, double px, double py, double pz, long out_block) {
    vx::collider_voxel_cell_at_point(
        m::<WH>(world),
        collider as CRaw,
        v3(px, py, pz),
        out_block as *mut vx::VoxelCoord,
    ).0 as jbyte
});

// 读取单个体素格子是否实心（collider_voxel_edit 的读对偶）。
// out_solid 是调用方分配、能被 Java `boolean` 写入的 1 字节地址。
jni!(boolean colliderVoxelGet(long world, long collider, int x, int y, int z, long out_solid) {
    vx::collider_voxel_read_cell(
        m::<WH>(world),
        collider as CRaw,
        x as i64, y as i64, z as i64,
        out_solid as *mut u8,
    ).0 as jbyte
});

jni!(void colliderBuilderDestroy(long builder) { col::collider_builder_destroy(m::<CBH>(builder)); });

jni_e_c!(double_array colliderGetTranslation(env _env, class _class, long world, long handle) { vec3_to_j_double_array(_env, col::collider_get_translation(cp::<WH>(world), handle as CRaw)) });
jni_e_c!(double_array colliderGetRotation(env _env, class _class, long world, long handle) { quat_to_j_double_array(_env, col::collider_get_rotation(cp::<WH>(world), handle as CRaw)) });
jni!(void colliderGetTranslationOut(long world, long handle, long out_translation) { col::collider_get_translation_out(cp::<WH>(world), handle as CRaw, pm::<Vec3>(out_translation)); });
jni!(void colliderGetRotationOut(long world, long handle, long out_rotation) { col::collider_get_rotation_out(cp::<WH>(world), handle as CRaw, pm::<Quat>(out_rotation)); });
jni!(long colliderGetShapeSize(long world, long handle) { to_jint(col::collider_get_shape_count(cp::<WH>(world), handle as CRaw)) });

jni!(boolean colliderSetPose(long world, long handle, double x, double y, double z, double qi, double qj, double qk, double qw) { col::collider_set_pose(m::<WH>(world), handle as CRaw, v3(x, y, z), qt(qi, qj, qk, qw)).0 as jbyte });
jni!(boolean colliderSetTranslation(long world, long handle, double x, double y, double z) { col::collider_set_translation(m::<WH>(world), handle as CRaw, v3(x, y, z)).0 as jbyte });
jni!(boolean colliderSetRotation(long world, long handle, double qi, double qj, double qk, double qw) { col::collider_set_rotation(m::<WH>(world), handle as CRaw, qt(qi, qj, qk, qw)).0 as jbyte });
jni!(boolean colliderSetSensor(long world, long handle, int sensor) { col::collider_set_sensor(m::<WH>(world), handle as CRaw, jb(sensor)).0 as jbyte });
jni!(boolean colliderSetFriction(long world, long handle, double friction) { col::collider_set_friction(m::<WH>(world), handle as CRaw, friction).0 as jbyte });
jni!(boolean colliderSetRestitution(long world, long handle, double restitution) { col::collider_set_restitution(m::<WH>(world), handle as CRaw, restitution).0 as jbyte });
jni!(boolean colliderSetCollisionGroups(long world, long handle, int memberships, int filter) { col::collider_set_collision_groups(m::<WH>(world), handle as CRaw, grp(memberships, filter)).0 as jbyte });
jni!(boolean colliderSetSolverGroups(long world, long handle, int memberships, int filter) { col::collider_set_solver_groups(m::<WH>(world), handle as CRaw, grp(memberships, filter)).0 as jbyte });
jni!(boolean colliderSetActiveEvents(long world, long handle, int bits) { col::collider_set_active_events(m::<WH>(world), handle as CRaw, bits as u32).0 as jbyte });
jni!(boolean colliderSetActiveHooks(long world, long handle, int bits) { col::collider_set_active_hooks(m::<WH>(world), handle as CRaw, bits as u32).0 as jbyte });
jni!(boolean colliderSetContactForceEventThreshold(long world, long handle, double threshold) { col::collider_set_contact_force_event_threshold(m::<WH>(world), handle as CRaw, threshold).0 as jbyte });
jni!(double colliderGetDensity(long world, long handle) { col::collider_get_density(cp::<WH>(world), handle as CRaw) });

jni!(long rigidBodyBuilderCreate(int status) { to_jlong(rb::rigid_body_builder_create(body_status(status))) });

jni!(void rigidBodyBuilderSetTranslation(long builder, double x, double y, double z) { rb::rigid_body_builder_set_translation(m::<RBH>(builder), v3(x, y, z)); });
jni!(void rigidBodyBuilderSetRotation(long builder, double x, double y, double z) { rb::rigid_body_builder_set_rotation(m::<RBH>(builder), v3(x, y, z)); });
jni!(void rigidBodyBuilderSetPose(long builder, double x, double y, double z, double qi, double qj, double qk, double qw) { rb::rigid_body_builder_set_pose(m::<RBH>(builder), v3(x, y, z), qt(qi, qj, qk, qw)); });
jni!(void rigidBodyBuilderSetAdditionalMassProperties(long builder, double cx, double cy, double cz, double mass, double lx, double ly, double lz) { rb::rigid_body_builder_set_additional_mass_properties(m::<RBH>(builder), v3(cx, cy, cz), mass, v3(lx, ly, lz)); });
jni!(void rigidBodyBuilderSetLinvel(long builder, double x, double y, double z) { rb::rigid_body_builder_set_linvel(m::<RBH>(builder), v3(x, y, z)); });
jni!(void rigidBodyBuilderSetAngvel(long builder, double x, double y, double z) { rb::rigid_body_builder_set_angvel(m::<RBH>(builder), v3(x, y, z)); });
jni!(void rigidBodyBuilderSetGravityScale(long builder, double value) { rb::rigid_body_builder_set_gravity_scale(m::<RBH>(builder), value); });
jni!(void rigidBodyBuilderSetLinearDamping(long builder, double value) { rb::rigid_body_builder_set_linear_damping(m::<RBH>(builder), value); });
jni!(void rigidBodyBuilderSetAngularDamping(long builder, double value) { rb::rigid_body_builder_set_angular_damping(m::<RBH>(builder), value); });
jni!(void rigidBodyBuilderSetCanSleep(long builder, int value) { rb::rigid_body_builder_set_can_sleep(m::<RBH>(builder), jb(value)); });
jni!(void rigidBodyBuilderSetEnabledRotations(long builder, int x, int y, int z) { rb::rigid_body_builder_set_enabled_rotations(m::<RBH>(builder), jb(x), jb(y), jb(z)); });
jni!(void rigidBodyBuilderSetUserData(long builder, long low, long high) { rb::rigid_body_builder_set_user_data(m::<RBH>(builder), low as u64, high as u64); });
jni!(void rigidBodyBuilderSetAdditionalMass(long builder, double mass) { rb::rigid_body_builder_set_additional_mass(m::<RBH>(builder), mass); });

jni!(long rigidBodyBuilderBuild(long builder) { to_jlong(rb::rigid_body_builder_build(m::<RBH>(builder))) });

jni!(void rigidBodyBuilderDestroy(long builder) { rb::rigid_body_builder_destroy(m::<RBH>(builder)); });

jni!(int rigidBodyGetStatus(long world, long handle) { rb::rigid_body_get_status(cp::<WH>(world), handle as RRaw) as jint });
jni!(boolean rigidBodySetStatus(long world, long handle, int status, int wake_up) { rb::rigid_body_set_status(m::<WH>(world), handle as RRaw, body_status(status), jb(wake_up)).0 as jbyte });

jni_e_c!(double_array rigidBodyGetTranslation(env _env, class _class, long world, long body) { vec3_to_j_double_array(_env, rb::rigid_body_get_translation(cp::<WH>(world), body as RRaw)) });
jni_e_c!(double_array rigidBodyGetRotation(env _env, class _class, long world, long body) { quat_to_j_double_array(_env, rb::rigid_body_get_rotation(cp::<WH>(world), body as RRaw)) });
jni!(void rigidBodyGetTranslationOut(long world, long body, long out_translation) { rb::rigid_body_get_translation_out(cp::<WH>(world), body as RRaw, pm::<Vec3>(out_translation)); });
jni!(void rigidBodyGetRotationOut(long world, long body, long out_rotation) { rb::rigid_body_get_rotation_out(cp::<WH>(world), body as RRaw, pm::<Quat>(out_rotation)); });
jni!(boolean rigidBodySetPose(long world, long body, double x, double y, double z, double qi, double qj, double qk, double qw, int wake_up) { rb::rigid_body_set_pose(m::<WH>(world), body as RRaw, v3(x, y, z), qt(qi, qj, qk, qw), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodySetTranslation(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_set_translation(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodySetRotation(long world, long body, double qi, double qj, double qk, double qw, int wake_up) { rb::rigid_body_set_rotation(m::<WH>(world), body as RRaw, qt(qi, qj, qk, qw), jb(wake_up)).0 as jbyte });
jni!(double rigidBodyGetMass(long world, long body) { rb::rigid_body_get_mass(m::<WH>(world), body as RRaw) });
jni_e_c!(double_array rigidBodyGetForce(env _env, class _class, long world, long body) { vec3_to_j_double_array(_env, rb::rigid_body_get_force(cp::<WH>(world), body as RRaw)) });
jni_e_c!(double_array rigidBodyGetLinvel(env _env, class _class, long world, long body) { vec3_to_j_double_array(_env, rb::rigid_body_get_linvel(cp::<WH>(world), body as RRaw)) });
jni!(void rigidBodyGetLinvelOut(long world, long body, long out_linvel) { rb::rigid_body_get_linvel_out(cp::<WH>(world), body as RRaw, pm::<Vec3>(out_linvel)); });
jni!(boolean rigidBodySetLinvel(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_set_linvel(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni_e_c!(double_array rigidBodyGetAngvel(env _env, class _class, long world, long body) { vec3_to_j_double_array(_env, rb::rigid_body_get_angvel(cp::<WH>(world), body as RRaw)) });
jni!(void rigidBodyGetAngvelOut(long world, long body, long out_angvel) { rb::rigid_body_get_angvel_out(cp::<WH>(world), body as RRaw, pm::<Vec3>(out_angvel)); });
jni!(boolean rigidBodySetAngvel(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_set_angvel(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyAddForce(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_add_force(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyAddForceAtPoint(long world, long body, double x, double y, double z, double px, double py, double pz, int wake_up) { rb::rigid_body_add_force_at_point(m::<WH>(world), body as RRaw, v3(x, y, z), v3(px, py, pz), jb(wake_up)).0 as jbyte });
#[cfg(feature = "relative-force")]
jni!(boolean rigidBodyAddForceAtLocalPoint(long world, long body, double x, double y, double z, double lx, double ly, double lz, int wake_up) { rb::rigid_body_add_force_at_local_point(m::<WH>(world), body as RRaw, v3(x, y, z), v3(lx, ly, lz), jb(wake_up)).0 as jbyte });
#[cfg(feature = "relative-force")]
jni!(boolean rigidBodyAddTorqueAtLocalPoint(long world, long body, double x, double y, double z, double lx, double ly, double lz, int wake_up) { rb::rigid_body_add_torque_at_local_point(m::<WH>(world), body as RRaw, v3(x, y, z), v3(lx, ly, lz), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyResetForce(long world, long body, int wake_up) { rb::rigid_body_reset_force(m::<WH>(world), body as RRaw, jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyAddTorque(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_add_torque(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyResetTorque(long world, long body, int wake_up) { rb::rigid_body_reset_torque(m::<WH>(world), body as RRaw, jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyApplyImpulse(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_apply_impulse(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyApplyTorqueImpulse(long world, long body, double x, double y, double z, int wake_up) { rb::rigid_body_apply_torque_impulse(m::<WH>(world), body as RRaw, v3(x, y, z), jb(wake_up)).0 as jbyte });
jni!(boolean rigidBodyEnableCcd(long world, long body, int enabled) { rb::rigid_body_enable_ccd(m::<WH>(world), body as RRaw, jb(enabled)).0 as jbyte });
jni!(boolean rigidBodySleep(long world, long body) { rb::rigid_body_sleep(m::<WH>(world), body as RRaw).0 as jbyte });
jni!(boolean rigidBodyWakeUp(long world, long body, int strong) { rb::rigid_body_wake_up(m::<WH>(world), body as RRaw, jb(strong)).0 as jbyte });
jni!(boolean rigidBodyIsSleeping(long world, long body) { rb::rigid_body_is_sleeping(cp::<WH>(world), body as RRaw).0 as jbyte });

macro_rules! query_filter_args {
    ($flags:ident,$memberships:ident,$filter:ident,$use_groups:ident,$exclude_collider:ident,$use_exclude_collider:ident,$exclude_rigid_body:ident,$use_exclude_rigid_body:ident) => {
        qfilter(
            $flags,
            $memberships,
            $filter,
            $use_groups,
            $exclude_collider,
            $use_exclude_collider,
            $exclude_rigid_body,
            $use_exclude_rigid_body,
        )
    };
}

jni!(long queryCastRay(long world, double ox, double oy, double oz, double dx, double dy, double dz, double max_toi, int solid, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_hit) {
    let world_ptr = cp::<WH>(world);
    if world_ptr.is_null() {
        er::set_error(er::ERR_NULL_POINTER, "world is null");
        return 0;
    }
    let filter_desc = query_filter_args!(flags, memberships, filter, use_groups, exclude_collider, use_exclude_collider, exclude_rigid_body, use_exclude_rigid_body);
    let hit = qu::query_cast_ray(world_ptr, v3(ox, oy, oz), v3(dx, dy, dz), max_toi, jb(solid), filter_desc);
    if out_hit != 0
        && let Some(out) = unsafe { pm::<RayHit>(out_hit).as_mut() } {
            *out = hit;
        }
    hit.collider as jlong
});

jni!(int queryCastRays(long world, long rays, int ray_count, double max_toi, int solid, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_hits, int capacity) {
    qu::query_cast_rays(cp::<WH>(world), p::<f64>(rays), u32_from_jint(ray_count), max_toi, jb(solid), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), pm::<RayHit>(out_hits), u32_from_jint(capacity)) as jint
});

jni!(long queryProjectPoint(long world, double x, double y, double z, double max_dist, int solid, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_projection) {
    let mut collider: CRaw = 0;
    let projection = qu::query_project_point(cp::<WH>(world), v3(x, y, z), max_dist, jb(solid), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), &mut collider as *mut CRaw);
    if let Some(out) = unsafe { pm::<PointProjection>(out_projection).as_mut() } { *out = projection; }
    collider as jlong
});

jni!(int queryIntersectPointCount(long world, double x, double y, double z, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body) {
    qu::query_intersect_point_count(cp::<WH>(world), v3(x, y, z), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body)) as jint
});

jni!(int queryIntersectAabbCount(long world, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body) {
    qu::query_intersect_aabb_count(cp::<WH>(world), aa(min_x,min_y,min_z,max_x,max_y,max_z), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body)) as jint
});

jni!(int queryIntersectObb(long world, double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_handles, int capacity) {
    qu::query_intersect_obb(cp::<WH>(world), Obb { center: v3(cx,cy,cz), half_extents: v3(hx,hy,hz), rotation: qt(qi,qj,qk,qw) }, query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), pm::<CRaw>(out_handles), u32_from_jint(capacity)) as jint
});

jni!(int queryIntersectSphere(long world, double cx, double cy, double cz, double radius, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_handles, int capacity) {
    qu::query_intersect_sphere(cp::<WH>(world), Sphere { center: v3(cx,cy,cz), radius }, query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), pm::<CRaw>(out_handles), u32_from_jint(capacity)) as jint
});

jni!(int queryIntersectVoxelAabb(long world, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_handles, int capacity) {
    vx::query_intersect_voxel_aabb(cp::<WH>(world), aa(min_x,min_y,min_z,max_x,max_y,max_z), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), pm::<CRaw>(out_handles), u32_from_jint(capacity)) as jint
});

jni!(int queryIntersectVoxelAabbCount(long world, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body) {
    vx::query_intersect_voxel_aabb_count(cp::<WH>(world), aa(min_x,min_y,min_z,max_x,max_y,max_z), query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body)) as jint
});

jni!(int queryIntersectVoxelObb(long world, double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_handles, int capacity) {
    vx::query_intersect_voxel_obb(cp::<WH>(world), Obb { center: v3(cx,cy,cz), half_extents: v3(hx,hy,hz), rotation: qt(qi,qj,qk,qw) }, query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body), pm::<CRaw>(out_handles), u32_from_jint(capacity)) as jint
});

jni!(int queryIntersectVoxelObbCount(long world, double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body) {
    vx::query_intersect_voxel_obb_count(cp::<WH>(world), Obb { center: v3(cx,cy,cz), half_extents: v3(hx,hy,hz), rotation: qt(qi,qj,qk,qw) }, query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body)) as jint
});

jni!(long queryCastShape(long world, int shape_type, double a, double b, double c, double d, double tx, double ty, double tz, double qi, double qj, double qk, double qw, double vx, double vy, double vz, double max_toi, double target_distance, int stop_at_penetration, int compute_impact_geometry_on_penetration, int flags, int memberships, int filter, int use_groups, long exclude_collider, int use_exclude_collider, long exclude_rigid_body, int use_exclude_rigid_body, long out_hit) {
    let hit = qu::query_cast_shape(
        cp::<WH>(world),
        sd(shape_type, a, b, c, d),
        v3(tx,ty,tz),
        qt(qi,qj,qk,qw),
        v3(vx,vy,vz),
        ShapeCastOptionsDesc { max_time_of_impact: max_toi, target_distance, stop_at_penetration: jb(stop_at_penetration), compute_impact_geometry_on_penetration: jb(compute_impact_geometry_on_penetration) },
        query_filter_args!(flags,memberships,filter,use_groups,exclude_collider,use_exclude_collider,exclude_rigid_body,use_exclude_rigid_body),
    );
    if let Some(out) = unsafe { pm::<ShapeCastHit>(out_hit).as_mut() } { *out = hit; }
    hit.collider as jlong
});

jni!(int neuralBoundsRequiredWeightCount(int hidden_width, int hidden_layers) {
    neu::neural_bounds_required_weight_count(u32_from_jint(hidden_width), u32_from_jint(hidden_layers)) as jint
});

jni!(long worldInsertDynamicCuboids(long world, double x, double y, double z, double qi, double qj, double qk, double qw, double lvx, double lvy, double lvz, long cuboids, int cuboid_count, double density, double friction, double restitution, int collision_memberships, int collision_filter, int solver_memberships, int solver_filter) {
    com::world_insert_dynamic_cuboids(m::<WH>(world), v3(x,y,z), qt(qi,qj,qk,qw), v3(lvx,lvy,lvz), p::<f64>(cuboids), u32_from_jint(cuboid_count), density, friction, restitution, grp(collision_memberships, collision_filter), grp(solver_memberships, solver_filter)) as jlong
});
jni!(long worldInsertStaticTrimesh(long world, long vertices_xyz, int vertex_xyz_len, long indices, int index_len, double friction, double restitution) {
    com::world_insert_static_trimesh(m::<WH>(world), p::<f64>(vertices_xyz), u32_from_jint(vertex_xyz_len), p::<u32>(indices), u32_from_jint(index_len), friction, restitution) as jlong
});
jni!(boolean worldRegisterTerrainGravityPolyhedron(long world, long vertices_xyz, int n_vertices, long face_indices, int n_faces, double density) {
    ev::world_register_terrain_gravity_polyhedron(m::<WH>(world), p::<f64>(vertices_xyz), u32_from_jint(n_vertices), p::<u32>(face_indices), u32_from_jint(n_faces), density).0 as jbyte
});
jni!(boolean worldRegisterTerrainGravityDem(long world, long dem, int nx, int ny, double resolution, double reference_radius, double surface_density) {
    ev::world_register_terrain_gravity_dem(m::<WH>(world), p::<f64>(dem), u32_from_jint(nx), u32_from_jint(ny), resolution, reference_radius, surface_density).0 as jbyte
});
jni!(boolean worldRegisterTerrainGravityMascon(long world) {
    ev::world_register_terrain_gravity_mascon(m::<WH>(world)).0 as jbyte
});
jni!(boolean worldUnregisterTerrainGravity(long world) {
    ev::world_unregister_terrain_gravity(m::<WH>(world)).0 as jbyte
});
jni!(long worldInsertStaticVoxelAabb(long world, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int small_voxel_limit, int mesh_voxel_limit, double friction, double restitution) {
    vx::world_insert_static_voxel_aabb(m::<WH>(world), aa(min_x,min_y,min_z,max_x,max_y,max_z), voxel_size_x, voxel_size_y, voxel_size_z, VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: Bool::FALSE, small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) }, friction, restitution) as jlong
});
jni!(long worldInsertDynamicVoxelObb(long world, double cx, double cy, double cz, double hx, double hy, double hz, double qi, double qj, double qk, double qw, double voxel_size_x, double voxel_size_y, double voxel_size_z, int mode, int small_voxel_limit, int mesh_voxel_limit, double density, double friction, double restitution) {
    vx::world_insert_dynamic_voxel_obb(m::<WH>(world), Obb { center: v3(cx,cy,cz), half_extents: v3(hx,hy,hz), rotation: qt(qi,qj,qk,qw) }, voxel_size_x, voxel_size_y, voxel_size_z, VoxelColliderOptions { mode: voxel_mode(mode), dynamic_body: Bool::TRUE, small_voxel_limit: u32_from_jint(small_voxel_limit), mesh_voxel_limit: u32_from_jint(mesh_voxel_limit) }, density, friction, restitution) as jlong
});

jni!(long jointBuilderCreate(int joint_type, double ax, double ay, double az, double b, double c) {
    to_jlong(jo::joint_builder_create(self::joint_type(joint_type), v3(ax, ay, az), b, c))
});
jni!(void jointBuilderDestroy(long builder) { jo::joint_builder_destroy(m::<JBH>(builder)); });
jni!(void jointBuilderSetContactsEnabled(long builder, int enabled) { jo::joint_builder_set_contacts_enabled(m::<JBH>(builder), jb(enabled)); });
jni!(void jointBuilderSetLocalAnchor1(long builder, double x, double y, double z) { jo::joint_builder_set_local_anchor1(m::<JBH>(builder), v3(x,y,z)); });
jni!(void jointBuilderSetLocalAnchor2(long builder, double x, double y, double z) { jo::joint_builder_set_local_anchor2(m::<JBH>(builder), v3(x,y,z)); });
jni!(void jointBuilderSetLimits(long builder, int axis, double min, double max) { jo::joint_builder_set_limits(m::<JBH>(builder), joint_axis(axis), min, max); });
jni!(void jointBuilderSetMotorVelocity(long builder, int axis, double target_vel, double factor) { jo::joint_builder_set_motor_velocity(m::<JBH>(builder), joint_axis(axis), target_vel, factor); });
jni!(void jointBuilderSetMotorPosition(long builder, int axis, double target_pos, double stiffness, double damping) { jo::joint_builder_set_motor_position(m::<JBH>(builder), joint_axis(axis), target_pos, stiffness, damping); });
jni!(long worldInsertImpulseJoint(long world, long body1, long body2, long builder, int wake_up) { jo::world_insert_impulse_joint(m::<WH>(world), body1 as RRaw, body2 as RRaw, m::<JBH>(builder), jb(wake_up)) as jlong });
jni!(boolean worldRemoveImpulseJoint(long world, long handle, int wake_up) { jo::world_remove_impulse_joint(m::<WH>(world), handle as JRaw, jb(wake_up)).0 as jbyte });

jni!(long characterControllerCreate() { to_jlong(cc::character_controller_create()) });
jni!(void characterControllerDestroy(long controller) { cc::character_controller_destroy(m::<CCH>(controller)); });
jni!(void characterControllerSetUp(long controller, double x, double y, double z) { cc::character_controller_set_up(m::<CCH>(controller), v3(x,y,z)); });
jni!(void characterControllerSetOffsetAbsolute(long controller, double offset) { cc::character_controller_set_offset_absolute(m::<CCH>(controller), offset); });
jni!(void characterControllerSetOffsetRelative(long controller, double offset) { cc::character_controller_set_offset_relative(m::<CCH>(controller), offset); });
jni!(void characterControllerSetSlide(long controller, int slide) { cc::character_controller_set_slide(m::<CCH>(controller), jb(slide)); });
jni!(void characterControllerSetAutostep(long controller, int enabled, double max_height, double min_width, int include_dynamic_bodies) { cc::character_controller_set_autostep(m::<CCH>(controller), jb(enabled), max_height, min_width, jb(include_dynamic_bodies)); });
jni!(void characterControllerSetSnapToGround(long controller, int enabled, double distance) { cc::character_controller_set_snap_to_ground(m::<CCH>(controller), jb(enabled), distance); });
jni!(void characterControllerSetSlopeAngles(long controller, double max_climb_angle, double min_slide_angle) { cc::character_controller_set_slope_angles(m::<CCH>(controller), max_climb_angle, min_slide_angle); });
jni!(boolean characterControllerMoveShape(long world, long controller, double dt, int shape_type, double a, double b, double c, double d, double tx, double ty, double tz, double qi, double qj, double qk, double qw, double dx, double dy, double dz, long out_movement) {
    let movement = cc::character_controller_move_shape(cp::<WH>(world), m::<CCH>(controller), dt, sd(shape_type,a,b,c,d), v3(tx,ty,tz), qt(qi,qj,qk,qw), v3(dx,dy,dz));
    if let Some(out) = unsafe { pm::<EffectiveCharacterMovement>(out_movement).as_mut() } { *out = movement; }
    movement.grounded.0 as jbyte
});
jni!(boolean characterControllerMoveShapeWithTerrain(long world, long controller, double dt, int shape_type, double a, double b, double c, double d, double tx, double ty, double tz, double qi, double qj, double qk, double qw, double dx, double dy, double dz, long out_movement) {
    let movement = cc::character_controller_move_shape_with_terrain(cp::<WH>(world), m::<CCH>(controller), dt, sd(shape_type,a,b,c,d), v3(tx,ty,tz), qt(qi,qj,qk,qw), v3(dx,dy,dz));
    if let Some(out) = unsafe { pm::<EffectiveCharacterMovement>(out_movement).as_mut() } { *out = movement; }
    movement.grounded.0 as jbyte
});
jni!(int characterControllerCollisionCount(long controller) { cc::character_controller_collision_count(cp::<CCH>(controller)) as jint });
jni!(long characterControllerGetCollision(long controller, int index, long out_collision) {
    let collision = cc::character_controller_get_collision(cp::<CCH>(controller), u32_from_jint(index));
    if let Some(out) = unsafe { pm::<CharacterCollision>(out_collision).as_mut() } { *out = collision; }
    collision.collider as jlong
});
jni!(boolean characterControllerSolveImpulses(long world, long controller, double dt, int shape_type, double a, double b, double c, double d, double character_mass) {
    cc::character_controller_solve_impulses(m::<WH>(world), m::<CCH>(controller), dt, sd(shape_type,a,b,c,d), character_mass).0 as jbyte
});

jni!(void worldClearEvents(long world) { ev::world_clear_events(m::<WH>(world)); });
jni!(int worldCollisionEventCount(long world) { ev::world_collision_event_count(cp::<WH>(world)) as jint });
jni!(long worldGetCollisionEvent(long world, int index, long out_event) {
    let event = ev::world_get_collision_event(cp::<WH>(world), u32_from_jint(index));
    if let Some(out) = unsafe { pm::<CER>(out_event).as_mut() } { *out = event; }
    event.collider1 as jlong
});
jni!(int worldGetCollisionEvents(long world, long out_events, int capacity) {
    ev::world_get_collision_events(cp::<WH>(world), pm::<CER>(out_events), u32_from_jint(capacity)) as jint
});
jni!(int worldContactForceEventCount(long world) { ev::world_contact_force_event_count(cp::<WH>(world)) as jint });
jni!(long worldGetContactForceEvent(long world, int index, long out_event) {
    let event = ev::world_get_contact_force_event(cp::<WH>(world), u32_from_jint(index));
    if let Some(out) = unsafe { pm::<ContactForceEventRecord>(out_event).as_mut() } { *out = event; }
    event.collider1 as jlong
});
jni!(int worldGetContactForceEvents(long world, long out_events, int capacity) {
    ev::world_get_contact_force_events(cp::<WH>(world), pm::<ContactForceEventRecord>(out_events), u32_from_jint(capacity)) as jint
});
jni!(void worldClearContactPairFilterCallback(long world) { ev::world_clear_contact_pair_filter_callback(m::<WH>(world)); });
jni!(void worldClearIntersectionPairFilterCallback(long world) { ev::world_clear_intersection_pair_filter_callback(m::<WH>(world)); });

// =========================================================================
// Force law API — Coulomb friction, air drag, external force, Newton gravity
// =========================================================================
jni!(boolean worldSetCoulombFrictionLaw(long world, double static_coefficient, double dynamic_coefficient, double velocity_threshold, int enabled) {
    ev::world_set_coulomb_friction_law(m::<WH>(world), CoulombFrictionLaw {
        static_coefficient, dynamic_coefficient, velocity_threshold, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearCoulombFrictionLaw(long world) { ev::world_clear_coulomb_friction_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });
jni!(boolean worldSetAirDragLaw(long world, double fluid_vx, double fluid_vy, double fluid_vz, double density, double viscosity, double char_len, double ref_area, double cd, double re_limit, int enabled) {
    ev::world_set_air_drag_law(m::<WH>(world), AirDragLaw {
        fluid_velocity: v3(fluid_vx, fluid_vy, fluid_vz), density, dynamic_viscosity: viscosity,
        characteristic_length: char_len, reference_area: ref_area, drag_coefficient: cd,
        reynolds_stokes_limit: re_limit, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearAirDragLaw(long world) { ev::world_clear_air_drag_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });
jni!(boolean worldSetExternalForceLaw(long world, double buoyancy_gravity_x, double buoyancy_gravity_y, double buoyancy_gravity_z, double fluid_density, double displaced_volume, int buoyancy_enabled, double charge, double electric_x, double electric_y, double electric_z, double magnetic_x, double magnetic_y, double magnetic_z, int em_enabled, double spring_x, double spring_y, double spring_z, double spring_stiffness, double spring_damping, int elastic_enabled, double gravity_source_x, double gravity_source_y, double gravity_source_z, double gravitational_parameter, int gravity_enabled, int enabled) {
    ev::world_set_external_force_law(m::<WH>(world), ExternalForceLaw {
        buoyancy_gravity: v3(buoyancy_gravity_x, buoyancy_gravity_y, buoyancy_gravity_z),
        fluid_density, displaced_volume, buoyancy_enabled: jb(buoyancy_enabled),
        charge, electric_field: v3(electric_x, electric_y, electric_z),
        magnetic_field: v3(magnetic_x, magnetic_y, magnetic_z), electromagnetic_enabled: jb(em_enabled),
        spring_anchor: v3(spring_x, spring_y, spring_z), spring_stiffness, spring_damping,
        elastic_enabled: jb(elastic_enabled),
        gravity_source: v3(gravity_source_x, gravity_source_y, gravity_source_z),
        gravitational_parameter, gravity_enabled: jb(gravity_enabled),
        enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearExternalForceLaw(long world) { ev::world_clear_external_force_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });
jni!(boolean worldSetNewtonGravityLaw(long world, double gravitational_constant, double min_distance, double max_distance, int enabled) {
    ev::world_set_newton_gravity_law(m::<WH>(world), NewtonGravityLaw {
        gravitational_constant, min_distance, max_distance, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearNewtonGravityLaw(long world) { ev::world_clear_newton_gravity_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Force law API — solar-wind pressure / dynamical friction / MOND gravity
// (PHYSICS_EXPANSION_PLAN C1: mirrors world_set_solar_wind_pressure_law &
// friends from mps-core events.rs.  See crates/mps-core/src/rapier/events.rs.)
// =========================================================================
jni!(boolean worldSetSolarWindPressureLaw(long world, double proton_density, double v_sw_mps, double wind_dir_x, double wind_dir_y, double wind_dir_z, double effective_area_m2, int enabled) {
    ev::world_set_solar_wind_pressure_law(m::<WH>(world), SolarWindPressureLaw {
        proton_density, v_sw_mps, wind_direction: v3(wind_dir_x, wind_dir_y, wind_dir_z),
        effective_area_m2, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearSolarWindPressureLaw(long world) { ev::world_clear_solar_wind_pressure_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });
jni!(boolean worldSetDynamicalFrictionLaw(long world, double background_density, double coulomb_log, int enabled) {
    ev::world_set_dynamical_friction_law(m::<WH>(world), DynamicalFrictionLaw {
        background_density_kg_m3: background_density, coulomb_log, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearDynamicalFrictionLaw(long world) { ev::world_clear_dynamical_friction_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });
jni!(boolean worldSetMonDGravityLaw(long world, double newtonian_a, double mond_a_zero, double direction_x, double direction_y, double direction_z, int enabled) {
    ev::world_set_mond_gravity_law(m::<WH>(world), MonDGravityLaw {
        newtonian_a, mond_a_zero, direction: v3(direction_x, direction_y, direction_z),
        enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearMonDGravityLaw(long world) { ev::world_clear_mond_gravity_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Force law API — Eddington-limited radiation pressure (PHYSICS_EXPANSION_PLAN C2)
// =========================================================================
jni!(boolean worldSetEddingtonRadiationPressureLaw(long world, double mass_kg, double opacity, double source_x, double source_y, double source_z, double effective_area_m2, int enabled) {
    ev::world_set_eddington_radiation_pressure_law(m::<WH>(world), EddingtonRadiationPressureLaw {
        mass_kg, opacity, source_position: v3(source_x, source_y, source_z),
        effective_area_m2, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearEddingtonRadiationPressureLaw(long world) { ev::world_clear_eddington_radiation_pressure_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Force law API — X-ray disc bolometric irradiation (PHYSICS_EXPANSION_PLAN C3)
// =========================================================================
jni!(boolean worldSetXrayIrradiationLaw(long world, double k_t_eff_kev, double r_in_km, double spectral_hardening, double source_x, double source_y, double source_z, double effective_area_m2, int enabled) {
    ev::world_set_xray_irradiation_law(m::<WH>(world), XrayIrradiationLaw {
        k_t_eff_kev, r_in_km, spectral_hardening, source_position: v3(source_x, source_y, source_z),
        effective_area_m2, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearXrayIrradiationLaw(long world) { ev::world_clear_xray_irradiation_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Force law API — Pulsar magnetic-dipole torque (PHYSICS_EXPANSION_PLAN C3)
// =========================================================================
jni!(boolean worldSetPulsarMagneticDipoleLaw(long world, double moment_of_inertia, double ns_radius_m, double period_ms, double period_derivative, double pulsar_x, double pulsar_y, double pulsar_z, double spin_x, double spin_y, double spin_z, double mu_x, double mu_y, double mu_z, int enabled) {
    ev::world_set_pulsar_magnetic_dipole_law(m::<WH>(world), PulsarMagneticDipoleLaw {
        moment_of_inertia, ns_radius_m, period_ms, period_derivative,
        pulsar_position: v3(pulsar_x, pulsar_y, pulsar_z),
        spin_axis: v3(spin_x, spin_y, spin_z),
        body_dipole_moment: v3(mu_x, mu_y, mu_z),
        enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearPulsarMagneticDipoleLaw(long world) { ev::world_clear_pulsar_magnetic_dipole_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Force law API — Jeans-escape drag (PHYSICS_EXPANSION_PLAN C4)
// =========================================================================
jni!(boolean worldSetJeansEscapeLaw(long world, double n_exo, double temperature, double escape_parameter, double mass_kg, double dir_x, double dir_y, double dir_z, double effective_area_m2, int enabled) {
    ev::world_set_jeans_escape_law(m::<WH>(world), JeansEscapeLaw {
        n_exo, temperature, escape_parameter, mass_kg,
        escape_direction: v3(dir_x, dir_y, dir_z),
        effective_area_m2, enabled: jb(enabled),
    }).0 as jbyte
});
jni!(boolean worldClearJeansEscapeLaw(long world) { ev::world_clear_jeans_escape_law(m::<WH>(world)); Bool::TRUE.0 as jbyte });

// =========================================================================
// Event ring buffer — lock-free dispatch
// =========================================================================
jni!(boolean worldInitCollisionEventRing(long world, int capacity) { ev::world_init_collision_event_ring(m::<WH>(world), u32_from_jint(capacity)).0 as jbyte });
jni!(boolean worldInitContactForceEventRing(long world, int capacity) { ev::world_init_contact_force_event_ring(m::<WH>(world), u32_from_jint(capacity)).0 as jbyte });
jni!(int worldDrainCollisionEventRing(long world, long out_events, int capacity) { ev::world_drain_collision_event_ring(cp::<WH>(world), pm::<CER>(out_events), u32_from_jint(capacity)) as jint });
jni!(int worldDrainContactForceEventRing(long world, long out_events, int capacity) { ev::world_drain_contact_force_event_ring(cp::<WH>(world), pm::<ContactForceEventRecord>(out_events), u32_from_jint(capacity)) as jint });
jni!(int worldCollisionEventRingLen(long world) { ev::world_collision_event_ring_len(cp::<WH>(world)) as jint });
jni!(int worldContactForceEventRingLen(long world) { ev::world_contact_force_event_ring_len(cp::<WH>(world)) as jint });
jni!(boolean worldSetEventDispatchMode(long world, int mode) { ev::world_set_event_dispatch_mode(m::<WH>(world), u32_from_jint(mode)).0 as jbyte });

// =========================================================================
// Aerodynamics — surface & voxel force applications
// =========================================================================
use mps_core::rapier::aerodynamics as aero_jni;
jni!(boolean aeroApplySurfaces(long world, long body, double wind_x, double wind_y, double wind_z, double density, long surfaces, int surface_count, int wake_up, long out_report) {
    aero_jni::aero_apply_surfaces(m::<WH>(world), body as RRaw, v3(wind_x, wind_y, wind_z), density,
        p::<AeroSurface>(surfaces), u32_from_jint(surface_count), jb(wake_up),
        pm::<AeroForceReport>(out_report)).0 as jbyte
});

// =========================================================================
// Fluid dynamics — AABB drag & buoyancy
// =========================================================================
use mps_core::rapier::fluid as fluid_jni;
jni!(boolean fluidApplyAabbForces(long world, long body, double center_x, double center_y, double center_z, double half_x, double half_y, double half_z, double density, double linear_drag, double quadratic_drag, double angular_drag, double flow_x, double flow_y, double flow_z, double gravity_x, double gravity_y, double gravity_z, double body_half_x, double body_half_y, double body_half_z, double body_volume, int wake_up, long out_report) {
    fluid_jni::fluid_apply_aabb_forces(m::<WH>(world), body as RRaw,
        FluidVolume { center: v3(center_x, center_y, center_z), half_extents: v3(half_x, half_y, half_z), density, linear_drag, quadratic_drag, angular_drag, flow_velocity: v3(flow_x, flow_y, flow_z), gravity: v3(gravity_x, gravity_y, gravity_z) },
        v3(body_half_x, body_half_y, body_half_z), body_volume, jb(wake_up),
        pm::<FluidForceReport>(out_report)).0 as jbyte
});

// =========================================================================
// Trajectory — projectile ballistics
// =========================================================================
use mps_core::rapier::trajectory as traj_jni;
jni!(boolean trajectoryApplyForcesToBody(long world, long body, double gravity_x, double gravity_y, double gravity_z, double flow_x, double flow_y, double flow_z, double mass, double ref_area, double density, double drag_coeff, double lift_coeff, double lift_x, double lift_y, double lift_z, int wake_up, long out_report) {
    traj_jni::trajectory_apply_forces_to_body(m::<WH>(world), body as RRaw,
        TrajectoryEnvironment { gravity: v3(gravity_x, gravity_y, gravity_z), flow_velocity: v3(flow_x, flow_y, flow_z), mass, reference_area: ref_area, density, drag_coefficient: drag_coeff, lift_coefficient: lift_coeff, lift_direction: v3(lift_x, lift_y, lift_z) },
        jb(wake_up), pm::<TrajectoryForceReport>(out_report)).0 as jbyte
});

// =========================================================================
// Molecular dynamics — Lennard-Jones & Coulomb potential calculators + forces
// =========================================================================
// `mol` alias is declared in the top-level `use mps_core::rapier::{...}` block.
jni!(double molecularLennardJonesPotential(double r, double epsilon, double sigma) { mol::molecular_lennard_jones_potential(r, epsilon, sigma) });
jni!(double molecularCoulombPotential(double r, double q1, double q2, double k, double eps) { mol::molecular_coulomb_potential(r, q1, q2, k, eps) });
jni!(double molecularVacuumCoulombConstant() { mol::molecular_vacuum_coulomb_constant() });

// Apply intermolecular forces between two rigid bodies in the world.
// `particle_a` / `particle_b` are `long` pointers to a `MolecularParticle`
// byte buffer (80 bytes, C layout: position@0, velocity@24, mass@48, charge@56,
// epsilon@64, sigma@72 — each Vec3 is 24 bytes). `out_report` is a `long`
// pointer to a 128-byte `MolecularPairReport` buffer (displacement@0, distance@24,
// lennard_jones_potential@32, coulomb_potential@40, total_potential@48,
// lennard_jones_force@56, coulomb_force@80, total_force@104). The caller fills
// the two particle buffers with Unsafe; the report buffer is written back.
jni!(boolean molecularApplyPairForces(long world, long body_a, long body_b, long particle_a, long particle_b, double coulomb_constant, double relative_permittivity, double cutoff_radius, double softening, int lennard_jones_enabled, int coulomb_enabled, int wake_up, long out_report) {
    mol::molecular_apply_pair_forces(
        m::<WH>(world), body_a as RRaw, body_b as RRaw,
        unsafe { *p::<MolecularParticle>(particle_a) }, unsafe { *p::<MolecularParticle>(particle_b) },
        MolecularForceLaw { coulomb_constant, relative_permittivity, cutoff_radius, softening, lennard_jones_enabled: jb(lennard_jones_enabled), coulomb_enabled: jb(coulomb_enabled) },
        jb(wake_up), pm::<MolecularPairReport>(out_report)).0 as jbyte
});
jni!(boolean molecularApplyPairForcesFlag(long world, long body_a, long body_b, long particle_a, long particle_b, double coulomb_constant, double relative_permittivity, double cutoff_radius, double softening, int lennard_jones_enabled, int coulomb_enabled, int wake_up, long out_report) {
mol::molecular_apply_pair_forces_flag(
m::<WH>(world), body_a as RRaw, body_b as RRaw,
unsafe { *p::<MolecularParticle>(particle_a) }, unsafe { *p::<MolecularParticle>(particle_b) },
MolecularForceLaw { coulomb_constant, relative_permittivity, cutoff_radius, softening, lennard_jones_enabled: jb(lennard_jones_enabled), coulomb_enabled: jb(coulomb_enabled) },
jb(wake_up), pm::<MolecularPairReport>(out_report)) as jbyte
});

// =========================================================================
// Fracture mechanics — Griffith / S-N / Miner / stress intensity / fragments
// =========================================================================
// All `out_report` args are `long` pointers to C-layout report buffers written
// back to the caller (read with Unsafe). Report sizes/offsets (bytes):
//   StressIntensityReport 24: stress_intensity@0, critical@8(u8), safety_factor@16
//   GriffithReport        32: critical_stress@0, energy_release_rate@8, critical_er_rate@16, will_fracture@24(u8)
//   MinerDamageReport     24: damage@0, remaining_life_fraction@8, failed@16(u8)
//   SnCurveReport        16: cycles_to_failure@0, infinite_life@8(u8)
//   FractureEnergyReport 32: available_energy@0, surface_energy_required@8, fragment_kinetic_energy@16, will_fracture@24(u8)
//   FractureModeReport   24: mode@0(u32), driving_stress@8, mixed_mode_ratio@16
//   FractureReplaceReport 12: fragment_count@0(u32), joint_count@4(u32), removed_source@8(u8)
// `material` (fractureGriffithCriterion) is a `long` to a 40-byte FractureMaterial
//   buffer: youngs_modulus@0, poisson_ratio@8, fracture_toughness@16, surface_energy@24, density@32.
// `fragments` (worldReplaceBodyWithFractureFragments) is a `long` to an array of
//   96-byte FractureFragmentDesc buffers: local_center@0, half_extents@24, initial_velocity@48,
//   density@72, friction@80, restitution@88.
jni!(boolean fractureStressIntensityFactor(double stress, double crack_length, double geometry_factor, double fracture_toughness, long out_report) {
    fr::fracture_stress_intensity_factor(stress, crack_length, geometry_factor, fracture_toughness, pm::<StressIntensityReport>(out_report)).0 as jbyte
});
jni!(boolean fractureGriffithCriterion(double stress, double crack_length, long material, long out_report) {
    fr::fracture_griffith_criterion(stress, crack_length, unsafe { *p::<FractureMaterial>(material) }, pm::<GriffithReport>(out_report)).0 as jbyte
});
jni!(boolean fractureMinerDamage(long cycle_counts, int count, long cycles_to_failure, long out_report) {
    fr::fracture_miner_damage(p::<f64>(cycle_counts), p::<f64>(cycles_to_failure), count as u32, pm::<MinerDamageReport>(out_report)).0 as jbyte
});
jni!(boolean fractureSnCurveLife(double stress_amplitude, double coefficient, double exponent, double endurance_limit, long out_report) {
    fr::fracture_sn_curve_life(stress_amplitude, coefficient, exponent, endurance_limit, pm::<SnCurveReport>(out_report)).0 as jbyte
});
jni!(boolean fractureEnergyRelease(double strain_energy, double new_surface_area, double surface_energy, double kinetic_energy, long out_report) {
    fr::fracture_energy_release(strain_energy, new_surface_area, surface_energy, kinetic_energy, pm::<FractureEnergyReport>(out_report)).0 as jbyte
});
jni!(boolean fractureModeFromStress(double tensile_stress, double shear_stress, double compressive_stress, long out_report) {
    fr::fracture_mode_from_stress(tensile_stress, shear_stress, compressive_stress, pm::<FractureModeReport>(out_report)).0 as jbyte
});
jni!(boolean worldReplaceBodyWithFractureFragments(long world, long source_body, long fragments, int fragment_count, int connect_fragments, int remove_source, long out_body_handles, long out_joint_handles, int capacity, long out_report) {
    fr::world_replace_body_with_fracture_fragments(
        m::<WH>(world), source_body as RRaw, p::<FractureFragmentDesc>(fragments),
        fragment_count as u32, jb(connect_fragments), jb(remove_source),
        pm::<RRaw>(out_body_handles), pm::<JRaw>(out_joint_handles), capacity as u32,
        pm::<FractureReplaceReport>(out_report)).0 as jbyte
});

#[cfg(feature = "anvilkit-bridge")]
jni!(long anvilKitAppCreate() { to_jlong(ak::anvilkit_app_create()) });
#[cfg(feature = "anvilkit-bridge")]
jni!(void anvilKitAppDestroy(long app) { ak::anvilkit_app_destroy(m::<AKH>(app)); });
#[cfg(feature = "anvilkit-bridge")]
jni!(void anvilKitAppUpdate(long app) { ak::anvilkit_app_update(m::<AKH>(app)); });
#[cfg(feature = "anvilkit-bridge")]
jni!(long anvilKitAppSpawnBody(long app, double tx, double ty, double tz, double qi, double qj, double qk, double qw, int status) {
    ak::anvilkit_app_spawn_body(m::<AKH>(app), v3(tx, ty, tz), qt(qi, qj, qk, qw), u32_from_jint(status)) as jlong
});
#[cfg(feature = "anvilkit-bridge")]
jni!(long anvilKitAppSpawnBodyWithCollider(long app, double tx, double ty, double tz, double qi, double qj, double qk, double qw, int status, int shape_type, double a, double b, double c, double d) {
    ak::anvilkit_app_spawn_body_with_collider(m::<AKH>(app), v3(tx, ty, tz), qt(qi, qj, qk, qw), u32_from_jint(status), sd(shape_type, a, b, c, d)) as jlong
});
#[cfg(feature = "anvilkit-bridge")]
jni!(boolean anvilKitAppSetTransform(long app, long entity_bits, double tx, double ty, double tz, double qi, double qj, double qk, double qw) {
    ak::anvilkit_app_set_transform(m::<AKH>(app), entity_bits as u64, v3(tx, ty, tz), qt(qi, qj, qk, qw)).0 as jbyte
});
// =========================================================================
// Material mechanics — Hooke / elastic moduli / yield / fracture / fatigue / beam
// =========================================================================
// All functions take `f64` inputs and a `long out` pointer to a caller-allocated
// `f64` slot; the computed value is written there and the result is `boolean`
// (false on invalid input or null out). `principal_stresses` writes 3 `f64` into
// a 24-byte buffer; `miners_damage` reads `count` `f64` from a `ratios` buffer.
jni!(boolean materialMechanicsHookesLawUniaxial(double stress, double youngs_modulus, long out) { mm::material_mechanics_hookes_law_uniaxial(stress, youngs_modulus, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsStressFromStrain(double youngs_modulus, double strain, long out) { mm::material_mechanics_stress_from_strain(youngs_modulus, strain, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsShearModulus(double youngs_modulus, double poisson_ratio, long out) { mm::material_mechanics_shear_modulus(youngs_modulus, poisson_ratio, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsBulkModulus(double youngs_modulus, double poisson_ratio, long out) { mm::material_mechanics_bulk_modulus(youngs_modulus, poisson_ratio, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsLameLambda(double youngs_modulus, double poisson_ratio, long out) { mm::material_mechanics_lame_lambda(youngs_modulus, poisson_ratio, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsVonMisesStress(double sx, double sy, double sz, double txy, double tyz, double tzx, long out) { mm::material_mechanics_von_mises_stress(sx, sy, sz, txy, tyz, tzx, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsVonMisesYieldCheck(double von_mises_stress, double yield_stress, long out) { mm::material_mechanics_von_mises_yield_check(von_mises_stress, yield_stress, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsTrescaShearStress(double sigma_1, double sigma_3, long out) { mm::material_mechanics_tresca_shear_stress(sigma_1, sigma_3, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsTrescaYieldCheck(double sigma_1, double sigma_3, double yield_stress, long out) { mm::material_mechanics_tresca_yield_check(sigma_1, sigma_3, yield_stress, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsKiCenterCrack(double stress, double crack_half_length, long out) { mm::material_mechanics_ki_center_crack(stress, crack_half_length, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsKiEdgeCrack(double stress, double crack_length, long out) { mm::material_mechanics_ki_edge_crack(stress, crack_length, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsFractureCheck(double stress_intensity, double fracture_toughness, long out) { mm::material_mechanics_fracture_check(stress_intensity, fracture_toughness, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsCriticalCrackLength(double stress, double fracture_toughness, long out) { mm::material_mechanics_critical_crack_length(stress, fracture_toughness, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsBasquinStressAmplitude(double cycles_to_failure, double fatigue_strength_coefficient, double fatigue_exponent, long out) { mm::material_mechanics_basquin_stress_amplitude(cycles_to_failure, fatigue_strength_coefficient, fatigue_exponent, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsBasquinCyclesToFailure(double stress_amplitude, double fatigue_strength_coefficient, double fatigue_exponent, long out) { mm::material_mechanics_basquin_cycles_to_failure(stress_amplitude, fatigue_strength_coefficient, fatigue_exponent, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsCoffinMansonStrainAmplitude(double cycles_to_failure, double ductility_coefficient, double ductility_exponent, long out) { mm::material_mechanics_coffin_manson_strain_amplitude(cycles_to_failure, ductility_coefficient, ductility_exponent, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsGoodmanCorrection(double stress_amplitude, double mean_stress, double ultimate_tensile, long out) { mm::material_mechanics_goodman_correction(stress_amplitude, mean_stress, ultimate_tensile, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsNortonCreepRate(double stress, double temperature, double a, double n, double activation_energy, double gas_constant, long out) { mm::material_mechanics_norton_creep_rate(stress, temperature, a, n, activation_energy, gas_constant, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsBeamBendingStress(double bending_moment, double distance_from_neutral_axis, double area_moment_of_inertia, long out) { mm::material_mechanics_beam_bending_stress(bending_moment, distance_from_neutral_axis, area_moment_of_inertia, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsBeamDeflectionCenterPointLoad(double load, double span, double youngs_modulus, double moment_of_inertia, long out) { mm::material_mechanics_beam_deflection_center_point_load(load, span, youngs_modulus, moment_of_inertia, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsEulerBucklingLoad(double youngs_modulus, double moment_of_inertia, double effective_length_factor, double column_length, long out) { mm::material_mechanics_euler_buckling_load(youngs_modulus, moment_of_inertia, effective_length_factor, column_length, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsSlendernessRatio(double effective_length_factor, double column_length, double radius_of_gyration, long out) { mm::material_mechanics_slenderness_ratio(effective_length_factor, column_length, radius_of_gyration, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsPrincipalStresses(double sx, double sy, double sz, double txy, double tyz, double tzx, long out) { mm::material_mechanics_principal_stresses(sx, sy, sz, txy, tyz, tzx, pm::<f64>(out)).0 as jbyte });
jni!(boolean materialMechanicsMinersDamage(long ratios, int count, long out) { mm::material_mechanics_miners_damage(p::<f64>(ratios), count as u32, pm::<f64>(out)).0 as jbyte });

// =========================================================================
// Thermodynamics — ideal gas law & polytropic processes
// =========================================================================
jni!(boolean thermodynamicsIdealGasPressure(double volume, double moles, double temperature, long out) { th::thermodynamics_ideal_gas_pressure(volume, moles, temperature, pm::<f64>(out)).0 as jbyte });
jni!(boolean thermodynamicsIdealGasVolume(double pressure, double moles, double temperature, long out) { th::thermodynamics_ideal_gas_volume(pressure, moles, temperature, pm::<f64>(out)).0 as jbyte });
jni!(boolean thermodynamicsIdealGasTemperature(double pressure, double volume, double moles, long out) { th::thermodynamics_ideal_gas_temperature(pressure, volume, moles, pm::<f64>(out)).0 as jbyte });
jni!(boolean thermodynamicsPolytropicPressure(double p1, double v1, double v2, double gamma, long out) { th::thermodynamics_polytropic_pressure(p1, v1, v2, gamma, pm::<f64>(out)).0 as jbyte });
jni!(boolean thermodynamicsPolytropicWork(double p1, double v1, double p2, double v2, double gamma, long out) { th::thermodynamics_polytropic_work(p1, v1, p2, v2, gamma, pm::<f64>(out)).0 as jbyte });
#[cfg(feature = "anvilkit-bridge")]
jni!(int anvilKitAppSyncToWorld(long app, long world) {
    ak::anvilkit_app_sync_to_world(m::<AKH>(app), m::<WH>(world)) as jint
});
#[cfg(feature = "anvilkit-bridge")]
jni!(long anvilKitAppEntityToBody(long app, long entity_bits) {
    ak::anvilkit_app_entity_to_body(cp::<AKH>(app), entity_bits as u64) as jlong
});
#[cfg(feature = "anvilkit-bridge")]
jni!(long anvilKitAppEntityToCollider(long app, long entity_bits) {
    ak::anvilkit_app_entity_to_collider(cp::<AKH>(app), entity_bits as u64) as jlong
});
#[cfg(feature = "anvilkit-bridge")]
jni!(boolean anvilKitAppApplyAeroSurfaces(long app, long world, long entity_bits, double wind_x, double wind_y, double wind_z, double air_density, long surfaces, int surface_count, int wake_up, long out_report) {
    ak::anvilkit_app_apply_aero_surfaces(m::<AKH>(app), m::<WH>(world), entity_bits as u64, v3(wind_x, wind_y, wind_z), air_density, p::<AeroSurface>(surfaces), u32_from_jint(surface_count), jb(wake_up), pm::<AeroForceReport>(out_report)).0 as jbyte
});
#[cfg(feature = "anvilkit-bridge")]
jni!(boolean anvilKitAppApplyAeroVoxelGrid(long app, long world, long entity_bits, double wind_x, double wind_y, double wind_z, double air_density, long voxels, int size_x, int size_y, int size_z, double voxel_size, double origin_x, double origin_y, double origin_z, double drag_coefficient, double lift_coefficient, int wake_up, long out_report) {
    ak::anvilkit_app_apply_aero_voxel_grid(m::<AKH>(app), m::<WH>(world), entity_bits as u64, v3(wind_x, wind_y, wind_z), air_density, p::<u8>(voxels), u32_from_jint(size_x), u32_from_jint(size_y), u32_from_jint(size_z), voxel_size, v3(origin_x, origin_y, origin_z), drag_coefficient, lift_coefficient, jb(wake_up), pm::<AeroForceReport>(out_report)).0 as jbyte
});
#[cfg(feature = "anvilkit-bridge")]
jni!(boolean anvilKitAppApplyFluidAabbForces(long app, long world, long entity_bits, double center_x, double center_y, double center_z, double half_x, double half_y, double half_z, double density, double linear_drag, double quadratic_drag, double angular_drag, double flow_x, double flow_y, double flow_z, double gravity_x, double gravity_y, double gravity_z, double body_half_x, double body_half_y, double body_half_z, double body_volume, int wake_up, long out_report) {
    ak::anvilkit_app_apply_fluid_aabb_forces(
        m::<AKH>(app),
        m::<WH>(world),
        entity_bits as u64,
        FluidVolume {
            center: v3(center_x, center_y, center_z),
            half_extents: v3(half_x, half_y, half_z),
            density,
            linear_drag,
            quadratic_drag,
            angular_drag,
            flow_velocity: v3(flow_x, flow_y, flow_z),
            gravity: v3(gravity_x, gravity_y, gravity_z),
        },
        v3(body_half_x, body_half_y, body_half_z),
        body_volume,
        jb(wake_up),
        pm::<FluidForceReport>(out_report)
    ).0 as jbyte
});
#[cfg(feature = "anvilkit-bridge")]
jni!(boolean anvilKitAppApplyTrajectoryForces(long app, long world, long entity_bits, double gravity_x, double gravity_y, double gravity_z, double flow_x, double flow_y, double flow_z, double mass, double reference_area, double density, double drag_coefficient, double lift_coefficient, double lift_x, double lift_y, double lift_z, int wake_up, long out_report) {
    ak::anvilkit_app_apply_trajectory_forces(
        m::<AKH>(app),
        m::<WH>(world),
        entity_bits as u64,
        TrajectoryEnvironment {
            gravity: v3(gravity_x, gravity_y, gravity_z),
            flow_velocity: v3(flow_x, flow_y, flow_z),
            mass,
            reference_area,
            density,
            drag_coefficient,
            lift_coefficient,
            lift_direction: v3(lift_x, lift_y, lift_z),
        },
        jb(wake_up),
        pm::<TrajectoryForceReport>(out_report)
    ).0 as jbyte
});

jni!(double spaceKeplerPeriod(double mu, double semi_major_axis) { sf::space_kepler_period(mu, semi_major_axis) });
jni!(double spaceKeplerSemiMajorAxis(double mu, double period) { sf::space_kepler_semi_major_axis(mu, period) });
jni!(boolean spaceHohmannTransfer(double mu, double radius1, double radius2, long out_transfer) {
    sf::space_hohmann_transfer(mu, radius1, radius2, pm::<HohmannTransfer>(out_transfer)).0 as jbyte
});
jni!(boolean spaceAtmosphericDragAcceleration(double vx, double vy, double vz, double avx, double avy, double avz, double density, double drag_coefficient, double area, double mass, long out_acceleration) {
    sf::space_atmospheric_drag_acceleration(v3(vx, vy, vz), v3(avx, avy, avz), density, drag_coefficient, area, mass, pm::<Vec3>(out_acceleration)).0 as jbyte
});
jni!(boolean spaceApplyAtmosphericDragToBody(long world, long body, double avx, double avy, double avz, double density, double drag_coefficient, double area, double mass, int wake_up, long out_acceleration) {
    sf::space_apply_atmospheric_drag_to_body(m::<WH>(world), body as RRaw, v3(avx, avy, avz), density, drag_coefficient, area, mass, jb(wake_up), pm::<Vec3>(out_acceleration)).0 as jbyte
});
jni!(boolean spaceTriadAttitude(double b1x, double b1y, double b1z, double b2x, double b2y, double b2z, double r1x, double r1y, double r1z, double r2x, double r2y, double r2z, long out_attitude) {
    sf::space_triad_attitude(v3(b1x, b1y, b1z), v3(b2x, b2y, b2z), v3(r1x, r1y, r1z), v3(r2x, r2y, r2z), pm::<Quat>(out_attitude)).0 as jbyte
});
jni!(boolean spaceQuaternionDerivative(double qi, double qj, double qk, double qw, double wx, double wy, double wz, long out_derivative) {
    sf::space_quaternion_derivative(qt(qi, qj, qk, qw), v3(wx, wy, wz), pm::<QuaternionDerivative>(out_derivative)).0 as jbyte
});
jni!(boolean spaceEkfPredictScalar(double state, double covariance, double nonlinear_delta, double jacobian, double process_noise, long out_prediction) {
    sf::space_ekf_predict_scalar(state, covariance, nonlinear_delta, jacobian, process_noise, pm::<ScalarKalman>(out_prediction)).0 as jbyte
});
jni!(double spaceEkfGainScalar(double covariance, double measurement_jacobian, double measurement_noise) {
    sf::space_ekf_gain_scalar(covariance, measurement_jacobian, measurement_noise)
});
jni!(boolean spaceEkfUpdateScalar(double predicted_state, double predicted_covariance, double measurement, double predicted_measurement, double kalman_gain, double measurement_jacobian, long out_update) {
    sf::space_ekf_update_scalar(predicted_state, predicted_covariance, measurement, predicted_measurement, kalman_gain, measurement_jacobian, pm::<ScalarKalman>(out_update)).0 as jbyte
});

jni!(long rtreeCreate() { to_jlong(rt::rtree_create()) });
jni!(void rtreeDestroy(long tree) { rt::rtree_destroy(m::<RTH>(tree)); });
jni!(void rtreeClear(long tree) { rt::rtree_clear(m::<RTH>(tree)); });
jni!(int rtreeLen(long tree) { rt::rtree_len(cp::<RTH>(tree)) as jint });
jni!(boolean rtreeInsert(long tree, long id, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { rt::rtree_insert(m::<RTH>(tree), id as u64, aa(min_x,min_y,min_z,max_x,max_y,max_z)).0 as jbyte });
jni!(boolean rtreeUpdate(long tree, long id, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { rt::rtree_update(m::<RTH>(tree), id as u64, aa(min_x,min_y,min_z,max_x,max_y,max_z)).0 as jbyte });
jni!(boolean rtreeRemove(long tree, long id) { rt::rtree_remove(m::<RTH>(tree), id as u64).0 as jbyte });
jni!(void rtreeRebuild(long tree) { rt::rtree_rebuild(m::<RTH>(tree)); });
jni!(int rtreeQueryAabbCount(long tree, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { rt::rtree_query_aabb_count(m::<RTH>(tree), aa(min_x,min_y,min_z,max_x,max_y,max_z)) as jint });
jni!(int rtreeQueryAabb(long tree, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, long out_ids, int capacity) { rt::rtree_query_aabb(m::<RTH>(tree), aa(min_x,min_y,min_z,max_x,max_y,max_z), pm::<u64>(out_ids), u32_from_jint(capacity)) as jint });

jni!(long crbTreeCreate() { to_jlong(crt::crb_tree_create()) });
jni!(void crbTreeDestroy(long tree) { crt::crb_tree_destroy(m::<CRTH>(tree)); });
jni!(void crbTreeClear(long tree) { crt::crb_tree_clear(m::<CRTH>(tree)); });
jni!(int crbTreeLen(long tree) { crt::crb_tree_len(cp::<CRTH>(tree)) as jint });
jni!(boolean crbTreeInsert(long tree, long id, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { crt::crb_tree_insert(m::<CRTH>(tree), id as u64, aa(min_x,min_y,min_z,max_x,max_y,max_z)).0 as jbyte });
jni!(boolean crbTreeUpdate(long tree, long id, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { crt::crb_tree_update(m::<CRTH>(tree), id as u64, aa(min_x,min_y,min_z,max_x,max_y,max_z)).0 as jbyte });
jni!(boolean crbTreeRemove(long tree, long id) { crt::crb_tree_remove(m::<CRTH>(tree), id as u64).0 as jbyte });
jni!(int crbTreeQueryAabbCount(long tree, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z) { crt::crb_tree_query_aabb_count(cp::<CRTH>(tree), aa(min_x,min_y,min_z,max_x,max_y,max_z)) as jint });
jni!(int crbTreeQueryAabb(long tree, double min_x, double min_y, double min_z, double max_x, double max_y, double max_z, long out_ids, int capacity) { crt::crb_tree_query_aabb(cp::<CRTH>(tree), aa(min_x,min_y,min_z,max_x,max_y,max_z), pm::<u64>(out_ids), u32_from_jint(capacity)) as jint });

// =========================================================================
// Zero-copy bridge functions — eliminate per-frame JNI allocation
// =========================================================================
use mps_core::rapier::bridge as br;

jni!(int bridgeBulkBodySnapshot(long world, long out_address, int capacity) {
    br::bulk_body_snapshot_to_direct_buffer(cp::<WH>(world), out_address, capacity) as jint
});

jni!(boolean bridgeVec3ToSlot(double x, double y, double z, long slot) {
    br::write_vec3_to_slot(slot, v3(x, y, z)).into()
});

jni!(boolean bridgeQuatToSlot(double i, double j, double k, double w, long slot) {
    br::write_quat_to_slot(slot, qt(i, j, k, w)).into()
});

jni!(int bridgeWriteF64Slice(long values, int value_count, long slot, int capacity) {
    if value_count < 0 {
        er::set_error(er::ERR_INVALID_ARGUMENT, "invalid value count");
        return 0;
    }
    let v = unsafe { std::slice::from_raw_parts(p::<f64>(values), value_count as usize) };
    br::write_f64_slice(slot, v, capacity) as jint
});

jni!(long bridgeVoxelColliderFromDirectBuffer(long world, long voxel_address, int size_x, int size_y, int size_z, double voxel_size_x, double voxel_size_y, double voxel_size_z, double origin_x, double origin_y, double origin_z, int mode, int dynamic_body, int small_voxel_limit, int mesh_voxel_limit) {
    br::voxel_collider_from_direct_buffer(m::<WH>(world), voxel_address, size_x, size_y, size_z, voxel_size_x, voxel_size_y, voxel_size_z, origin_x, origin_y, origin_z, mode, dynamic_body != 0, small_voxel_limit, mesh_voxel_limit)
});

// =========================================================================
// Shared Arena — zero-JNI physics state read/write
// =========================================================================

jni!(boolean worldCreateSharedArena(long world, int max_bodies, int max_colliders, int max_events, int max_commands, long out_address, long out_size) {
    wo::world_create_shared_arena(m::<WH>(world), u32_from_jint(max_bodies), u32_from_jint(max_colliders), u32_from_jint(max_events), u32_from_jint(max_commands), pm::<u64>(out_address), pm::<u64>(out_size)).0 as jbyte
});
jni!(void worldDestroySharedArena(long world) { wo::world_destroy_shared_arena(m::<WH>(world)); });
jni!(long worldGetSharedArenaAddress(long world) { wo::world_get_shared_arena_address(cp::<WH>(world)) as jlong });
jni!(long worldGetSharedArenaSize(long world) { wo::world_get_shared_arena_size(cp::<WH>(world)) as jlong });
/// Returns the arena wrapped as a Java DirectByteBuffer.
///
/// This uses `NewDirectByteBuffer` — a standard JNI API since Java 1.4.
/// The returned ByteBuffer wraps the native arena memory directly, enabling
/// zero-JNI reads/writes from pure `java.nio.ByteBuffer` / `java.nio.DoubleBuffer`.
#[unsafe(export_name = "Java_org_polaris2023_mps_rapier_RapierNative_worldGetArenaDirectByteBuffer")]
#[allow(non_snake_case)]
pub extern "system" fn worldGetArenaDirectByteBuffer(
    env: JNIEnv,
    _class: jclass,
    world: jlong,
) -> ljni::sys::jobject {
    catch_unwind(AssertUnwindSafe(|| {
        let world = world as *mut WH;
        let addr = wo::world_get_shared_arena_address(world);
        let size = wo::world_get_shared_arena_size(world);
        if addr == 0 || size == 0 {
            return std::ptr::null_mut();
        }
        let env_raw: *mut JNIEnv = &raw const env as *mut _;
        let env = unsafe { &mut *env_raw };
        unsafe { env.new_direct_byte_buffer(addr as _, size as _) }
            .map(|bb| bb.as_raw())
            .unwrap_or(std::ptr::null_mut())
    }))
    .unwrap_or(std::ptr::null_mut())
}

// =========================================================================
// Space flight — apply-to-body functions
//
// NOTE: These accept `out_accel` as a native-memory output pointer (long).
// Callers allocate 3×f64 (=24 bytes) of native memory and pass the address.
// =========================================================================

jni!(boolean spaceApplyJ2ForceToBody(long world, long body, double mu, double equatorial_radius, double j2, double mass, int wake_up, long out_acceleration) {
    sf::space_apply_j2_force_to_body(m::<WH>(world), body as RRaw, mu, equatorial_radius, j2, mass, jb(wake_up), pm::<Vec3>(out_acceleration)).0 as jbyte
});

jni!(boolean spaceApplySolarRadiationPressureToBody(long world, long body, double sun_x, double sun_y, double sun_z, double solar_flux, double reflectivity, double area, double mass, int wake_up, long out_acceleration) {
    sf::space_apply_solar_radiation_pressure_to_body(m::<WH>(world), body as RRaw, v3(sun_x, sun_y, sun_z), solar_flux, reflectivity, area, mass, jb(wake_up), pm::<Vec3>(out_acceleration)).0 as jbyte
});

jni!(boolean spaceApplyGravityGradientTorqueToBody(long world, long body, double ix, double iy, double iz, double mu, int wake_up, long out_torque) {
    sf::space_apply_gravity_gradient_torque_to_body(m::<WH>(world), body as RRaw, v3(ix, iy, iz), mu, jb(wake_up), pm::<Vec3>(out_torque)).0 as jbyte
});

// =========================================================================
// Cosmos — 太空刚体演算 (mps-cosmos)
//
// 与 `mps-core` 的 world 不同，`CosmosWorld` 是一个面向轨道演算的独立
// 物理 world：自行持有 `RigidBodySet`/`PhysicsPipeline`，仅复用
// `mps-formula` 的纯计算。这里把它的一组核心 `pub` API 包成 JNI export，
// 供 Java 端做太空场景演练。
//
// 句柄约定：
//   `long world`   —— `*mut CosmosWorld`
//   `long builder` —— `*mut RigidBodyBuilder`（由 `cosmosSatelliteBuilder` /
//                     `cosmosFixedBodyBuilder` 返回，**插入后所有权转移**
//                     给 `CosmosWorld`；不插入则调用方用
//                     `cosmosBuilderDestroy` 释放）
//   `long body`    —— `RigidBodyHandle` 的 `into_raw_parts()` 打包成的单个
//                     64 位（高 32 = index，低 32 = generation）。之所以不
//                     拆成两个 jint，是为了和 rapier 的 `RigidBodyHandleRaw`
//                     在 ABI 上对齐（后者也是单 u64）。
// =========================================================================
use mps_cosmos::gravity::CelestialSource;
use mps_cosmos::rapier3d::prelude::{RigidBodyBuilder, RigidBodyHandle, Vector};
use mps_cosmos::world::{
    CosmosWorld, CosmosWorldConfig, OrbitIntegration, StepResult, StepSkipReason,
};

/// `RigidBodyHandle` ↔ `jlong` 打包。高 32 位存 index，低 32 位存
/// generation —— 与 `RigidBodyHandle::into_raw_parts()` 顺序一致。
fn pack_handle(h: RigidBodyHandle) -> jlong {
    let (idx, generation) = h.into_raw_parts();
    (((idx as u64) << 32) | (generation as u64)) as i64
}

fn unpack_handle(packed: jlong) -> RigidBodyHandle {
    let packed = packed as u64;
    let idx = ((packed >> 32) & 0xFFFF_FFFF) as u32;
    let generation = (packed & 0xFFFF_FFFF) as u32;
    RigidBodyHandle::from_raw_parts(idx, generation)
}

/// 由 `CelestialBodyId`（整数 0..=9）拿 `&'static CelestialBody`；非法则
/// `None`。对应 `mps_formula::celestial_data::celestial_body_id_from_u32`。
fn celestial_by_id(id: jint) -> Option<&'static mps_formula::celestial_data::CelestialBody> {
    let id = u32::try_from(id).ok()?;
    mps_formula::celestial_data::celestial_body_id_from_u32(id)
        .map(mps_formula::celestial_data::get_celestial_body)
}

/// 取裸指针指向的 `RigidBodyBuilder` 的可变引用（builder 链式 set 用）。
/// 0（null）则返回 `None`（与 catch_unwind 兜底一致，避免 panic）。
fn builder_mut(builder: jlong) -> Option<&'static mut RigidBodyBuilder> {
    if builder == 0 {
        return None;
    }
    unsafe { (builder as *mut RigidBodyBuilder).as_mut() }
}

// 构造一个动态刚体 builder（质量 kg、初始位置/速度）。返回 `*mut` 指针
// 给 Java；后续交给 `cosmosInsertBody` 插入。
jni!(long cosmosSatelliteBuilder(double mass, double px, double py, double pz, double vx, double vy, double vz, double radius) {
    to_jlong(Box::into_raw(Box::new(
        mps_cosmos::bodies::satellite_builder(mass, Vector::new(px, py, pz), Vector::new(vx, vy, vz), radius)
    )))
});
// 构造固定（静态）刚体 builder —— 适合做 n-body 引力源中心本体。
jni!(long cosmosFixedBodyBuilder(double px, double py, double pz) {
    to_jlong(Box::into_raw(Box::new(
        mps_cosmos::bodies::fixed_body_builder(Vector::new(px, py, pz))
    )))
});
// 链式设惯量/阻尼等常见 builder 属性后再交给 `cosmosInsertBody`。这里
// 暴露线性/角阻尼、平移锁定、`gravity_scale` 几个最常用的。
jni!(void cosmosBuilderSetLinearDamping(long builder, double value) {
    if let Some(b) = builder_mut(builder) { b.linear_damping = value; }
});
jni!(void cosmosBuilderSetAngularDamping(long builder, double value) {
    if let Some(b) = builder_mut(builder) { b.angular_damping = value; }
});
jni!(void cosmosBuilderSetGravityScale(long builder, double value) {
    if let Some(b) = builder_mut(builder) { b.gravity_scale = value; }
});
// **激活**平移锁定（动态刚体不再平动，仅可转动）。
// `RigidBodyBuilder::lock_translations` 是消费 self 的链式 API，这里
// 把裸指针的 builder 取出、调用后再放回原地，等价于链尾 `.lock_translations()`。
jni!(void cosmosBuilderLockTranslations(long builder) {
    if builder != 0 {
        unsafe {
            let b = Box::from_raw(builder as *mut RigidBodyBuilder);
            let b = b.lock_translations();
            let _ = Box::into_raw(Box::new(b));
        }
    }
});
// 显式释放一个**未插入**的 builder。插入 `cosmosInsertBody` 后所有权已
// 转移，**不要**再调本函数（会 double-free）。
jni!(void cosmosBuilderDestroy(long builder) {
    if builder != 0 { drop(unsafe { Box::from_raw(builder as *mut RigidBodyBuilder) }); }
});

// 创建一个 `CosmosWorld`。
//
// 参数：
// - `dt`：积分步长（秒），合法范围 `0 < dt ≤ 30`；
// - `solver_iterations`、`ccd_substeps`：rapier 求解器参数；
// - `orbit_integration`：0 = `RapierForce`（默认），1 = `Verlet`，
//   2 = `Yoshida4`，3 = `Yoshida4Kahan`，4 = `ForestRuth8`，5 = `ForestRuth8Kahan`；
// - `verlet_substeps`：Verlet 路径的内部子步数（≥1，仅 `Verlet` 模式生效）；
// - `n_body_softening_sq`：n-body 互引力软化平方项（m²），0 表示无软化。
jni!(long cosmosWorldCreate(double dt, int solver_iterations, int ccd_substeps, int orbit_integration, int verlet_substeps, double n_body_softening_sq) {
    let orbit_integration = match u32_from_jint(orbit_integration) {
        1 => OrbitIntegration::Verlet,
        2 => OrbitIntegration::Yoshida4,
        3 => OrbitIntegration::Yoshida4Kahan,
        4 => OrbitIntegration::ForestRuth8,
        5 => OrbitIntegration::ForestRuth8Kahan,
        _ => OrbitIntegration::RapierForce,
    };
    let cfg = CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt,
        solver_iterations: u32_from_jint(solver_iterations),
        ccd_substeps: u32_from_jint(ccd_substeps),
        n_body_softening_sq,
        central_body: None,
        orbit_integration,
        verlet_substeps: u32_from_jint(verlet_substeps).max(1),
        ..CosmosWorldConfig::default()
    };
    to_jlong(Box::into_raw(Box::new(CosmosWorld::new(cfg))))
});
jni!(void cosmosWorldDestroy(long world) {
    if world != 0 { drop(unsafe { Box::from_raw(world as *mut CosmosWorld) }); }
});

// 设太阳位置（光压方向参考）。
jni!(void cosmosWorldSetSunPosition(long world, double x, double y, double z) {
    if let Some(w) = unsafe { (world as *mut CosmosWorld).as_mut() } {
        w.set_sun_position(Vector::new(x, y, z));
    }
});

// 设 n-body 中心天体（按整数 id：0=Sun,1=Mercury,2=Venus,3=Earth,4=Moon,
// 5=Mars,6=Jupiter,7=Saturn,8=Uranus,9=Neptune）。`id < 0` 清除中心天体。
jni!(boolean cosmosWorldSetCentralBody(long world, int id) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return Bool::FALSE.0 as jbyte; };
    let body = if id < 0 { None } else { celestial_by_id(id) };
    w.set_central_body(body);
    Bool::TRUE.0 as jbyte
});

// 注册一个天体引力源。`celestial_id` 见 `cosmosWorldSetCentralBody`；
// `max_sh_degree` 限制球谐展开最高阶（受 `body.max_degree` 上限约束）。
// 返回注册到世界中的源索引（≥0 成功；-1 参数错）。
jni!(int cosmosWorldAddCelestial(long world, int celestial_id, int max_sh_degree) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return -1; };
    let Some(body) = celestial_by_id(celestial_id) else { return -1; };
    let src = CelestialSource::new(body, u32_from_jint(max_sh_degree));
    w.add_celestial(src) as jint
});

// 把已插入的刚体登记为 n-body 互引力质点源（给定质量 kg）。
jni!(boolean cosmosWorldAddNBody(long world, long body, double mass) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return Bool::FALSE.0 as jbyte; };
    w.add_n_body(unpack_handle(body), mass);
    Bool::TRUE.0 as jbyte
});

// 一步到位：插入 builder 并把其质量登记为 n-body 源。builder 所有权转移
// （插入后不可再 `cosmosBuilderDestroy`）。返回打包的 body 句柄（0 = 失败）。
jni!(long cosmosWorldInsertBodyAsGravitySource(long world, long builder, double mass) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return 0; };
    if builder == 0 { return 0; }
    let b = *unsafe { Box::from_raw(builder as *mut RigidBodyBuilder) };
    pack_handle(w.insert_body_as_gravity_source(b, mass))
});

// 插入 builder，返回打包的 body 句柄（0 = 失败）。builder 所有权转移。
jni!(long cosmosWorldInsertBody(long world, long builder) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return 0; };
    if builder == 0 { return 0; }
    let b = *unsafe { Box::from_raw(builder as *mut RigidBodyBuilder) };
    pack_handle(w.insert_body(b))
});

// 设置某刚体的环境扰动配置（大气阻力 + 太阳光压 + 太阳风动压 +
// Chandrasekhar 动力学摩擦 —— 本次扩 11 参数是新接口 moonshoot4：太阳风与
// 动摩擦签名向后兼容 由 ..Default::default() 保证 zero 即关闭）。
// 旧 Kotlin 端用 9-arg `cosmosWorldSetPerturbation(legacy)`，新加扩展参数走
// `cosmosWorldSetPerturbationExt` 可控开启太阳风/动摩擦等 扩展。
jni!(boolean cosmosWorldSetPerturbation(
    long world, long body,
    double drag_coefficient, double area, int enable_drag,
    double reflectivity, double optical_area, int enable_solar
) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return Bool::FALSE.0 as jbyte; };
    w.set_perturbation(unpack_handle(body), mps_cosmos::world::PerturbationConfig {
        drag_coefficient, area, enable_drag: enable_drag != 0,
        reflectivity, optical_area, enable_solar: enable_solar != 0,
        ..Default::default()
    });
    Bool::TRUE.0 as jbyte
});

// 扩展版：开启所有 4 类环境扰动力（呼 参见 cosmos_world_set_perturbation。
// 旧版仅大气阻力和光压；此为后加的太阳风动压与 Chandrasekhar 动力学摩擦）。
jni!(boolean cosmosWorldSetPerturbationExt(
    long world, long body,
    double drag_coefficient, double area, int enable_drag,
    double reflectivity, double optical_area, int enable_solar,
    double solar_wind_proton_density, double solar_wind_speed,
    double solar_wind_area, int enable_solar_wind,
    double friction_background_density, double friction_coulomb_log,
    int enable_dynamical_friction
) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return Bool::FALSE.0 as jbyte; };
    w.set_perturbation(unpack_handle(body), mps_cosmos::world::PerturbationConfig {
        drag_coefficient, area, enable_drag: enable_drag != 0,
        reflectivity, optical_area, enable_solar: enable_solar != 0,
        solar_wind_proton_density, solar_wind_speed, solar_wind_area,
        enable_solar_wind: enable_solar_wind != 0,
        friction_background_density, friction_coulomb_log,
        enable_dynamical_friction: enable_dynamical_friction != 0,
        enable_eclipse: false,
        enable_tidal: false,
        love_number_k2: 0.299,
        tidal_q: 12.0,
        tidal_radius: 0.0,
    });
    Bool::TRUE.0 as jbyte
});

// 推进一步，返回一个 `int` 编码的 `StepResult`：
// - `>0`：`Stepped(n)` —— 实际推进的秒数 ×1000（即 n_millisec）；
// - `-1`：`Substepped`（拆子步完成；具体子步数/子步 dt 不便塞进单 int，
//   如需细节用 `cosmosWorldStepDetailed`）；
// - `-2`：`Skipped(NonFinite)`（dt 为 NaN/Inf）；
// - `-3`：`Skipped(NonPositive)`（dt ≤ 0）；
// - `-4`：`Skipped(TooLarge)`（dt 超过 30s 硬上限）。
//
// 这个"压成单 int"的设计是为了让 Java 端的常见 `if (r > 0)` 判断简单。
jni!(int cosmosWorldStep(long world, double dt) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return -2; };
    match w.step(dt) {
        StepResult::Stepped(n) => ((n * 1000.0).round() as i64).max(1) as jint,
        StepResult::Substepped { .. } => -1,
        StepResult::Skipped(StepSkipReason::NonFinite) => -2,
        StepResult::Skipped(StepSkipReason::NonPositive) => -3,
        StepResult::Skipped(StepSkipReason::TooLarge) => -4,
    }
});

// `step_n`：循环 `n` 次推进 `dt`，任一步非法整批拒。
// 返回 0 = 成功；1 = NonFinite；2 = NonPositive；3 = TooLarge。
jni!(int cosmosWorldStepN(long world, double dt, int n) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return 1; };
    match w.step_n(dt, u32_from_jint(n)) {
        Ok(()) => 0,
        Err(StepSkipReason::NonFinite) => 1,
        Err(StepSkipReason::NonPositive) => 2,
        Err(StepSkipReason::TooLarge) => 3,
    }
});

// 取刚体当前位置（3×f64）。`out_translation` 指向 24 字节 native 缓冲。
// 返回 1 成功 / 0 句柄无效或 world 为 null。
jni!(int cosmosBodyTranslationOut(long world, long body, long out_translation) {
    let w = unsafe { (world as *const CosmosWorld).as_ref() };
    let Some(w) = w else { return 0; };
    let Some(p) = w.body_translation(unpack_handle(body)) else { return 0; };
    if let Some(out) = unsafe { pm::<Vec3>(out_translation).as_mut() } {
        out.x = p.x; out.y = p.y; out.z = p.z;
    }
    1
});

// 取刚体当前线速度（3×f64）。
jni!(int cosmosBodyLinvelOut(long world, long body, long out_linvel) {
    let w = unsafe { (world as *const CosmosWorld).as_ref() };
    let Some(w) = w else { return 0; };
    let Some(v) = w.body_linvel(unpack_handle(body)) else { return 0; };
    if let Some(out) = unsafe { pm::<Vec3>(out_linvel).as_mut() } {
        out.x = v.x; out.y = v.y; out.z = v.z;
    }
    1
});

// 取刚体质量（kg）。`NaN` 表示句柄无效。
jni!(double cosmosBodyMass(long world, long body) {
    let w = unsafe { (world as *const CosmosWorld).as_ref() };
    let Some(w) = w else { return f64::NAN; };
    w.body_mass(unpack_handle(body)).unwrap_or(f64::NAN)
});

// 当前动态刚体数量。
jni!(int cosmosWorldDynamicBodyCount(long world) {
    let w = unsafe { (world as *const CosmosWorld).as_ref() };
    let Some(w) = w else { return 0; };
    w.dynamic_body_count() as jint
});

// 动态刚体数量（用于 sizing cosmosWorldDynamicBodySnapshot：先拿到 N
// 再在 Java 端分配 `long[N]` 与 `double[N * 7]` 直接 buffer）。与 mps-core
// `worldDynamicBodySnapshotCount` ABI 平行（见 性能分析.MD §11.1/§12.1，
// M1 + L1 落地基线）。
jni!(int cosmosWorldDynamicBodySnapshotCount(long world) {
    let w = unsafe { (world as *const CosmosWorld).as_ref() };
    let Some(w) = w else { return 0; };
    w.dynamic_body_count() as jint
});

// 批量快照动态刚体 handle + pose（7 f64/body：pos3 + quat4）。详见
// mps-cosmos `cosmos_world_dynamic_body_snapshot` 文档。
//
// **签名平行 mps-core `worldDynamicBodySnapshot`**：`long world, long
// out_handles, long out_values, int capacity` —— `out_handles` / `out_values`
// 是 Java 端用 `Unsafe.allocateMemory` / `ByteBuffer.allocateDirect` 分配的
// **native 直接内存指针**（不是 jbyteArray / jdoubleArray）。这样：
//   - 0 JNI env 拷贝，1 次连续 native memcpy；
//   - 0 Java heap 短命对象，0 minor GC 压力（性能分析.MD §11.2 / M2 的诉求
//     在此形态下自动满足）；
//   - Java 端可映射到 `MappedByteBuffer` 或 `MemorySegment` (Java 22+ FFM
//     路径)，与 mps-core 路径用同一份代码模板。
//
// Java 端推荐用法（替代 N 次 cosmosBodyTranslationOut 往返）：
//   int n = cosmosWorldDynamicBodySnapshotCount(world);
//   long handlesPtr = Unsafe.allocateMemory(n * 8L);
//   long valuesPtr  = Unsafe.allocateMemory(n * 7L * 8);
//   int written = cosmosWorldDynamicBodySnapshot(world, handlesPtr, valuesPtr, n);
//   // values[i*7..i*7+3] = pos, values[i*7+3..i*7+7] = quat(i,j,k,w)
//
// 容量非法 / world null 时返回 0；失败原因由 abiLastErrorCode() 报告。
jni!(int cosmosWorldDynamicBodySnapshot(
        long world,
        long out_handles,
        long out_values,
        int capacity
    ) {
    mps_cosmos::ffi::cosmos_world_dynamic_body_snapshot(
        world as isize as *const CosmosWorld,
        out_handles as isize as *mut u64,
        out_values as isize as *mut f64,
        u32_from_jint(capacity),
    ) as jint
});

// =========================================================================
// Cosmos Shared Arena — zero-JNI physics state read/write
//
// 与 mps-core 的 `world*SharedArena` 平行：把 cosmos world 的共享内存 arena
// 暴露给 Java，供其用 native-order `ByteBuffer` 做命令环写入 + body 槽零拷贝
// 回读。底层调用 `mps_cosmos::ffi::cosmos_world_*`（与 `crates/mps-cosmos/
// include/cosmos.h` 的 C ABI 一致）。布局契约见 `mps_cosmos::arena` 的常量
// （HEADER_SIZE / BODY_SLOT_STRIDE / CMD_SLOT_STRIDE / OFF_*）。
//
// 句柄约定同 cosmos world：`long world` = `*mut CosmosWorld`（由
// `cosmosWorldCreate` 返回）。`out_address` / `out_size` 是 Java 端用
// `Unsafe.allocateMemory` 或 `ByteBuffer.allocateDirect` 分配的 8 字节 native
// 指针，写入 arena 基地址 / 总字节大小。
// =========================================================================

// 创建共享 arena。`out_address` / `out_size` 传 0（null）可跳过对应输出。
// 返回 1 = 成功；0 = 已存在 / 容量非法 / world 为 null（原因见
// `abiLastErrorCode`）。
jni!(boolean cosmosWorldCreateSharedArena(long world, int max_bodies, int max_commands, long out_address, long out_size) {
    mps_cosmos::ffi::cosmos_world_create_shared_arena(
        world as isize as *mut CosmosWorld,
        u32_from_jint(max_bodies),
        u32_from_jint(max_commands),
        pm::<u64>(out_address),
        pm::<u64>(out_size),
    ) as jbyte
});
// 销毁共享 arena（若有的话）。world 为 null 是 no-op。销毁前 Java 必须已释放映射
// 该 arena 的 `ByteBuffer`，否则 use-after-free。
jni!(void cosmosWorldDestroySharedArena(long world) {
    mps_cosmos::ffi::cosmos_world_destroy_shared_arena(world as isize as *mut CosmosWorld);
});
// 取 arena 基地址（无 arena 时返回 0）。供 Java 映射 `ByteBuffer` 的地址来源。
jni!(long cosmosWorldGetSharedArenaAddress(long world) {
    mps_cosmos::ffi::cosmos_world_get_shared_arena_address(world as isize as *const CosmosWorld) as jlong
});
// 取 arena 总字节大小（无 arena 时返回 0）。供 Java 映射 `ByteBuffer` 的容量来源。
jni!(long cosmosWorldGetSharedArenaSize(long world) {
    mps_cosmos::ffi::cosmos_world_get_shared_arena_size(world as isize as *const CosmosWorld) as jlong
});

/// Returns the cosmos arena wrapped as a Java DirectByteBuffer.
///
/// 与核心 `worldGetArenaDirectByteBuffer` 平行，仅指向 cosmos world 的 arena。
/// 使用标准 JNI `NewDirectByteBuffer`：返回的 `ByteBuffer` 直接包裹 native arena
/// 内存，Java 侧可用 `ByteBuffer` / `DoubleBuffer` 做零 JNI 读写。
#[unsafe(export_name = "Java_org_polaris2023_mps_rapier_RapierNative_cosmosWorldGetArenaDirectByteBuffer")]
#[allow(non_snake_case)]
pub extern "system" fn cosmosWorldGetArenaDirectByteBuffer(
    env: JNIEnv,
    _class: jclass,
    world: jlong,
) -> ljni::sys::jobject {
    catch_unwind(AssertUnwindSafe(|| {
        let world = world as isize as *mut CosmosWorld;
        let addr = mps_cosmos::ffi::cosmos_world_get_shared_arena_address(world);
        let size = mps_cosmos::ffi::cosmos_world_get_shared_arena_size(world);
        if addr == 0 || size == 0 {
            return std::ptr::null_mut();
        }
        let env_raw: *mut JNIEnv = &raw const env as *mut _;
        let env = unsafe { &mut *env_raw };
        unsafe { env.new_direct_byte_buffer(addr as _, size as _) }
            .map(|bb| bb.as_raw())
            .unwrap_or(std::ptr::null_mut())
    }))
    .unwrap_or(std::ptr::null_mut())
}

// ── Phase 5e: 软体 JNI 网关（暴露 15 个 soft_body_* FFI 给 Java/FFM）──────────
// 与 Phase 5a/5b/5d/5f 的 mps-core FFI 一一对应。FFM 侧直接在 Java 25 经
// Linker 链接这些 `Java_org_polaris2023_mps_rapier_RapierNative_*` 符号，故本
// 文件即 JNI + FFM 共享的网关。返回 id 类沿用 `u32::MAX` 哨兵（转 jlong 后 Java
// 侧比较 == 0xFFFFFFFFL）；布尔类返回 `Bool` 经 `.0 as jbyte`（0/1）。

// Phase 5a: 通用软体构造（任意拓扑：质点 / 弹簧 / XPBD 距离约束 / 四面体）
jni!(long softBodyCreate(long world, double gravity_x, double gravity_y, double gravity_z) { sb::soft_body_create(m::<WH>(world), v3(gravity_x, gravity_y, gravity_z)) as jlong });
jni!(long softBodyAddParticle(long world, int id, double x, double y, double z, double mass, int pinned) { sb::soft_body_add_particle(m::<WH>(world), id as u32, x, y, z, mass, jb(pinned)) as jlong });
jni!(boolean softBodyAddSpring(long world, int id, int a, int b, double stiffness, double damping) { sb::soft_body_add_spring(m::<WH>(world), id as u32, a as u32, b as u32, stiffness, damping).0 as jbyte });
jni!(boolean softBodyAddDistanceConstraint(long world, int id, int a, int b, double compliance) { sb::soft_body_add_distance_constraint(m::<WH>(world), id as u32, a as u32, b as u32, compliance).0 as jbyte });
jni!(boolean softBodyAddTetrahedron(long world, int id, int a, int b, int c, int d) { sb::soft_body_add_tetrahedron(m::<WH>(world), id as u32, a as u32, b as u32, c as u32, d as u32).0 as jbyte });
jni!(boolean softBodyAddTriangle(long world, int id, int a, int b, int c) { sb::soft_body_add_triangle(m::<WH>(world), id as u32, a as u32, b as u32, c as u32).0 as jbyte });
jni!(boolean softBodyAddBending(long world, int id, int p, int q) { sb::soft_body_add_bending(m::<WH>(world), id as u32, p as u32, q as u32).0 as jbyte });
jni!(boolean softBodyConfigureSolver(long world, int id, int solver_mode, int iterations, double compliance) { sb::soft_body_configure_solver(m::<WH>(world), id as u32, solver_mode as u32, iterations as u32, compliance).0 as jbyte });
jni!(long softBodyBuildTetraMesh(long world, double gravity_x, double gravity_y, double gravity_z, long particles, int particles_len, long tets, int tets_len, double particle_mass, double compliance, int iterations) { sb::soft_body_build_tetra_mesh(m::<WH>(world), v3(gravity_x, gravity_y, gravity_z), p::<Vec3>(particles), particles_len as u32, p::<u32>(tets), tets_len as u32, particle_mass, compliance, iterations as u32) as jlong });

// Phase 4 + 5d: voxel 网格软体构造 + 破坏联动
jni!(long softBodyVoxelBuild(long world, long voxels, int voxels_len, int size_x, int size_y, int size_z, double voxel_size, double origin_x, double origin_y, double origin_z, double particle_mass, double stiffness, double damping, int pin_boundary) { sb::soft_body_voxel_build(m::<WH>(world), p::<u8>(voxels), voxels_len as u32, size_x as u32, size_y as u32, size_z as u32, voxel_size, v3(origin_x, origin_y, origin_z), particle_mass, stiffness, damping, jb(pin_boundary)) as jlong });
jni!(boolean softBodyVoxelDig(long world, int id, int cell_x, int cell_y, int cell_z) { sb::soft_body_voxel_dig(m::<WH>(world), id as u32, cell_x as u32, cell_y as u32, cell_z as u32).0 as jbyte });

// Phase 5b: 查询 / 读回 / 生命周期
jni!(boolean softBodySetGravity(long world, int id, double gravity_x, double gravity_y, double gravity_z) { sb::soft_body_set_gravity(m::<WH>(world), id as u32, v3(gravity_x, gravity_y, gravity_z)).0 as jbyte });
jni!(long softBodyCount(long world) { sb::soft_body_count(cp::<WH>(world)) as jlong });
jni!(long softBodyParticleCount(long world, int id) { sb::soft_body_particle_count(cp::<WH>(world), id as u32) as jlong });
jni!(boolean softBodyGetParticle(long world, int id, int index, long out_pos, long out_vel) { sb::soft_body_get_particle(cp::<WH>(world), id as u32, index as u32, pm::<Vec3>(out_pos), pm::<Vec3>(out_vel)).0 as jbyte });
jni!(boolean softBodyRemoveParticle(long world, int id, int index) { sb::soft_body_remove_particle(m::<WH>(world), id as u32, index as u32).0 as jbyte });
jni!(boolean softBodyDestroy(long world, int id) { sb::soft_body_destroy(m::<WH>(world), id as u32).0 as jbyte });

// Phase 5i: 拓扑读回（渲染用）— 批量读回粒子位置/逆质量 + 边 + 四面体。
jni!(int softBodyReadParticles(long world, int id, long out_pos, long out_inv_mass, int capacity) { sb::soft_body_read_particles(cp::<WH>(world), id as u32, pm::<Vec3>(out_pos), pm::<f64>(out_inv_mass), capacity as u32) as jint });
jni!(int softBodyReadEdges(long world, int id, long out_edges, int capacity) { sb::soft_body_read_edges(cp::<WH>(world), id as u32, pm::<u32>(out_edges), capacity as u32) as jint });
jni!(int softBodyReadTetrahedra(long world, int id, long out_tets, int capacity) { sb::soft_body_read_tetrahedra(cp::<WH>(world), id as u32, pm::<u32>(out_tets), capacity as u32) as jint });
jni!(int softBodyReadTriangles(long world, int id, long out_tris, int capacity) { sb::soft_body_read_triangles(cp::<WH>(world), id as u32, pm::<u32>(out_tris), capacity as u32) as jint });

// ── Phase 7: 风场/空气阻力 + 休眠 + 诊断 ──────────────────────────────────
jni!(boolean softBodyApplyWind(long world, int id, double ax, double ay, double az, double drag) { sb::soft_body_apply_wind(m::<WH>(world), id as u32, Vec3 { x: ax, y: ay, z: az }, drag).0 as jbyte });
jni!(boolean softBodyClearWind(long world, int id) { sb::soft_body_clear_wind(m::<WH>(world), id as u32).0 as jbyte });
jni!(boolean softBodySleep(long world, int id) { sb::soft_body_sleep(m::<WH>(world), id as u32).0 as jbyte });
jni!(boolean softBodyWake(long world, int id) { sb::soft_body_wake(m::<WH>(world), id as u32).0 as jbyte });
jni!(boolean softBodyIsSleeping(long world, int id) { sb::soft_body_is_sleeping(cp::<WH>(world), id as u32).0 as jbyte });
jni!(double softBodyKineticEnergy(long world, int id) { sb::soft_body_kinetic_energy(cp::<WH>(world), id as u32) });
jni!(double softBodyTotalVolume(long world, int id) { sb::soft_body_total_volume(cp::<WH>(world), id as u32) });
// Phase 8: 锚定软体任意质点到刚体 + 解绑
jni!(boolean softBodyAttachParticle(long world, int id, int particle, long body, double ax, double ay, double az) { sb::soft_body_attach_particle(m::<WH>(world), id as u32, particle as u32, body as u64, Vec3 { x: ax, y: ay, z: az }).0 as jbyte });
jni!(boolean softBodyDetachParticle(long world, int id, int particle) { sb::soft_body_detach_particle(m::<WH>(world), id as u32, particle as u32).0 as jbyte });
// Phase 9: 撕裂阈值（应变阈值，>0 开启，<=0/disabled 关闭）
jni!(boolean softBodySetTearStrain(long world, int id, double strainToBreak, int enabled) { sb::soft_body_set_tear_strain(m::<WH>(world), id as u32, strainToBreak, enabled as u8).0 as jbyte });
// Phase 10: 塑性（永久变形，橡皮泥/记忆棉）
jni!(boolean softBodySetPlasticity(long world, int id, double yieldStrain, double creep, int enabled) { sb::soft_body_set_plasticity(m::<WH>(world), id as u32, yieldStrain, creep, enabled as u8).0 as jbyte });
// Phase 11: 充气/气压（闭合三角网格沿法向吹胀）
jni!(boolean softBodySetPressure(long world, int id, double pressure) { sb::soft_body_set_pressure(m::<WH>(world), id as u32, pressure).0 as jbyte });
// Phase 12: 软体自碰撞(self-collision)：空间哈希 broad-phase + 逐迭代位置投影
jni!(boolean softBodySetSelfCollision(long world, int id, double radius, double stiffness) { sb::soft_body_set_self_collision(m::<WH>(world), id as u32, radius, stiffness).0 as jbyte });
// Phase 13: 运行时改单条弹簧刚度(Hookean k)
jni!(boolean softBodySetSpringStiffness(long world, int id, int index, double stiffness) { sb::soft_body_set_spring_stiffness(m::<WH>(world), id as u32, index as u32, stiffness).0 as jbyte });
// Phase 13: 运行时改单条 XPBD 距离约束柔度(compliance α)
jni!(boolean softBodySetDistanceConstraintCompliance(long world, int id, int index, double compliance) { sb::soft_body_set_distance_constraint_compliance(m::<WH>(world), id as u32, index as u32, compliance).0 as jbyte });
// Phase 14: 软软碰撞(soft-soft / cross-body)：world 层空间哈希 + 逐软体对投影
jni!(boolean softBodySetCrossCollision(long world, int id, double radius, double stiffness) { sb::soft_body_set_cross_collision(m::<WH>(world), id as u32, radius, stiffness).0 as jbyte });
// Phase 16: 体积守恒约束(独立柔度)
jni!(boolean softBodySetVolumeConservation(long world, int id, double compliance) { sb::soft_body_set_volume_conservation(m::<WH>(world), id as u32, compliance).0 as jbyte });
// Phase 17: 软体间黏连(可撕黏附)
jni!(boolean softBodySetCohesion(long world, int id, double radius, double stiffness, double breakDistance) { sb::soft_body_set_cohesion(m::<WH>(world), id as u32, radius, stiffness, breakDistance).0 as jbyte });
// Phase 18: 全局内部阻尼
jni!(boolean softBodySetDamping(long world, int id, double d) { sb::soft_body_set_damping(m::<WH>(world), id as u32, d).0 as jbyte });
// Phase 5f: 软体-刚体碰撞（proxy collider 桥接）
jni!(boolean softBodyEnableCollision(long world, int id, double particle_radius, int enabled) { sb::soft_body_enable_collision(m::<WH>(world), id as u32, particle_radius, jb(enabled)).0 as jbyte });
