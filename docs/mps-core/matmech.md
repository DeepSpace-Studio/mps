# rapier/matmech.rs

## 作用
材料力学 C ABI — `mps_formula::material_mechanics` 纯公式的薄封装（胡克定律、弹性模量、屈服判据、断裂力学、疲劳、蠕变、梁理论）。标量结果复用 `ffi_scalar`；多值结果（`principal_stresses`、`miners_damage`）显式写出。不触碰 `WorldHandle`/Rapier 状态。Rust 模块名 `matmech`，符号前缀 `material_mechanics_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 弹性：`material_mechanics_hookes_law_uniaxial`、`material_mechanics_stress_from_strain`、`material_mechanics_shear_modulus`、`material_mechanics_bulk_modulus`、`material_mechanics_lame_lambda`、`material_mechanics_principal_stresses`。
- 屈服：`material_mechanics_von_mises_stress`/`_yield_check`、`material_mechanics_tresca_shear_stress`/`_yield_check`。
- 断裂：`material_mechanics_ki_center_crack`、`material_mechanics_ki_edge_crack`、`material_mechanics_fracture_check`、`material_mechanics_critical_crack_length`。
- 疲劳/蠕变：`material_mechanics_basquin_stress_amplitude`/`_cycles_to_failure`、`material_mechanics_coffin_manson_strain_amplitude`、`material_mechanics_goodman_correction`、`material_mechanics_norton_creep_rate`、`material_mechanics_miners_damage`。
- 梁/柱：`material_mechanics_beam_bending_stress`、`material_mechanics_beam_deflection_center_point_load`、`material_mechanics_euler_buckling_load`、`material_mechanics_slenderness_ratio`。

## 依赖
- `mps_formula::material_mechanics` — 纯公式。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/matmech.rs`、`material_mechanics.rs`。
