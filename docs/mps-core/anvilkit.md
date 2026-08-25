# rapier/anvilkit.rs

## 作用
anvilkit 桥接模块，把外部的 anvilkit ECS 物理世界与本项目 Rapier 物理世界对接。
通过 `AnvilKitAppState` 缓存 anvilkit 的 `App` 实例，以及实体→刚体句柄、实体→碰撞体句柄、约束→关节句柄的映射表，供 Java 侧通过 C-ABI 调用来创建/同步物理体。
本文件是 `#[cfg(feature = "anvilkit-bridge")]` 下暴露 FFI 的胶水层，把 anvilkit 组件、Rapier 构建器、材料属性等互相转换并写入对应世界。

## 关键导出
- `anvilkit_app_create` — 创建一个空的 anvilkit 物理 App 句柄。
- `anvilkit_app_destroy` — 释放 anvilkit App 句柄。
- `anvilkit_app_update` — 推进 anvilkit ECS 世界一帧。
- `anvilkit_app_spawn_body` — 在 anvilkit 世界生成一个刚体（仅本体，无碰撞体）。
- `anvilkit_app_spawn_body_with_collider` — 生成刚体并附带碰撞体。
- `anvilkit_app_set_transform` — 设置某实体的位姿（平移+旋转）。
- `anvilkit_app_set_material` — 设置实体碰撞体的材料属性。
- `anvilkit_app_sync_to_world` — 把 anvilkit 状态同步到 Rapier 世界。
- `anvilkit_app_entity_to_body` / `anvilkit_app_entity_to_collider` — 查询实体对应的刚体/碰撞体句柄。
- `anvilkit_app_create_constraint` / `anvilkit_app_constraint_to_joint` / `anvilkit_app_remove_constraint` — 约束与关节的创建、绑定、删除。
- `anvilkit_app_apply_aero_surfaces` / `apply_aero_voxel_grid` / `apply_fluid_aabb_forces` / `apply_trajectory_forces` — 施加气动、流体、轨迹等外力。
- `material_stress_strain_linear` / `material_elastic_collision_relative_speed` / `material_hertz_contact_force` — 材料应力应变与接触力计算（Hertz 接触）。

## 依赖
- 本 crate子模块：`crate::rapier::ffi`（句柄/类型/打包函数）、`crate::rapier::aerodynamics`、`crate::rapier::fluid`、`crate::rapier::trajectory`、`crate::rapier::error::ffi_guard`。
- 外部 crate：`anvilkit`（core::math、ecs）、`rapier3d`(RigidBodyBuilder/ColliderBuilder/RigidBodyType)、`dashmap`(DashMap)。
