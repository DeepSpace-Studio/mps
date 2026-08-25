# rapier/batch.rs

## 作用
批量碰撞体创建管线(Box3D 风格),目标是把 N 个形状登记进 `ColliderBatch` 后合并为单个 `Collider::compound` 一次性插入,以摊薄 arena 分配与广相重建开销。文件头文档描述合并策略:对相同 friction/restitution/density/collision_groups/solver_groups/sensor/body_parent 的请求分组,静态(parentless)形状塞进同一 `compound`,带父节点的回退为逐 collider 插入。三个 Box3D 预设(`default/sticky/bouncy`)给出常见参数组合。

## 关键导出
- `struct ColliderRequest` — 单个批量请求参数描述。
- `struct ColliderBatch` — 批量管理器本体;方法 `new/len/is_empty/execute/batch_add_colliders/merge_static_shapes`,内部维护 `BatchEntry` 分组表。
- `struct Box3DPreset` — 预设参数集;三个构造 `box3d_default()/box3d_sticky()/box3d_bouncy()` 与 FFI 入口 `box3d_preset_*`。
- `extern "C"` 入口(5 项):`world_batch_add_colliders`、`world_merge_static_shapes`、`box3d_preset_default`、`box3d_preset_sticky`、`box3d_preset_bouncy`。
- `const MAX_BATCH_REQUESTS`、`const MAX_COMPOUND_PARTS`(私有)。
- `impl crate::rapier::world::PhysicsWorld`(crate 扩展)— 在 `PhysicsWorld` 上加 `batch_add_colliders`/`merge_static_shapes` 方法。

## 依赖
- 外部 crate:`rapier3d::math::Pose`、`rapier3d::prelude::{Collider, ColliderBuilder, ColliderSet, ...}`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`、`crate::rapier::world::PhysicsWorld`(impl 直接操作 world 的刚体/碰撞体集合)。
