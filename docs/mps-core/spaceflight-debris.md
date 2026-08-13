# rapier/spaceflight/debris.rs

## 作用
碎片与碰撞风险: SGP4 J2 长期进动速率、碎片碰撞概率 (高斯径向/航迹协方差)。空间环境对抗体预警和编目相关的两个 C ABI 入口。

## 关键导出
- `space_debris_collision_probability(miss_distance, combined_radius, sigma_radial, sigma_intrack, out_probability) -> Bool` — 计算两个物体碰撞概率，写入 `CollisionProbability { probability, combined_sigma }`，概率 clamp 到 [0,1]。
- `space_sgp4_j2_secular_rates(semi_major_axis, eccentricity, inclination, mean_motion, equatorial_radius, j2, out_rates) -> Bool` — 基于 SGP4 模型由 J2 摄动给出近地点幅角/升交点赤经/平近点角的长期进动速率，写入 `Sgp4SecularRates`。

## 依赖
- 本 crate 子模块: `super::*` (即 mod.rs 的 finite/write_out/ffi_guard/error 常量/FFI 类型)。
- 外部 crate: 无直接 (通过 super 间接依赖 rapier3d)。
