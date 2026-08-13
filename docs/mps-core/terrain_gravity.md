# rapier/terrain_gravity.rs

## 作用
实现不规则天体与地形的重力模型,文档头声明支持三类:
1. **多面体引力** — Werner & Scheeres (1997),对常密度多面体精确(仅受离散化误差影响)。
2. **地表质量分布** — 基于 DEM 的地形重力,支持直接卷积与 FFT 卷积两种方式。
3. **月球 Mascon 模型** — GRAIL 导出的质量瘤(Mass Concentration)。
提供纯 Rust 接口(`polyhedron_gravity`、`terrain_gravity_*`、`lunar_mascon_*`)与对应的 `extern "C"` ABI 入口,供 Java 侧调用。`TerrainGrid`、`LunarMascon` 为导出数据结构。

## 关键导出
- `pub fn polyhedron_gravity(...)` — 常密度多面体引力(非 C ABI,纯函数)。
- `pub struct TerrainGrid` — DEM 地形网格数据容器(行 186)。
- `pub fn terrain_gravity_direct(...)` / `pub fn terrain_gravity_fft(...)` — DEM 地形重力(直接 / FFT 卷积)。
- `pub struct LunarMascon` — 月球质量瘤参数结构(行 360)。
- `pub fn lunar_mascon_gravity / _count / _get(...)` — 月球 mascon 引力查询(纯 Rust)。
- `pub extern "C" fn terrain_polyhedron_gravity(...)` — 多面体引力的 C ABI 入口。
- `pub extern "C" fn terrain_gravity_dem(...)` / `terrain_gravity_dem_fft(...)` — DEM 地形重力的 C ABI 入口。
- `pub extern "C" fn terrain_lunar_mascon_gravity / _count / _get(...)` — 月球 mascon 的 C ABI 入口。
- 常量:`MAX_VERTICES = 100_000`、`MAX_FACES = 200_000`。

## 依赖
- `rapier3d::prelude::Vector` — 向量与引力计算。
- `crate::rapier::error` — `ERR_INVALID_ARGUMENT`、`ERR_NOT_FOUND`、`ERR_NULL_POINTER`、`ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `Bool`、`Vec3`,及 `vec3_finite`、`vec3_from_rapier`、`vec3_to_rapier`。
- 物理常量自行定义(如 `G = 6.67430e-11`),不依赖外部公式 crate。
