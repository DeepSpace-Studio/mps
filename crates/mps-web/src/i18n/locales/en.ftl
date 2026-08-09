# MPS Motion Physics System — English translations
# Fluent .ftl format (https://projectfluent.org/)

# ---- Navigation ----
nav-home = Home
nav-quickstart = Quickstart
nav-architecture = Architecture
nav-gravity = Gravity
nav-integrators = Integrators
nav-formula = Formula
nav-voxel = Voxel
nav-events = Events
nav-arena = Arena
nav-cosmos = Cosmos
nav-jni = JNI
nav-ffm = FFM
nav-api = API

# ---- Language switcher ----
lang-zh = 中文
lang-en = English

# ---- Home page ----
home-hero-tag = / MPS PHYSICS OBSERVATORY
home-hero-title = Motion Physics System (Meters Per Second)
home-hero-desc = High-precision Rust physics engine based on { $rapier }. Full API exposed via C FFI ({ $ffi } functions) and Java JNI ({ $jni } methods). { $tests } tests, { $gravity } gravity models, { $integrators } symplectic integrators, zero-copy shared-memory Arena, { $modules } formula modules and { $bodies } celestial bodies.
home-cta-quickstart = Quickstart
home-cta-api = API Reference

home-stat-tests = Tests
home-stat-formula-fns = Formula Fns
home-stat-formula-modules = Formula Modules
home-stat-celestial = Celestial Bodies

home-section-directory = Module Directory
home-section-formula-modules = Formula Modules ({ $count })
home-section-key-features = Key Features
home-section-architecture = Architecture

home-mod-core-title = Core Engine
home-mod-core-desc = World, rigid bodies, colliders, joints, queries, controllers
home-mod-cosmos-title = Cosmos Rigid Body
home-mod-cosmos-desc = CosmosWorld, Verlet orbit integration, n-body gravity, perturbations
home-mod-physics-title = Physics Systems
home-mod-physics-desc = Gravity, terrain, force registry, events, aerodynamics, fluid
home-mod-formula-title = Domain Formulas
home-mod-formula-desc = 33 modules — spaceflight, astrophysics, nuclear, relativity, quantum, etc.
home-mod-integration-title = Integration
home-mod-integration-desc = Arena shared memory, JNI/FFM bindings, Java ecosystem
home-mod-reference-title = Reference
home-mod-reference-desc = Full API tables, precision & performance, optimization guide

home-feat-gravity-title = High-Precision Gravity
home-feat-gravity-desc = Spherical harmonics (EGM2008 8×8), ellipsoidal gravity, J2-J6 zonal harmonics, quadrupole tensor. Auto-selects optimal model by orbital altitude.
home-feat-integrators-title = Symplectic Integrators
home-feat-integrators-desc = Leapfrog, Yoshida 4th order, Forest-Ruth 8th order. Kahan compensation: 15→30 significant digits. Post-Newtonian 1PN+2PN corrections.
home-feat-celestial-title = Built-in Celestials
home-feat-celestial-desc = 10 solar system bodies with precision data (JPL DE441). Earth EGM2008, Moon LP165 + 12 Mascons (GRAIL), Mars Mars50c.
home-feat-terrain-title = Terrain Gravity
home-feat-terrain-desc = Polyhedral gravity (Werner-Scheeres), DEM terrain mass distribution, FFT acceleration. Lunar Mascon model prevents low-orbit decay.
home-feat-registry-title = ForceRegistry
home-feat-registry-desc = Typed force registry. Any force implementing ForceLaw trait auto-dispatches; world step auto-aggregates reports, no manual dispatch needed.
home-feat-jni-title = JNI + Shared Memory
home-feat-jni-desc = Java 21 JNI full binding ({ $count } methods). Shared-memory Arena (DirectByteBuffer) for zero-JNI read/write, only 1 world_step call per frame.

home-callout = All formulas live in a standalone crate { $crate } — pure Rust, no Rapier or WorldHandle dependency.

# ---- Formula mini-stat labels (home module grid) ----
formula-cat-spaceflight = Spaceflight
formula-cat-nuclear = Nuclear
formula-cat-mechanics = Mechanics
formula-cat-astrophysics = Astrophysics
formula-cat-relativity = Relativity
formula-cat-quantum = Quantum
formula-cat-electromagnetism = Electromagnetism
formula-cat-fluid = Fluid Dynamics

