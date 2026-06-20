# msp_rigid_body

`msp_rigid_body` is a Rust native physics library built on `rapier3d-f64`.
It exposes one native `cdylib` to Java through both JNI and Java FFM.

The project is intended to provide a stable Java-facing rigid body API while
keeping Rapier-owned world state, bodies, colliders, events, and query pipelines
inside Rust.

```text
Java 21 JNI / Java 25 FFM
  -> Rust C ABI / JNI wrappers
    -> project Rapier wrapper modules
      -> rapier3d-f64
```

## Repository Layout

```text
src/
  abi/       Java FFM metadata and JNI wrappers
  helper/    JNI helper utilities
  rapier/    physics world, bodies, colliders, queries, events, voxel, indexes

test21/      Java 21 JNI smoke test project
test25/      Java 25 FFM smoke test project
```

## Native API Surface

The Rust crate defines C-compatible ABI types in `src/rapier/ffi.rs`.
External callers use opaque native pointers for world and builder ownership,
and packed `u64` handles for Rapier rigid bodies, colliders, and joints.

Supported areas include:

- World creation, stepping, gravity, integration parameters, body snapshots.
- Rigid body creation, insertion, pose/velocity mutation, forces, impulses, CCD, sleep/wakeup.
- Collider creation, insertion, runtime material/group/event settings.
- Air-drag and lift accumulation for surface samples, driven by Rapier rigid body motion.
- Ray, point, AABB, OBB, sphere, shape-cast, and voxel-shaped queries.
- Collision and contact-force event queues.
- Joints and character controller through JNI.
- Compact tree and RTree spatial indexes.
- Extended bounds/collider builders: capsule, SSV, ellipsoid, prism, cylinder, shell, kDOP, FDH, neural bounds.
- Voxel collider construction from raw grids, AABB, and OBB.

## Java Entry Points

### Java 21 JNI

`test21` uses `RigidBodyNative` JNI methods plus higher-level Java helpers in
`org.polaris2023.msp_rigid_body.util`.

Run:

```powershell
cd test21
.\gradlew.bat check
```

### Java 25 FFM

`test25` uses `RigidBodyFfm` with Java's Foreign Function & Memory API.
It covers:

- World, rigid body, collider, and CRbTree basics.
- Voxel AABB/OBB build stats and collider creation.
- Voxel AABB/OBB intersection queries.
- Regular runtime queries: ray cast, point projection, AABB/OBB/sphere intersection, shape cast.
- Rigid body runtime mutation: pose, velocity, force/torque, impulse, CCD, sleep/wakeup.
- Air-drag and lift surface accumulation helpers for body motion.
- Collider runtime mutation: pose, sensor, friction, restitution, groups, event bits, hooks, contact-force threshold.
- Collision and contact-force event bulk reads plus event clearing.

Run:

```powershell
cd test25
.\gradlew.bat check
```

## Voxel Colliders

Voxel colliders can be created from:

- Raw occupancy grids: native memory or Java `byte[]`.
- Axis-aligned bounding boxes: `collider_builder_create_voxel_aabb`.
- Oriented bounding boxes: `collider_builder_create_voxel_obb`.

Build modes are controlled by `VoxelColliderOptions`:

- `Auto`: choose from voxel count and dynamic/static body usage.
- `Cuboids`: one cuboid per solid voxel.
- `GreedyCuboids`: merge adjacent solid voxels into larger cuboids.
- `SurfaceMesh`: generate an exterior triangle mesh for large static voxel sets.

`VoxelBuildStats` can be used before building to inspect cell count, solid count,
selected mode, estimated parts, estimated vertices/triangles, and generated grid size.

Java 21 includes a `VoxelGrid` helper with:

```text
get, set, clear, solidCount, fillBox, fillAabb, fillSphere,
copyFrom, union, subtract, intersect, toByteArray, address
```

## Verification

Current verified commands:

```powershell
cargo test

cd test21
.\gradlew.bat check

cd ..\test25
.\gradlew.bat check
```

Expected result:

```text
Rust tests: passed
Java 21 JNI smoke test: passed
Java 25 FFM smoke test: passed
```

## Current Gaps

The main remaining integration work is Java 25 FFM parity for areas that already
exist in Rust/JNI:

- Character controller.
- Joints.
- Advanced collider builders such as heightmap, convex hull, point-cloud bounds, kDOP, FDH, and neural bounds.
