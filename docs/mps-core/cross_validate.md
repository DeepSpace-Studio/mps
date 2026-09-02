# rapier/cross_validate.rs

## 作用
多公式交叉验证引力。一个注册进 `ForceLaw` 体系的引力定律：每次 `world_step`，对每个动态体用 N 条独立公式"线"（Newton 点质量、J2–J6 带谐、四极张量、MOND 增强、Schwarzschild 相对论修正）经 `rayon` 并行计算引力加速度，互相校验一致性后以 `F = m·a` 施加。`CrossValidateAggregation` 决定合成方式：`NewtonAnchored`（默认，牛顿线为基准，各非牛顿线只在相对差 ≤ 容差时贡献有界修正，否则被否决）、`Mean`、`Median`。超差结果记入 `world_get_cross_validate_last_divergence` 供诊断。

## 关键导出
- `world_set_cross_validate_gravity(world, config*)` — 注册交叉验证引力定律（`CrossValidateGravityConfig`：中心天体参数、公式线掩码 `CrossValidateLineMask`、容差、修正混合因子）。
- `world_set_cross_validate_gravity_flag(...)` — 布尔模式变体。
- `world_clear_cross_validate_gravity(world)` — 注销回默认重力。
- `world_get_cross_validate_last_divergence(...)` — 最近一次发散诊断。
- `world_cross_validate_default_config(...)` — 填充默认配置。
- 内部：`CrossValidateGravityLaw`（`ForceLaw` 实现）、`CrossValidateAttractor`/`Aggregation`/`LineMask`/`GravityConfig`。

## 依赖
- `crate::rapier::forces`（`ForceLaw`/`ForceFacade` 注册体系）。
- `rayon` — 公式线并行。
- `mps-formula` 引力公式（Newton/MOND/J2/相对论修正等）。
- `crate::rapier::ffi`/`error`。

## 测试
`mps-test/src/rapier/cross_validate.rs` — 各聚合模式、容差否决、注册/注销。