# ---- Quickstart page ----
quickstart-tag = / QUICKSTART
quickstart-title = Quickstart
quickstart-desc = Set up the MPS physics engine development environment from scratch.
quickstart-step1-title = Install Rust Toolchain
quickstart-step1-desc = Install Rust 1.75+ and cargo. Recommend using rustup.
quickstart-step2-title = Clone the Repository
quickstart-step2-desc = git clone and enter the rigid-body directory.
quickstart-step3-title = Build the Core Library
quickstart-step3-desc = cargo build --workspace compiles all crates.
quickstart-step4-title = Run Tests
quickstart-step4-desc = cargo test --workspace executes { $tests } integration tests.
quickstart-step5-title = Generate C Headers
quickstart-step5-desc = cargo build -p mps-core triggers cbindgen to generate rigid_body.h.

# ---- Architecture page ----
arch-tag = // MPS
arch-title = Architecture Overview
arch-desc = Crate layering and per-frame data flow from Java top to Rapier bottom.
arch-stack-title = Crate stack
arch-stack-lead = Top-to-bottom: Java bindings → C ABI → mps-core → Rapier3D-f64.
arch-stack-diagram = Java 21 JNI / Java 25 FFM
  └─ Rust C ABI (483 functions)
       ├─ mps-formula  — 33 pure formula modules (300+ functions)
       ├─ mps-core     — physics engine + Rapier wrapper (World, bodies, colliders, queries, events)
       ├─ mps-cosmos   — cosmos rigid body (separate world, Verlet orbit integration)
       ├─ mps-jni      — JNI bindings (311 methods, incl. cosmos batch)
       ├─ mps-ffm      — FFM metadata
       └─ mps-test     — integration tests (incl. cosmos 19)
arch-layers-title = Layer responsibilities
arch-layer-formula-title = mps-formula
arch-layer-formula-desc = { $modules } pure formula modules (spaceflight / astrophysics / nuclear etc.). No WorldHandle dependency; callable from anywhere.
arch-layer-core-title = mps-core
arch-layer-core-desc = Rapier3D-f64 wrapper + shared Arena + C-ABI. World, bodies, colliders, joints, queries, force registry, event system.
arch-layer-cosmos-title = mps-cosmos
arch-layer-cosmos-desc = CosmosWorld: separate physics domain. Verlet orbit integration + n-body mutual gravity. Does not share World with mps-core.
arch-layer-jni-title = mps-jni
arch-layer-jni-desc = Java 21 JNI full bindings, { $methods } methods (including cosmos batch API).
arch-layer-test-title = mps-test
arch-layer-test-desc = { $tests } integration tests, also serves as the module-mirror crate verifying source structure.
arch-layer-ffm-title = mps-ffm
arch-layer-ffm-desc = Java 25 Foreign Function & Memory API metadata; gradually replacing JNI.
arch-flow-title = Per-frame data flow
arch-flow-lead = Java frame-renderer side, 5-step MPS loop:
arch-flow-step-1 = Java reads Arena data in the DirectByteBuffer, updates body target positions / forces.
arch-flow-step-2 = Exactly one world_step(world, dt) FFI call — the only sync point across the boundary.
arch-flow-step-3 = mps-core dispatches internally: ForceRegistry aggregates forces → Rapier collision solver → joint solver → event dispatch.
arch-flow-step-4 = Arena layout: position / velocity / force-accumulator SoA writes go directly back into the DirectByteBuffer.
arch-flow-step-5 = Java side reads Arena zero-copy, renders / extracts physics. Events drained from ring buffer.
arch-tenets-title = Design tenets
arch-tenet-zero-copy = Zero-copy: Java and Rust swap body state via shared memory Arena; no per-object JNI read/write.
arch-tenet-formula-pure = Pure formula layer: mps-formula doesn't depend on WorldHandle or Rapier; pure-function callable in any context.
arch-tenet-ffi-stable = Stable C ABI: cbindgen generates rigid_body.h; every breaking change must bump the banner version.
arch-build-title = Build pipeline
arch-build-cbindgen = mps-core's build.rs triggers cbindgen at build time, emitting pub C-ABI types/functions into rigid_body.h.
arch-build-xtask = xtask auto-counts TEST_COUNT / JNI_METHOD_COUNT / CORE_FFI_COUNT and refreshes mps-web/src/metrics.rs.

