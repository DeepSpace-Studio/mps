# rapier/acoustics_ffi.rs

## 作用
声学 C ABI — `mps_formula::acoustics` 纯标量公式的薄封装（扩散损失、吸收、混响 RT60、阻抗、透射/质量定律、Helmholtz 共振、Doppler、屏障衰减、声呐品质因数）。标量结果复用 `ffi_scalar`（null `out` 或 `None` → `Bool::FALSE`）；`doppler_shift` 额外接受一个 `Bool`（C `uint8_t`）的 `approach` 标志。不触碰 `WorldHandle`/Rapier 状态。波动/模态/结构类的 `acoustic_*` FFI 已在 `mps_formula::acoustics` 内，不在此重复封装。Rust 模块名 `acoustics_ffi`，符号前缀 `acoustics_`。同 thermo：函数显式写出以规避 cbindgen 不展开宏的问题。

## 关键导出
- 扩散/吸收：`acoustics_spherical_spreading_loss`、`acoustics_cylindrical_spreading_loss`、`acoustics_thorp_absorption`。
- 混响：`acoustics_sabine_rt60`、`acoustics_eyring_rt60`。
- 阻抗/透射：`acoustics_acoustic_impedance`、`acoustics_transmission_coefficient`、`acoustics_mass_law_tl`、`acoustics_helmholtz_resonance_frequency`。
- 传播效应：`acoustics_doppler_shift`、`acoustics_maekawa_barrier_attenuation`、`acoustics_active_sonar_echo_level`。

## 依赖
- `mps_formula::disciplines::electromagnetism`（声学公式当前放在该模块）— 12 个纯函数。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/acoustics_ffi.rs`、`acoustics.rs`。
