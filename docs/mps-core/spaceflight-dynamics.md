# rapier/spaceflight/dynamics.rs

## 作用
相对运动与制导 (Clohessy-Wiltshire)、机械臂 DH 变换/逆解、柔性模态/晃荡/对接缓冲、变分方程、bang-off-bang 制导、辐射剂量等。覆盖交会对接、机械臂、柔性航天器建模等 kid of 14 个 C ABI 入口。

## 关键导出
- `space_arm_first_joint_inverse(wrist_x, wrist_y) -> f64` — 平面机械臂第一关节角 (atan2)。
- `space_arm_third_joint_angle(planar_radius, vertical_offset, link2, link3, elbow_up) -> f64` — 余弦定理求第三关节角，不可达返回 NaN。
- `space_artificial_potential_guidance` — 人工势场制导。
- `space_bang_off_bang_profile` — bang-off-bang 推力时间分配剖面。
- `space_cw_derivative` — Clohessy-Wiltshire 相对运动状态导数。
- `space_dh_transform` — Denavit-Hartenberg 关节变换。
- `space_docking_buffer_energy` / `space_docking_glideslope_command` — 对接缓冲能量与下滑道指令。
- `space_flexible_mode_derivative` / `space_slosh_pendulum_derivative` — 柔性模态/晃荡摆状态导数。
- `space_manipulator_dynamics_diag` / `space_mass_properties_two_body` — 机械臂动力学对角质量阵、两体质量属性。
- `space_radiation_absorbed_dose` — 辐射吸收剂量。
- `space_variational_two_body` — 两体变分 (状态转移矩阵) 导数。

## 依赖
- 本 crate 子模块: `super::*`。
- 外部 crate: 间接经 super 的 rapier3d::Vector。
