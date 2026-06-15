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
    bounds.rs
    collider.rs
    compat.rs
    controller.rs
    crbtree.rs
    dop.rs
    error.rs
    events.rs
    ffi.rs
    joints.rs
    mod.rs
    neural.rs
    query.rs
    rigid_body.rs
    rtree.rs
    voxel.rs
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
Java 21 JNI has broader helper coverage through `SpatialIndex`.

RTree is the next obvious Java 25 FFM parity gap.

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
- Regular queries: ray, point projection, AABB, OBB, sphere, shape cast.
- Voxel-shaped AABB/OBB queries.
- Collision event and contact-force event bulk reads.
- Event clearing.

## 16. Recommended Next Steps

Recommended implementation order:

1. Java 25 FFM RTree parity:
   create, destroy, clear, len, insert, update, remove, rebuild, query count, query handles.

2. Java 25 FFM character controller:
   controller lifecycle, configuration setters, move shape, collision count/read, solve impulses.

3. Java 25 FFM joints:
   joint builder creation, anchors, limits, motors, insert/remove.

4. Java 25 FFM advanced collider builders:
   heightmap, convex hull, point-cloud bounds, kDOP, FDH, neural bounds.

5. Higher-level Java 25 helper layer:
   `RigidBodyFfm` is currently low-level. A safer Java 25 utility layer similar
   to `test21/util` would reduce direct `MemorySegment` handling in user code.
