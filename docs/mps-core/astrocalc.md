# rapier/astrocalc.rs

## 作用
天体物理 C ABI — `mps_formula::astrophysics` 纯标量公式的薄封装（Roche/Hill 球、哈勃定律、NFW 暗物质晕、黑体辐射、Jeans 判据、恒星结构、双星、系外行星、星系）。标量结果复用 `ffi_scalar`；两个二元组结果（`roche_limit`、`habitable_zone_boundaries`）写两个 `f64` 输出。不触碰 `WorldHandle`/Rapier 状态——N 体 / `astro_*` FFI（Barnes-Hut、FMM、共振等）已在别处，不在此重复。Rust 模块名 `astrocalc`，符号前缀 `astrophysics_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 恒星/辐射：`astrophysics_mass_luminosity_relation`、`astrophysics_eddington_luminosity`(+`_solar`)、`astrophysics_blackbody_spectral_radiance`、`astrophysics_wien_displacement`、`astrophysics_main_sequence_lifetime`、`astrophysics_mass_radius_relation`、`astrophysics_chandrasekhar_mass_limit`(+`_kg`)、`astrophysics_ss73_disk_temperature`、`astrophysics_nickel56_decay_luminosity`。
- 引力势场：`astrophysics_hill_sphere_radius`、`astrophysics_roche_limit`、`astrophysics_nfw_density`/`_enclosed_mass`/`_circular_velocity`、`astrophysics_hubble_velocity`/`_distance`。
- 恒星形成/双星/系外行星：`astrophysics_jeans_mass`/`_length`、`astrophysics_lane_emden_first_zero`、`astrophysics_mass_function`、`astrophysics_binary_semi_major_axis`、`astrophysics_transit_depth`、`astrophysics_radial_velocity_semi_amplitude`、`astrophysics_habitable_zone_boundaries`。

## 依赖
- `mps_formula::astrophysics` — 纯公式。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/astrocalc.rs`、`astrophysics.rs`。
