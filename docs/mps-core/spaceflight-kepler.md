# rapier/spaceflight/kepler.rs

## 作用
轨道力学基础: 开普勒周期/半长轴、轨道根数↔状态矢量互转、Lambert 时间方程、Hohmann 转移、Tsiolkovsky 火箭方程、大气阻力引起的半长轴衰减率。8 个 C ABI 入口。

## 关键导出
- `space_elements_to_state(elements: OrbitalElements, mu, out_state) -> Bool` — 轨道根数转状态矢量 (位置+速度)，写入 `StateVector`。
- `space_state_to_elements` — 状态矢量转轨道根数 (反向)。
- `space_hohmann_transfer` — Hohmann 双脉冲转移 Δv。
- `space_kepler_period(mu, a) -> f64` — 开普勒第三定律周期 (2π√(a³/μ))。
- `space_kepler_semi_major_axis(mu, period) -> f64` — 周期反推半长轴。
- `space_lambert_time_elliptic` — 椭圆 Lambert 时间方程。
- `space_semi_major_axis_decay_rate` — 大气阻力下半长轴衰减率。
- `space_tsiolkovsky_delta_v` — 齐奥尔科夫斯基理想 Δv。

## 依赖
- 本 crate 子模块: `super::*`。
- 外部 crate: 经 super 用 rapier3d::Vector 做旋转变换。
