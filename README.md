# mps_rigid_body

`mps_rigid_body` is a Rust native physics library built on `rapier3d-f64`.
It exposes one native `cdylib` to Java through both JNI and Java FFM.

The project provides a stable Java-facing rigid body API while keeping Rapier-owned
world state, bodies, colliders, events, and query pipelines inside Rust.

```text
Java 21 JNI / Java 25 FFM
  └─ Rust C ABI / JNI wrappers
       ├─ mps-formula — 28 pure formula modules (300+ functions)
       ├─ mps-core — physics engine + Rapier wrapper
       ├─ mps-jni — JNI bindings
       └─ mps-ffm — FFM metadata
```

## Repository Layout

```text
crates/
  mps-core/       physics world, bodies, colliders, queries, events, forces, voxel
  mps-formula/    28 pure physics/engineering formula modules
  mps-jni/        Java JNI bindings
  mps-ffm/        Java FFM metadata
  mps-test/       332 integration tests

docs/             documentation site (dark-theme, dual-language zh/en)
```

## Formula Library (mps-formula)

The formula crate provides 28 modules with 300+ pure Rust functions covering
physics, aerospace, and engineering domains. No dependency on Rapier or WorldHandle.

| Module | Functions | Domain |
|--------|-----------|--------|
| `spaceflight` | 88 | orbital mechanics, attitude control, thermal, propulsion, environment |
| `material_mechanics` | 26 | elasticity, plasticity, fracture, fatigue, beam theory |
| `nuclear` | 23 | decay, binding energy, fission/fusion, neutronics |
| `relativity` | 23 | Lorentz, Schwarzschild, Kerr, ISCO, gravitational redshift |
| `thermodynamics` | 23 | conduction, radiation, phase change, gas laws, cycles |
| `quantum` | 20 | wave functions, tunneling, harmonic oscillator, hydrogen atom |
| `astrophysics` | 19 | N-body, Barnes-Hut, FMM, Lane-Emden, Eddington, Hubble |
| `fluid` | 18 | buoyancy/drag, SPH, Navier-Stokes, Bernoulli, turbulence |
| `electromagnetism` | 16 | Lorentz, Faraday, Maxwell, Biot-Savart, Poynting, wave |
| `aerodynamics` | 5 | surface force, voxel aero, force estimation |
| `molecular` | 8 | Lennard-Jones, Coulomb, pair interaction |
| `acoustics` | 7 | modal analysis, wave equation, resonance, spatialization |
| `biomechanics` | 4 | Hill muscle model, joint constraints |
| `celestial_data` | 1 | 10 solar system bodies (JPL DE441) |
| `chaos` | 6 | Lorenz attractor, double pendulum, Lyapunov exponents |
| `continuum` | 5 | FEM shape functions, strain/stress tensors |
| `control_theory` | 7 | PID, state-space, MPC, LQR |
| `gravitational_models` | 6 | spherical harmonics (EGM2008 8×8), ellipsoid, polyhedron |
| `integrators` | 7 | Leapfrog, Yoshida 4, Forest-Ruth 8, post-Newtonian |
| `physchem` | 4 | Gray-Scott reaction-diffusion, catalysis |
| `plasma` | 7 | Debye shielding, Vlasov, PIC, MHD, magnetic reconnection |
| `softbody` | 5 | XPBD constraints, hyperelastic constitutive models |
| `superfluidity` | 4 | Gross-Pitaevskii, vortex lattice, quantized circulation |
| `topology` | 3 | persistent homology, Betti numbers |
| `trajectory` | 6 | 6DOF ballistic/glide trajectory, RK4 integration |
| `transmission` | 3 | gear ratios, torque distribution |
| `wave_optics` | 5 | Kirchhoff diffraction, Fresnel propagation, interference |

## Native API Surface

The Rust crate defines C-compatible ABI types in `crates/mps-core/src/rapier/ffi/`.
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

## Formula Modules

Each formula module follows a two-layer architecture:

- **mps-formula**: Pure computation — input values, output values, no `WorldHandle`, `RigidBody`, or Rapier state.
- **mps-core**: C ABI wrappers + Rapier interaction — reads body state, calls formula, applies forces/torques.

All existing C ABI function names, parameters, error codes, and `_flag` variants are preserved for backward compatibility.

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
cargo test -p mps-test              # 332 integration tests
cargo check --workspace              # full workspace check

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

## Documentation

Online documentation at `docs/index.html` — dark-theme dual-language (zh/en) site covering all modules, API reference, integration guides, and performance data.

## Current Gaps

The main remaining integration work is Java 25 FFM parity for areas that already
exist in Rust/JNI:

- Character controller.
- Joints.
- Advanced collider builders such as heightmap, convex hull, point-cloud bounds, kDOP, FDH, and neural bounds.