# mps_rigid_body Project Analysis

## 1. Project Positioning

`mps_rigid_body` is a Rust native physics wrapper around `rapier3d-f64`.
The crate builds as both `rlib` and `cdylib`, with the `cdylib` used by Java.

The project is not a standalone Rust app. It is a cross-language physics runtime:

```text
Java 21 JNI / Java 25 FFM
  -> Rust C ABI / JNI wrappers
    -> src/rapier wrapper modules
      -> rapier3d-f64
```

Rust owns the physics world, rigid bodies, colliders, joints, events, query
pipeline, and spatial indexes. Java receives opaque pointers or packed handles
and calls back into Rust for all physics operations.

## 2. Source Layout

```text
src/
  lib.rs
  abi/
    ffm.rs
    jni.rs
    mod.rs
  helper/
    helper.rs
    mod.rs
  rapier/
    acoustics.rs
    aerodynamics.rs
    anvilkit.rs
    astrophysics.rs
    biomechanics.rs
    bounds.rs
    chaos.rs
    collider.rs
    compat.rs
    continuum.rs
    control_theory.rs
    controller.rs
    crbtree.rs
    dop.rs
    electromagnetism.rs
    error.rs
    events.rs
    ffi.rs
    fluid.rs
    fracture.rs
    joints.rs
    mod.rs
    molecular.rs
    neural.rs
    physchem.rs
    plasma.rs
    quantum.rs
    query.rs
    relativity.rs
    rigid_body.rs
    rtree.rs
    softbody.rs
    spaceflight.rs
    superfluidity.rs
    thermodynamics.rs
    topology.rs
    trajectory.rs
    transmission.rs
    voxel.rs
    wave_optics.rs
    world.rs

test21/
  Java 21 JNI smoke test and Java helper API

test25/
  Java 25 FFM smoke test and FFM binding sample
```

`src/rapier` is the core implementation. `src/abi` adapts it to external Java
entry points. `src/helper` contains JNI support utilities.

## 3. ABI Design

`src/rapier/ffi.rs` is the ABI type center. It defines `#[repr(C)]` structs used
by C ABI, JNI, and Java FFM.

Important ABI structs:

- `Vec3`: 3D vector.
- `Quat`: quaternion.
- `Bool`: one-byte boolean wrapper.
- `ShapeDesc`: generic primitive shape descriptor.
- `InteractionGroupsDesc`: collision/solver group descriptor.
- `QueryFilterDesc`: query filtering descriptor.
- `ShapeCastOptionsDesc`: shape cast options.
- `RayHit`, `PointProjection`, `ShapeCastHit`: query result records.
- `CollisionEventRecord`, `ContactForceEventRecord`: event records.
- `VoxelColliderOptions`, `VoxelBuildStats`: voxel build configuration and statistics.
- `NeuralBoundsDesc`: neural bounds configuration.
- `TrajectoryState`, `TrajectoryEnvironment`, `TrajectoryForceReport`: point-mass trajectory force model.
- `TrajectoryGlideState`, `TrajectoryGlideEnvironment`, `TrajectoryGlideReport`: boost-glide / skip-glide style trajectory model.
- `OrbitalElements`, `StateVector`: orbital element and Cartesian state conversion.
- `ScalarKalman`, `LeastSquaresAttitude`, `GnssObservation`: estimation and navigation formula results.
- `HohmannTransfer`, `FriisLink`, `SolarPanelPower`, `ThermalBalance`, `RadiatorPower`: mission engineering formula results.
- `CmgExchange`, `CmgRobustInverse`, `RigidBodyEulerDerivative`, `QuaternionDerivative`: attitude dynamics/control result records.
- `MassProperties`, `ContactForceModel`, `BatteryEquivalentCircuit`, `HallThrusterPerformance`: subsystem engineering result records.

The project intentionally does not expose Rapier internal Rust types directly.
External callers use stable C-compatible types.

### 3.1 Opaque Handles

Long-lived Rust-owned objects are exposed as opaque pointers:

- `WorldHandle`
- `RigidBodyBuilderHandle`
- `ColliderBuilderHandle`
- `JointBuilderHandle`
- `CharacterControllerHandle`
- `RTreeHandle`
- `CRbTreeHandle`

Java stores these as `long` or `MemorySegment`, depending on JNI vs FFM.

### 3.2 Packed Rapier Handles

Rapier handles are packed into nonzero `u64` values:

```text
packed = ((generation << 32) | id) + 1
```

`0` is reserved for null/invalid handles. This is used for rigid bodies,
colliders, and impulse joints.

### 3.3 FFM-Friendly Out Pointers

Some native structs are returned through out-pointer variants for Java FFM.
This avoids platform-specific complexity around returning larger structs by value.

Examples:

- `world_get_gravity_out`
- `rigid_body_get_translation_out`
- `rigid_body_get_rotation_out`
- `collider_get_translation_out`
- `collider_get_rotation_out`
- `query_cast_ray_out`
- `query_project_point_out`
- `query_cast_shape_out`
- `voxel_aabb_build_stats_out`
- `voxel_obb_build_stats_out`

For one-byte success/failure paths, Java FFM uses `_flag` functions returning
`u8` instead of `Bool` struct returns.

## 4. Java Access Layers

### 4.1 JNI

`src/abi/jni.rs` exposes JNI-compatible `Java_*` functions and delegates to the
Rust C ABI implementation.

Responsibilities:

- Convert Java primitives to Rust ABI structs.
- Convert Java arrays/native memory addresses to Rust slices.
- Convert Rust result structs back to Java arrays or native output buffers.
- Catch Rust panics at the JNI boundary.

Java 21 wrappers live in `test21/src/main/java/.../util`.

### 4.2 FFM

`test25/.../RigidBodyFfm.java` is a Java 25 FFM binding sample. It declares Java
`MemoryLayout` definitions matching the Rust `repr(C)` ABI and uses downcall
handles for native functions.

The FFM surface currently covers:

- World lifecycle and stepping.
- Gravity and collider count.
- Rigid body builder, insertion, pose/velocity/force/impulse mutation, CCD, sleep/wakeup.
- Collider builder, insertion, parent insertion, runtime material/group/event mutation.
- CRbTree basics.
- Voxel AABB/OBB build stats and collider creation.
- Voxel AABB/OBB queries.
- Regular queries: ray, point projection, AABB, OBB, sphere, shape cast.
- Collision/contact-force event bulk reads and event clearing.

## 5. World Module

`src/rapier/world.rs` owns the Rapier simulation state:

```rust
PhysicsPipeline
Vector gravity
IntegrationParameters
IslandManager
BroadPhaseBvh
NarrowPhase
RigidBodySet
ColliderSet
ImpulseJointSet
MultibodyJointSet
CCDSolver
CallbackPhysicsHooks
CollectingEventHandler
```

Main functions:

- `world_create`
- `world_destroy`
- `world_step`
- `world_set_gravity`
- `world_get_gravity`
- `world_get_gravity_out`
- `world_set_integration_parameters`
- `world_get_integration_parameters`
- `world_body_snapshot`
- `world_dynamic_body_snapshot`
- `world_update_body_poses`
- `world_update_body_velocities`
- `world_get_rigid_body_set_size`
- `world_get_collider_set_size`

`world_step` validates the timestep and advances the Rapier pipeline using the
world's hooks and event handler.

## 6. Rigid Bodies

Implemented in `src/rapier/rigid_body.rs`.

Supported body statuses:

- `Dynamic`
- `Fixed`
- `KinematicPositionBased`
- `KinematicVelocityBased`

Builder capabilities:

- translation
- rotation
- pose
- linear/angular velocity
- additional mass
- additional mass properties
- gravity scale
- damping
- sleep behavior
- enabled rotations
- user data

Runtime capabilities:

- get/set status
- get/set translation
- get/set rotation
- set pose
- get/set linear velocity
- get/set angular velocity
- add force
- add torque
- apply impulse
- apply torque impulse
- enable CCD
- sleep
- wake up
- sleeping state query
- remove/copy body

FFM uses byte-returning `_flag` variants for success/failure mutation calls.

## 7. Colliders

Implemented in `src/rapier/collider.rs`.

Primitive shapes:

- ball
- cuboid
- capsule Y/X/Z
- cylinder
- round cylinder
- cone
- round cone
- round cuboid

Additional builders:

- OBB
- sphere
- heightmap
- convex hull
- point-cloud bounds
- double BV
- skewed OBB
- discrete OBB
- fused collapsing bounds
- edge BVH
- medial spheres

Builder settings:

- translation
- rotation
- pose
- sensor
- friction
- restitution
- density
- collision groups
- solver groups
- active events
- active hooks
- contact-force event threshold

