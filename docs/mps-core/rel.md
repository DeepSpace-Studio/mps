# rapier/rel.rs

## 作用
相对论 C ABI — `mps_formula` 相对论公式的薄封装（Kerr/Schwarzschild 黑洞、引力波、相对论运动学、引力透镜、宇宙学、Hawking 辐射）。标量结果复用 `ffi_scalar`。不触碰 `WorldHandle`/Rapier 状态。Rust 模块名 `rel`，符号前缀 `relativity_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 黑洞：`relativity_kerr_horizon_radii`、`relativity_kerr_ergosphere_radius`、`relativity_kerr_frame_dragging_frequency`、`relativity_schwarzschild_isco`、`relativity_kerr_isco`、`relativity_schwarzschild_effective_potential`、`relativity_photon_sphere_radius`、`relativity_hawking_temperature`、`relativity_reissner_nordstrom_horizons`。
- 引力波：`relativity_gw_strain_amplitude`、`relativity_chirp_mass`、`relativity_gw_frequency_derivative`、`relativity_gw_inspiral_snr`、`relativity_gw_inspiral_time_to_coalescence`。
- 运动学：`relativity_relativistic_total_energy`、`relativity_relativistic_momentum`、`relativity_relativistic_energy_from_momentum`、`relativity_relativistic_aberration`、`relativity_relativistic_doppler_longitudinal`/`_transverse`/`_beaming_factor`。
- 引力效应/宇宙学：`relativity_gravitational_redshift`、`relativity_einstein_radius`、`relativity_cosmological_redshift`、`relativity_redshift_from_wavelengths`、`relativity_lense_thirring_angular_frequency`、`relativity_hubble_recession_velocity`/`_distance`、`relativity_flat_universe_lookback_time`。

## 依赖
- `mps_formula`（相对论公式模块）— 纯函数。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/rel.rs`、`relativity.rs`。
