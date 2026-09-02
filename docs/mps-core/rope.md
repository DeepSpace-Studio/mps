# rope.rs — 绳索/缆绳体(单向 cable 约束,组合层)

`crates/mps-core/src/rapier/rope.rs`。一个**绳索体**是沿 `start → end` 直线布点的质点链(`segments + 1` 个粒子),由 XPBD 距离约束串接,以普通 `SoftBody` 存入 `SoftBodySet`。与 `soft_chain_create` / `soft_cloth_create` 同属组合层——不发明新物理,只复用 fork 的 Phase 19 各向异性 `DistanceConstraint`。

## 核心机制:单向缆绳

`DistanceConstraint` 的张力/压缩两侧 compliance 分离(`compliance` 张力侧、`compression` 压缩侧,Phase 19)。`RopeDesc.unilateral` 置位时,把压缩侧 compliance 设为 [`ROPE_CABLE_COMPRESSION_COMPLIANCE`] = 1e9——XPBD 投影权重 α/dt² ≈ 3.6e12,远大于典型逆质量,压缩侧修正量 ~1e-12,等效于**只抗拉不抗压**:缆绳松弛时自由缩短,绷紧时按 `stretch_compliance` 抗拉伸。`unilateral` 清零时两侧同 compliance,行为是普通弹性绳。

## RopeDesc 字段

| 字段 | 含义 |
|---|---|
| `segments` | 段数(粒子数 = segments+1),≥1,≤ `ROPE_MAX_PARTICLES`(65536) |
| `start` / `end` | 两端点(直线布点),跨度 > 1e-9 |
| `particle_mass` | 自由粒子质量 |
| `stretch_compliance` | 张力侧 compliance,0 = 不可伸长 |
| `slack` | 松弛系数:每段 rest = 跨度/segments × (1+slack) |
| `iterations` | XPBD 每子步 Gauss-Seidel 迭代次数 |
| `unilateral` | 缆绳模式开关 |
| `pin_mode` | `RopePinMode`:0 自由 / 1 钉起点 / 2 钉终点 / 3 双端 |

创建后自动切到 `Xpbd` 求解器(距离约束在默认 `MassSpring` 路径下无效)。每体重力继承世界重力。

## 与既有 FFI 的关系

- `soft_body_build_rope` 是**双向** XPBD 绳(弹性绳);本模块的增量是单向缆绳、slack、直线跨度布点。
- 后续操作全部复用既有 `soft_body_*`:`soft_body_attach_particle`(端点锚到刚体,粒子 0 = start、粒子 segments = end)、`soft_body_scale_rest_length`(绞盘收/放,同时缩放弹簧与距离约束)、`soft_body_read_particles` / `soft_body_read_edges`(渲染读回)。

## 已知边界

悬挂的绳链全程处于拉伸状态(真实物理),压缩侧在纯重力悬垂场景不参与——单向与双向在"两针固定+重力"下终态一致。差异只在端部受外力挤压时出现(系泊、拖拽中缆绳被推拢),单向性由 `rope_cable_mode_makes_compression_free` 结构测试直接锁定(compression == 1e9)。

## 测试与 JNI

- 测试:`crates/mps-test/src/rapier/rope.rs`(链拓扑、缆绳结构、slack rest、悬垂有界、绞盘收放、锚定跟随刚体、坏参数表)。
- JNI:`softRopeCreate(world, segments, sx/sy/sz, ex/ey/ez, particle_mass, stretchCompliance, slack, iterations, unilateral, pin_mode)`,后续操作复用 softBody* 绑定。
