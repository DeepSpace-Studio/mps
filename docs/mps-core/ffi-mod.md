# rapier/ffi/mod.rs

## 作用
`ffi` 子模块的汇聚文件，是物理引擎对外 FFI 类型与转换函数的根。
本身只声明两个子模块 `convert` 和 `types`，并通过 `pub use convert::*`、`pub use types::*` 把它们全部再导出到上层命名空间。
所有 C-ABI 用得到的句柄类型、标量结构体、以及 FFI 与 Rapier 之间的转换逻辑都集中从这里暴露。

## 关键导出
- `pub mod convert;` — 重导出 FFI↔Rapier 转换函数子模块。
- `pub mod types;` — 重导出 FFI 数据类型（句柄/值类型）子模块。
- `pub use convert::*;` — 把 `convert` 全部公有符号导出到本层。
- `pub use types::*;` — 把 `types` 全部公有符号导出到本层。

（本文件自身不定义任何 `pub fn`/`pub struct`，真正的导出符号都在 `convert.rs` 与 `types.rs` 中。）

## 依赖
- 子模块 `crate::rapier::ffi::convert`、`crate::rapier::ffi::types`（同目录）。
- 间接依赖：`mps_formula::ffi::types`(被 types.rs 再导出)、`rapier3d`。
