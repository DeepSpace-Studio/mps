# rapier/shared_arena/ring.rs

## 作用
`shared_arena` 子模块之一，实现两个无锁 SPSC 环形队列：命令环（Java 写、Rust 读）与事件环（Rust 写、Java 读）。
命令环在 `world_step` 开头被排空，事件环在步进后写入碰撞/接触力事件。内存序是核心：命令排空用普通 `read_unaligned`（Java 已以 Release 发布），事件推送用 `Release`（写索引）/`Acquire`（读索引）。修改此文件后需用 Miri 复核 SPSC 测试。

## 关键导出
- `drain_commands` — 排空命令环，返回 `(cmd_type, body_index, arg0, arg1, arg2)` 元组列表，并复位写索引（pub）。
- `cmd_slot_ptr` / `event_slot_ptr` — 命令槽/事件槽裸指针计算（按容量取模环绕，impl 内）。
- `push_collision_event` — 推入碰撞事件（started/sensor/removed 等标记，pub）。
- `push_contact_force_event` — 推入接触力事件（含总力/最大力分量，pub）。
- `flush_events_from_handler` — 把 `crate::rapier::events::CollectingEventHandler` 收集到的事件桥接写入事件环（pub）。
- 事件环满时静默丢弃事件的环形边界检查逻辑。

## 依赖
- `std::sync::atomic::Ordering`，`super::{CMD_SLOT_STRIDE, EVENT_SLOT_STRIDE, OFF_CMD_WRITE}`。
- `super::SharedPhysicsArena`（全部为其 `impl` 块）。
- `crate::rapier::events::CollectingEventHandler`（事件桥接来源）。
