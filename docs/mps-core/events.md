# rapier/events.rs

## 作用
碰撞/接触力事件的收集与分发,同时承载全部 `world_set/clear/get_*_law` 力法配置 FFI。文件头文档说明三条并行事件通道:`Mutex` 保护的 legacy Vec 队列、`EventRing<T>` 单生产者单消费者(SPSC)无锁环形缓冲(物理线程写、Java drain 线程读,Release/Acquire 游标同步)、以及 `CallbackSlot` 类型化函数指针回调。初始化类操作(环重建、回调注册、模式切换)通过 `init_guard()`/`step_active` 与 `world_step` 互斥,冲突返回 `ERR_UNSUPPORTED` 而非 UB。

## 关键导出
- `CollectingEventHandler`(crate 级)— 实现 Rapier `EventHandler`,收集碰撞与接触力事件,并保存自定义物理报告。
- `CallbackPhysicsHooks`(crate 级)— 实现 Rapier `PhysicsHooks`,承载接触对/相交对过滤回调。
- `EventRing<T>` / `CollisionEventRing` / `ContactForceEventRing`(crate 级)— SPSC 环形缓冲及其类型别名。
- `EventInitGuard<'a>` / `StepGuard<'a>`(crate 级)— 初始化期与步进期的互斥守卫。
- `CustomPhysicsState`(crate 级)— 各力法参数与自定义物理报告的集中状态。
- `PendingForce` — 待施加力记录(供 `world.rs` 的 scratch 缓冲使用)。
- `extern "C"` 入口(~62 项):力法配置 `world_set/clear/get_{coulomb_friction, air_drag, external_force, newton_gravity, solar_wind_pressure, dynamical_friction, mond_gravity, eddington_radiation_pressure, xray_irradiation, pulsar_magnetic_dipole, jeans_escape}_law(_flag)`;事件读取 `world_clear_events`、`world_collision/contact_force_event_count`、`world_get_collision/contact_force_event(s)`;环形缓冲 `world_init/drain/clear_*_event_ring(s)`、`*_ring_len/stats`;回调 `world_set/clear_contact_pair/intersection_pair_filter_callback`、`world_register_collision/contact_force_callback`、`world_unregister_callback`、`world_set_event_dispatch_mode`;`world_get_custom_physics_report`。

## 依赖
- 外部 crate:`parking_lot::{Mutex, RwLock}`、`rapier3d::geometry::{CollisionEvent, CollisionEventFlags, ContactPair, SolverFlags}`、`rapier3d::prelude::{ColliderSet, ContactForceEvent, EventHandler, PhysicsHooks, Real, RigidBodySet, Vector}`、`smallvec::SmallVec`、`std::cell::UnsafeCell`、`std::sync::atomic::*`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(各 `*Law` 描述体、`CollisionEventRecord`、`ContactForceEventRecord`、`EventDispatchMode`、`CustomPhysicsReport` 等)、`crate::rapier::forces::ForceLawType`。
