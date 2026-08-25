# rapier/spaceflight/mod.rs

## 作用
空间飞行子模块的枢纽 (原 2610 行 spaceflight.rs 拆分的第 1 部分)。集中存放所有 per-domain 子模块共用的零开销数值辅助与常量，子模块用 `use super::*;` 即可拿到全部导入，无需各自重述 30+ 项 import 列表 (per OPTIMIZATION.md §3)。

## 关键导出
- `pub mod debris/dynamics/gnss/kepler/perturbation/propulsion/rotation/thermal` — 8 个按领域拆分的子模块声明。
- `pub use debris::*; ... pub use thermal::*;` — 把全部子模块符号 re-export 到 `spaceflight` 命名空间，保持 ABI 路径稳定 (无 ABI_VERSION bump)。
- `pub(crate) use rapier3d::prelude::Vector` — Rapier 向量类型，供子模块直接用。
- `pub(crate) use crate::rapier::error::{ERR_*, ffi_guard, set_error, clear_error}` — 错误辅助。
- `pub(crate) use crate::rapier::ffi::{Bool, Vec3, Quat, WorldHandle, RigidBodyHandleRaw, 各 FFI 结构体..., vec3_from_rapier, vec3_to_rapier, unpack_rigid_body_handle}` — 全部 FFI ABI 类型与转换辅助。
- `const EPS: f64 = 1e-12` / `const SIGMA = 5.670e-8` / `const SPEED_OF_LIGHT = 299_792_458.0` — 公用数值常量 (容差、Stefan-Boltzmann、光速)。
- `fn finite(values: &[f64]) -> bool` — 输入有限性检查 (每个 C ABI 入口必用)。
- `fn write_out<T: Copy>(out, value) -> Bool` — 安全写回输出指针 (null 时设 ERR_INVALID_ARGUMENT)。

## 依赖
- 本 crate 子模块: `crate::rapier::error`, `crate::rapier::ffi`。
- 外部 crate: `rapier3d` (Vector), `std::f64::consts` (PI/TAU)。
