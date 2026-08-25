# rapier/compat.rs

## 作用
提供与早期 API 兼容的世界构造与查询入口:向 Rapier 世界插入由多个长方体(cuboid)组成的动态刚体、插入静态三角网格(trimesh)碰撞体,以及对刚体做 AABB 相交查询(先计数再批量取回)。这些入口把 Desc 描述结构(`AabbDesc`、`Quat`、`Vec3`、`InteractionGroupsDesc`、`QueryFilterDesc`)转换为 Rapier 原生类型,并对参数做有限性与容量校验。命名以 `world_*` / `query_*` 前缀区分,便于向后兼容旧调用方。

## 关键导出
- `pub extern "C" fn world_insert_dynamic_cuboids(...)` — 由一组长方体构造并插入动态刚体,返回 `RigidBodyHandleRaw`。
- `pub extern "C" fn world_insert_static_trimesh(...)` — 插入静态三角网格碰撞体。
- `pub extern "C" fn query_intersect_aabb_rigid_body_count(...)` — 统计与给定 AABB 相交的刚体数量。
- `pub extern "C" fn query_intersect_aabb_rigid_bodies(...)` — 批量取回与 AABB 相交的刚体 handle。
- 内部常量:`DYNAMIC_LINEAR_DAMPING`(0.4)、`DYNAMIC_ANGULAR_DAMPING`(0.18)、`MAX_DYNAMIC_CUBOIDS`(100_000)、`MAX_TRIMESH_VERTICES`、`MAX_TRIMESH_INDICES`;辅助 `valid_aabb`(非 pub)。

## 依赖
- `rapier3d::math::Vector` 与 `rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder}` — 刚体/碰撞体构造。
- `hashbrown::HashSet` — 相交查询结果去重。
- `crate::rapier::error` — 错误码与 `ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `AabbDesc`、`InteractionGroupsDesc`、`MAX_OUTPUT_CAPACITY`、`Quat`、`QueryFilterDesc`、`RigidBodyHandleRaw`、`Vec3`、`WorldHandle`,及 `interaction_groups_to_rapier`、`isometry_from_parts`、`pack_rigid_body_handle`、`quat_finite`、`query_filter_from_desc`、`vec3_finite`、`vec3_to_rapier`。
- `std::slice` — 跨 FFI 边界的可读切片构造。
