# rapier/spaceflight/rotation.rs

## 作用
姿态确定与控制: 四元数/Euler 微分、CMG (控制矩陀螺) 力矩交换、TRIAD/最小二乘定姿、标量 EKF 预测/更新、磁场力矩器、表面充电、太阳阵扰动力矩。直接给 Rapier 刚体加力矩 (含 _flag 变体)。16 个 C ABI 入口。

## 关键导出
- `space_quaternion_derivative` / `space_rigid_body_euler_derivative` — 姿态运动学/动力学方程导数。
- `space_apply_cmg_torque_to_body[_flag](world, body_handle, gimbal_axis, wheel_momentum, gimbal_rate, wake_up, out_exchange) -> Bool` — 给刚体施加 CMG 力矩，返回 CmgExchange。
- `space_apply_magnetic_torquer_to_body[_flag]` — 给刚体施加磁力矩器力矩。
- `space_cmg_exchange` — CMG 力矩交换纯计算。
- `space_cmg_robust_pseudoinverse_diag` — CMG 鲁棒伪逆 (对角加权)。
- `space_triad_attitude` — TRIAD 双矢量定姿。
- `space_least_squares_attitude_two_vector` — 双矢量最小二乘定姿。
- `space_ekf_predict_scalar` / `space_ekf_update_scalar` / `space_ekf_gain_scalar` — 标量扩展 Kalman 滤波。
- `space_magnetic_torquer_dipole` — 磁力矩器偶极矩。
- `space_solar_array_pd_torque` — 太阳阵 P-D 控制扰动力矩。
- `space_surface_charging_current_balance` — 表面充放电电流平衡。

## 依赖
- 本 crate 子模块: `super::*` (unpack_rigid_body_handle, vec3_to_rapier)。
- 外部 crate: 经 super 间接用 rapier3d::prelude。
