# rapier/ffi/force_queue.rs

## 作用
共享内存力队列 —— Java↔Rust 零拷贝施力通道。Java 侧把 `(body_id, force[3], torque[3]?)` 写进 DirectByteBuffer 的槽位，Rust 在 `world_step` 内消费，绕过逐刚体 JNI 调用。内存布局：64 字节对齐的 `ForceQueueHeader`（capacity/head/tail/generation/stride/flags）+ 槽位位图（每槽 1 bit）+ 载荷区（`capacity × stride × 8` 字节；`stride = 6` 为纯力，`7` 为力+扭矩）。同步协议为单生产者/单消费者无锁设计：Java 是各槽载荷与位图位的唯一写者，Rust 是位图为 1 的槽的唯一读者；`head`/`tail` 用 release/acquire；位图位 `atomic_or` 置位 / `atomic_andnot` 清除（单写者语义免 CAS 环）；`generation` 在 `head` 回绕时递增解决 ABA；`flags` bit0 为可选的 paused 门（不保护 enqueue/cancel）。capacity 必须是 2 的幂（掩码取模）。

## 关键导出
- `pub struct ForceQueueHeader`（`#[repr(C, align(64))]` + `#[java_struct]`）— 队列头部；经 `mps-bindgen-macro` 标注生成 Java 侧结构（包 `org.polaris2023.mps.ffi`）。
- `pub unsafe fn bitmap(header*)` / `payload` / `payload_mut` — 由头指针定位位图/载荷区的内部辅助。
- `pub extern "C" fn rigid_body_consume_force_queue(world, header*) -> u32` — 消费整个队列：逐槽读取置位的 `(body_id, force[, torque])` 并施加到刚体，返回消费条数；调用点在 `world_step` 流水线内（`mps-jni` 亦直接暴露 `rigidBodyConsumeForceQueue`）。

## 依赖
- `core::sync::atomic`（`AtomicU64` + acquire/release 序）。
- `mps-bindgen-macro::{java_struct, ...}` — 惰性标注宏，标记参与 Java 绑定生成的 repr(C) 结构。
- `crate::rapier::ffi::convert::unpack_rigid_body_handle`、`crate::rapier::error`。

## 测试
`mps-test/src/rapier/force_queue.rs`（协议/布局）与 `force_queue_integration.rs`（端到端消费施力）。
