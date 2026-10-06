# 0004 — 不透明句柄

## 背景

Java 和别的 FFI 宿主不能依赖 Rapier 的句柄类型、生命周期或 arena 布局。把 Rust 引用送过 JNI 没有稳定 ABI。

## 决定

- 世界和 builder 以不透明指针交接，由成对的 create / destroy 管理。
- 刚体、碰撞体、关节使用 packed `u64`（`RigidBodyHandleRaw`、`ColliderHandleRaw`、`ImpulseJointHandleRaw` 一类别名）。
- `#[repr(C)]` 结构只放 C 基本类型、定长数组和这些句柄。
- 大块零拷贝走共享 arena 或 force queue 的已发布布局，不走把 `Vec` 指针当成所有权交出去。

## 后果

代际失效是调用者的责任：对象销毁后旧 `u64` 不得再使用。JNI 层的 `to_jlong` 一类辅助只做整数加宽，不恢复类型安全。布局变更按 ABI 版本走，见 arena 守门。

## 相关代码

- `crates/mps-core/src/rapier/ffi/types.rs`
- `crates/mps-core/src/rapier/ffi/convert.rs`
- `crates/mps-core/src/rapier/shared_arena/`
- `crates/mps-jni/src/lib.rs`
