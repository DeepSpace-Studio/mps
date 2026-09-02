# rapier/nucphys.rs

## 作用
核物理 C ABI — `mps_formula::nuclear` 纯标量公式的薄封装（放射性衰变、半经验质量公式、反应 Q 值、聚变/裂变能量、反应堆物理、衰减）。标量结果复用 `ffi_scalar`；常数型（`f64` 恒定值）的封装直接写常量。不触碰 `WorldHandle`/Rapier 状态。Rust 模块名 `nucphys`，符号前缀 `nuclear_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出（按主题分组）
- 衰变：`nuclear_decay_constant`、`nuclear_remaining_nuclei`、`nuclear_activity`、`nuclear_half_life`、`nuclear_mean_lifetime`、`nuclear_specific_activity`、`nuclear_half_value_layer`。
- 结合能/Q 值：`nuclear_bethe_weizsaecker_binding_energy`、`nuclear_binding_energy_per_nucleon`、`nuclear_reaction_q_value`、`nuclear_dt_fusion_energy`/`_q_value`、`nuclear_dd_fusion_branch1_energy`/`branch2_energy`、`nuclear_u235_fission_energy`。
- 反应堆/原子质量：`nuclear_four_factor_formula`、`nuclear_reaction_rate`、`nuclear_atomic_mass_approx`。

## 依赖
- `mps_formula::nuclear` — 纯公式。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/nucphys.rs`、`nuclear.rs`。
