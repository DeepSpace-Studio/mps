# mps-core 源码导览

`mps-core` 是 `mps_rigid_body` 的物理世界 + Rapier 封装 + C ABI crate（`crates/mps-core`）。本文档按子模块分区，链接到 `docs/mps-core/` 下每个源文件的作用分析。所有分析基于真实源码。

源码根: `crates/mps-core/src/`。crate 入口 `lib.rs` 把 `rapier` 模块整体导出。

---

## 入口与模块根

- [lib.md](mps-core/lib.md) — `src/lib.rs`：crate 入口，re-export `rapier` 模块与 `jni_api`，anvilkit-bridge feature 门控。
- [rapier-mod.md](mps-core/rapier-mod.md) — `src/rapier/mod.rs`：`rapier` 顶层模块，声明并 re-export 全部子模块。

## 物理世界核心

- [world.md](mps-core/world.md) — `world.rs`：`PhysicsWorld` 主结构与 `world_*` C ABI 入口（创建/步进/重力/积分参数/共享竞技场/相对力）。
- [rigid_body.md](mps-core/rigid_body.md) — `rigid_body.rs`：刚体 builder/创建/位姿/速度/力/脉冲/CCD/sleep 等 `rigid_body_*` 入口。
- [collider.md](mps-core/collider.md) — `collider.rs`：碰撞体 builder/创建/材质/分组/事件/钩子等 `collider_*` 入口。
- [events.md](mps-core/events.md) — `events.rs`：碰撞/接触力事件队列 + hooks 回调。
- [forces.md](mps-core/forces.md) — `forces.rs`：力法则注册表、BodyForceLog、ForceFacade。

## 查询与交互

- [query.md](mps-core/query.md) — `query.rs`：射线/点/AABB/OBB/球/形状投掷等 `query_*` 空间查询入口。
- [interaction.md](mps-core/interaction.md) — `interaction.rs`：交互组、碰撞分组、接触交互配置。
- [joints.md](mps-core/joints.md) — `joints.rs`：关节 builder 与 `joint_*` 入口。
- [controller.md](mps-core/controller.md) — `controller.rs`：角色控制器与碰撞响应。
- [batch.md](mps-core/batch.md) — `batch.rs`：批量操作（批量创建/查询/事件读取）。

## 空间索引与体素

- [crbtree.md](mps-core/crbtree.md) — `crbtree.rs`：紧凑 R 树（CRbTree）空间索引。
- [rtree.md](mps-core/rtree.md) — `rtree.rs`：R 树空间索引。
- [voxel.md](mps-core/voxel.md) — `voxel.rs`：体素碰撞体构建（AABB/OBB/网格）与体素查询。

## 边界与范围

- [bounds.md](mps-core/bounds.md) — `bounds.rs`：扩展边界 builder（胶囊/SSV/椭球/棱柱/圆柱/壳/kDOP/FDH/神经边界）。
- [dop.md](mps-core/dop.md) — `dop.rs`：k-DOP 有向离散定向多面体边界。
- [neural.md](mps-core/neural.md) — `neural.rs`：神经学习边界（NeuralBounds）。

## 物理域扩展

- [aerodynamics.md](mps-core/aerodynamics.md) — `aerodynamics.rs`：气动面/体素网格施力 `aero_*` 入口。
- [cloth.md](mps-core/cloth.md) — `cloth.rs`：布料体（矩形网格 + 结构/剪切/弯曲三族弹簧，`soft_cloth_*` 入口）。
- [rope.md](mps-core/rope.md) — `rope.rs`：绳索/缆绳体（单向 cable 约束 + 绞盘，`soft_rope_create` 入口）。
- [balloon.md](mps-core/balloon.md) — `balloon.rs`：气囊/充气体（闭合受压球壳 + Phase 11 压力模型，`soft_balloon_create` 入口）。
- [granular.md](mps-core/granular.md) — `granular.rs`：颗粒体（DEM 接触模型 + Coulomb 摩擦，`granular_*` 入口）。
- [fluid.md](mps-core/fluid.md) — `fluid.rs`：流体力学力（浮力/阻力/SPH 等）。
- [molecular.md](mps-core/molecular.md) — `molecular.rs`：分子力（Lennard-Jones/Coulomb）。
- [fracture.md](mps-core/fracture.md) — `fracture.rs`：断裂力学与刚体碎裂 `fracture_*` 入口。
- [trajectory.md](mps-core/trajectory.md) — `trajectory.rs`：轨迹力估计/积分/施加 `trajectory_*` 入口。
- [terrain_gravity.md](mps-core/terrain_gravity.md) — `terrain_gravity.rs`：不规则天体/地形重力（多面体/DEM/Mascon）。

