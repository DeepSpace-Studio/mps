# DESIGN.md — 设计规则索引

> **来源说明**：本文件的原始版本不在本仓库中，但 `crates/mps-test` 的守门测试按章节号引用它（`DESIGN.md §N`）。本文件是对这些引用的**还原**：只收录代码中被实际引用、且可由代码验证的规则。原始正文（总体架构叙述等）已不可考。新增引用时请同步维护本表；工程优化章节见 [OPTIMIZATION.md](OPTIMIZATION.md)。

### §3.2 — `ERR_*` 错误码双侧独立声明
`ERR_OK` / `ERR_NULL_POINTER` / `ERR_INVALID_ARGUMENT` / `ERR_NOT_FOUND` / `ERR_CAPACITY` / `ERR_UNSUPPORTED` / `ERR_INTERNAL` 在 `mps-formula::error` 与 `mps_core::rapier::error` **各自独立声明**，且数值必须一致。

理由：这是对 cbindgen 的刻意规避——cbindgen 不识别 `pub use`，常量必须在每个 crate 内字面声明才会进入生成的 `rigid_body.h`。

守门：`crates/mps-test/src/rapier/error_consistency.rs`（引用 DESIGN.md §3.2 + OPTIMIZATION.md §1）。任何一侧改动数值而另一侧未同步，该测试以精确的双侧定义位置报错——**禁止静默该测试**；若确需改变某个码值，选一侧为规范源、双侧同改。

### 模块镜像规则（无章节号）
每当 `mps-core::rapier::*`（或 cosmos）新增/删除/重命名一个子模块，`mps-test/src/rapier/<name>.rs`（或对应层级）必须同步保持 lockstep。不做解析、仅按目录清单比对。

守门：`crates/mps-test/src/rapier/verify_module_mirror.rs`（引用 DESIGN.md + EDITION.md 规则 + OPTIMIZATION.md §8）。防两类事故：删除源文件留下悬空 `mod` 声明（编译错误），以及新增模块没有测试文件（静默漏覆盖）。
