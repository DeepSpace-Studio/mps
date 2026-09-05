# rapier/fracture_mesh.rs

## 作用
可碎裂复合刚体（fracture mesh body）的生命周期管理层。一个 fracture mesh body 是"单个刚体 + 多个碰撞体 + 预定义碎块描述"的复合体：创建时作为普通刚体入世界，碎块描述缓存备用；触发碎裂时调用 `fracture.rs` 的 `world_replace_body_with_fracture_fragments` 把源刚体替换为 N 个独立动态刚体（可选用弱关节连接）。触发方式四种：手动 `trigger`；应力强度/Griffith 阈值（`set_trigger` + `set_stress` 达阈值自动碎裂）；疲劳累积（`set_trigger` mode 3 + `add_fatigue_damage` 累计到 1.0 自动碎裂）；自动冲击损伤（`enable_impact_damage` 后每步 `world_step` 把求解器接触冲量 × 系数累积进 `impact_damage`，达阈值自动碎裂）。碎块来源两种：手工描述数组，或 Voronoi 预切割（给定局部 AABB + 种子点集自动生成碎块）。纯组合层，无新物理（Voronoi 胞元数学在 `mps-formula::voronoi`）。

## 关键导出
- `pub extern "C" fn fracture_mesh_body_create(...)` — 由形状 + 碎块描述数组创建，返回稳定 id（`u32::MAX` 表错误）。
- `pub extern "C" fn fracture_mesh_body_create_with_voronoi(...)` — 由形状 + AABB + 种子点集创建，碎块由 Voronoi 胞元盒拟合自动生成；`edge_shrink`（0..0.5）按比例收缩半尺寸避免碎片初始互相穿插；重复种子合并、退化胞元跳过，至少需 1 个有效胞元。
- `pub extern "C" fn fracture_mesh_body_trigger(...)` — 手动触发碎裂（碎块继承源刚体线速度；密度为 0 的碎块继承材质密度）。
- `pub extern "C" fn fracture_mesh_body_set_trigger(...)` — 设置触发模式：0=Manual、1=StressIntensity、2=Griffith、3=Fatigue。
- `pub extern "C" fn fracture_mesh_body_set_trigger_stress(...)` — mode 1 便捷封装。
- `pub extern "C" fn fracture_mesh_body_set_stress(...)` — 上报当前应力强度；达阈值自动碎裂。
- `pub extern "C" fn fracture_mesh_body_add_fatigue_damage(...)` — 累积疲劳损伤（0..=1），mode 3 时达 1.0 自动碎裂。
- `pub extern "C" fn fracture_mesh_body_enable_impact_damage(...)` — 启用自动冲击损伤：`world_step` 步进后轮询窄相接触对（`narrow_phase.contact_pairs()` + `total_impulse_magnitude()`，不依赖事件订阅），冲量 × `scale` 累加，达 `threshold` 自动触发碎裂；未启用时 O(1)。持续静置接触的小冲量也会累积（类疲劳语义），阈值由调用方按材料定。
- `pub extern "C" fn fracture_mesh_body_get_impact_damage(...)` — 查询当前累积冲击损伤。
- 内部：`accumulate_impact_damage`（`world_step` 挂钩，刚体管线步进后调用；先快照启用体 → 反查 rigid-body handle → 按 pair 累积 → 阈值命中者经 `trigger_fracture_mesh` 统一碎裂）、`trigger_fracture_mesh`（手动/自动共用的触发核心，含碎片→颗粒分区与无刚体碎片时的直删路径；碎片线速度 = 源体线速度 + 描述符相对速度，密度 0 继承材质密度）。
- `pub extern "C" fn fracture_mesh_body_link_granular_debris(...)` — 碎屑路由：链接 granular 体后，触发碎裂时最大半尺寸 < `size_threshold` 的碎片变为在碎片世界位置生成、携带源体线速度的单颗 DEM 颗粒（`granular_bodies[id].add_particle`），其余仍为刚体碎片；`granular_id == u32::MAX` 解除链接。全碎片均为碎屑时直接移除源体、不建刚体碎片；链接指向已销毁 granular 体时静默回退为刚体碎片。
- `pub extern "C" fn fracture_mesh_body_is_fractured(...)` — 查询是否已碎裂。
- `pub extern "C" fn fracture_mesh_body_remove(...)` — 移除（未碎裂时删除源刚体；已碎裂则只清元数据）。
- 内部：`insert_fracture_mesh_body`（两个创建入口共用的校验+插入路径）、`FractureMeshBody`（世界内 `fracture_mesh_bodies` 哈希表，id 单调分配）、`FractureTrigger` 枚举、`MAX_FRACTURE_MESH_PARTS = 1024`。

## 依赖
- `mps_formula::voronoi` — `voronoi_fragments_from_seeds`（种子 → 碎块描述；胞元顶点枚举 + 最近邻精确剪枝，见 `crates/mps-formula/src/voronoi.rs` 模块文档）。
- `crate::rapier::fracture` — `world_replace_body_with_fracture_fragments`、`material_valid`、`fragment_valid`（已提为 `pub(crate)`）。
- `crate::rapier::ffi` — `Bool`、`FractureMaterial`、`FractureFragmentDesc`、`ShapeDesc`（`shape_from_desc`）、`RigidBodyHandleRaw`、`WorldHandle`、handle 打包与 `vec3_*` 辅助。
- `crate::rapier::error` — 错误码与 `ffi_guard`。
- `rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyHandle}` — 源刚体/碰撞体构造。

## 测试
`mps-test/src/rapier/fracture_mesh.rs`：创建/状态查询、非法输入拒绝（空指针、容量、材质、碎块描述）、手动触发一次性语义、应力/Griffith 阈值自动碎裂、疲劳累积自动碎裂、模式校验、移除与未找到错误、null world；Voronoi 创建 + 触发碎裂（2 种子 → 碎成 2 块）、非法输入拒绝（null 种子、0 种子、反转 AABB、越界 shrink、坏材质、null world）；自动冲击损伤（落地自动碎裂端到端、默认不累积、scale/threshold/id/out 指针/null world 校验）；碎屑路由（小块→颗粒/大块→刚体混合、全颗粒直删路径、校验与解除链接、碎裂后拒绝链接）。
`mps-test/src/rapier/voronoi.rs`：公式层单元测试——单种子胞元 = AABB（8 顶点/体积/形心）、双种子对半切分、2×2 种子铺满盒子、碎片与模板字段透传、shrink 收缩比例、重复种子合并、非法输入拒绝（空种子/反转或扁平 AABB/越界 shrink/非有限种子）、种子数上限（512 拒绝、恰好 512 接受）。
