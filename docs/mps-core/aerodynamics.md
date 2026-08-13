# rapier/aerodynamics.rs

## 作用
实现刚体的气动/空气动力计算,通过一组气动面(AeroSurface)或体素网格(voxel grid)对 Rapier 世界中的刚体施加气动力与力矩。支持风场速度、空气密度参数,并将合力/合力矩汇总到 `AeroForceReport`。提供两组入口:直接按气动面列表施加,以及按体素网格(掩码)施加,二者各有一个 `_flag` 变体用于按条件开关。力/力矩的累加使用 `KahanVec3` 以减少浮点误差。

## 关键导出
- `pub extern "C" fn aero_apply_surfaces(...)` — 按气动面列表对刚体施加气动力,写入 `AeroForceReport`。
- `pub extern "C" fn aero_apply_voxel_grid(...)` — 按体素网格对刚体施加气动阻力/力。
- `pub extern "C" fn aero_apply_voxel_grid_flag(...)` — 带开关标志的体素网格气动施加。
- `pub extern "C" fn aero_apply_surfaces_flag(...)` — 带开关标志的气动面施加。
- `pub extern "C" fn aero_estimate_surface_force(...)` — 估计单个气动面产生的力(不写入世界)。
- 内部辅助:`voxel_index` / `voxel_solid` / `make_report`(非 pub,用于索引与报告构造)。

## 依赖
- `rapier3d::prelude::Vector` — 向量类型。
- `crate::rapier::error` — `ERR_NULL_POINTER`、`ERR_CAPACITY`、`ERR_INVALID_ARGUMENT` 等错误码,及 `ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `AeroForceReport`、`AeroSurface`、`Bool`、`MAX_OUTPUT_CAPACITY`、`RigidBodyHandleRaw`、`Vec3`、`WorldHandle`,以及 `unpack_rigid_body_handle`、`vec3_finite`、`vec3_from_rapier`、`vec3_to_rapier`。
- `crate::rapier::math::KahanVec3` — Kahan 求和累加器,降低合力/力矩浮点误差。