Runtime settings:

- pose
- sensor
- friction
- restitution
- collision groups
- solver groups
- active events
- active hooks
- contact-force event threshold
- density readback

Insertion paths:

- `world_insert_collider`
- `world_insert_collider_with_parent`

## 8. Queries

Implemented in `src/rapier/query.rs`.

Supported query operations:

- ray cast
- batch ray cast
- point projection
- point intersection count
- AABB intersection count and handle output
- OBB intersection count and handle output
- sphere intersection count and handle output
- shape cast

The query pipeline is built from the world's broad phase, narrow phase, bodies,
colliders, and `QueryFilterDesc`.

`QueryFilterDesc` supports:

- query flags
- interaction groups
- excluded collider
- excluded rigid body

FFM uses out-pointer variants for ray, point projection, and shape cast results.

## 9. Events

Implemented in `src/rapier/events.rs`.

The event handler stores:

- collision events
- contact-force events

Events are collected into bounded vectors protected by `parking_lot::Mutex`.
The maximum stored event count is `16_384`.

Public event API:

- `world_clear_events`
- `world_collision_event_count`
- `world_get_collision_event`
- `world_get_collision_events`
- `world_contact_force_event_count`
- `world_get_contact_force_event`
- `world_get_contact_force_events`

Java 25 FFM currently uses the bulk read functions and slices the returned
`MemorySegment` to the number of records written.

## 10. Voxel System

Implemented in `src/rapier/voxel.rs`.

Voxel colliders can be built from:

- raw occupancy grid memory
- Java `byte[]` through JNI
- AABB
- OBB

Build modes:

- `Auto`
- `Cuboids`
- `GreedyCuboids`
- `SurfaceMesh`

Default thresholds:

```text
small_voxel_limit = 128
mesh_voxel_limit = 20_000
```

Important native functions:

- `collider_builder_create_voxels`
- `collider_builder_create_voxels_auto`
- `collider_builder_create_voxel_aabb`
- `collider_builder_create_voxel_aabb_auto`
- `collider_builder_create_voxel_obb`
- `collider_builder_create_voxel_obb_auto`
- `voxel_build_stats`
- `voxel_aabb_build_stats`
- `voxel_obb_build_stats`
- `voxel_aabb_build_stats_out`
- `voxel_obb_build_stats_out`

Voxel query functions:

- `query_intersect_voxel_aabb`
- `query_intersect_voxel_aabb_count`
- `query_intersect_voxel_obb`
- `query_intersect_voxel_obb_count`

Direct insertion helpers:

- `world_insert_static_voxel_aabb`
- `world_insert_dynamic_voxel_obb`

### 10.1 Bounding Boxes as Voxel Sources

Using bounding boxes to define voxels is feasible and implemented.

For AABB, every generated cell is solid because the grid is exactly the box
volume. For OBB, the system first builds a world-space AABB around the oriented
box, then samples voxel cell centers against the OBB's local half-extents.

This keeps the voxel builder pipeline unified: AABB/OBB paths generate an owned
occupancy grid and then reuse the same voxel collider build modes.

### 10.2 Java VoxelGrid

Java 21 includes `VoxelGrid`, which supports:

- `get`
- `set`
- `clear`
- `solidCount`
- `fillBox`
- `fillAabb`
- `fillSphere`
- `copyFrom`
- `union`
- `subtract`
- `intersect`
- `toByteArray`
- `address`

This helper makes simple voxel content possible without manually writing every
occupancy byte.

## 11. Spatial Indexes

The project includes two standalone spatial index wrappers:

- `CRbTree`
- `RTree`

CRbTree has Java 25 FFM coverage for create, insert, and AABB count query.
RTree now has Java 25 FFM coverage for create, destroy, clear, len, insert,
update, remove, rebuild, AABB query count, and AABB query handle output.

Java 21 JNI has broader helper coverage through `SpatialIndex`.

## 12. Extended Geometry

Additional geometry modules:

- `bounds.rs`: capsule, SSV, ellipsoid, prism, cylinder, spherical shell.
- `dop.rs`: kDOP and FDH builders.
- `neural.rs`: neural bounds builder and neural bounds queries.
- `compat.rs`: compatibility helpers such as dynamic cuboids and static trimesh insertion.

Most of these are exposed through Rust C ABI and Java 21 JNI. Java 25 FFM does
not yet cover all of them.

## 13. Joints and Character Controller

`joints.rs` supports impulse joint creation and removal for several joint kinds.
`controller.rs` wraps Rapier's character controller movement and collision data.

These are available through JNI wrappers, but Java 25 FFM parity is still open.

## 14. Verification Status

Verified commands:

```powershell
cargo test

cd test21
.\gradlew.bat check

cd ..\test25
.\gradlew.bat check
```

Current expected result:

```text
Rust tests: passed
Java 21 JNI smoke test: passed
Java 25 FFM smoke test: passed
```

Recent local verification:

```text
cargo check: passed
cargo test: passed, 172 tests
```

## 15. Current Java 25 FFM Coverage

Java 25 FFM currently covers:

- ABI version.
- World create/destroy/step.
- Gravity get/set.
- Collider count.
- Rigid body create/insert/get translation/get rotation.
- Rigid body pose, translation, rotation mutation.
- Rigid body linear/angular velocity mutation.
- Rigid body force, torque, impulse, torque impulse.
- Rigid body CCD, sleep, wakeup, sleeping state.
- Collider builder create for primitive cuboid-like shape.
- Voxel AABB/OBB builder stats and collider creation.
- Collider builder material/event settings.
- Collider insertion and insertion with parent.
- Collider runtime pose, sensor, material, group, event, hook, threshold mutation.
- Collider translation, rotation, and density reads.
- CRbTree create/insert/AABB count query.
- RTree create/destroy/clear/len/insert/update/remove/rebuild/AABB query.
- Regular queries: ray, point projection, AABB, OBB, sphere, shape cast.
- Voxel-shaped AABB/OBB queries.
- Collision event and contact-force event bulk reads.
- Event clearing.

The generated C header also exposes the `space_*`, `trajectory_*`, and other
formula symbols, but the Java 25 sample helper does not yet wrap the full
formula surface. Java 25 callers can still bind these functions manually with
FFM downcall handles if they mirror the `repr(C)` layouts from
`include/rigid_body.h`.

## 16. Recommended Next Steps

Recommended implementation order:

1. Java 25 FFM character controller:
   controller lifecycle, configuration setters, move shape, collision count/read, solve impulses.

2. Java 25 FFM joints:
   joint builder creation, anchors, limits, motors, insert/remove.

3. Java 25 FFM advanced collider builders:
   heightmap, convex hull, point-cloud bounds, kDOP, FDH, neural bounds.

4. Higher-level Java 25 helper layer:
   `RigidBodyFfm` is currently low-level. A safer Java 25 utility layer similar
   to `test21/util` would reduce direct `MemorySegment` handling in user code.

## 17. src Feature Map

This section summarizes the concrete feature responsibility of each `src`
module. It is intended as a quick map for future development.

### 17.1 Crate and ABI Entrypoints

- `src/lib.rs`
  - Crate root.
  - Installs the global allocator.
  - Re-exports ABI-facing structs and handles.
  - Wires `abi`, `helper`, and `rapier` modules.

- `src/abi/ffm.rs`
  - Minimal C ABI capability probes.
  - Exposes `abi_version`, `abi_supports_ffm`, and `abi_supports_jni`.
  - Java 25 FFM calls the native library symbols directly; this file is not the
    whole FFM binding, only the native capability surface.

- `src/abi/jni.rs`
  - JNI bridge for Java 21.
  - Converts Java primitive arguments into Rust ABI structs.
  - Wraps Rust C ABI calls as `Java_org_polaris2023_msp_1rigid_1body_RigidBodyNative_*`.
  - Handles Java arrays, native output buffers, error boundaries, and panic safety.

- `src/helper/helper.rs`
  - JNI helper utilities for array/native-memory conversion.

### 17.2 Core Physics Runtime

- `src/rapier/world.rs`
  - Owns the Rapier physics world state.
  - Creates/destroys worlds, steps simulation, manages gravity and integration parameters.
  - Exposes body/collider counts.
  - Provides body snapshots and batch body pose/velocity updates.

- `src/rapier/rigid_body.rs`
  - Builds and inserts rigid bodies.
  - Supports dynamic, fixed, and kinematic body types.
  - Provides runtime pose, velocity, force, torque, impulse, CCD, sleep, and wakeup control.
  - Includes byte-returning `_flag` variants for Java FFM-friendly mutation calls.

