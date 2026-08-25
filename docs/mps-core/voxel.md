# rapier/voxel.rs

## 作用
体素网格到碰撞体的转换与体素区域查询。把 Java 侧传入的 `&[u8]` 体素数组按 `VoxelGrid` 视图解读(x/y/z 尺寸、每轴体素边长、原点),用 `rayon` 并行提取实体体素/表面,生成 Rapier 复合形状或体素形状碰撞体;同时支持 AABB/OBB 体素专用构建、构建统计上报,以及体素区域的相交查询与静/动态体素体的直接插入世界。

## 关键导出
- `struct VoxelGrid<'a>` — 借用式体素网格视图(`voxels: &[u8]`、`size_*`、`voxel_size_*`、`origin`),含 `index/is_solid` 等内部方法。
- `fn build_voxel_collider` — 供内部/其他模块复用的体素→ColliderBuilder 构建函数。
- `extern "C"` 入口(~17 项):`collider_builder_create_voxels(_auto)`、`collider_builder_create_voxel_aabb(_auto)`、`collider_builder_create_voxel_obb(_auto)`、`voxel_build_stats`、`voxel_aabb/obb_build_stats(_out)`、`query_intersect_voxel_aabb(_count)`、`query_intersect_voxel_obb(_count)`、`world_insert_static_voxel_aabb`、`world_insert_dynamic_voxel_obb`。
- 上限常量(私有):`MAX_VOXEL_CELLS`(262144)、`MAX_COMPOUND_PARTS`(100000)、`MAX_SURFACE_VERTICES`、`MAX_SURFACE_TRIANGLES`。

## 依赖
- 外部 crate:`rayon::prelude::*`(并行体素扫描)、`rapier3d::math::{Pose, Rotation, Vector}`、`rapier3d::prelude::{ColliderBuilder, SharedShape}`、`std::slice`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(`AabbDesc`/`Obb`/`QueryFilterDesc`/`VoxelBuildStats`/`VoxelColliderMode`/`VoxelColliderOptions`/`ColliderBuilderHandle`/`WorldHandle` 等及 quat/vec3/mode 转换)。