## 空间飞行子模块（src/rapier/spaceflight/）

原 2610 行 `spaceflight.rs` 按 OPTIMIZATION.md §3 拆分为 8 个 per-domain 子模块，共用 [spaceflight-mod.md](mps-core/spaceflight-mod.md) 中的辅助与常量。

- [spaceflight-mod.md](mps-core/spaceflight-mod.md) — `mod.rs`：子模块枢纽，共用数值辅助与 FFI 类型 re-export。
- [spaceflight-debris.md](mps-core/spaceflight-debris.md) — `debris.rs`：SGP4 J2 进动 + 碎片碰撞概率。
- [spaceflight-dynamics.md](mps-core/spaceflight-dynamics.md) — `dynamics.rs`：相对运动制导/机械臂/柔性/对接/变分/bang-off-bang。
- [spaceflight-gnss.md](mps-core/spaceflight-gnss.md) — `gnss.rs`：GNSS 伪距/双差、Friis 链路、雷达测速。
- [spaceflight-kepler.md](mps-core/spaceflight-kepler.md) — `kepler.rs`：开普勒轨道力学基础（根数↔状态/Lambert/Hohmann/Tsiolkovsky）。
- [spaceflight-perturbation.md](mps-core/spaceflight-perturbation.md) — `perturbation.rs`：J2/大气阻力/太阳辐射压/重力梯度等摄动力施加到刚体。
- [spaceflight-propulsion.md](mps-core/spaceflight-propulsion.md) — `propulsion.rs`：推进与电源（Hall/Sabatier/电池/太阳板/接触力）。
- [spaceflight-rotation.md](mps-core/spaceflight-rotation.md) — `rotation.rs`：姿态确定与控制（CMG/TRIAD/EKF/磁场力矩器）。
- [spaceflight-thermal.md](mps-core/spaceflight-thermal.md) — `thermal.rs`：热控（热管/辐射器/气闸/SPE 氧气/Whipple）。

## C ABI 层（src/rapier/ffi/）

- [ffi-mod.md](mps-core/ffi-mod.md) — `ffi/mod.rs`：FFI 模块根。
- [ffi-convert.md](mps-core/ffi-convert.md) — `ffi/convert.rs`：`Vec3`/`Quat` 与 Rapier `Vector`/`Rotation` 双向转换、pack/unpack 句柄。
- [ffi-types.md](mps-core/ffi-types.md) — `ffi/types.rs`：所有 `#[repr(C)]` ABI 结构体与句柄类型别名（WorldHandle、ColliderHandleRaw 等）。

## 共享竞技场（src/rapier/shared_arena/）

零拷贝 Rust↔外部 内存竞技场，供 `mps-jni` DirectByteBuffer 桥接。

- [shared-arena-mod.md](mps-core/shared-arena-mod.md) — `mod.rs`：`SharedPhysicsArena` 主结构与 C ABI 入口。
- [shared-arena-layout.md](mps-core/shared-arena-layout.md) — `layout.rs`：竞技场内存布局。
- [shared-arena-header.md](mps-core/shared-arena-header.md) — `header.rs`：竞技场头部元数据。
- [shared-arena-holes.md](mps-core/shared-arena-holes.md) — `holes.rs`：空闲槽位管理（holes 链表）。
- [shared-arena-ring.md](mps-core/shared-arena-ring.md) — `ring.rs`：事件环形缓冲区。

## 兼容、桥接与错误

- [compat.md](mps-core/compat.md) — `compat.rs`：旧 API 兼容的刚体插入与 AABB 查询。
- [bridge.md](mps-core/bridge.md) — `bridge.rs`：Rust↔Java 零拷贝内存桥（JNI DirectByteBuffer，`pub unsafe fn`，非 extern "C"）。
- [error.md](mps-core/error.md) — `error.rs`：线程局部错误槽、`ffi_guard`、`last_error_*` ABI、7 个 `ERR_*` 常量。

## 特性桥接

- [anvilkit.md](mps-core/anvilkit.md) — `anvilkit.rs`：`anvilkit-bridge` feature 下的 AnvilKit 集成（app create/update/destroy），`#[cfg(feature = "anvilkit-bridge")]` 门控。
