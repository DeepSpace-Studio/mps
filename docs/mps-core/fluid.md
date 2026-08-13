# rapier/fluid.rs

## 作用
流体力学的 FFI 层。把 `mps_formula::fluid` 的实际计算(基于流体体积的力估算、Navier-Stokes 简化步进、SPH 核 poly6/spiky/viscosity、伯努利压力)通过 C ABI 暴露,并对世界中的刚体施加由流体力得出的合力与合力矩(`fluid_apply_aabb_forces(_flag)`)。

## 关键导出
- `extern "C"` 入口(11 项):`fluid_estimate_aabb_forces`、`fluid_apply_aabb_forces(_flag)`、`fluid_navier_stokes_simplified_step`、`fluid_sph_poly6_kernel`、`fluid_sph_spiky_gradient`、`fluid_sph_viscosity_laplacian`、`fluid_sph_estimate_density`、`fluid_sph_estimate_forces`、`fluid_bernoulli_pressure`、`fluid_bernoulli_report`。
- (无 pub struct/enum/trait/const,纯 FFI 文件)。

## 依赖
- 外部 crate:`mps_formula::fluid`(实际流体计算函数,如 `compute_fluid_forces` 等)。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(`BernoulliReport`、`Bool`、`FluidForceReport`、`FluidVolume`、`NavierStokesReport`、`RigidBodyHandleRaw`、`SphForceReport`、`SphParticle`、`Vec3`、`WorldHandle`、`finite_positive`、`unpack_rigid_body_handle`、vec3_* 辅助)。
