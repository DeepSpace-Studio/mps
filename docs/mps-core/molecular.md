# rapier/molecular.rs

## 作用
分子级相互作用(静电力 / Lennard-Jones)的纯计算与施加 FFI 层。势/力计算委派给 `mps_formula::molecular`,本模块负责参数校验、错误回报、把计算出的成对力施加到指定刚体对(`molecular_apply_pair_forces(_flag)`、`molecular_pair_interaction`),并提供真空库仑常量访问入口。

## 关键导出
- `extern "C"` 入口(8 项):`molecular_lennard_jones_potential`、`molecular_lennard_jones_force`、`molecular_coulomb_potential`、`molecular_coulomb_force`、`molecular_pair_interaction`、`molecular_apply_pair_forces(_flag)`、`molecular_vacuum_coulomb_constant`。
- (无 pub struct/enum/trait/const,纯 FFI 文件)。

## 依赖
- 外部 crate:`mps_formula::molecular`(Lennard-Jones / Coulomb 实际函数)。
- 本 crate 子模块:`crate::rapier::error`(ERR_INVALID_ARGUMENT/ERR_NOT_FOUND/ERR_NULL_POINTER/clear_error/ffi_guard/set_error)、`crate::rapier::ffi`(`Bool`、`MolecularForceLaw`、`MolecularPairReport`、`MolecularParticle`、`RigidBodyHandleRaw`、`Vec3`、`WorldHandle`、unpack_rigid_body_handle、vec3_* 辅助)。
