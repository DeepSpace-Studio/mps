# rapier/mod.rs

## 作用
`rapier` 子模块入口声明。集中声明本 crate 内部全部物理子模块(子目录文件),并通过 `pub use mps_formula::*` 把底层公式 crate 的 27 个领域公式包(acoustics、astrophysics、celestial_data、chaos、cosmology、gravitational_models 等)平铺再导出到 `rapier::` 命名空间。`anvilkit` 在 feature `anvilkit-bridge` 下条件编译。

## 关键导出
- `pub mod {aerodynamics, batch, bounds, bridge, collider, compat, controller, crbtree, dop, error, events, ffi, fluid, forces, fracture, interaction, joints, molecular, neural, query, rigid_body, rtree, shared_arena, spaceflight, terrain_gravity, trajectory, voxel, world}` — 物理子模块声明。
- `pub mod anvilkit`(feature `anvilkit-bridge`)— AnvilKit 桥子模块。
- `pub use mps_formula::*`(27 项)— 将 `mps_formula` 各领域包(acoustics/astrophysics/biomechanics/celestial_data/chaos/continuum/control_theory/cosmology/electromagnetism/gravitational_models/galactic_dynamics/heliophysics/high_energy_astro/integrators/math/physchem/planetary_science/plasma/quantum/relativity/softbody/stellar/superfluidity/thermodynamics/topology/transmission/wave_optics)再导出为 `rapier::`。

## 依赖
- 外部 crate:`mps_formula`(领域公式库)。
- 本 crate 子模块:见上 `pub mod` 列表。
