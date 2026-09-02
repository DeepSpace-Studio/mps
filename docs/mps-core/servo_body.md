# rapier/servo_body.rs

## 作用
PD/PID 伺服体 —— **第六种体类型**：一个动态刚体由 `PdController`/`PidController` 驱动到目标位姿（位置 + 旋转）和/或目标速度。每次 `servo_body_update` 计算当前状态与目标的速度级修正并经 `set_linvel`/`set_angvel` 写回——刚体是被求解器"驱动"到目标，而非运动学吸附，因此仍能与其它体正确碰撞。存在 `world.servo_bodies` 哈希表（稳定 id）。

## 关键导出
- `servo_body_create(world, shape, translation, kp, kd, ki, axes)` — 建刚体 + 伺服控制器（`axes` 按位选轴）。
- `servo_body_set_target_position` / `_set_target_rotation`（四元数）/ `_set_target_velocity` / `_set_target_angular_velocity` — 目标设定。
- `servo_body_update(world, id, dt)` — 每步伺服修正。
- `servo_body_get_translation` / `_get_velocity` / `_get_rigid_body_handle` — 状态查询。
- `servo_body_destroy` — 移除（保留刚体句柄可查询）。
- 内部：`ServoBody`、`ServoController`。

## 依赖
- fork `PdController`/`PidController`（或内实现的 PD/PID 速度级控制）。
- `crate::rapier::ffi`（`ShapeDesc`、`Quat`、handle 打包）、`crate::rapier::error`。

## 测试
`mps-test/src/rapier/servo_body.rs` — 创建/目标驱动收敛/销毁等。
