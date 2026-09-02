# cloth.rs — 布料体(网格拓扑软体,组合层)

`crates/mps-core/src/rapier/cloth.rs`。一个**布料体**是 cols × rows 的矩形质点网格,由三族弹簧桥接,以普通 `SoftBody` 存入世界的 `SoftBodySet`。与 `soft_chain_create`(骨骼软体 Phase 1)同属"组合层":不发明新物理,只复用 fork 里已有的原子操作。

## 弹簧三族

| 族 | 连接 | 刚度 | 作用 |
|---|---|---|---|
| 结构(structural) | 网格横/竖相邻 | `stiffness` | 承载拉伸形状 |
| 剪切(shear) | 每格两条对角线 | `stiffness · shear_ratio` | 抵抗面内错切 |
| 弯曲(bend) | 隔一格邻居 | `stiffness · bend_ratio` | 经典 mass-spring 弯曲代理,`0` = 自由褶皱 |

默认求解器是 `MassSpring`,只消费 `springs`,所以三族都是普通弹簧(不是 XPBD 距离约束)。每体重力初始化为**世界重力**(与 `soft_body_voxel_build` 故意置零、留给地形耦合接口不同)。

## FFI

- `soft_cloth_create(world, ClothDesc) -> u32` — 一次调用建好整张网格;返回 `SoftBodyId`,错误返回 `u32::MAX` + 线程局部错误码(与 `soft_body_build_grid`/`soft_body_build_rope` 同约定)。参数校验:网格 ≥ 2×2、总数 ≤ `CLOTH_MAX_PARTICLES`(512×512)、轴向量归一化后不平行、`ClothPinMode`(0 自由 / 1 四角 / 2 `col==0` 边=旗杆 / 3 `row==0` 边=窗帘杆)。
- 粒子计数/位置读回/风/撕裂/睡眠/能量等全部直接复用既有 `soft_body_*` FFI(`soft_body_read_particles`、`soft_body_apply_wind`、`soft_body_set_tear_strain` + `soft_body_tear_now`、…)。

## 与 `soft_body_build_grid` 的区别

`soft_body_build_grid` 填充 3D 盒子(6 邻接、XPBD、边界整体钉死)——果冻块;布料是**2 维**的,带三族弹簧划分、任意平面朝向(`u_axis`/`v_axis`)、按边选择性 pin——旗帜/窗帘/桌布的表述。

## 测试

`crates/mps-test/src/rapier/cloth.rs`:网格拓扑与弹簧计数(39 = 17 结构 + 12 剪切 + 10 弯曲)、零 ratio 关闭旁族、重力下旗帜下垂且钉死边不动、风沿列单调偏移、超应变撕裂删弹簧、坏参数表(ERR_INVALID_ARGUMENT/ERR_CAPACITY)、自由落体有界。
