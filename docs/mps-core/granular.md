# granular.rs — 颗粒体(DEM 粒子云,Phase 36)

fork 侧 `rapier/src/dynamics/granular.rs` + mps-core FFI `crates/mps-core/src/rapier/granular.rs`。**颗粒体**是 DEM(离散元)粒子云:`GranularParticle / GranularWorld / GranularParams` 脚手架与 Phase 35 `fluid.rs` 同构(朴素 O(n²) 邻居、半隐式 Euler、不碰 SoA、fork 本地 f64 数学、bit-identical),把流体的压力/黏度核换成颗粒接触模型。

## 接触模型(fork 侧)

- **径向排斥**:两粒子重叠(`d < r_i + r_j`)时线性弹簧推离 `k_n·δ`,法向速度比例阻尼 `c_n`(只推不拉)。
- **Coulomb 摩擦**:切向阻尼力(系数 `tangential_damping`),钳位到 `μ·|F_n|`——颗粒体区别于流体的定义性项,堆体能保持休止角。
- 每对 (i,j) 只访问一次,力对称施加(牛顿第三定律);固定粒子序 + 纯 f64 → 确定性。

## mps-core FFI

- `granular_create(world, gravity, particle_radius, normal_stiffness, normal_damping, friction, tangential_damping) -> u32`(`u32::MAX` 错误,参数校验:`tangential_damping ∈ [0,1]` 等)
- `granular_add_particle(world, id, x/y/z, vx/vy/vz, mass, radius) -> u32`(粒子索引)
- `granular_particle_count` / `granular_read_particles(out_pos, out_vel, capacity)`(双通道,短容量返回真实数量)
- `granular_step(world, id, dt)`——可选手动子步;`world_step` 已自动推进全部颗粒体(照 fluids 的位置,在刚体管线之后)。

`PhysicsWorld.granular_bodies: Vec<GranularWorld>`(world.rs)。

## 测试

- fork 单元测试(`rapier` 仓库,4 个):自由落体解析解、重叠排斥(COM 不动)、摩擦减速切向滑移(μ=0.8 vs μ=0 差分)、bit-identical 确定性。
- mps-test 集成(4 个):创建 + world_step 自动推进(下落 + 排斥)、手动 step 钩子(零重力匀速位移精确校验)、短缓冲/空通道读回、坏参数表 + 未知 id。

## JNI

`softGranularCreate / softGranularAddParticle / softGranularParticleCount / softGranularReadParticles / softGranularStep`,后续(Phase 37+)计划:voxel 挖掘联动(`soft_body_voxel_dig` → 生成颗粒)、与刚体的 proxy 碰撞耦合(照 fluid_proxies 模板)、空间哈希 broad-phase。

## Phase 37:voxel 挖掘 → 颗粒生成联动

`granular_link_voxel_dig(world, dig_grain_body, grain_mass, grain_radius)` 建立链接后,`collider_voxel_edit(solid=0)` 真正挖掉一个格子(changed=true)时,会在格子世界中心向链接的颗粒体生成一颗颗粒(零初速);`dig_grain_body = u32::MAX` 解链;`granular_get_voxel_dig_link` 查询当前链接。已挖空/未链接不生成。这是"挖月壤"链路的最后一环:voxel 地形 + `collider_voxel_edit` 挖掘 + 颗粒体承接碎屑。

## 数值注意

显式积分稳定性:`k_n / m · dt² < 1`。默认 `k_n=800` 配 `m≥0.05`、`dt=1/60` 安全;调参先加大 `normal_damping` 再加 `k_n`。
