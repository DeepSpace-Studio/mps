# rapier/ffi/convert.rs

## 作用
FFI 与 Rapier 之间的类型转换核心。把 C-ABI 侧的标量/句柄结构体（`Vec3`、`Quat`、`RigidBodyHandleRaw` 等）与 Rapier 原生类型（`Vector`、`Rotation`、`RigidBodyHandle`、`ColliderHandle`、`ImpulseJointHandle` 等）相互转换。
同时提供各类 `u32` 原始码到枚举（刚体状态、形状类型、关节类型、交互组、KDOP 预设等）的双向映射，以及形状描述 `ShapeDesc` 到 Rapier `SharedShape` 的构造与合法性校验。
句柄打包采用 `(generation<<32 | id) + 1` 方案，避免 0 作为有效句柄。

## 关键导出
- `vec3_to_rapier` / `vec3_from_rapier` — `Vec3` ↔ Rapier `Vector` 互转（pub）。
- `pack_rigid_body_handle` / `unpack_rigid_body_handle` — 刚体句柄打包/解包（pub）。
- `pack_collider_handle` / `unpack_collider_handle` — 碰撞体句柄打包/解包（pub）。
- `pack_impulse_joint_handle` / `unpack_impulse_joint_handle` — 冲量关节句柄打包/解包（pub）。
- `quat_to_rapier` / `quat_from_rapier` / `isometry_from_parts` — 四元数/位姿转换（pub(crate)）。
- `body_status_from_raw` / `body_status_to_rapier` / `body_status_to_raw` — 刚体状态枚举与 `u32`/Rapier 互转。
- `shape_from_desc` / `shape_desc_valid` — 由 `ShapeDesc` 构造 Rapier 形状并校验合法性。
- `force_law_type_from_u32` / `force_law_type_tag` — `ForceLawType` 与 `u32` 互转。
- `query_filter_from_desc` / `shape_cast_options_to_rapier` / `joint_axis_to_rapier` — 查询过滤器、形状投射选项、关节轴转换。
- `vec3_finite` / `quat_finite` / `finite_positive` — 数值有限性检查辅助。

## 依赖
- `super::types`（`Vec3`、`Quat`、`ShapeDesc`、`BodyStatus` 等 C-ABI 类型）。
- `rapier3d`（Vector/Rotation/Pose/SharedShape/RigidBodyHandle/ColliderHandle 等）。
- `crate::rapier::forces::ForceLawType`。