# ---- Gravity page ----
grav-tag = // MPS
grav-title = Gravity Models
grav-desc = Complete gravity model catalogue, from point-mass to lunar Mascon.
grav-models-title = Model catalogue
grav-models-lead = 5 model families, auto-selected by orbital altitude and precision need.
grav-col-name = Name
grav-col-use = Use case
grav-col-cost = Cost
grav-row-newton = Deep-space cruise; two-body approximation
grav-row-sh = Low-orbit high precision; EGM2008 8×8 model
grav-row-ellipsoid = Earth oblateness; fast ellipsoid gravity
grav-row-zonal = J2 primary up to J6 sectoral / zonal
grav-row-quad = Arbitrary quadrupole tensor integration
grav-row-poly = Asteroid / irregular-body surface (Werner–Scheeres)
grav-row-mascon = Low lunar orbit (GRAIL 12 mass anomalies)
grav-bodies-title = Built-in bodies
grav-body-earth-title = Earth EGM2008
grav-body-earth-desc = 8×8 spherical-harmonic truncation; Jason / CHAMP / GRACE orbit simulation grade.
grav-body-moon-title = Moon LP165 + Mascon
grav-body-moon-desc = LP165 spherical harmonics + 12 GRAIL Mascon mass anomalies; prevents low-orbit simulation crashing.
grav-body-mars-title = Mars Mars50c
grav-body-mars-desc = 50th-order harmonic truncation, including polar oblateness and seasonal CO₂ cap approximation.
grav-body-sun-title = Sun point source
grav-body-sun-desc = JPL DE441 sun GM = 1.32712440018e20 m³/s²; origin of all planetary perturbations.
grav-auto-title = Auto model selection
grav-auto-lead = Engine switches models adaptively based on current orbital altitude and reference radius.
grav-auto-note = Low orbit (< 200 km) uses spherical harmonics + Mascon; medium orbit J2-J6; high / deep-space falls back to point-mass.
grav-api-title = C ABI
grav-api-desc = Gravity-related functions are exposed via the mps-core C-ABI:
grav-bodies-grid-title = Body catalogue

# ---- Integrators page ----
int-tag = // MPS
int-title = Symplectic Integrators
int-desc = Leapfrog, Yoshida 4, Forest–Ruth 8th-order symplectic integrators.
int-why-title = Why symplectic
int-why-lead = Classical RK4 drifts energy for long-duration orbital evolution.
int-why-body = Symplectic integrators preserve the Hamiltonian structure; the energy error is bounded and oscillates periodically, suitable for 10⁴-year orbit simulation.
int-catalog-title = Integrator catalogue
int-col-name = Integrator
int-col-order = Order
int-col-notes = Notes
int-row-leapfrog = Kick-Drift-Kick; 2nd order time-reversible; typical orbit workhorse.
int-row-yoshida4 = 4th order split symmetric (3 substeps); energy error ~O(dt⁴).
int-row-forest-ruth = 8th order Forest–Ruth class; deep-space high-precision planetary approximation.
int-kahan-title = Kahan error compensation
int-kahan-lead = Every symplectic integrator has a _kahan variant; Kahan algorithm compensates f64 truncation.
int-kahan-li-1 = Standard version stable to 15 significant digits.
int-kahan-li-2 = Kahan variant improves to 30 significant digits.
int-kahan-li-3 = Only when long-double-equivalent precision is required — ~2× slower.
int-kahan-note = Kahan maintains a compensation term c = (sum - t) - y for every summation step.
int-pn-title = Post-Newtonian corrections
int-pn-lead = Near a large GM body, relativistic effects are observable.
int-pn-li-1pn = post_newtonian_1pn — first-order post-Newtonian (Mercury perihelion).
int-pn-li-2pn = post_newtonian_2pn — second-order post-Newtonian (Lense–Thirring on high-eccentricity orbit).
int-pn-li-full = post_newtonian_full — combined PN terms.
int-adaptive-title = Adaptive step control
int-adaptive-desc = adaptive_step_size + step_accepted controls integrator step:
int-diag-title = Numerical diagnostics
int-diag-energy = Specific energy ε = v²/2 - GM/r — must stay conserved.
int-diag-am = Specific angular momentum h = r × v — strictly preserved by symplectic methods.
int-diag-kepler = keplerian_elements() converts to six elements to monitor semi-major-axis drift.

