# rapier/collider.rs

## 作用
碰撞体构造器与全部 `collider_*` / `world_*_collider` C ABI 入口。支持丰富形状(AABB/OBB/sphere/heightmap/convex hull/point cloud/double BV/skewed/discrete OBB/fused collapsing bounds/edge BVH/medial spheres/halfspace 等)的创建、聚合成 `ColliderBuilder`、构建、插入/移除/复制到世界,以及姿态/传感器/摩擦/恢复系数/密度/碰撞解算分组/激活事件与 hooks 阈值的读写。

## 关键导出
- `extern "C"` 入口(~60 项):
  - 构造:`collider_builder_create(_halfspace/_ex/_obb/_sphere/_heightmap/_convex_hull/_point_cloud/_double_bv/_skewed_obb/_discrete_obb/_fused_collapsing_bounds/_edge_bvh/_medial_spheres)`、`collider_builder_build/destroy`、`collider_destroy_raw`。
  - setter:`set_translation/rotation/pose/sensor/friction/restitution/density/collision_groups/solver_groups/active_events/active_hooks/contact_force_event_threshold`。
  - 世界增删:`world_insert_collider(_with_parent)`、`world_remove_collider(_flag)`、`world_copy_collider`。
  - 查询/setter:`collider_get/set_translation(_out/_flag)`、`get/set_rotation(_out/_flag)`、`set_pose(_flag)`、`set_sensor(_flag)`、`set_friction/restitution/collision_groups/solver_groups/active_events/active_hooks/contact_force_event_threshold(_flag)`、`get_shape_count`、`get_density`。
- (无 pub struct/enum/trait)。

## 依赖
- 外部 crate:`rapier3d::math::{Pose, Rotation, Vector}`、`rapier3d::na::Unit`、`rapier3d::prelude::{Array2, Collider, ColliderBuilder, SharedShape, TypedShape}`、`smallvec::SmallVec`、`std::slice`。
- 本 crate 子模块:`crate::convert::quat_to_rapier`、`crate::rapier::error`、`crate::rapier::ffi`(AabbDesc/Obb/Sphere/ShapeDesc/ColliderBuilderHandle/ColliderHandleRaw/InteractionGroupsDesc/WorldHandle 等与一组 shape_desc/active_events/quat/vec3/句柄打包辅助)。