- `src/rapier/collider.rs`
  - Builds and inserts colliders.
  - Supports primitive and advanced collider builders.
  - Provides builder-level material, group, sensor, active event/hook, and threshold settings.
  - Provides runtime collider pose/material/group/event/hook mutation.
  - Supports standalone collider insertion and parented collider insertion.

- `src/rapier/events.rs`
  - Collects collision events and contact-force events during `world_step`.
  - Provides count, single read, bulk read, and clear functions.
  - Stores event queues in bounded mutex-protected vectors.

### 17.3 Query and Movement Systems

- `src/rapier/query.rs`
  - Ray cast and batch ray cast.
  - Point projection.
  - Point, AABB, OBB, and sphere intersections.
  - Shape cast.
  - FFM-friendly out-pointer variants for ray, point projection, and shape cast.

- `src/rapier/aerodynamics.rs`
  - Surface-sample air force accumulation for Rapier rigid bodies.
  - Computes drag and lift from wind velocity, air density, exposed area, and surface normal.
  - Applies forces at world-space surface points with `add_force_at_point`.
  - Includes pure estimation helpers and batch application helpers.

- `src/rapier/controller.rs`
  - Character controller lifecycle and configuration.
  - Supports movement with collision reporting and impulse solving.
  - JNI coverage exists; Java 25 FFM parity is still pending.

### 17.4 Geometry Builders

- `src/rapier/bounds.rs`
  - Capsule, swept sphere volume, ellipsoid, prism, cylinder, and spherical shell builders.
  - Matching intersection queries for these bound types.

- `src/rapier/dop.rs`
  - kDOP and FDH collider builders.
  - Useful for approximate convex bounds from point clouds or directional hulls.

- `src/rapier/neural.rs`
  - Neural bounds descriptor and required weight count calculation.
  - Neural bounds collider builder.
  - Neural bounds intersection queries.

- `src/rapier/voxel.rs`
  - Raw occupancy-grid voxel colliders.
  - AABB/OBB-to-voxel generation.
  - Build modes: cuboids, greedy cuboids, surface mesh, auto.
  - Voxel build statistics.
  - Voxel-shaped AABB/OBB queries.
  - Direct static AABB and dynamic OBB voxel body insertion helpers.

- `src/rapier/compat.rs`
  - Compatibility helpers for common workflows.
  - Includes dynamic cuboid batch insertion and static trimesh insertion.
  - Includes AABB rigid body query helpers.

### 17.5 Aerospace, Controls, and Engineering Formula Layer

The project now includes a broad formula layer in addition to Rapier collision
and dynamics wrappers. This layer has two categories:

1. Pure ABI formulas that return derived values through scalar returns or
   out-pointer result structs.
2. Rapier-integrated formulas that compute forces/torques and apply them
   directly to a `RigidBodyHandleRaw` inside a `WorldHandle`.

- `src/rapier/trajectory.rs`
  - Point-mass trajectory force model with gravity, drag, lift, and optional body force application.
  - Boost-glide / skip-glide style trajectory state equations using velocity,
    flight-path angle, altitude, downrange, exponential atmosphere,
    ballistic coefficient, lift-to-drag ratio, and bank angle.
  - `trajectory_apply_forces_to_body` applies the point-mass force report to a
    Rapier rigid body.

- `src/rapier/spaceflight.rs`
  - Orbital mechanics:
    Kepler third law, orbital elements to/from Cartesian state, J2 perturbation,
    Clohessy-Wiltshire relative motion, Lambert elliptic transfer time,
    Hohmann transfer, semi-major-axis drag decay, SGP4-style J2 secular rates.
  - Attitude dynamics and control:
    quaternion kinematics, Euler rigid-body angular acceleration, CMG angular
    momentum exchange, CMG robust damped pseudo-inverse, gravity-gradient
    torque, magnetic torquer dipole, Bang-Off-Bang attitude profile, solar
    array PD torque.
  - Navigation and estimation:
    TRIAD attitude determination, two-vector least-squares attitude wrapper,
    scalar EKF predict/gain/update, GNSS pseudorange, double-difference carrier phase,
    Sagnac phase/rate helper, radar range/range-rate.
  - Space environment and spacecraft subsystem formulas:
    scale-height atmosphere, atmospheric drag acceleration, solar radiation
    pressure, radiation absorbed dose, atomic oxygen erosion, surface charging
    current balance, Whipple shield critical projectile estimate.
  - Thermal, ECLSS, power, and propulsion:
    heat-pipe thermal resistance, radiator power, thermal equilibrium,
    single-phase fluid loop heat transfer, CO2 mass balance, Sabatier methane
    rate, SPE oxygen rate, solar-panel power, battery equivalent circuit,
    Hall thruster performance, Tsiolkovsky rocket equation.
  - Robotics and docking:
    D-H transform, six-axis first joint inverse, third joint angle, simplified
    diagonal manipulator dynamics, artificial potential guidance, docking
    glideslope command, docking buffer energy absorption, airlock depressurization.
  - Structure and flexible dynamics:
    structural natural frequency, flexible mode derivative, slosh pendulum derivative,
    Hunt-Crossley contact force, two-body mass properties.

Rapier-integrated `spaceflight` entry points:

- `space_apply_j2_force_to_body`
- `space_apply_atmospheric_drag_to_body`
- `space_apply_solar_radiation_pressure_to_body`
- `space_apply_gravity_gradient_torque_to_body`
- `space_apply_magnetic_torquer_to_body`
- `space_apply_cmg_torque_to_body`

These functions locate a rigid body inside `WorldHandle`, compute the requested
spaceflight acceleration/torque, and call Rapier `add_force` or `add_torque`.
The matching `_flag` variants return `u8` for FFM-friendly success/failure.

- `src/rapier/control_theory.rs`
  - Control-oriented formulas and stepping utilities, including state-space
    stepping.
  - Useful as the natural home for future generic controller primitives before
    they are bound to a Rapier body or AnvilKit entity.

Other domain formula modules are present for specialized pure ABI formulas:

- `acoustics.rs`
- `astrophysics.rs`
- `biomechanics.rs`
- `chaos.rs` — Lorenz attractor (RK4), Lyapunov exponent (orbit divergence + Rosenstein
  algorithm), bifurcation diagrams (Lorenz rho/sigma/beta scan + logistic map),
  double pendulum (Lagrangian mechanics, RK4), chaos detection (Lyapunov +
  Grassberger–Procaccia correlation dimension), logistic map (period-doubling route).
- `continuum.rs`
- `electromagnetism.rs`
- `fracture.rs`
- `molecular.rs`
- `physchem.rs`
- `plasma.rs` — Debye shielding & plasma parameters (λ_D, ω_pe, ω_pi, N_D, v_th),
  Boris particle pusher (PIC, symplectic), trilinear field interpolation,
  charge deposition (cloud-in-cell), Vlasov moments (n, u, T, q),
  1D Poisson solve (Thomas algorithm), magnetic X-point detection (∇B Jacobian),
  Sweet–Parker / Petschek reconnection rates, Alfvén speed, Lundquist number.
- `quantum.rs`
- `relativity.rs` — Lorentz factor & boost matrix (3D), relativistic velocity addition,
  Schwarzschild metric & radius, light deflection, effective potential,
  gravitational time dilation (stationary + orbital), length contraction,
  relativistic particle (E, p, rapidity), invariant mass.
  (See Section 17.8 for detailed coverage.)
- `softbody.rs`
- `superfluidity.rs` — Biot–Savart vortex segment velocity, vortex ring self-induced
  velocity, quantised circulation (h/m quantum), Gross–Pitaevskii order parameter
  (vortex core profile, phase winding), GP energy density (kinetic + interaction + trap),
  GP imaginary-time amplitude evolution, vortex reconnection (closest approach + X-swap),
  vortex tangle statistics (line density, curvature, kinetic energy),
  2D grid sampling for visualisation, healing length & sound speed for ⁴He.
- `thermodynamics.rs`
- `topology.rs`
- `trajectory.rs` — Point-mass trajectory force model with gravity, drag, lift,
  boost-glide / skip-glide trajectory state equations (RK4),
  `trajectory_apply_forces_to_body` applies forces to a Rapier rigid body.
  (See Section 17.5 for details.)
- `transmission.rs`
- `wave_optics.rs` — Wavenumber & plane wave, spherical wave (1/r decay),
  Huygens–Fresnel point-source superposition, Fresnel diffraction (paraxial, rectangular
  aperture), Fresnel–Kirchhoff diffraction (obliquity factor cosθ),
  Young's double-slit interference with single-slit sinc envelope,
  thin-film interference (OPD, half-wave loss, rainbow spectrum),
  Fresnel zone analysis (radius, phasor sum), 2D diffraction grid sampling.