# ---- Formula page ----
form-tag = // MPS
form-title = Formula Modules
form-desc = 33 pure-Rust formula modules spanning spaceflight to quantum physics.
form-intro-pure = All formulas live in the standalone crate mps-formula — pure Rust, no Rapier or WorldHandle dependency.
form-mod-kepler = kepler.rs — Kepler equation iterative solver / element conversion
form-mod-dynamics = dynamics.rs — orbit dynamics / two-centre approximation
form-mod-perturbation = perturbation.rs — third-body perturbation / drag / solar pressure
form-mod-propulsion = propulsion.rs — Tsiolkovsky / propellant budget
form-mod-rotation = rotation.rs — rigid-body attitude dynamics / quaternions
form-mod-thermal = thermal.rs — thermal balance / solar irradiance
form-mod-debris = debris.rs — debris cloud evolution / collision probability
form-mod-gnss = gnss.rs — GNSS pseudorange / multi-frequency
form-mod-trajectory = trajectory.rs — Lambert problem / transfer orbit
form-mod-astrophysics = astrophysics.rs — stellar structure / luminosity function
form-mod-stellar = stellar.rs — main sequence / H-R diagram / evolution
form-mod-galactic = galactic_dynamics.rs — rotation curves / density-wave theory
form-mod-cosmology = cosmology.rs — FLRW metric / Hubble flow
form-mod-helio = heliophysics.rs — solar wind / corona parameters
form-mod-high-energy = high_energy_astro.rs — blackbody / synchrotron / inverse Compton
form-mod-celestial = celestial_data.rs — JPL DE441 10-body precision parameters
form-mod-planetary = planetary_science.rs — planetary interiors / tides
form-mod-mechanics = material_mechanics.rs — stress / strain / yield
form-mod-material = material_mechanics (subset) — elasticity tensor / anisotropy
form-mod-biomech = biomechanics.rs — joint torque / muscle models
form-mod-control = control_theory.rs — PID / LQR / state-space
form-mod-chaos = chaos.rs — Lorenz / Rössler attractors
form-mod-topology = topology.rs — homotopy / topological invariants
form-mod-softbody = softbody.rs — soft body / spring-mass
form-mod-relativity = relativity.rs — Lorentz transform / time dilation
form-mod-transmission = transmission.rs — signal transmission / link budget
form-mod-quantum = quantum.rs — Schrödinger equation / Hamiltonian / spin
form-mod-em = electromagnetism.rs — Maxwell equations / Poynting vector
form-mod-nuclear = nuclear.rs — decay / half-life / cross section
form-mod-fluid = fluid.rs — Navier–Stokes / Euler / Bernoulli
form-mod-plasma = plasma.rs — plasma frequency / MHD
form-mod-superfluidity = superfluidity.rs — superfluid helium / two-fluid model
form-mod-continuum = continuum.rs — continuum mechanics
form-mod-physchem-title = Physics + Chemistry
form-mod-physchem = physchem.rs — reaction kinetics / equilibrium constant
form-mod-thermo = thermodynamics.rs — entropy / enthalpy / engine efficiency
form-mod-molecular = molecular.rs — ideal gas / molecular dynamics
form-mod-wave-optics = wave_optics.rs — interference / diffraction / polarization
form-mod-acoustics = acoustics.rs — acoustics / Doppler
form-mod-aero = aerodynamics.rs — lift / drag / airfoils
form-support-title = Supporting modules
form-support-intro = A few shared primitives inside mps-formula that all 33 modules reuse:
form-call-title = Calling from Java
form-call-desc = All formula functions are exposed via C ABI with no WorldHandle dependency:

