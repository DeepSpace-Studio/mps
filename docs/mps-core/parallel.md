# rapier/parallel.rs

## 作用
mps-core 每帧工作的多线程执行支持。背景:rapier 的 `parallel` feature 已经把 `PhysicsPipeline::step` 内部的碰撞检测/求解器放到 rayon 线程池上,但 step 外围的一切——力法则求值、地形引力采样(每体 O(faces)/O(格网))、O(n²) 成对引力、外部力施加、渲染循环的快照导出——此前全是单线程;在多动态体或重地形引力源的场景下这些串行段主导帧预算。

设计是**两段式逐体求值**:每个力法则本来就有「填充(只读 bodies 计算每体力)→ 施加(`ForceFacade::add_force` 改写刚体)」两段。填充段通过 `par_map_bodies` 把逐体闭包映射到 rayon 池上并**保序**,施加段保持串行回放。保序使并行输出与串行**位一致**(每体力由恰好一个任务以不变的算术序列产生),因此阈值切换永远不会改变仿真结果。

唯一有跨体耦合的法则——牛顿成对引力——用 `pairwise_gravity_accumulate`:刚体切成固定的 128 体 chunk,每个 chunk 对 `(a, b), a ≤ b` 由一个 rayon 任务把跨对力累加进各自不相交的缓冲,最后按固定字典序归并。归并序与线程数/调度无关,结果完全确定(相对串行上三角循环只有浮点重结合的末位差异)。

低于阈值(默认 `PAR_MIN_ITEMS` = 128,可按调用点收紧/放宽)一律在调用线程上串行执行——小场景保持完全的旧行为、零调度开销。

线程池:所有并行段跑在 rayon 全局池上,与 rapier `parallel` feature 共用;力填充与 `pipeline.step` 在时间上不重叠,共享池不会超额订阅。池大小默认为全部逻辑核,可按优先级用 `parallel_set_thread_count` FFI(仅首次使用前生效)、`MPS_CORE_THREADS`(首次使用时读一次)或 `RAYON_NUM_THREADS`(rayon 原生,始终生效)配置。

并行化落点(按改造位置):
- `interaction.rs` 全部 8 个逐体法则(AirDrag、SolarWindPressure、DynamicalFriction、MonDGravity、Eddington、Xray、PulsarMagneticDipole 双桶、JeansEscape)的填充循环;Pulsar 的 `max_re` 类归约用 f64::max 折叠(精确、与串行位一致)。
- `terrain_gravity.rs::TerrainGravityLaw`(最重的逐体采样,阈值 `TERRAIN_GRAVITY_MIN_ITEMS` = 32)。
- `events.rs::apply_custom_external_forces_with_facade`(浮力/电磁/弹簧/点引力,每体产出 ≤4 条 PendingForce,展平顺序与串行一致)。
- `interaction.rs::NewtonianGravityForceLaw`(chunk-pair 分解,阈值 `GRAVITY_PAR_MIN_BODIES` = 256;以下保持串行上三角位一致路径)。
- `world.rs` 的 `world_body_snapshot` / `world_dynamic_body_snapshot` FFI(先串行收集 handle,并行算每体 13-f64 位姿,串行回写调用方缓冲)。

未并行化(有意保留串行):step 内 soft-body/fluid/granular 代理同步(需要 `get_mut` 逐代理写,RigidBodySet 无并行可变访问 API)、step 后 `reset_forces` 循环(O(n) 纯置零)、Coulomb hook 重扫(稳态 O(1))。旧版 `pairwise_gravity` / `per_body_air_drag` 自由函数仅测试引用,保留作为串行参考实现。

## 关键导出
- `const PAR_MIN_ITEMS`(128)、`GRAVITY_CHUNK`(128)、`GRAVITY_PAR_MIN_BODIES`(256)、`TERRAIN_GRAVITY_MIN_ITEMS`(32)——并行阈值;chunk 大小固定(不随池宽变化)以保证位级确定性。
- `fn thread_count() -> u32` — 全局池线程数(惰性建池)。
- `fn set_thread_count(n: u32) -> bool` — 尝试重设全局池;池已初始化后失败。
- `fn par_map_bodies(handles, bodies, min_items, compute) -> Vec<R>` — 逐体并行映射核心 helper;`compute: Fn(RigidBodyHandle, &RigidBody) -> R + Sync` 纯只读;低于 `min_items` 或单核时串行,两条路径位一致。
- `fn pairwise_gravity_accumulate(body_data, g, min_dist, max_dist_sq) -> Vec<Vector>` — chunk-pair 并行 O(n²) 成对引力累加(牛顿第三定律,每对只算一次),确定性有序归并。
- `fn body_pose_snapshots(handles, bodies, with_velocity) -> Vec<[f64; 13]>` — 快照导出 helper(pos3+quat4+linvel3+angvel3)。
- `extern "C"` 入口(2 项):`parallel_thread_count`、`parallel_set_thread_count`。

## 依赖
- 外部 crate:`rayon::prelude`(全局池 + `par_iter`)、`rapier3d::prelude::{RigidBody, RigidBodyHandle, RigidBodySet}`(`&RigidBodySet` 跨线程共享读已被 `bridge.rs` 先例验证)。
- 本 crate 子模块:`crate::rapier::error`(ffi_guard/set_error/clear_error)、`crate::rapier::ffi`(Bool/Vec3/Quat 转换、pack)。
- 测试:`mps-test/src/rapier/parallel.rs` 10 个用例——并行/串行阈值两侧对照串行参考(空中阻力、成对引力、外部力、地形引力、脉冲星力矩)、跨运行确定性、快照与真实状态逐值一致、线程池 FFI 往返。