These modules should remain pure and ABI-oriented unless a formula clearly
maps to a Rapier force, torque, collider, query, or world-step operation.

### 17.6 AnvilKit Integration Layer

- `src/rapier/anvilkit.rs`
  - Owns an AnvilKit ECS app state and maps entities to Rapier rigid bodies,
    colliders, and constraints.
  - Supports spawning bodies/colliders, setting transforms/materials, syncing
    entities to a Rapier world, and resolving entity-to-body/collider handles.
  - Already includes domain-force application helpers:
    `anvilkit_app_apply_aero_surfaces`,
    `anvilkit_app_apply_aero_voxel_grid`,
    `anvilkit_app_apply_fluid_aabb_forces`,
    `anvilkit_app_apply_trajectory_forces`.

Current AnvilKit integration pattern:

```text
entity_bits
  -> anvilkit_app_entity_to_body
  -> RigidBodyHandleRaw
  -> trajectory/spaceflight/aerodynamics/fluid function applies force or torque
```

The existing AnvilKit functions use this pattern for aerodynamics, fluid
forces, and trajectory forces. Spaceflight formulas can be integrated in the
same style by adding `anvilkit_app_apply_space_*` wrappers that resolve the
entity body handle and delegate to `space_apply_*_to_body`.

Recommended AnvilKit wrappers to add next:

- `anvilkit_app_apply_space_j2_force`
- `anvilkit_app_apply_space_atmospheric_drag`
- `anvilkit_app_apply_space_solar_radiation_pressure`
- `anvilkit_app_apply_space_gravity_gradient_torque`
- `anvilkit_app_apply_space_magnetic_torquer`
- `anvilkit_app_apply_space_cmg_torque`

This keeps formulas reusable at three levels:

```text
Pure formula ABI
  -> Rapier WorldHandle/body apply function
    -> AnvilKit entity wrapper
```

### 17.7 Spatial Indexes and Support

- `src/rapier/crbtree.rs`
  - Compact RbTree spatial index.
  - Create, destroy, clear, len, insert, update, remove, AABB query count, AABB query handles.
  - Partially covered by Java 25 FFM.

- `src/rapier/rtree.rs`
  - RTree spatial index.
  - Create, destroy, clear, len, insert, update, remove, rebuild, AABB query count, AABB query handles.
  - Covered in Rust tests, Java 21 JNI helpers, and Java 25 FFM smoke tests.

- `src/rapier/joints.rs`
  - Impulse joint builders and insertion/removal.
  - Supports fixed, revolute, prismatic, rope, spring, and spherical joint styles.
  - Supports anchors, limits, motor velocity, and motor position.
  - Java 25 FFM parity is still pending.

- `src/rapier/error.rs`
  - Thread-local last-error code/message support.
  - Used for invalid pointer/capacity/unsupported operation reporting.

- `src/rapier/ffi.rs`
  - Shared ABI structs, enums, handle packing/unpacking, and conversion helpers.
  - The source of truth for C/JNI/FFM memory layout.

- `src/rapier/mod.rs`
  - Module wiring for the Rapier wrapper layer.

### 17.8 New Domains: Chaos, Relativity, Superfluidity, Plasma, Wave Optics

The project has expanded with five new domain modules implementing advanced
physics models beyond the Rapier core. All are pure ABI-oriented formula modules
(exceptions noted).

- `src/rapier/chaos.rs`
  - **Lorenz attractor**: RK4 integration, single-step and N-step batch (output buffer).
  - **Lyapunov exponent**: orbit-divergence method (reference + perturbed trajectory,
    periodic renormalisation) for the Lorenz system; Rosenstein algorithm (delay-coordinate
    embedding, nearest-neighbour divergence) for arbitrary time series.
  - **Bifurcation diagrams**: Lorenz parameter scan (σ, ρ, β with Poincaré section
    at x-local-maxima); logistic map r-parameter scan.
  - **Double pendulum (Lagrangian mechanics)**: α₁, α₂ from the Euler–Lagrange equations,
    RK4 integration with energy-conservation check.
  - **Chaos detection**: composite analysis using Lyapunov exponent + Grassberger–Procaccia
    correlation dimension with confidence scoring.
  - **Logistic map**: single-step, N-step iterate, and bifurcation diagram.

- `src/rapier/relativity.rs`
  - **Lorentz kinematics**: γ = 1/√(1−β²), 4×4 Lorentz boost matrix (3D velocity),
    four-vector transformation, relativistic velocity addition (3D general formula),
    rapidity φ = artanh(β), β-from-γ inversion.
  - **Schwarzschild metric**: g_tt, g_rr coefficients, Schwarzschild radius,
    light deflection angle δφ = 4GM/(bc²), effective potential with GR correction
    (−GM L² / c² r³), periapsis precession estimate.
  - **Gravitational time dilation**: stationary factor √(1−rs/r), circular orbital
    factor √(1−3rs/2r) with ISCO boundary (r > 1.5 rs), lightweight single-value query.
  - **Length contraction**: L = L₀/γ along motion direction.
  - **Relativistic particle physics**: γ, total energy (E = γmc²), kinetic energy
    (K = (γ − 1)mc²), momentum p = γmv, rapidity; massless (photon) support;
    invariant mass from E and p (E²/c⁴ − p²/c²) with tachyon detection.

- `src/rapier/superfluidity.rs`
  - **Biot–Savart vortex dynamics**: induced velocity from a straight vortex segment
    (analytical formula with singular endpoint/collinear rejection);
    vortex ring self-induced velocity v = (κ/4πR)[ln(8R/ξ) − ½].
  - **Quantised circulation**: circulation quantum κ₀ = h/m for ⁴He;
    loop-integral circulation with quantisation check; quantum-number estimation
    from tangential velocity samples.
  - **Gross–Pitaevskii model**: order parameter ψ = √n·exp(iφ) with density-depleted
    vortex core (Pade profile) and 2π phase winding; energy density (kinetic +
    interaction + trapping potential); imaginary-time amplitude evolution ∂ψ/∂τ
    = −(g|ψ|² − μ)ψ/ħ.
  - **Vortex reconnection**: closest-point detection between two segments;
    X-topology swap reconnection model with energy dissipation estimate.
  - **Vortex tangle statistics**: total length, average curvature, kinetic energy,
    vortex line density.
  - **GP grid sampling**: 2D cross-section of wavefunction for visualisation.
  - **Constants**: healing length ξ = ħ/√(2mgn), sound speed c = √(gn/m),
    helium mass and scattering length.

- `src/rapier/plasma.rs`
  - **Plasma parameters**: Debye length λ_D, electron/ion plasma frequencies ω_pe/ω_pi,
    Debye sphere count N_D, thermal velocity v_th.
  - **PIC building blocks**: Boris pusher (second-order symplectic: half-E → B-rotation
    → half-E); trilinear field interpolation from 3D grid; charge deposition
    ρ = q·w/V_cell, j = ρ·v.
  - **Vlasov moments**: density n, bulk velocity u, temperature T, heat flux q
    from particle ensemble.
  - **Self-consistent field**: 1D Poisson solver (Thomas algorithm for tridiagonal system),
    electric field E = −dφ/dx.
  - **Magnetic reconnection**: 2D X-point detection via B ≈ 0 + ∇B Jacobian determinant
    < 0 (hyperbolic null); Sweet–Parker rate (1/√S); Petschek fast rate (π/4ln S);
    Alfvén speed v_A; Lundquist number S.
  - **PIC step report**: kinetic energy, field energy, max E/B fields.

- `src/rapier/wave_optics.rs`
  - **Wave fundamentals**: wavenumber k = 2π/λ, plane wave E = A₀ exp(i(k·r − φ₀)),
    spherical wave E = A₀ exp(ikr)/r with 1/r decay.
  - **Huygens–Fresnel principle**: N point-source superposition for field at
    arbitrary observation point.
  - **Fresnel diffraction**: near-field scalar diffraction from a rectangular aperture
    (parabolic phase approximation, numerical Riemann sum).
  - **Fresnel–Kirchhoff diffraction integral**: full scalar diffraction with obliquity
    factor cosθ and −i/λ prefactor.
  - **Young's double-slit**: interference pattern I = cos²(πdx/λD)·sinc²(πax/λD)
    with single-slit envelope; 1D array sampling for caching.
  - **Thin-film interference**: OPD = 2n₂t cosθ_t, half-wave loss analysis,
    Fresnel reflection coefficients, normalised intensity; multi-wavelength
    spectrum batch (rainbow colour synthesis).
  - **Fresnel zones**: zone radius r_n = √(nλD), alternating π phase, phasor
    sum of first N zones.
  - **Grid sampling**: 2D Fresnel diffraction pattern caching for rendering.

