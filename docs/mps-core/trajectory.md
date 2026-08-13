# rapier/trajectory.rs

## 作用
实现弹道/轨迹的力估计与积分步进,并为 Rapier 世界中的刚体施加轨迹力。核心计算委托给 `mps_formula::trajectory`(下层公式 crate),本模块负责 C ABI 封装、参数校验与结果回填。除常规气动/重力轨迹外,还提供滑翔(glide)轨迹的力估计与积分步进。所有入口都包裹 `ffi_guard` 以防 panic 跨 FFI 边界。

## 关键导出
- `pub extern "C" fn trajectory_estimate_forces(...)` — 估计某轨迹状态所受气动/重力合力(`TrajectoryForceReport`)。
- `pub extern "C" fn trajectory_integrate_step(...)` — 推进一个积分步,输出下一状态与力报告。
- `pub extern "C" fn trajectory_apply_forces_to_body(...)` — 将轨迹力施加到世界中的刚体。
- `pub extern "C" fn trajectory_apply_forces_to_body_flag(...)` — 带开关标志的施加到刚体版本。
- `pub extern "C" fn trajectory_glide_estimate(...)` — 滑翔轨迹的力估计(`TrajectoryGlideReport`)。
- `pub extern "C" fn trajectory_glide_integrate_step(...)` — 滑翔轨迹的积分步进(`TrajectoryGlideState`)。

## 依赖
- `mps_formula::trajectory::{compute_forces, integrate_step}` — 实际力计算与积分的下层公式实现(核心逻辑所在)。
- `crate::rapier::error` — 错误码与 `ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `Bool`、`RigidBodyHandleRaw`、`TrajectoryEnvironment`、`TrajectoryForceReport`、`TrajectoryGlideEnvironment`、`TrajectoryGlideReport`、`TrajectoryGlideState`、`TrajectoryState`、`WorldHandle`,及 `unpack_rigid_body_handle`、`vec3_from_rapier`、`vec3_to_rapier`。
