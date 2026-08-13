# rapier/error.rs

## 作用
实现线程局部的"最后错误"槽位,并与 `mps-formula` 下层 crate 共享同一错误槽。错误槽实体位于 `mps_formula::error`(最底层),因此公式代码与 Rapier FFI 层写入的错误落在同一处,Java 侧通过 `last_error_*` ABI 统一读取。本模块重导出共享实现,新增导出访问器以及 `ffi_guard` panic 屏障(确保 Rust panic 不跨 FFI 边界 unwind 而 abort 宿主 JVM)。错误码以本地 `pub const` 形式重声明,便于 cbindgen 在生成的 C 头中输出为 `#define`(编译期断言将其钉死为 `mps_formula::error` 的规范值)。

## 关键导出
- `pub use mps_formula::error::{clear_error, set_error}` — 重导出清错/设错函数。
- `pub const ERR_OK / ERR_NULL_POINTER / ERR_INVALID_ARGUMENT / ERR_NOT_FOUND / ERR_CAPACITY / ERR_UNSUPPORTED / ERR_INTERNAL` — 错误码(0–6)。
- `pub fn ffi_guard<R>(default, f)` — 包裹 FFI 入口,panic 转 `ERR_INTERNAL` 并返回失败哨兵。
- `pub extern "C" fn last_error_code() -> u32` — 取当前线程最后错误码。
- `pub extern "C" fn last_error_message() -> *const c_char` — 取最后错误消息(借用于线程局部槽,不可释放/存储)。
- `pub extern "C" fn last_error_clear()` — 将错误槽复位为 `ERR_OK` / "ok"。
- `pub extern "C" fn error_code_name(code) -> *const c_char` — 错误码转可读名称(C ABI)。

## 依赖
- `mps_formula::error` — 实际错误槽实现及 `clear_error`/`set_error`/各 `ERR_*` 规范值(通过 `pub use` 与编译期 `assert!` 绑定)。
- `std::os::raw::c_char` — C 字符串指针类型。
- `std::panic::{AssertUnwindSafe, catch_unwind}` — `ffi_guard` 的 panic 捕获。
- 被几乎所有其他 `rapier` 模块依赖(`crate::rapier::error::{...}`),是 FFI 错误处理的公共基础设施。