# ---- Voxel page ----
vox-tag = // MPS
vox-title = Voxel System
vox-desc = Dense voxel grid → collider build → polyhedron-gravity bridge.
vox-overview-title = Overview
vox-overview-lead = VoxelGrid + build_voxel_collider is the end-to-end pipeline from voxel map to simulation.
vox-overview-body = Useful for lunar-lander surface representation, irregular terrain elevation maps, and proximity fly-by simulations needing fast near-field collision.
vox-grid-title = VoxelGrid data model
vox-grid-desc = rapier::voxel::VoxelGrid<'a> borrows cells / dims / origin / scale:
vox-grid-li-1 = cells: 8- or 16-bit occupancy (0 = empty, non-0 = solid)
vox-grid-li-2 = dims + origin + scale: world-frame dimensions / origin / metres-per-voxel
vox-grid-li-3 = Borrow semantics: grid doesn't own cells; can come directly from mmap or Java ByteBuffer
vox-build-title = build_voxel_collider
vox-build-lead = Entry function converts the grid into a Rapier collider and inserts it into World.
vox-build-note = Internally MarchingCubes + convex decomposition: roughly every 1 m³ → 1 convex hull; complexity O(N log N).
vox-terrain-title = Terrain-gravity bridge
vox-terrain-desc = Voxel grids can be fed directly into rapier::terrain_gravity polyhedron gravity:
vox-terrain-li-direct = terrain_gravity_direct — brute O(N²) vertex sum; baseline / validation.
vox-terrain-li-fft = terrain_gravity_fft — frequency-domain convolution O(N log N); preferred for large grids.
vox-terrain-li-poly = polyhedron_gravity — Werner–Scheeres raw API.
vox-cases-title = Typical use cases
vox-case-lunar-title = Lunar surface landing
vox-case-lunar-desc = 100 m lunar grid + 12 Mascon blocks; simulate lander gravity-gradient disturbance on descent.
vox-case-terrain-title = Terrain region mapping
vox-case-terrain-desc = DEM elevation → VoxelGrid → simulation; pairs well with land-flight trajectory.
vox-case-proximity-title = Near-field collision performance
vox-case-proximity-desc = Because convex decomposition yields a bounded collider count, n-body mutual collision is O(N·log M).

# ---- Events page ----
evt-tag = // MPS
evt-title = Event System
evt-desc = Collision + contact-force events, three dispatch modes, C callback ABI.
evt-types-title = Event types
evt-types-lead = Two built-in physics event types, both records in mps-formula/ffi/types/core.rs.
evt-col-type = Type
evt-col-fields = Fields
evt-row-collision = started, collider1, collider2, sensor, removed
evt-row-contact = collider1, collider2, total_force, max_force_direction, max_force_magnitude
evt-modes-title = Dispatch modes
evt-mode-poll-title = Poll
evt-mode-poll-desc = Events pushed to a background Vec; application drains on demand.
evt-mode-callback-title = Callback
evt-mode-callback-desc = Direct unsafe extern "C" callback dispatch; real time but the callback must avoid heavy Rust callbacks.
evt-mode-both-title = Both
evt-mode-both-desc = Both pushed to queue and dispatched to callback; used for tests and hot replay.
evt-ring-title = Ring buffer
evt-ring-desc = High-frequency path uses SPSC EventRing<T>, lockless single-producer / single-consumer.
evt-ring-li-1 = MAX_EVENT_RECORDS = 16 384 — single ring cap.
evt-ring-li-2 = Producer: Rapier callback thread inside world_step.
evt-ring-li-3 = Consumer: Java render frame thread; drain() returns the latest N.
evt-forces-title = ForceLaw dispatch
evt-forces-lead = rapier::forces::ForceRegistry typed registration; every step aggregates a ForceReport.
evt-force-coulomb = CoulombFrictionLaw — static / dynamic friction + velocity threshold.
evt-force-airdrag = AirDragLaw — Reynolds-adaptive Stokes / Newton regime switching.
evt-force-external = ExternalForceLaw — buoyancy + electromagnetic + spring + gravity composite.
evt-force-newton = NewtonGravityLaw — body-body N-body pairwise attraction; scalable G.
evt-force-custom = Custom ForceLaw trait — any force struct scheduled automatically after register().
evt-forces-note = ForceReport contains per-body total force / torque / type labels; can be overlaid on a debug UI.
evt-abi-title = C callback ABI
evt-abi-desc = Collision / contact-force callback signatures are exposed as unsafe extern "C":

