# rapier/query.rs

## 作用
空间查询 C ABI 入口集合。提供射线投射(`query_cast_ray(_out/_rays)`)、点投影(`query_project_point(_out)`)、相交计数与列举(对 AABB/OBB/Sphere 多种区域谓词,带 `_count/_count_all/_counts`(批量计数)/`_`(单区域)/`_all`(全切片)变体)、形状投射(`query_cast_shape(_out)`)。所有函数把 FFI 描述体(如 `AabbDesc`、`Obb`、`Sphere`、`QueryFilterDesc`、`ShapeCastOptionsDesc`)转成 Rapier 内部几何,再调用相应的 Rapier query 机制。

## 关键导出
- `extern "C"` 入口(~24 项):`query_cast_ray(_out/_rays)`、`query_project_point(_out)`、`query_intersect_point_count`、`query_intersect_aabb_count/_all/_counts`、`query_intersect_obb_count/_all/_counts`、`query_intersect_sphere_count/_all/_counts`、`query_intersect_aabb(_all)`、`query_intersect_obb(_all)`、`query_intersect_sphere(_all)`、`query_intersect_aabb_rigid_body_count_all/_all`、`query_cast_shape(_out)`。
- (无 pub struct/enum/trait;纯 FFI 函数文件)。

## 依赖
- 外部 crate:`rapier3d::geometry::{Aabb, Ray}`、`rapier3d::parry::shape::FeatureId`、`rapier3d::prelude::SharedShape`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(AabbDesc/Obb/Sphere/ShapeDesc/QueryFilterDesc/ShapeCastOptionsDesc/RayHit/PointProjection/ShapeCastHit/MAX_OUTPUT_CAPACITY/WorldHandle/ColliderHandleRaw 及 quat/vec3/shape_desc/query_filter/shape_cast_options 转换辅助)。
