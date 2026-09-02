# balloon.rs — 气囊/充气体(闭合受压球壳,组合层)

`crates/mps-core/src/rapier/balloon.rs`。一个**气囊体**是闭合的 UV 球壳(纬度环 + 南北极各一粒子),三角形喂给 fork 的 Phase 11 压力模型。与 `soft_cloth_create` / `soft_rope_create` 同属组合层——不发明新物理。

## 机制

- **壳网格**:`rings × segments` 环带粒子 + 2 极点;每个 quad 拆两个三角形,极点扇形封口。`add_triangle` 自动注册三条边为 XPBD 距离约束(带去重),所以壳线框(环边/经线/quad 对角线/极点辐条)免费得到。
- **压力**:每粒子 `F_i = Σ_t P·area(t)·n̂(t)`(法线按质心朝外定向),在 XPBD predict 步与重力/风一起施加(Phase 11)——闭合壳对称膨胀,`pressure = 0` 时字段不设(零每步成本)。
- **柔软度**:全部壳边共享张力侧 `edge_compliance`,0 = 不可伸长蒙皮,越大越像橡皮气球(充到边张力平衡内压为止)。

## BalloonDesc

| 字段 | 含义 |
|---|---|
| `rings` / `segments` | 纬度环数(≥2)/ 每环经度段数(≥3);粒子数 = rings·segments+2,上限 `BALLOON_MAX_PARTICLES`(4096,`add_triangle` 去重是 O(已有边) 扫描,建造成本 O(n²),上限据此保守) |
| `center` / `radius` | 球心 / 半径 |
| `particle_mass` | 壳粒子质量 |
| `edge_compliance` | 壳边张力侧 compliance(≥0) |
| `pressure` | 初始内压(0 = 出生未充气,事后 `soft_body_set_pressure` 充) |
| `iterations` | XPBD 每子步迭代 |

创建后自动切 `Xpbd` 求解器;每体重力继承世界重力。**体积守恒(`soft_body_set_volume_conservation`)不适用于气囊**——它约束四面体,壳没有四面体,膨胀由压力模型维持。

## 与既有 FFI 的关系

充/放气 = `soft_body_set_pressure`(运行时泵气/放气);风 = `soft_body_apply_wind`;锚定(系留气球)= `soft_body_attach_particle`;渲染读回 = `soft_body_read_particles` / `soft_body_read_surface_mesh`;顶点数/三角形数 = `soft_body_particle_count` / `soft_body_read_surface_triangle_count`。

## 物理边界(重要)

自由悬浮的无压壳在均匀重力场中**零内应力、整体自由下落**(等效原理)——不会"瘪"。真实气球瘪掉需要环境气压差或地面,引擎壳体两者都没有;接 proxy 碰撞落地后被压扁是后续课题。差异由 `balloon_unpressurized_free_fall_stays_coherent` 锁定(形状摊开度不变 + 整体下落)。

## 测试与 JNI

- 测试:`crates/mps-test/src/rapier/balloon.rs`(闭合壳粒子/三角形计数、壳面半径校验、充气膨胀、自由下落保持形状、运行时泵气/放气回缩、坏参数表 + 容量上限)。
- JNI:`softBalloonCreate(world, rings, segments, cx/cy/cz, radius, particle_mass, edge_compliance, pressure, iterations)`,后续操作复用 softBody* 绑定。