# ---- Arena page ----
arena-tag = // MPS
arena-title = Shared Memory Arena
arena-desc = DirectByteBuffer — zero-copy bridge between Java and Rust rigid-body state.
arena-why-title = Why an Arena
arena-why-lead = Classic JNI per-object get/setField costs 10 ms+ per frame at 1000 bodies.
arena-why-body = The shared Arena places rigid-body SoA data in one direct byte buffer; Java reads / writes without entering the Rust boundary. Only one world_step FFI call per frame.
arena-layout-title = Memory layout
arena-layout-desc = rapier::shared_arena header + SoA slot array + recycle ring.
arena-mods-title = Sub-modules
arena-mod-header = header.rs — magic / version / capacity validation.
arena-mod-layout = layout.rs — BodySlot field offset constants.
arena-mod-ring = ring.rs — SPSC event ring reuse.
arena-mod-holes = holes.rs — O(1) recycling of freed slots.
arena-flow-title = Per-frame protocol
arena-flow-step-1 = Java side arena_write_* sets this frame's target positions / forces / torques.
arena-flow-step-2 = world_step(world, dt) single FFI triggers simulation.
arena-flow-step-3 = arena_read_* reads updated positions / velocities directly from DirectByteBuffer.
arena-flow-note = No GetFieldID / CallObjectMethod at any point; JNI reference table stays empty, no GC risk.
arena-java-title = Java side example
arena-java-desc = DirectByteBuffer + JNI is the standard Java 21 idiom:

# ---- Cosmos page ----
cosmos-tag = // MPS
cosmos-title = Cosmos Rigid Body
cosmos-desc = CosmosWorld — separate orbital-scale physics domain.
cosmos-what-title = What Cosmos is
cosmos-what-lead = mps-cosmos provides an orbital physics domain that does not share a World.
cosmos-what-body = Unlike mps-core's Rapier, CosmosWorld uses a Verlet symplectic integrator and n-body pairwise mutual gravity, suited to long-duration orbital evolution without need for rigid-body contact solving.
cosmos-mods-title = Sub-modules
cosmos-mod-kepler-title = kepler.rs
cosmos-mod-kepler-desc = Kepler equation iteration / six-element ↔ Cartesian conversion.
cosmos-mod-dynamics-title = dynamics.rs
cosmos-mod-dynamics-desc = Two-centre approximation / third-body perturbation / drag.
cosmos-mod-perturbation-title = perturbation.rs
cosmos-mod-perturbation-desc = Tidal / Yarkovsky / Poynting–Robertson.
cosmos-mod-propulsion-title = propulsion.rs
cosmos-mod-propulsion-desc = Tsiolkovsky equation / specific impulse / propellant budget.
cosmos-mod-rotation-title = rotation.rs
cosmos-mod-rotation-desc = Quaternion attitude dynamics / spin stabilization.
cosmos-mod-thermal-title = thermal.rs
cosmos-mod-thermal-desc = Thermal balance / solar irradiance / eclipse cycle.
cosmos-mod-debris-title = debris.rs
cosmos-mod-debris-desc = Debris cloud evolution / Kessler syndrome model.
cosmos-mod-gnss-title = gnss.rs
cosmos-mod-gnss-desc = L1 / L5 pseudorange / multi-frequency ionosphere-free solution.
cosmos-nbody-title = n-body mutual gravity
cosmos-nbody-lead = Every CosmosWorld frame computes GM · m / r² pairwise across all bodies, O(N²).
cosmos-nbody-note = For N ≤ 20 the direct pairwise is faster than a BH tree; this is exactly the scale of a solar-system-level simulation.
cosmos-bodies-title = Body catalogue ({ $count })
cosmos-bodies-desc = JPL DE441 provides the GM, orbital elements, and phase of 10 primary bodies; injected into CosmosWorld at init.
cosmos-jni-title = JNI integration
cosmos-jni-desc = mps-jni exposes cosmos_batch_* methods: submit many spacecraft states at once rather than per-body; tracking-queue friendly.

# ---- 404 page ----
not-found-title = Page Not Found
not-found-desc = The page you requested does not exist. Please return to the home page.
not-found-back = Back to Home

# ---- Footer ----
footer-text = MPS Motion Physics System v{ $version } — GitHub
