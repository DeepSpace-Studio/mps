# rapier/forces.rs

## 作用
力法注册中心。文件头说明它把以前分散在 `world_step` 各处的硬编码 `if let Some(law)` 力分支抽象为统一注册表:实现 `ForceLaw` 的力法注册进 `ForceRegistry`(由 `PhysicsWorld` 持有),`world_step` 只需调用 `registry.apply_all()`,新增力法变成「定义结构体 + 注册」。包含每体每步的力贡献表(`ForceContributionTable`)、Kahan 求和的 `ForceReport`、`BodyForceLog` 日志、`ForceFacade` 提供安全写入沙盒,以及 `ForceLawType` 标识与 rely 的 Kahan 向量工具。

## 关键导出
- `trait ForceLaw: Send + Sync` — 力法接口,定义 `apply_at` 等由 `apply_all` 统一调用。
- `enum ForceLawType` — 力法类型分类(配合 `ForceContributionTable` 索引)。
- `struct ForceRegistry` — 注册表本体;方法 `new/register/unregister/find_by_type/unregister_by_type/get/get_mut/law_indices/apply_at/apply_all/len/is_empty`(`unregister` 返回 bool)。
- `struct ForceLawHandle(pub u64)` + `raw()` — 力法注册句柄。
- `struct ForceContribution` — 单条力贡献记录。
- `struct ForceContributionTable` + `ForceContributionTableIter` — 按力法类型分桶的力贡献表;方法 `get/get_copy/iter/values/is_empty/add`。
- `struct ForceReport` — 单帧力汇总报告;`add`/`to_legacy_report()/drain_report()`。
- `struct BodyForceLog` — 每体力日志缓冲。
- `struct ForceFacade<'a>` — `ForceLaw::apply` 调用期的力/力矩写入外观;方法 `new/add_force/add_torque/add_force_at_point/push_force/push_torque/update_reynolds/drain_report`。
- `const NUM_FORCE_TYPES`、`ForceLawType::label`。

## 依赖
- 外部 crate:`rapier3d::prelude::{ColliderSet, NarrowPhase, RigidBodyHandle, RigidBodySet, Vector}`、`smallvec::SmallVec`。
- 本 crate 子模块:`crate::rapier::ffi::CustomPhysicsReport`、`crate::rapier::math::KahanVec3`。
