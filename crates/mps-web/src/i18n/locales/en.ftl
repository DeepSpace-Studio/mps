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

# ---- 404 page ----
not-found-title = Page Not Found
not-found-desc = The page you requested does not exist. Please return to the home page.
not-found-back = Back to Home

# ---- Footer ----
footer-text = MPS Motion Physics System v{ $version } — GitHub