## 18. New Feature Recommendations

The project already has a broad Rust/JNI surface. The most valuable next work is
to improve Java 25 FFM parity and then add higher-level ergonomics.

### 18.1 High Priority

1. Spaceflight Rapier and AnvilKit wrappers

   The pure formula ABI is broad, and several formulas already apply directly
   to Rapier bodies. The next integration step is to expose matching AnvilKit
   entity helpers.

   Add AnvilKit wrappers for:

   - J2 force
   - atmospheric drag
   - solar radiation pressure
   - gravity-gradient torque
   - magnetic torquer torque
   - CMG torque

   Reason: this completes the path from mission/space formulas to actual
   simulation behavior:

   ```text
   formula -> Rapier body force/torque -> AnvilKit entity-level API
   ```

2. Java 25 FFM character controller

   Add FFM bindings for:

   - controller create/destroy
   - up vector
   - offset absolute/relative
   - slide
   - autostep
   - snap to ground
   - slope angles
   - move shape
   - collision count/read
   - solve impulses

   Reason: character movement is a complete gameplay-facing feature, and JNI
   already proves the API shape.

3. Java 25 FFM joints

   Add FFM bindings for:

   - joint builder creation/destruction
   - contacts enabled
   - local anchors
   - limits
   - motor velocity
   - motor position
   - insert/remove impulse joint

   Reason: constraints are essential for physics gameplay and mechanical setups.

### 18.2 Medium Priority

4. Java 25 FFM advanced collider builders

   Add FFM coverage for:

   - heightmap
   - convex hull
   - point cloud bounds
   - double BV
   - skewed/discrete OBB
   - fused collapsing bounds
   - edge BVH
   - medial spheres
   - kDOP
   - FDH
   - neural bounds

   Reason: the Rust and JNI APIs already expose these builders, but FFM cannot
   use most of them yet.

5. Java 25 high-level helper layer

   Build a safer utility package for Java 25 similar to `test21/util`.

   Suggested wrappers:

   - `FfmPhysicsWorld`
   - `FfmRigidBody`
   - `FfmCollider`
   - `FfmQuery`
   - `FfmSpatialIndex`
   - `FfmVoxelGrid`

   Reason: `RigidBodyFfm` is intentionally low-level. A helper layer would hide
   repetitive `MemorySegment` allocation and native ownership details.

6. Bulk query APIs

   Add batch query support beyond ray batches:

   - batch AABB intersection counts
   - batch sphere intersection counts
   - batch OBB intersection counts
   - batch shape casts

   Reason: games and simulations often need many queries per tick. Batch calls
   reduce Java/native call overhead.

7. Air-drag / lift helper layer

   Add higher-level wrappers that can turn geometry samples into
   `AeroSurface` batches and feed them into `aero_apply_surfaces`.

   Reason: this is the cleanest path for an anvilkit-style integration. The
   Rust side stays physics-centric while the caller decides how to sample
   block faces, voxel shells, or mesh triangles.

### 18.3 Longer-Term Features

6. Voxel asset pipeline

   Possible additions:

   - voxelize heightmaps
   - voxelize triangle meshes
   - bitset or RLE compressed voxel input
   - chunked voxel collider streaming
   - material IDs instead of boolean occupancy

   Reason: current voxel support is functional for generated boxes/grids, but a
   content pipeline would make it practical for terrain and destructible scenes.

7. Contact material callbacks or material table

   Add a stable ABI-safe material table for friction/restitution/contact behavior.

   Reason: direct callbacks across FFI are hard to keep safe. A material table
   would provide configurable behavior without exposing arbitrary callback code.

8. Debug geometry export

   Add APIs to export collider/query debug geometry:

   - collider AABBs
   - voxel generated cuboids/mesh stats
   - contact points/normals
   - broad phase query debug output

   Reason: debugging native physics from Java is hard without visibility.

9. Serialization snapshots

   Add optional snapshot save/load:

   - body state
   - collider descriptors
   - joints
   - integration parameters

   Reason: useful for replay, testing, rollback, and editor workflows.

## 19. Rust Project Structure (Complete)

### 19.1 Crate Configuration

`Cargo.toml`:
- **Name:** `mps_rigid_body`
- **Version:** 0.1.4
- **Edition:** 2024
- **Crate types:** `cdylib` + `rlib` (shared library for Java + static for Rust consumers)
- **Panic strategy:** `abort` (both dev and release profiles, for JNI safety)

Key dependencies:

| Dependency | Version | Notes |
|---|---|---|
| `rapier3d-f64` | 0.33.0 | Double-precision physics engine, `parallel` feature enabled |
| `jni` (ljni) | 0.21.0 | JNI bindings for Java 21 |
| `parking_lot` | 0.12.5 | Synchronization primitives |
| `hashbrown` | 0.17.1 | High-performance HashMap/HashSet |
| `smallvec` | 1.15.2 | Small-vector optimization |
| `mimalloc` | 0.1.52 | Global allocator (MiMalloc) |
| `anvilkit` | 0.1 (optional) | ECS bridge with `rapier` feature |
| `bevy_ecs` | 0.19.0 (optional) | ECS for AnvilKit bridge |
| `rayon` | 1.12 | Parallel iteration |
| `cbindgen` | 0.29.4 | Build-time C header generator |

Feature flags:
- `default = []` — no features by default
- `anvilkit-bridge` — enables `dep:anvilkit` + `dep:bevy_ecs`

### 19.2 Build System

```text
build.rs (cbindgen)
  -> generates include/rigid_body.h from #[no_mangle] pub extern "C" fn
  -> header is consumed by Java 25 FFM bindings
```

### 19.3 Global Allocator

```rust
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;
```

MiMalloc is chosen for its performance in multi-threaded workloads (Rapier uses `rayon` for parallel broad-phase/narrow-phase).

### 19.4 Module Architecture

```
src/lib.rs
  ├── abi/
  │   ├── mod.rs      — module wiring
  │   ├── jni.rs       — Java_org_polaris2023_msp_rigid_body_RigidBodyNative_* functions
  │   └── ffm.rs       — abi_version, abi_supports_ffm, abi_supports_jni probes
  ├── helper/
  │   ├── mod.rs
  │   └── helper.rs    — JNI array/native-memory conversion utilities
  └── rapier/
      ├── mod.rs       — module wiring (42 submodules)
      ├── ffi/
      │   ├── mod.rs   — re-exports types + convert; unit tests for packed handles
      │   ├── types.rs — all #[repr(C)] structs, enums, and opaque handles (~2400 lines)
      │   └── convert.rs — pack/unpack handle conversion
      ├── math.rs      — shared math utilities (finite checks, Vec3 arithmetic, write_out)
      ├── error.rs     — thread-local last-error state
      ├── world.rs     — PhysicsWorld struct and world_* functions
      ├── rigid_body.rs — rigid body builder and runtime operations
      ├── collider.rs  — collider builder and runtime operations
      ├── query.rs     — ray, point, AABB, OBB, sphere, shape cast queries
      ├── events.rs    — collision/contact-force event collection
      ├── joints.rs    — impulse joint builder and management
      ├── controller.rs — character controller
      ├── voxel.rs     — voxel collider generation (raw/AABB/OBB → cuboids/mesh)
      ├── crbtree.rs   — compact RbTree spatial index
      ├── rtree.rs     — RTree spatial index
      ├── bounds.rs    — capsule, SSV, ellipsoid, prism, cylinder, spherical shell
      ├── dop.rs       — kDOP and FDH builders
      ├── neural.rs    — neural bounds
      ├── compat.rs    — dynamic cuboid/static trimesh insertion helpers
      ├── aerodynamics.rs — aero surface force accumulation
      ├── fluid.rs     — fluid volume buoyancy/drag, Navier-Stokes, SPH
      ├── trajectory.rs — point-mass and boost-glide trajectory models
      ├── spaceflight.rs — orbital mechanics, attitude, navigation, subsystems
      ├── control_theory.rs — PID, state-space stepping, MPC
      ├── anvilkit.rs  — AnvilKit ECS ↔ Rapier bridge
      └── [domain formula modules] — acoustics, astrophysics, biomechanics, chaos,
          continuum, electromagnetism, fracture, molecular, physchem, plasma,
          quantum, relativity, softbody, superfluidity, thermodynamics, topology,
          transmission, wave_optics
```

