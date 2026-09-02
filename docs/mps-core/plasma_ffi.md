# rapier/plasma_ffi.rs

## 作用
等离子体磁流体 C ABI — `mps_formula` 等离子体公式的薄封装（磁压比、回旋频率、拉莫尔半径、磁镜比与损失锥）。标量结果复用 `ffi_scalar`。不触碰 `WorldHandle`/Rapier 状态。Rust 模块名 `plasma_ffi`，符号前缀 `plasma_`。同样逐函数显式写出以规避 cbindgen 宏限制。

## 关键导出
- `plasma_beta` — 热压/磁压比。
- `plasma_gyrofrequency` — 带电粒子回旋频率。
- `plasma_larmor_radius` — 拉莫尔回旋半径。
- `plasma_mirror_ratio` — 磁镜收敛比。
- `plasma_mirror_loss_cone_angle` — 磁镜损失锥角。

## 依赖
- `mps_formula`（等离子体公式模块）— 纯函数。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/plasma_ffi.rs`、`plasma.rs`。
