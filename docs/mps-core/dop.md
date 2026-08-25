# rapier/dop.rs

## 作用
k-DOP 与 FDH(Fixed-Direction Hull)离散方向包围壳的构造器实现。`DirectionHull` trait 定义按给定法向方向集计算每向最小/最大 slab,把点云投影到 slab 合成凸包,再 `ColliderBuilder` 化为 Rapier 复合凸包形状。`KdopHull` 用预置法向 `SmallVec<[Vector;13]>`(13 节约常见 26-DOP),`FdhHull<'a>` 借用调用方提供的方向数组。FFI 入口接 `KDOP preset` 枚举翻译。

## 关键导出
- `trait DirectionHull` — `directions()->&[Vector]` + 默认 `build(&self, &[Vector])->Option<ColliderBuilder>`。
- `struct KdopHull { directions: SmallVec<[Vector;13]> }` + `impl DirectionHull for KdopHull`(包了 `pub directions`)。
- `struct FdhHull<'a> { directions: &'a [Vector] }` + `impl DirectionHull for FdhHull<'_>`。
- `fn kdop_directions(preset: KdopPreset)-> SmallVec<[Vector;13]>` — preset→法向集转换,FFI 入口使用。
- `extern "C"` 入口(2 项):`collider_builder_create_kdop`、`collider_builder_create_fdh`。
- 私有助手 `Slab`、`normalize_direction`、`read_vectors`、`build_direction_hull`;上限 `MAX_RAW_POINTS`(1e6)、`MAX_RAW_DIRECTIONS`(4096)。

## 依赖
- 外部 crate:`rapier3d::prelude::{ColliderBuilder, Vector}`、`smallvec::{SmallVec, smallvec}`、`std::slice`。
- 本 crate 子模块:`crate::rapier::error`(ERR_CAPACITY/ERR_INVALID_ARGUMENT/ERR_NULL_POINTER/clear_error/ffi_guard/set_error)、`crate::rapier::ffi`(`ColliderBuilderHandle`、`KdopPreset`、`kdop_preset_from_raw`)。
