// mps-test - extracted integration tests for mps-core physics engine
// Each module mirrors a rapier submodule from mps-core

pub mod cosmos {
    pub mod bodies;
    pub mod gravity;
    pub mod integrator;
    pub mod orbit;
    pub mod perturbation;
    pub mod world;
}

pub mod rapier {
    pub mod acoustics;
    pub mod aerodynamics;
    pub mod anvilkit;
    pub mod astrophysics;
    pub mod biomechanics;
    pub mod bounds;
    pub mod bridge;
    pub mod celestial_data;
    pub mod chaos;
    pub mod collider;
    pub mod continuum;
    pub mod control_theory;
    pub mod controller;
    pub mod crbtree;
    pub mod dop;
    pub mod electromagnetism;
    pub mod error;
    pub mod events;
    pub mod ffi;
    pub mod fluid;
    pub mod forces;
    pub mod fracture;
    pub mod gravitational_models;
    pub mod integrators;
    pub mod interaction;
    pub mod joints;
    pub mod material_mechanics;
    pub mod math;
    pub mod molecular;
    pub mod neural;
    pub mod nuclear;
    pub mod physchem;
    pub mod plasma;
    pub mod quantum;
    pub mod query;
    pub mod relativity;
    pub mod rigid_body;
    pub mod rtree;
    pub mod shared_arena;
    pub mod softbody;
    pub mod spaceflight;
    pub mod superfluidity;
    pub mod terrain_gravity;
    pub mod thermodynamics;
    pub mod topology;
    pub mod trajectory;
    pub mod transmission;
    pub mod voxel;
    pub mod wave_optics;
    pub mod world;

    // CI守门测试：跨 crate 模块镜像对齐 (OPTIMIZATION.md §8)。
    pub mod verify_module_mirror;
    // ABI 锁定测试：pin shared_arena constants (OPTIMIZATION.md §10)。
    pub mod arena_compat;
    // 跨 crate 错误码一致性 (OPTIMIZATION.md §1 可选加固)。
    pub mod error_consistency;
    // 跨 crate 版本锁定 (ARENA_VERSION ↔ ABI_VERSION, OPTIMIZATION.md §N6)。
    pub mod version_consistency;
    // mps-web metrics.rs ↔ source counts 同步 (OPTIMIZATION.md §N3)。
    pub mod verify_metrics_sync;
}
