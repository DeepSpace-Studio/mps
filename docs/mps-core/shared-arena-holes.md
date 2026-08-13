# rapier/shared_arena/holes.rs

## 作用
`shared_arena` 子模块之一，负责把物理世界状态刷入竞技场中可变填充的"空洞"区——body 槽与 collider 槽，以及 body 句柄映射和按 `ForceLawType` 的力明细。
每个 body/collider 槽写入采用 per-slot 的 generation 计数器（先奇数=写中，后偶数=完成）保证 Java 端读到一致数据。`flush_all_bodies`/`flush_all_colliders` 在每帧 `world_step` 后调用，仅对"上次填过、本次缩水"的区间做 gen=0 回收（M3 尾清零优化，省去每帧清空所有空槽的带宽）。

## 关键导出
- `body_slot_ptr` — 返回第 index 个 body 槽的裸指针（pub）。
- `flush_body` — 把一个刚体的位置/速度/角速度/类型/睡眠/用户数据写入其槽（含 generation 协议，pub）。
- `clear_body_slot` — 把某 body 槽标记为空（gen=0，pub）。
- `flush_all_bodies` — 遍历刚体集写入所有活跃 body 槽、更新句柄映射与 body 计数，并按需回收（pub）。
- `write_body_handle` — 写入 arena 索引→Rapier `RigidBodyHandle` 的句柄映射（impl 内）。
- collider 槽相关方法（`collider_slot_ptr`/`flush_collider`/`clear_collider_slot`/`flush_all_colliders`）。
- 力明细（`force_law` 写入区）相关方法——按 `ForceLawType` 累计各力贡献。

## 依赖
- `std::sync::atomic::{AtomicU64, Ordering}`、`rapier3d::prelude::RigidBodyType`。
- `super::{BODY_SLOT_STRIDE, COLLIDER_SLOT_STRIDE, OFF_FORCE_LAW_COUNT}` 等常量/字段。
- `super::SharedPhysicsArena`（全部为其 `impl` 块）。