### 19.5 FFI Type System (src/rapier/ffi/types.rs)

This is the single source of truth for all cross-language types. Key categories:

**Opaque Handles (Rust-owned, passed as raw pointers):**

| Handle | Inner Type | Purpose |
|---|---|---|
| `WorldHandle` | `PhysicsWorld` | Full Rapier simulation state |
| `RigidBodyBuilderHandle` | `RigidBodyBuilder` | Rigid body configuration |
| `ColliderBuilderHandle` | `ColliderBuilder` | Collider configuration |
| `JointBuilderHandle` | `JointBuilderKind` | Joint configuration |
| `CharacterControllerHandle` | `CharacterControllerState` | Character controller state |
| `RTreeHandle` | `RTreeIndex` | RTree spatial index |
| `CRbTreeHandle` | `CRbTreeIndex` | Compact RbTree index |
| `AnvilKitAppHandle` | `AnvilKitAppState` | AnvilKit ECS state (feature-gated) |

**Packed Handles (non-zero u64, safe to store as Java `long`):**

| Type | Raw | Packing |
|---|---|---|
| `RigidBodyHandleRaw` | `u64` | `((gen << 32) \| id) + 1` |
| `ColliderHandleRaw` | `u64` | `((gen << 32) \| id) + 1` |
| `ImpulseJointHandleRaw` | `u64` | `((gen << 32) \| id) + 1` |

**Primitive ABI Types:**

- `Vec3` — `{x, y, z: f64}`, 24 bytes, repr(C)
- `Quat` — `{i, j, k, w: f64}`, 32 bytes, repr(C)
- `Bool` — `u8` wrapper (0 = false, 1 = true), repr(C)
- `BodyStatus` — enum: Dynamic/Fixed/KinematicPositionBased/KinematicVelocityBased
- `ShapeType` — enum: Ball/Cuboid/CapsuleY/X/Z/Cylinder/RoundCylinder/Cone/RoundCone/RoundCuboid
- `VoxelColliderMode` — enum: Auto/Cuboids/GreedyCuboids/SurfaceMesh
- `JointAxisDesc` — enum: LinX/LinY/LinZ/AngX/AngY/AngZ
- `JointTypeDesc` — enum: Fixed/Revolute/Prismatic/Rope/Spring/Spherical
- `NeuralActivation` — enum: Relu/Tanh/Sin/Linear
- `KdopPreset` — enum: K6=6/K14=14/K18=18/K26=26

**Descriptor Structs (input parameters):**

- `ShapeDesc`, `InteractionGroupsDesc`, `QueryFilterDesc`, `ShapeCastOptionsDesc`
- `AabbDesc`, `Obb`, `Sphere`, `Capsule`, `Ssv`, `Ellipsoid`, `Prism`, `Cylinder`, `SphericalShell`
- `VoxelColliderOptions`, `NeuralBoundsDesc`

**Result Structs (output records):**

- `PointProjection`, `RayHit`, `ShapeCastHit`
- `CollisionEventRecord`, `ContactForceEventRecord`
- `VoxelBuildStats`
- `EffectiveCharacterMovement`, `CharacterCollision`

**Aerodynamics / Fluid:**
- `AeroSurface`, `AeroForceReport`
- `FluidVolume`, `FluidForceReport`, `NavierStokesReport`
- `SphParticle`, `SphForceReport`

**Trajectory / Spaceflight:**
- `TrajectoryState`, `TrajectoryEnvironment`, `TrajectoryForceReport`
- `TrajectoryGlideState`, `TrajectoryGlideEnvironment`, `TrajectoryGlideReport`
- `OrbitalElements`, `StateVector`
- `HohmannTransfer`, `ScalarKalman`, `LeastSquaresAttitude`, `GnssObservation`
- `QuaternionDerivative`, `RigidBodyEulerDerivative`, `CmgExchange`, `CmgRobustInverse`
- `Sgp4SecularRates`, `CwState`, `CwDerivative`
- `SolarPanelPower`, `ThermalBalance`, `FriisLink`, `RadiatorPower`
- `Co2MassBalance`, `BatteryEquivalentCircuit`, `HallThrusterPerformance`
- `ContactForceModel`, `MassProperties`, `BangOffBangProfile`
- `CollisionProbability`, `AtomicOxygenErosion`, `FlexibleModeDerivative`, `SloshPendulumDerivative`
- `VariationalState`, `FluidLoopHeatTransfer`, `RadarMeasurement`, `AirlockDepressurization`
- `DhTransform`, `ManipulatorDynamics`

**Relativity:**
- `LorentzBoost`, `LorentzTransformedFrame`, `RelativisticParticle`
- `SchwarzschildMetric`, `GravitationalTimeDilation`, `LengthContraction`
- `RelativisticOrbitReport`, `RocheLimitReport`, `OrbitalResonanceReport`

**N-Body / Astrophysics:**
- `NBodyParticle`, `NBodySolverParams`, `NBodyForceReport`

**Molecular Dynamics:**
- `MolecularParticle`, `MolecularForceLaw`, `MolecularPairReport`

**Quantum:**
- `QuantumWaveFunction`, `QuantumBarrier`, `QuantumTunnelingReport`, `QuantumOscillatorReport`

**FEM / Continuum Mechanics:**
- `FemTetrahedron`, `FemShapeFunctionReport`, `FemConstitutiveReport`
- `NewmarkBetaParameters`, `NewmarkBetaReport`
- `BernoulliReport`
- `MaterialProperties`, `StressStrainReport`, `HertzContactReport`

**Fracture Mechanics:**
- `FractureMaterial`, `StressIntensityReport`, `GriffithReport`
- `MinerDamageReport`, `SnCurveReport`, `FractureEnergyReport`, `FractureModeReport`
- `FractureFragmentDesc`, `FractureReplaceReport`

**Thermodynamics:**
- `HeatConductionReport`, `PhaseChangeReport`, `ThermalRadiationReport`
- `FemHeatNode`, `FemHeatEdge`, `FemHeatDiffusionReport`
- `ThermalStressReport`, `ThermoelasticReport`

**Electromagnetism:**
- `ElectromagneticField`, `LorentzForceReport`, `MagneticFluxReport`
- `FaradayInductionReport`, `MaxwellPointReport`, `FdtdYeeReport`

**Acoustics:**
- `ModalAnalysisReport`, `StructuralModeReport`, `AcousticWaveReport`, `AcousticResonanceReport`
- `AcousticMaterial`, `AcousticContactDesc`, `AcousticExcitationReport`
- `ModalSynthesisReport`, `SpatializedSample`

**Soft Body:**
- `SoftSpring`, `SoftDistanceConstraint`, `SoftBendingConstraint`
- `SoftSphereCollision`, `SoftVolumeConstraint`, `SoftBodyStepReport`

**Biomechanics:**
- `HillMuscleDesc`, `HillMuscleState`, `HillMuscleReport`
- `SkeletalJointLimit`, `SkeletalConstraintReport`

**Mechanical Constraints:**
- `GearConstraintDesc`, `GearConstraintReport`
- `ScrewConstraintDesc`, `ScrewConstraintReport`
- `CamConstraintDesc`, `CamConstraintReport`
- `SpiralConstraintDesc`, `SpiralConstraintReport`

**Control Theory:**
- `PidGains`, `PidState`, `PidReport`
- `StateSpaceReport`, `MpcConfig`, `MpcReport`

**Topology Optimization:**
- `TopologyOptimizationParams`, `TopologyOptimizationReport`
- `SimpMaterialReport`, `DensityFieldStats`

**Coulomb/Drag/External Forces:**
- `CoulombFrictionLaw`, `AirDragLaw`, `ExternalForceLaw`, `CustomPhysicsReport`

**PhysChem:**
- `ChemicalReactionRate`, `GrayScottParams`, `ReactionDiffusionReport`
- `GrayScottReactionReport`, `CatalystEffect`, `CatalystReport`, `ConcentrationBuoyancyReport`

**Chaos:**
- `LorenzState`, `LorenzParams`, `LorenzStepReport`
- `LyapunovReport`, `BifurcationPoint`
- `DoublePendulumState`, `DoublePendulumParams`, `DoublePendulumAccel`
- `ChaosDetectionReport`, `ChaosDetectionParams`, `LogisticMapState`

**Superfluidity:**
- `VortexSegment`, `BiotSavartVelocity`, `VortexRing`, `VortexReconnectionReport`
- `GpOrderParameter`, `GpEnergyDensity`, `GpTimeEvolutionParams`
- `QuantisedCirculation`, `VortexTangleStats`, `GpGridPoint`

