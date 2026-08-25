# rapier/bridge.rs

## 作用
实现 Rust 与 Java(JVM)之间的**零拷贝内存桥接**。文档头说明其消除了若干 JNI 瓶颈:用预分配的 `DirectByteBuffer`/指针传递取代逐次 `newDoubleArray` 与整数组拷贝,用 `GetDirectBufferAddress`、`GetPrimitiveArrayCritical`(pin 而非 copy)等标准 JNI API(Java 8+)完成 `memcpy` 级批量传输。兼容 Fabric / Forge / NeoForge 及任意 JVM 8+ 应用,不依赖 Minecraft 内部 API。所有函数经 `catch_unwind` 防止 panic 跨 FFI 边界。注意:本模块导出为 `pub unsafe fn` / `pub fn`,**不**是 `extern "C"`(由 Java 侧通过 JNI 调用,而非 C ABI)。

## 关键导出
- `pub unsafe fn direct_double_buffer_as_slice(address, capacity)` — 从 Java DirectByteBuffer 零拷贝得到 `&mut [f64]`。
- `pub unsafe fn direct_byte_buffer_as_slice(address, capacity)` — 零拷贝得到 `&[u8]`。
- `pub unsafe fn direct_byte_buffer_as_slice_mut(...)` — 可写 `&mut [u8]` 变体。
- `pub fn write_vec3_to_slot(slot, value)` — 将 `Vec3` 写入直接缓冲槽位。
- `pub fn write_quat_to_slot(slot, value)` — 将 `Quat` 写入直接缓冲槽位。
- `pub fn write_f64_slice(slot, values, capacity)` — 批量写 `f64` 切片到缓冲槽。
- `pub fn bulk_body_snapshot_to_direct_buffer(...)` — 批量将刚体快照写入直接缓冲(单帧复用)。
- `pub fn get_double_array_critical(...)` / `get_byte_array_critical(...)` — 经 `GetPrimitiveArrayCritical` 获取临界数组指针。
- `pub fn voxel_collider_from_direct_buffer(...)` — 从直接缓冲批量构造体素碰撞体。

## 依赖
- `std::panic::{AssertUnwindSafe, catch_unwind}` — panic 隔离屏障。
- `std::slice` — 原始指针到切片的转换。
- `crate::rapier::ffi::{Vec3, Quat}` — 与 Java 交换的向量/四元数布局类型。
- 依赖 JVM 标准 JNI 调用约定(仅用 `GetDirectBufferAddress`、`GetPrimitiveArrayCritical` 等),不引入额外 crate。
