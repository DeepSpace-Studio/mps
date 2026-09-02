# rapier/emag.rs

## 作用
电磁学 C ABI — `mps_formula` 电磁标量公式的薄封装（平面波、趋肤效应、天线/口径、Friis 传输、反射/VSWR、传输线、Rayleigh 散射、Faraday 旋转）。标量结果复用 `ffi_scalar`（null `out` 或 `None` → `Bool::FALSE`）。不触碰 `WorldHandle`/Rapier 状态——纯计算器。Rust 模块名 `emag`，符号前缀 `electromagnetism_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 平面波/介质：`electromagnetism_poynting_magnitude_plane_wave`、`electromagnetism_phase_velocity`、`electromagnetism_wavelength_in_medium`、`electromagnetism_intrinsic_impedance`、`electromagnetism_skin_depth`、`electromagnetism_vacuum_wavelength`、`electromagnetism_wave_frequency`。
- 天线/传播：`electromagnetism_dipole_radiation_resistance`、`electromagnetism_half_wave_dipole_directivity`、`electromagnetism_effective_aperture`、`electromagnetism_far_field_distance`、`electromagnetism_friis_power_received`。
- 反射/传输线：`electromagnetism_reflection_coefficient`、`electromagnetism_vswr`、`electromagnetism_return_loss`、`electromagnetism_quarter_wave_transformer`、`electromagnetism_coaxial_impedance`、`electromagnetism_coaxial_cutoff_frequency`。
- 散射/磁光：`electromagnetism_rayleigh_scattering_cross_section`、`electromagnetism_faraday_rotation`、`electromagnetism_transmission_line_input_impedance`。

## 依赖
- `mps_formula`（`disciplines::electromagnetism`）— 纯公式。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/emag.rs`、`electromagnetism.rs`。
