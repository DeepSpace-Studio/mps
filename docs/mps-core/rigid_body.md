# rapier/rigid_body.rs

## 作用
刚体构造器与全部 `rigid_body_*` / `world_*_rigid_body` C ABI 入口。提供 `RigidBodyBuilder` 的创建/属性设置/构建/销毁,以及刚体在世界中的增删复制、姿态/速度/质量读取、力与力矩施加、瞬态冲量施加、CCD 开关、睡眠唤醒/状态查询等。所有函数经 `ffi_guard` 包装以防 panic 跨 FFI 边界。

## 关键导出
- `extern "C"` 入口(~66 项):
  - 构造器:`rigid_body_builder_create/build/destroy/destroy_raw`、`set_translation/rotation/pose/additional_mass(_properties)/linvel/angvel/gravity_scale/linear_damping/angular_damping/can_sleep/enabled_rotations/user_data`。
  - 世界增删:`world_insert/remove/copy_rigid_body`、`world_remove_rigid_body_flag`。
  - 状态/姿态:`rigid_body_get/set_status`、`get/set_translation(_out/_flag)`、`get/set_rotation(_out/_flag)`、`set/get_pose(_flag)`。
  - 物理:`get_mass/force/linvel(_out)/angvel(_out)`、`set_linvel/angvel(_flag)`、`add_force(_at_point/_at_local_point/_flag)`、`add_torque(_at_local_point/_flag)`、`reset_force/torque`、`apply_impulse(_flag)`、`apply_torque_impulse(_flag)`。
  - 其它:`enable_ccd(_flag)`、`sleep/wake_up(_flag)`、`is_sleeping(_flag)`。
- (无 pub struct/enum/trait;纯 FFI 函数文件)。

## 依赖
- 外部 crate:`rapier3d::dynamics::RigidBody`、`rapier3d::prelude::{MassProperties, RigidBodyBuilder, RigidBodyType}`。
- 本 crate 子模块:`crate::rapier::error`(ffi_guard/set_error/常量)、`crate::rapier::ffi`(BodyStatus/Bool/Quat/Vec3/WorldHandle/RigidBodyBuilderHandle/RigidBodyHandleRaw 及一系列 body_status/quat/vec3 转换与句柄打包辅助)。
