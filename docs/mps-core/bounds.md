# rapier/bounds.rs

## 作用
扩展形状(capsule/SSV/ellipsoid/prism/cylinder/spherical shell)的构造器与区域相交查询集合。这些形状大多不是 Rapier 内置第一类形状,本文件用 `ColliderBuilder` + `SharedShape` 包装或自定义接触形状方式补充构造器,并提供配合复合碰撞体评估的 AABB/OBB 等区域查询的 `_count/_count_all/_(单列)/_all(切片)` 变体。内部私有助手做几何有限性/有效性校验。

## 关键导出
- `pub(crate) fn capsule_shape / ssv_shape / cylinder_shape / spherical_shell_shape / ellipsoid_shape / prism_shape`(返回 `Option<(Pose, SharedShape)>`)— 把 FFI 几何描述体转成 Rapier 形状 + 偏置姿态,供 collider 构造器与查询内部使用。
- `extern "C"` 入口(~30 项):`collider_builder_create_{capsule, ssv, ellipsoid, prism, cylinder, spherical_shell}`;查询系列 `query_intersect_{capsule, ssv, ellipsoid, prism, cylinder, spherical_shell}{_count, _count_all, _, _all}`。
- `const EPSILON`、私有助手 `identity_pose` / `valid_segment`(私有)。

## 依赖
- 外部 crate:`rapier3d::math::{Pose, Rotation, Vector}`、`rapier3d::prelude::{ColliderBuilder, SharedShape}`、`smallvec::SmallVec`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(`Capsule`/`Ssv`/`Cylinder`/`Ellipsoid`/`Prism`/`SphericalShell`/`ColliderBuilderHandle`/`ColliderHandleRaw`/`QueryFilterDesc`/`WorldHandle`/`MAX_OUTPUT_CAPACITY` 等及 isometry/quat/query_filter/vec3 转换与句柄打包)。
