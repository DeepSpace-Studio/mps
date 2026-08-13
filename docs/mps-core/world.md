# rapier/world.rs

## 作用
物理世界核心 `PhysicsWorld` 与全部 `world_*` C ABI 入口。封装 Rapier3d 的 `PhysicsPipeline`、`IslandManager`、`BroadPhaseBvh`、`NarrowPhase`、刚体/碰撞体集合、关节集合、CCD 求解器,内嵌 `FrameWorkBuffers`(每帧可复用的 scratch)以避免分步堆分配。通过 `world_step` 推进物理仿真,并维护力法注册表/共享竞技场/相对力映射等扩展状态。

## 关键导出
- `PhysicsWorld` — 主物理世界结构,持有 pipeline、积分参数、岛屿/广相/窄相/关节/CCD、hooks、事件处理器与力注册表。
- `FrameWorkBufferspub(crate)` — 每帧复用的 scratch(body_log、friction_work、pending_forces、arena_idx_map 等)。
- `extern "C"` 入口(~28 项):`world_create/destroy/step`、`world_set/get_integration_parameters`、`world_set/get_gravity(_out)`、`world_get_rigid_body/collider_set_size`、`world_dynamic/body_snapshot(_count)`、`world_update_body_poses/velocities`、`world_get_force_registry_(typed_)count`、`world_create/destroy_shared_arena`、`world_get_shared_arena_address/size`、`world_reset_shared_arena_events`、`world_set/get/remove_relative_force_*`、`world_set/get_relative_force_enabled`。
- `ForceLawHandleRaw`(type alias)。
- `PhysicsWorld::newprintln` (crate 内)构造函数。

## 依赖
- 外部 crate:`rapier3d`(prelude 物理类型)、`std::sync::Arc`、`dashmap::DashMap`(feature `relative-force`)、`smallvec`。
- 本 crate 子模块:`crate::rapier::error`(ffi_guard/set_error/常量)、`crate::rapier::ffi`(WorldHandle、Vec3、Quat、句柄打包辅助等)、`crate::rapier::forces`(BodyForceLog/ForceFacade/ForceRegistry)、`crate::rapier::events`(CallbackPhysicsHooks/PendingForce)、`crate::rapier::shared_arena`(SharedPhysicsArena)。
