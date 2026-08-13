# rapier/ffi/types.rs

## 作用
FFI 层的"句柄"类型定义。所有 C-ABI 函数持有的不透明句柄（世界、构造器、控制器、索引等）都是在此定义的薄包装结构体，内部包着本项目对应 Rust 类型。
值类型（`Vec3`、`Quat`、各 `*Report` 等）大多从 `mps_formula::ffi::types` 再导出，本文件只补充那些指向本项目 `world`/`anvilkit`/`joints`/`controller`/`rtree`/`crbtree` 等具体实现的句柄。
`AnvilKitAppHandle` 仅在 `anvilkit-bridge` feature 下编译。

## 关键导出
- `WorldHandle` — 物理世界句柄，内含 `crate::rapier::world::PhysicsWorld`。
- `AnvilKitAppHandle` — anvilkit 应用句柄（feature=`anvilkit-bridge`），内含 `AnvilKitAppState`。
- `RigidBodyBuilderHandle` — 刚体构造器句柄，内含 `rapier3d::RigidBodyBuilder`。
- `ColliderBuilderHandle` — 碰撞体构造器句柄，内含 `rapier3d::ColliderBuilder`。
- `JointBuilderHandle` — 关节构造器句柄，内含 `crate::rapier::joints::JointBuilderKind`。
- `CharacterControllerHandle` — 角色控制器句柄，内含 `crate::rapier::controller::CharacterControllerState`。
- `RTreeHandle` — 包围盒树索引句柄，内含 `crate::rapier::rtree::RTreeIndex`。
- `CRbTreeHandle` — 自定义红黑树索引句柄，内含 `crate::rapier::crbtree::CRbTreeIndex`。
- `pub use mps_formula::ffi::types::*;` — 再导出标量值与报告结构体（如 `Vec3`、`Quat`、`OrbitalElements` 等）。

## 依赖
- 外部 crate：`mps_formula::ffi::types`（值类型来源）、`rapier3d`（构造器类型）。
- 本 crate子模块：`crate::rapier::world`、`crate::rapier::anvilkit`、`crate::rapier::joints`、`crate::rapier::controller`、`crate::rapier::rtree`、`crate::rapier::crbtree`。
