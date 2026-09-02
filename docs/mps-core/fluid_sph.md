# rapier/fluid_sph.rs

## 作用
SPH 流体体 FFI —— 流体 SPH 路线图 Phase 1。fork `FluidWorld`（`rapier/src/dynamics/fluid.rs`）之上的薄组合层：每个流体存于 `PhysicsWorld.fluids`（按 `Vec` 下标索引，返回给调用方的 id 即该下标）。本模块不做新物理，只在 FFI 边界搬运参数；流体的 step 与刚体耦合（碰撞代理回读）由 `world.rs` 的 step 流水线负责（Phase 2 起）。

## 关键导出
- `fluid_create(...)` — 建 SPH 流体世界，返回 id（`Vec` 下标）。
- `fluid_add_particle(world, id, pos, vel)` — 加粒子。
- `fluid_particle_count(world, id)` — 粒子数。
- `fluid_get_particle(world, id, index, out_pos, out_vel)` — 读粒子状态。
- `fluid_step(world, id, dt)` — 推进流体（SPH 力 + 积分）。
- `fluid_enable_collision(world, id, particle_radius, enabled)` — Phase 5f 式逐粒子碰撞代理（与 `soft_body_enable_collision` 同思路）。

## 依赖
- fork `rapier3d::dynamics::fluid`（`FluidWorld`）。
- `crate::rapier::ffi`（`Vec3`、`WorldHandle`）、`crate::rapier::error`。

## 测试
`mps-test/src/rapier/fluid_sph.rs` — 创建/加粒子/步进/代理耦合。
