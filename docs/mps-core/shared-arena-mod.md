# rapier/shared_arena/mod.rs

## 作用
共享内存物理竞技场（shared arena）的核心模块，用于消除 Java↔Rust 之间的逐次 JNI 调用开销。
通过一块连续内存（经 `DirectByteBuffer` 共享给 Java）承载物理世界状态：头部（版本/计数/flags/偏移）、body 槽、collider 槽、命令环形队列（Java→Rust）、事件环形队列（Rust→Java）。
每帧 `world_step`：Rust 先排空命令环 → 推进 Rapier 物理 → 把最新 body/collider 状态刷入槽 → 把碰撞/接触事件写入事件环；Java 纯内存读写读取状态，仅在提交时调用一次 JNI。
同步采用 per-slot 的 generation 计数器协议保证读写一致性。本文件定义 `SharedPhysicsArena` 结构体、分配/释放、常量与偏移，其余方法分散到 layout/header/holes/ring 子模块。

## 关键导出
- `SharedPhysicsArena` — 共享内存竞技场主结构体（含原始指针与各区域偏移、容量、原子索引）。
- `CommandType` — 命令类型枚举（AddForce/AddTorque/SetPose/... 等13种），经 `pub use layout::CommandType` 导出。
- `SharedPhysicsArena::new` — 按给定容量分配竞技场，返回竞技场与裸指针（用于传给 Java）。
- `SharedPhysicsArena::as_ptr` / `size` / `address` — 返回竞技场裸指针、字节大小、基地址。
- `ARENA_MAGIC` / `ARENA_VERSION` — 魔数 `"MPS_AREN"` 与布局版本号。
- `BODY_SLOT_STRIDE`/`COLLIDER_SLOT_STRIDE`/`CMD_SLOT_STRIDE`/`EVENT_SLOT_STRIDE` — 各槽/环步长（96/80/32/64 字节）。
- `HEADER_SIZE` / `INTEGRATION_PARAMS_SIZE` / `FORCE_SUMMARY_SIZE` — 头部/积分参数/力汇总区大小。
- `MAX_ARENA_BODIES`/`MAX_ARENA_COLLIDERS`/`MAX_ARENA_EVENTS`/`MAX_ARENA_COMMANDS`/`MAX_ARENA_TOTAL_BYTES` — 竞技场容量上限与256MiB总分配上限。
- `OFF_*` 常量（pub(super)）— `OFF_CMD_WRITE`(44)、`OFF_CMD_RING`(96)、`OFF_EVENT_RING`(104) 等头部偏移。
- `unsafe impl Send/Sync for SharedPhysicsArena` — 因 Java 并发访问而手动声明。

## 依赖
- 标准库：`std::alloc`(alloc_zeroed/dealloc)、`std::sync::atomic::AtomicU32`。
- 子模块：`crate::rapier::shared_arena::layout/header/holes/ring`（extend `impl` 块）。
- `rapier3d::prelude::RigidBodySet`/`RigidBodyType`（holes 模块 flush 用到）。
- `crate::rapier::events::CollectingEventHandler`（ring 模块事件桥接）。
