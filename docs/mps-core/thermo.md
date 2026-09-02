# rapier/thermo.rs

## 作用
热力学 C ABI — `mps_formula::thermodynamics` 纯标量公式的 `#[unsafe(no_mangle)]` 薄封装（理想气体状态方程、多方过程）。标量结果复用 [`crate::rapier::ffi::ffi_scalar`]：`out` 为 null 或公式返回 `None` → `Bool::FALSE`，否则写入并返回 `Bool::TRUE`。不触碰 `WorldHandle`/Rapier 状态——纯计算器。热传导/辐射/FEM 扩散的 C ABI 已在 `mps_formula` 的 thermal FFI 中，此处只封装气体状态公式。Rust 模块名 `thermo`，导出符号前缀 `thermodynamics_`。函数逐个显式写出（不用宏），因为 cbindgen 不展开声明宏，宏生成的 `pub extern "C" fn` 会被 `rigid_body.h` 静默遗漏。

## 关键导出
- `thermodynamics_ideal_gas_pressure` / `_volume` / `_temperature` — 理想气体状态方程三种未知量形式。
- `thermodynamics_polytropic_pressure` / `_work` — 多方过程压强与做功。

## 依赖
- `mps_formula::thermodynamics` — 纯公式。
- `crate::rapier::ffi::{Bool, ffi_scalar}`。

## 测试
`mps-test/src/rapier/thermo.rs`、`thermodynamics.rs`。
