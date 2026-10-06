# 0003 — 错误码双侧字面声明

## 背景

宿主需要稳定的 `u32` 错误码，并且要写进 cbindgen 生成的头文件。cbindgen 不解析依赖 crate，也把 `pub use` 的常量丢掉。纯公式还需要一种不触碰 FFI 全局状态的失败方式。

## 决定

- `ERR_OK` … `ERR_INTERNAL`（0 到 6）在 `mps-formula::error` 与 `mps_core::rapier::error` 各写一份字面常量。core 用编译期 `assert!` 钉住两者相等。
- 线程局部错误槽只放在 `mps-formula::error`。core 的 `last_error_*` 是导出的访问器。
- 每个 `extern "C"` 入口用 `ffi_guard` 把 panic 收成 `ERR_INTERNAL` 和该函数的失败哨兵。
- 纯公式的数值域用 `FormulaError` 返回 `Result`，不写错误槽。FFI 包装再映射成 `ERR_INVALID_ARGUMENT` 或 `ERR_NULL_POINTER`。
- release / dev 的 `panic` 都是 `unwind`。

## 后果

改一个码值必须改两个文件，并让 `error_consistency` 通过。不能为了「少写一份」改成 `pub use`，否则头文件会丢宏。`panic = "abort"` 会让守卫失效。

## 相关代码

- `crates/mps-formula/src/error.rs`
- `crates/mps-formula/src/domain.rs`
- `crates/mps-core/src/rapier/error.rs`
- `crates/mps-test/src/rapier/error_consistency.rs`
- 根 `Cargo.toml` 的 `[profile.release] panic`
