# mps — Motion Physics System

> **mps** is the project name. It is a double entendre:
> - **m/s** — meters per second, the SI unit of velocity (the quantity this engine ultimately governs);
> - **M**otion **P**hysics **S**ystem — the engine itself.
>
> The GitHub repository is named **`rigid-body`** (`Polari-Stars-MC/rigid-body`); within docs and code we refer to the project as **mps**.

`mps` is a Rust-native physics engine built on [`rapier3d-f64`](https://rapier.rs) (double-precision). It wraps Rapier's world state, bodies, colliders, events, and query pipelines behind a single stable native `cdylib` with a C ABI, and keeps all Rapier-owned state inside Rust.

External consumers drive the simulation through opaque world/builder pointers and packed `u64` handles for rigid bodies, colliders, and joints — no Rapier types leak across the boundary.

```text
mps (Motion Physics System)
 └─ Rust workspace (rigid-body repo)
      ├─ mps-formula — 28 pure physics/engineering formula modules (no Rapier)
      ├─ mps-core     — physics world + Rapier wrapper + C ABI surface
      ├─ mps-cosmos   — astrodynamics / flight-dynamics on top of mps-formula
      ├─ mps-jni      — optional Java JNI bindings (consumes the C ABI)
      ├─ mps-ffm      — optional Java 25 FFM metadata (consumes the C ABI)
      ├─ mps-web      — Dioxus 0.7 documentation site (SSR)
      ├─ mps-test     — integration test suite
      ├─ mps-build-common — shared cbindgen helper
      ├─ mps-bindgen-macro — #[java_struct]/#[java_enum] → Java codegen
      └─ xtask        — workspace automation (metrics, java codegen)
```

## Why f64

Rapier is compiled with `rapier3d-f64` (64-bit floats) rather than the default f32 build. This doubles memory and slows some ops but preserves precision for long-duration orbital, aerospace, and multi-body simulations where f32 drift is unacceptable.

## Repository Layout

```text
crates/
  mps-core/      physics world, bodies, colliders, queries, events, forces, voxel
  mps-formula/   28 pure physics/engineering formula modules
  mps-cosmos/    astrodynamics & flight dynamics
  mps-jni/       optional Java JNI bindings
  mps-ffm/       optional Java 25 FFM metadata
  mps-web/       Dioxus documentation site
  mps-test/      integration tests
  mps-build-common/  cbindgen helper shared by mps-core + mps-cosmos
  mps-bindgen-macro/ #[java_struct]/#[java_enum] → Java source generator
  xtask/         workspace automation

docs/            legacy docs — moved into crates/mps-web/
rapier/          vendored rapier3d-f64 fork (separate workspace, path dependency)
```

## Formula Library (mps-formula)

The formula crate provides **28 modules** with 300+ pure Rust functions spanning physics, aerospace, and engineering. It has **zero dependency on Rapier or `WorldHandle`** — pure input→output computation, which keeps it trivially reusable and unit-testable.

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

## Architecture: two layers

Every physics capability follows a strict two-layer split so the math stays pure and the engine stays encapsulated:

- **mps-formula** — pure computation. Takes values in, returns values out. No `WorldHandle`, no `RigidBody`, no Rapier state.
- **mps-core** — C ABI surface + Rapier interaction. Reads body state, calls a formula, applies the resulting force/torque back into the world.

All C ABI function names, parameters, error codes, and `_flag` variants are preserved for backward compatibility.

## Native API Surface

The C-compatible ABI lives in `crates/mps-core/src/rapier/ffi/`. Supported areas:

- World creation, stepping, gravity, integration parameters, body snapshots.
- Rigid body creation, insertion, pose/velocity mutation, forces, impulses, CCD, sleep/wakeup.
- Collider creation, insertion, runtime material/group/event settings.
- Air-drag and lift accumulation for surface samples, driven by Rapier rigid body motion.
- Ray, point, AABB, OBB, sphere, shape-cast, and voxel-shaped queries.
- Collision and contact-force event queues.
- Joints and character controller.
- Compact-tree and RTree spatial indexes.
- Extended collider builders: capsule, SSV, ellipsoid, prism, cylinder, shell, kDOP, FDH, neural bounds.
- Voxel collider construction from raw grids, AABB, and OBB.

## Voxel Colliders

Voxel colliders can be created from:

- Raw occupancy grids: native memory or a `byte[]` buffer.
- Axis-aligned bounding boxes: `collider_builder_create_voxel_aabb`.
- Oriented bounding boxes: `collider_builder_create_voxel_obb`.

Build modes are controlled by `VoxelColliderOptions`:

- `Auto`: choose from voxel count and dynamic/static body usage.
- `Cuboids`: one cuboid per solid voxel.
- `GreedyCuboids`: merge adjacent solid voxels into larger cuboids.
- `SurfaceMesh`: generate an exterior triangle mesh for large static voxel sets.

`VoxelBuildStats` can be used before building to inspect cell count, solid count, selected mode, estimated parts, estimated vertices/triangles, and generated grid size.

## Building & Testing

```powershell
cargo fmt --all --check        # formatting gate (CI)
cargo clippy --all-targets -- -D warnings   # lint gate (CI)
cargo test                     # full integration suite (mps-test)
cargo check --workspace        # full workspace type-check
cargo build --release          # build everything
```

CI runs on Ubuntu, Windows, and macOS (`macos-latest`) via `.github/workflows/ci.yml`.

## Documentation

Online documentation lives in `crates/mps-web/` — a Rust SSR site built with Dioxus 0.7 + dioxus-i18n (Fluent), published at `https://Polari-Stars-MC.github.io/rigid-body/`.

The `.github/workflows/pages.yml` workflow builds the site with `cargo build -p mps-web --release` (subscribing the Dioxus Router to the GitHub Pages base path via `DIOXUS_ASSET_ROOT`), launches the binary as a local SSR server, and exports each route to `_site/<path>/index.html` for GitHub Pages.

**Forks:** the base path is derived from `${{ steps.configure-pages.outputs.base_path }}`, which auto-adapts to the fork's repository name — no hard-coded `/rigid-body` path in either the Rust code or the workflow. To deploy a fork:

1. Enable Actions in the fork's **Settings → Actions → General**.
2. Set **Settings → Pages → Source** to *GitHub Actions*.
