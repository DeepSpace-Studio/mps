# rapier/spaceflight/perturbation.rs

## 作用
轨道摄动力与 tensorflow 力: J2 摄动、大气阻力、太阳辐射压、重力梯度力矩、Sagnac、原子氧侵蚀。是少数直接操作 Rapier 刚体的空间模块 (含 `_flag` 变体与 wake_up 参数)。15 个 C ABI 入口。

## 关键导出
- `space_apply_j2_force_to_body[_flag](world, body_handle, ...) -> Bool` — 给刚体施加 J2摄动力。
- `space_apply_atmospheric_drag_to_body[_flag]` — 给刚体施加大气阻力。
- `space_apply_solar_radiation_pressure_to_body[_flag]` — 给刚体施加太阳辐射压。
- `space_apply_gravity_gradient_torque_to_body[_flag]` — 给刚体施加重力梯度力矩。
- `space_atmospheric_drag_acceleration` — 大气阻力加速度 (纯计算)。
- `space_j2_acceleration` — J2加速度 (纯计算)。
- `space_solar_radiation_pressure_acceleration` — 太阳辐射压加速度。
- `space_gravity_gradient_torque` — 重力梯度力矩。
- `space_atmospheric_density_scale_height` — 大气密度标高。
- `space_atomic_oxygen_erosion` — 原子氧剥蚀 (材料退化)。
- `space_sagnac_phase_rate(area, angular_rate, wavelength) -> f64` — Sagnac 相位变化率。

## 依赖
- 本 crate 子模块: `super::*` (含 unpack_rigid_body_handle, vec3_to_rapier, vec3_from_rapier)。
- 外部 crate: 经 super 间接用 rapier3d::prelude。