**Wave Optics:**
- `ComplexAmplitude`, `PlaneWaveParams`, `DiffractionPoint`, `PointSource`
- `FresnelZoneReport`, `ThinFilmInterferenceReport`, `ThinFilmParams`
- `KirchhoffDiffractionPoint`, `SphericalWavePoint`, `ApertureDesc`, `YoungSlitPoint`

**Plasma:**
- `PicParticle`, `GridField`, `ChargeDensityCell`
- `PlasmaParamsReport`, `VlasovMomentReport`, `MagneticXPoint`, `PicStepReport`
- `BorisPusherParams`

### 19.6 FFI Convert Layer (src/rapier/ffi/convert.rs)

Provides bidirectional conversion between Rapier types and ABI types:

| Rust → ABI | ABI → Rust |
|---|---|
| `RigidBodyHandle` ↔ `RigidBodyHandleRaw` (pack/unpack) | |
| `ColliderHandle` ↔ `ColliderHandleRaw` (pack/unpack) | |
| `ImpulseJointHandle` ↔ `ImpulseJointHandleRaw` (pack/unpack) | |
| `na::Vector3<f64>` → `Vec3` | `Vec3` → `na::Vector3<f64>` |
| `na::Quaternion<f64>` → `Quat` | `Quat` → `na::Quaternion<f64>` |
| `na::Isometry3<f64>` → `(Vec3, Quat)` | `(Vec3, Quat)` → `na::Isometry3<f64>` |
| `RapierInteractionGroups` ↔ `InteractionGroupsDesc` | |
| `QueryFilter` ↔ `QueryFilterDesc` | |
| `ShapeCastOptions` ↔ `ShapeCastOptionsDesc` | |
| `bool` ↔ `Bool` | |

### 19.7 Math Utilities (src/rapier/math.rs)

Shared helper functions replacing previously duplicated code:

```rust
fn finite(value: f64) -> bool
fn finite_positive(value: f64) -> bool
fn finite_non_negative(value: f64) -> bool
fn clamp(value: f64, lo: f64, hi: f64) -> f64
fn write_out<T: Copy>(out: *mut T, value: T) -> Bool  // writes through pointer with null check
fn vec3_add/sub/scale/dot/cross/length/length_sq/normalize
```

### 19.8 Error Handling (src/rapier/error.rs)

Thread-local last-error state for FFI boundary error reporting:

```rust
fn set_error(code: u32, message: &str)
fn clear_error()
fn get_last_error_code() -> u32
fn get_last_error_message() -> *const c_char
```

Error codes:
- `ERR_NULL_POINTER` — output pointer is null
- `ERR_INVALID_HANDLE` — handle pointer is null
- `ERR_INVALID_BODY` — body handle is zero
- `ERR_INVALID_COLLIDER` — collider handle is zero
- `ERR_INVALID_JOINT` — joint handle is zero
- `ERR_UNSUPPORTED` — operation not supported for the given type

## 20. Java Minecraft Integration (test21)

### 20.1 NeoForge Mod Setup

`test21/` is a **NeoForge 1.21.1** mod project that integrates the Rust native library into Minecraft via JNI.

**Build configuration (`test21/build.gradle`):**
- **Minecraft version:** 1.21.1
- **NeoForge version:** 21.1.233
- **Java version:** 21
- **Mod ID:** `mps_rigid_body`
- **Mod name:** "MPS Rigid Body"
- **Author:** Polaris Stars

**Native library integration:**
```groovy
tasks.register('cargoBuildRelease', Exec)   // cargo build --release
tasks.register('syncNativeLibraryToRelease') // ensures library in target/release
tasks.register('copyNativeLibraryToResources', Copy)  // copies to resources/native/{os}/{arch}
```

The native library (`mps_rigid_body.dll`/`.so`/`.dylib`) is bundled as a mod resource and extracted at runtime. The library path is set via `java.library.path` system property.

### 20.2 Java Class Structure

```
org.polaris2023.msp_rigid_body/
  RigidBody.java          — Public API facade (static methods)
  RigidBodyNative.java    — JNI native method declarations

org.polaris2023.msp_rigid_body.util/
  PhysicsWorld.java       — World lifecycle wrapper
  RigidBody.java          — Rigid body wrapper (opaque long handle)
  Collider.java           — Collider wrapper
  CharacterController.java — Character controller wrapper
  SpatialIndex.java       — CRbTree/RTree wrapper
  NativeMemory.java       — Unsafe-based native memory allocation
  IParent.java            — Parent interface for collider insertion
  VoxelGrid.java          — Voxel occupancy grid helper
```

### 20.3 RigidBody.java (Public API)

The top-level `RigidBody` class provides a static facade over JNI calls:

```java
// ABI
static int abiVersion()
static boolean abiSupportsFfm()
static boolean abiSupportsJni()

// World
static long worldCreate(double gx, double gy, double gz)
static void worldDestroy(long world)
static void worldStep(long world, double dt)
static void worldSetGravity(long world, double x, double y, double z)
static double[] worldGetGravity(long world)

// Rigid Body Builder
static long rigidBodyBuilderCreate(int status)
static void rigidBodyBuilderDestroy(long builder)
static void rigidBodyBuilderSetTranslation(long builder, double x, double y, double z)
static long worldInsertRigidBody(long world, long builder)
static double[] rigidBodyGetTranslation(long world, long body)

// CRbTree
static long crbTreeCreate()
static void crbTreeDestroy(long tree)
static void crbTreeClear(long tree)
static int crbTreeLen(long tree)
static boolean crbTreeInsert(long tree, long id, double minX, ..., double maxZ)
static boolean crbTreeUpdate(...)
static boolean crbTreeRemove(long tree, long id)
static int crbTreeQueryAabbCount(...)
static int crbTreeQueryAabb(long tree, ..., long outIds, int capacity)

// RTree
static long rtreeCreate()
static void rtreeDestroy(long tree)
static boolean rtreeInsert(...)
static int rtreeQueryAabbCount(...)
```

### 20.4 NativeMemory.java

Uses `sun.misc.Unsafe` for direct native memory allocation, used as an output buffer for query results:

```java
class NativeMemory implements AutoCloseable {
    static NativeMemory longs(int count)
    static NativeMemory bytes(byte[] data)
    long address()
    long getLong(long offset)
    int getInt(long offset)
    double getDouble(long offset)
    boolean getBool(long offset)
    void putByte(long offset, int value)
    void putLong(long offset, long value)
    void putDouble(long offset, double value)
    double[] getVec3(long offset)
    long[] getLongs(int count)
    void close()  // frees native memory
}
```

### 20.5 PhysicsWorld.java (util)

Wraps a `WorldHandle` pointer:

```java
class PhysicsWorld {
    PhysicsWorld(double gx, double gy, double gz)   // worldCreate
    long handle()
    void step(double dt)                              // worldStep
    void setGravity(double x, double y, double z)
    double[] getGravity()
    RigidBody insertRigidBody(RigidBody body)        // worldInsertRigidBody
    void destroy()                                    // worldDestroy
}
```

### 20.6 util/RigidBody.java

Wraps a `RigidBodyHandleRaw` (packed u64):

```java
class RigidBody {
    RigidBody()              // empty (handle = 0)
    RigidBody(long body)     // from packed handle
    boolean isEmpty()
    long handle()
    double[] translation(PhysicsWorld world)    // getTranslation
    double[] rotation(PhysicsWorld world)       // getRotation
    double[] linvel(PhysicsWorld world)         // getLinvel
    double[] angvel(PhysicsWorld world)         // getAngvel
    RigidBody translation(PhysicsWorld, x, y, z, wakeUp)  // setTranslation
    RigidBody pose(PhysicsWorld, ...)           // setPose
    RigidBody linearVelocity(...)               // setLinearVelocity
    RigidBody angularVelocity(...)              // setAngularVelocity
    RigidBody addForce(PhysicsWorld, x, y, z)
    RigidBody addTorque(PhysicsWorld, x, y, z)
    RigidBody applyImpulse(PhysicsWorld, x, y, z)
    RigidBody applyTorqueImpulse(PhysicsWorld, x, y, z)
    RigidBody enableCcd(PhysicsWorld, boolean)
    RigidBody sleep(PhysicsWorld)
    RigidBody wakeUp(PhysicsWorld)
    boolean isSleeping(PhysicsWorld)
}
```

### 20.7 Smoke Test (JniSmokeTest.java)

The `runJniTest` Gradle task executes `JniSmokeTest`:
- Validates ABI version compatibility
- Creates a world with gravity (0, -9.81, 0)
- Steps the simulation
- Creates a rigid body builder, inserts it
- Reads back the body's translation
- Creates CRbTree, inserts entries, queries
- Creates RTree, inserts entries, queries
- Cleans up all resources

