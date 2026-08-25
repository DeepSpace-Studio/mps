# rapier/interaction.rs

## 作用
体间相互作用力的实际实现模块:牛顿引力、库仑摩擦、大气阻力(含太阳风、动力学摩擦、辐射压力、磁偶极辐射、X 射线辐射、Jeans 逃逸等多种天体物理力法),与 Rapier 刚体集耦合。文档说明这些力法由调用方经 `world_set_*_law` FFI 配置后,`world_step` 内部调用 `apply_body_interactions*` 把计算出的力注入物理流水线。各力法都 `impl ForceLaw` 注册进 `ForceRegistry`,旧版「每分支 if 配置」接口被收编为统一注册式调用。

## 关键导出
- `const G: f64` — 万有引力常量。
- `fn pairwise_gravity` — 直接作用于 `RigidBodySet` 的两两牛顿引力计算(供 `world_step` 调用)。
- `fn per_body_air_drag` — 每体大气阻力计算。
- `fn apply_body_interactions_with_facade`(crate 级)— `world_step` 内驱动的体间相互作用总分发器(含 pairwise 引力、库仑摩擦、各 ForceLaw 调用)。
- `struct CoulombFrictionParams`(crate 级)— 库仑摩擦参数集。
- `struct NewtonianGravityForceLaw / AirDragForceLaw / SolarWindPressureForceLaw / DynamicalFrictionForceLaw / MonDGravityForceLaw / EddingtonRadiationPressureForceLaw / XrayIrradiationForceLaw / PulsarMagneticDipoleForceLaw / JeansEscapeDragForceLaw`(均 crate 级,`impl ForceLaw`)— 实际注册进 registry 的力法结构体。

## 依赖
- 外部 crate:`rapier3d::prelude::{Vector, NarrowPhase, RigidBodyHandle}`、`smallvec::SmallVec`、`mps_formula::galactic_dynamics as gd`、`mps_formula::heliophysics as hph`、`mps_formula::high_energy_astro as hea`。
- 本 crate 子模块:`crate::rapier::ffi`(`AirDragLaw`/`CustomPhysicsReport`/`vec3_*` 转换)、`crate::rapier::math::KahanVec3`、`crate::rapier::forces{ForceFacade, ForceLaw, ForceLawType, ForceRegistry}`。
