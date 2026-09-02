# rapier/qphys.rs

## 作用
量子力学 C ABI — `mps_formula` 量子公式的薄封装（自由粒子/德布罗意、无限深势阱、氢原子、不确定性关系、费米黄金定则、康普顿散射、光电效应、Landau 能级、Rabi 振荡、Clebsch-Gordan 系数、简并微扰）。标量结果复用 `ffi_scalar`。不触碰 `WorldHandle`/Rapier 状态。Rust 模块名 `qphys`，符号前缀 `quantum_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 基础：`quantum_free_particle_energy`、`quantum_de_broglie_wavelength`、`quantum_infinite_well_energy`/`_wave_function`、`quantum_minimum_uncertainty_product`、`quantum_fine_structure_constant`、`quantum_angular_momentum_squared`、`quantum_time_evolution_phase`。
- 氢原子：`quantum_bohr_radius`、`quantum_hydrogen_energy_level`/`_orbital_radius`/`_transition_wavelength`。
- 散射/辐射：`quantum_compton_wavelength_shift`/`_scattered_wavelength`、`quantum_photoelectric_threshold`/`_max_kinetic`、`quantum_einstein_a_coefficient`。
- 高级：`quantum_fermi_golden_rule_linear`、`quantum_spin_orbit_energy`、`quantum_variational_hydrogen_energy`/`_optimal_alpha`、`quantum_coherent_state_photon_probability`、`quantum_spherical_harmonic_real`、`quantum_rabi_oscillation_probability`、`quantum_landau_level`、`quantum_clebsch_gordan_allowed`、`quantum_degenerate_perturbation_2x2`。

## 依赖
- `mps_formula`（量子公式模块）— 纯函数。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/qphys.rs`、`quantum.rs`。