### 20.8 Minecraft Mod Entry Point

The mod is registered via `neoforge.mods.toml`:
```toml
modId="mps_rigid_body"
displayName="MPS Rigid Body"
description="JNI-backed rigid body test mod."
```

NeoForge runs (`client`, `server`, `gameTestServer`, `data`) are configured in `build.gradle`. The native library is loaded at mod initialization via `System.loadLibrary("mps_rigid_body")`.

## 21. Java 25 FFM Integration (test25)

### 21.1 Project Setup

`test25/` is a **NeoForge 1.26.1** (Minecraft future version) mod project using Java 25's Foreign Function & Memory API instead of JNI.

**Build configuration:**
- **Minecraft version:** 1.26.1
- **NeoForge version:** 26.1.2
- **Java version:** 25
- **JVM args:** `--enable-native-access=ALL-UNNAMED`

**Key difference from test21:** The NeoForge moddev plugin is conditionally applied via `enableNeoForgePreview` flag (default: false). When disabled, test25 acts as a pure FFM test without Minecraft integration. When enabled, it also registers as a NeoForge mod.

### 21.2 FFM Smoke Test

`FfmSmokeTest` uses Java 25 FFM API to:
- Load the native library via `SymbolLookup.libraryLookup()`
- Define `MemoryLayout` for each `repr(C)` struct
- Create `MethodHandle` downcall handles for each native function
- Call Rust functions directly through FFM (no JNI layer)
- Validate results against expected values

**Coverage includes:**
- World create/destroy/step
- Gravity get/set
- Rigid body lifecycle (create, insert, translate, rotate, apply force)
- Collider creation and insertion
- Voxel AABB/OBB collider generation
- CRbTree and RTree operations
- Ray, point projection, AABB, OBB, sphere, shape cast queries
- Collision and contact-force event reads
- Event clearing

### 21.3 FFM vs JNI Comparison

| Aspect | JNI (test21) | FFM (test25) |
|---|---|---|
| Java version | 21 | 25 |
| Minecraft version | 1.21.1 | 1.26.1 (preview) |
| Native access | `System.loadLibrary` + JNI functions | `SymbolLookup` + MethodHandle downcalls |
| Handle type | `long` (64-bit integer) | `MemorySegment` (safe memory reference) |
| Struct layout | Manual offset calculation | `MemoryLayout` definition |
| Safety | JNI boundary with panic catching | Direct C ABI with out-pointer validation |
| Performance | JNI overhead per call | Near-zero overhead (direct C ABI) |
| Build integration | JNI header generation | C header via cbindgen (already done) |

### 21.4 FFM Struct Layout Example

Java 25 `MemoryLayout` definitions mirror Rust `repr(C)`:

```java
// Rust: pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
static final StructLayout VEC3 = MemoryLayout.structLayout(
    ValueLayout.JAVA_DOUBLE.withName("x"),
    ValueLayout.JAVA_DOUBLE.withName("y"),
    ValueLayout.JAVA_DOUBLE.withName("z")
);

// Rust: pub struct RayHit { collider: u64, time_of_impact: f64, ... }
static final StructLayout RAY_HIT = MemoryLayout.structLayout(
    ValueLayout.JAVA_LONG.withName("collider"),
    ValueLayout.JAVA_DOUBLE.withName("time_of_impact"),
    VEC3.withName("normal"),
    ValueLayout.JAVA_INT.withName("feature")
);
```

## 22. How to Use in Minecraft

### 22.1 JNI Path (Minecraft 1.21.1, Java 21)

**Step 1: Build the native library**
```powershell
cargo build --release
```

**Step 2: Run the mod**
```powershell
cd test21
.\gradlew.bat runClient
```

The native library is automatically copied to `build/generated/resources/native/{os}/{arch}/` and loaded by the mod.

**Step 3: Use in mod code**
```java
import org.polaris2023.msp_rigid_body.RigidBody;

// Create physics world
long world = RigidBody.worldCreate(0, -9.81, 0);

// Create a rigid body
long builder = RigidBody.rigidBodyBuilderCreate(0); // 0 = Dynamic
RigidBody.rigidBodyBuilderSetTranslation(builder, 0, 64, 0);
long body = RigidBody.worldInsertRigidBody(world, builder);
RigidBody.rigidBodyBuilderDestroy(builder);

// Step simulation
RigidBody.worldStep(world, 1.0 / 20.0);

// Read position
double[] pos = RigidBody.rigidBodyGetTranslation(world, body);
// pos[0] = x, pos[1] = y, pos[2] = z

// Cleanup
RigidBody.worldDestroy(world);
```

### 22.2 FFM Path (Minecraft 1.26.1+, Java 25)

**Step 1: Build the native library**
```powershell
cargo build --release
```

**Step 2: Enable NeoForge preview**
```powershell
cd test25
.\gradlew.bat runClient -Penable_neoforge_preview=true
```

**Step 3: Use FFM bindings in mod code**

The `RigidBodyFfm.java` class provides high-level wrappers:

```java
import org.polaris2023.msp_rigid_body.ffm.RigidBodyFfm;

// Create physics world
try (var world = RigidBodyFfm.World.create(0, -9.81, 0)) {
    // Create rigid body
    var body = RigidBodyFfm.RigidBody.builder()
        .status(BodyStatus.DYNAMIC)
        .translation(0, 64, 0)
        .build(world);
    
    // Step simulation
    world.step(1.0 / 20.0);
    
    // Read position
    var pos = body.getTranslation();
    // pos.x(), pos.y(), pos.z()
}
// World auto-closes via try-with-resources
```

### 22.3 Minecraft-Specific Integration Points

**Entity Physics Replacement:**
- Replace Minecraft's built-in AABB collision with Rapier's full rigid body dynamics
- Entities become `Dynamic` rigid bodies
- Terrain blocks become `Fixed` colliders (generated from voxel grids)

**Voxel Terrain Colliders:**
```java
// Create collider from Minecraft chunk data
byte[] occupancyGrid = chunkToOccupancyGrid(chunk); // 16x16x16 or similar
long colliderBuilder = RigidBodyNative.colliderBuilderCreateVoxels(
    occupancyGrid, occupancyGrid.length,
    sizeX, sizeY, sizeZ,
    centerX, centerY, centerZ,
    scaleX, scaleY, scaleZ,
    mode, // VoxelColliderMode.Auto
    options
);
```

**Spatial Queries:**
```java
// Find entities near a position (replaces Minecraft's entity lookup)
long rtree = RigidBody.rtreeCreate();
// ... insert entity AABBs into rtree ...
int count = RigidBody.rtreeQueryAabbCount(
    rtree, x - radius, y - radius, z - radius,
    x + radius, y + radius, z + radius
);
```

**Character Controller:**
```java
// Replace Minecraft's player movement with Rapier character controller
long controller = RigidBodyNative.controllerCreate(0.4); // offset
RigidBodyNative.controllerSetSlideEnabled(controller, true);
RigidBodyNative.controllerSetAutostep(controller, 0.6, 0.2, true);
// ... each tick:
EffectiveCharacterMovement movement = RigidBodyNative.controllerMoveShape(
    controller, world, dx, dy, dz, 0.0, 1.0, // filter flags, groups
    outCollisions, maxCollisions
);
```

### 22.4 Performance Considerations for Minecraft

1. **Native library loading:** The library is extracted from the JAR to a temp directory on first load. Subsequent loads use the cached path.

2. **Thread safety:** The Rust side uses `parking_lot::Mutex` for event queues. Physics stepping is single-threaded per world. Multiple worlds can run in parallel.

3. **Memory management:** Opaque handles must be explicitly destroyed. Java `long` values are stored as primitive fields. FFM uses `MemorySegment` which is garbage-collected with Arena scoping.

4. **Chunk-based physics:** For large Minecraft worlds, create one Rapier world per loaded chunk region and synchronize at boundaries.

5. **Voxel collider caching:** Voxel collider generation is expensive. Cache generated colliders per chunk and regenerate only when blocks change.

6. **Spatial queries:** Use CRbTree/RTree for entity lookups instead of iterating all entities. Insert entity AABBs each tick and query by region.

## 23. Verification Commands

```powershell
# Rust tests (172 tests)
cargo test

# Java 21 JNI smoke test
cd test21
.\gradlew.bat check

# Java 25 FFM smoke test
cd ..\test25
.\gradlew.bat check

# Full build
cargo build --release
cd test21 && .\gradlew.bat build
cd ..\test25 && .\gradlew.bat build
```
