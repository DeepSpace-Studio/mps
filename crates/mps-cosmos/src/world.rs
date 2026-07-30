//! `CosmosWorld` — 基于 `rapier3d-f64` 的太空物理场景。
//!
//! 仿 [`mps_core::rapier::world::PhysicsWorld`] 的字段布局，但去掉 C ABI /
//! 共享 arena / 力律登记表 / 事件钩子，仅保留太空演练所需的 rapier 后端
//! 加上：
//! - 一组注册的天体引力源（[`CelestialSource`]）
//! - 一组参与 n-body 互引力的动态质点源（[`NBodySource`]）
//! - 可选环境扰动力（大气阻力、太阳光压）的 per-body 配置
//!
//! 推进循环 [`CosmosWorld::step`]：在每个物理子步之前，对所有动态刚体
//! 累加「天体引力 + n-body 互引力 + 环境扰动力」的合**力**（加速度 × 质量），
//! 然后交给 `PhysicsPipeline::step` 完成 Rapier 的常规积分/约束求解。

use crate::gravity::{
    CelestialSource, NBodySource, celestial_acceleration, gm_from_mass, n_body_acceleration,
};
use crate::orbit::BodyState;
use crate::perturbation::{atmospheric_drag_force, solar_pressure_force};
use mps_formula::celestial_data::AU;
use rapier3d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodyBuilder, RigidBodyHandle,
    RigidBodySet, Vector,
};

/// 单刚体的环境扰动配置。
#[derive(Clone, Copy, Debug)]
pub struct PerturbationConfig {
    /// 大气阻力系数 Cd。
    pub drag_coefficient: f64,
    /// 迎风截面积（m²）。
    pub area: f64,
    /// 是否施加该天体的大气阻力。需配合 `central_body` 设置才能取密度。
    pub enable_drag: bool,
    /// 光压系数 Cr。
    pub reflectivity: f64,
    /// 受光截面积（m²）。
    pub optical_area: f64,
    /// 是否施加太阳光压。
    pub enable_solar: bool,
}

impl Default for PerturbationConfig {
    fn default() -> Self {
        Self {
            drag_coefficient: 2.2,
            area: 0.0,
            enable_drag: false,
            reflectivity: 1.3,
            optical_area: 0.0,
            enable_solar: false,
        }
    }
}

/// 太空世界的配置。
#[derive(Clone, Debug)]
pub struct CosmosWorldConfig {
    /// 全局加速度锚（一般为 ZERO：太空场景无统一重力，引力由天体源贡献）。
    pub gravity: Vector,
    /// 积分步长（秒）。
    pub dt: f64,
    /// 约束求解迭代次数。
    pub solver_iterations: u32,
    /// CCD 子步数。
    pub ccd_substeps: u32,
    /// n-body 互引力的软化平方项（m²），避免两体无限接近时 1/r² 发散。
    ///
    /// 物理上引力的"硬截断"（`integrator.rs` 内 `dist_sq < 1.0` 跳过）只在距离
    /// <1m 时生效，对真实航天器间距永远不会触发；而两体近距离交会（如编队
    /// 飞行、对接接近）若 `softening_sq = 0`，1/r² 在数值上会瞬态冲高。
    /// 默认 `1e3` m²（约 31.6m 软化长度）——对千米级以上轨道间距无感，仅在
    /// 极近距离起到数值限幅。设为 `0.0` 则完全无软化（仅保留 1m 硬截断）。
    pub n_body_softening_sq: f64,
    /// n-body 中心天体（用于环境扰动力：大气密度/太阳方向参考）。
    /// 若为 `None` 则不施加基于中心天体的环境扰动。
    pub central_body: Option<&'static mps_formula::celestial_data::CelestialBody>,
    /// 轨道积分模式（默认沿用旧的 rapier 力注入路径）。
    ///
    /// - [`OrbitIntegration::RapierForce`]：把合力用 `add_force` 喂给 rapier，
    ///   走 semi-implicit Euler。简单但长弧相位误差大（1s 步长一圈 LEO 漂~700km）。
    /// - [`OrbitIntegration::Verlet`]：天体引力 + n-body 互引力用 velocity-Verlet
    ///   显式积分直接写回 `translation`/`linvel`，rapier 只负责碰撞/约束/姿态。
    ///   长弧精度提高一个量级。阻力/光压并入 Verlet 的加速度函数。
    pub orbit_integration: OrbitIntegration,
    /// Verlet 路径的内部子步数：一次 `step(dt)` 内做 `substeps` 次小步 Verlet，
    /// 每次 `dt/substeps` 秒。子步越多相位误差越小；1 内部子步即 Verlet 在 `dt` 内
    /// 走一整步。仅在 `orbit_integration = Verlet` 时生效。
    pub verlet_substeps: u32,
}

/// 轨道积分模式。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrbitIntegration {
    /// （默认）用 rapier 的 `add_force` 路径走 semi-implicit Euler。
    #[default]
    RapierForce,
    /// 显式 velocity-Verlet 积分轨道，rapier 只处理碰撞/姿态。
    Verlet,
}

impl Default for CosmosWorldConfig {
    fn default() -> Self {
        Self {
            gravity: Vector::ZERO,
            dt: 1.0 / 60.0,
            solver_iterations: 4,
            ccd_substeps: 4,
            n_body_softening_sq: 1e3,
            central_body: None,
            orbit_integration: OrbitIntegration::default(),
            verlet_substeps: 1,
        }
    }
}

/// 太空物理世界。所有公开 API 自行管理内部 `RigidBodySet` 等。
///
/// 手写 `Clone`（而非 derive）因为 `PhysicsPipeline` 不实现 `Clone`——它是
/// 无状态的工作对象（每次 `step` 内部重建临时结构），克隆时用 `::new()`
/// 恢复即可。用途：场景快照/回滚（演练器 undo、Monte Carlo 多世界并行）。
/// 成本是深拷贝整个 body/collider set；超大规模场景应考虑 `Arc` 共享只读配置。
pub struct CosmosWorld {
    pipeline: PhysicsPipeline,
    gravity: Vector,
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,

    celestials: Vec<CelestialSource>,
    n_body_sources: Vec<NBodySource>,
    n_body_softening_sq: f64,
    central_body: Option<&'static mps_formula::celestial_data::CelestialBody>,

    /// per-body 环境扰动配置，按 handle 的 arena index 存储。
    perturbations: Vec<Option<PerturbationConfig>>,
    /// 太阳在世界中的位置（用于光压方向），默认放在原点。
    sun_position: Vector,
    /// 轨道积分模式（见 [`CosmosWorldConfig::orbit_integration`]）。
    orbit_integration: OrbitIntegration,
    /// Verlet 子步数。
    verlet_substeps: u32,
}

/// 一次 `step` 的诊断结果。调用方原本只能靠 `step` 的静默 return 猜
/// "为什么没推进"，现在能直接判。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepResult {
    /// 正常推进了 `dt` 秒（RapierForce 路径下就是入参 dt；Verlet 路径下
    /// 是整步 dt，内部子步已自行处理）。
    Stepped(f64),
    /// 由于子步切分，被拆成 `n` 个 `dt/n` 秒小步完成（RapierForce 路径
    /// 下 `dt > MAX_STEP_DT` 时启用）。
    Substepped { substeps: u32, sub_dt: f64 },
    /// `dt` 非法（NaN / ≤0 / 超过单步上限），整步被跳过。
    Skipped(StepSkipReason),
}

/// `step` 跳过的原因。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepSkipReason {
    /// `dt` 为 NaN 或无穷。
    NonFinite,
    /// `dt <= 0`。
    NonPositive,
    /// `dt` 超过单步安全上限（当前 10s，防止误把"一帧"当"一小时"喂进来
    /// 后让积分发散）。需要更长推进请用 `step_n` 或循环 `step`。
    TooLarge,
}

/// 单步允许的最大 dt（秒）。超过则 RapierForce 路径会做子步切分以保精度；
/// Verlet 路径由 `verlet_substeps` 控制子步，不受此上限约束。
const MAX_STEP_DT: f64 = 10.0;

impl CosmosWorld {
    pub fn new(config: CosmosWorldConfig) -> Self {
        let integration_parameters = IntegrationParameters {
            dt: config.dt,
            num_solver_iterations: config.solver_iterations as usize,
            max_ccd_substeps: config.ccd_substeps as usize,
            ..IntegrationParameters::default()
        };
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: config.gravity,
            integration_parameters,
            islands: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            celestials: Vec::new(),
            n_body_sources: Vec::new(),
            n_body_softening_sq: config.n_body_softening_sq,
            central_body: config.central_body,
            perturbations: Vec::new(),
            sun_position: Vector::ZERO,
            orbit_integration: config.orbit_integration,
            verlet_substeps: config.verlet_substeps.max(1),
        }
    }

    /// 设太阳位置（光压方向参考）。
    pub fn set_sun_position(&mut self, pos: Vector) {
        self.sun_position = pos;
    }

    /// 设/改 n-body 中心天体（用于环境扰动力：大气密度/太阳方向参考）。
    /// 传 `None` 清除，则后续不施加基于中心天体的大气阻力。
    pub fn set_central_body(
        &mut self,
        body: Option<&'static mps_formula::celestial_data::CelestialBody>,
    ) {
        self.central_body = body;
    }

    /// 注册一个天体引力源。返回其索引便于后续移除/启用切换。
    pub fn add_celestial(&mut self, source: CelestialSource) -> usize {
        self.celestials.push(source);
        self.celestials.len() - 1
    }

    /// 注册一个 n-body 互引力质点源（给定质量 kg）。若刚体已插入，可直接
    /// 调 [`Self::add_n_body_handle`]。
    pub fn add_n_body(&mut self, handle: RigidBodyHandle, mass: f64) {
        let gm = gm_from_mass(mass);
        self.n_body_sources.push(NBodySource { handle, gm });
    }

    /// 设置某刚体的环境扰动配置。
    pub fn set_perturbation(&mut self, handle: RigidBodyHandle, cfg: PerturbationConfig) {
        let idx = handle.into_raw_parts().0 as usize;
        if idx >= self.perturbations.len() {
            self.perturbations.resize(idx + 1, None);
        }
        self.perturbations[idx] = Some(cfg);
    }

    /// 插入一个已配置好的刚体 builder，返回其句柄。
    pub fn insert_body(&mut self, builder: RigidBodyBuilder) -> RigidBodyHandle {
        let mut rb = builder.build();
        // Rapier builder 只把 additional_mass_properties 暂存到
        // `additional_local_mprops`，要等 pipeline.step 里的
        // `handle_user_changes_to_rigid_bodies` 才会并入 `local_mprops` 并据
        // 此计算 effective_inv_mass。在 step 之前调用方若立即需要 mass/受力
        // 大小正确，就显式重算一次。
        rb.recompute_mass_properties_from_colliders(&self.colliders);
        self.bodies.insert(rb)
    }

    /// 插入刚体并将其质量登记为 n-body 源（一步到位）。
    pub fn insert_body_as_gravity_source(
        &mut self,
        builder: RigidBodyBuilder,
        mass: f64,
    ) -> RigidBodyHandle {
        let handle = self.insert_body(builder);
        self.add_n_body(handle, mass);
        handle
    }

    /// 取刚体当前位置。
    pub fn body_translation(&self, handle: RigidBodyHandle) -> Option<Vector> {
        self.bodies.get(handle).map(|b| b.translation())
    }

    /// 取刚体线速度。
    pub fn body_linvel(&self, handle: RigidBodyHandle) -> Option<Vector> {
        self.bodies.get(handle).map(|b| b.linvel())
    }

    /// 取刚体质量。
    pub fn body_mass(&self, handle: RigidBodyHandle) -> Option<f64> {
        self.bodies.get(handle).map(|b| b.mass())
    }

    /// 取刚体完整状态切片（用于轨道诊断）。
    pub fn body_state(&self, handle: RigidBodyHandle) -> Option<BodyState> {
        self.bodies
            .get(handle)
            .map(|b| BodyState::new(b.translation(), b.linvel()))
    }

    /// 当前动态刚体数量。
    pub fn dynamic_body_count(&self) -> usize {
        self.bodies.iter().filter(|(_, b)| b.is_dynamic()).count()
    }

    /// 推进一个步长。
    ///
    /// 按 `orbit_integration` 配置走两条路径之一：
    /// - `RapierForce`：把合力用 `add_force` 注入，rapier 内部 semi-implicit Euler。
    ///   `dt > MAX_STEP_DT` 时内部自动拆成若干 ≤`MAX_STEP_DT` 的子步，每子步
    ///   重注入力，返回 [`StepResult::Substepped`]。
    /// - `Verlet`：天体引力 + n-body 用 velocity-Verlet 显式积分写回 translation/linvel，
    ///   rapier 只跑碰撞/约束/姿态。子步数由 `verlet_substeps` 控制。
    ///
    /// 返回 [`StepResult`]，调用方可据此判断"为什么没推进"。
    pub fn step(&mut self, dt: f64) -> StepResult {
        if !dt.is_finite() {
            return StepResult::Skipped(StepSkipReason::NonFinite);
        }
        if dt <= 0.0 {
            return StepResult::Skipped(StepSkipReason::NonPositive);
        }
        if dt > 30.0 {
            // 硬上限：30s 以上的单步一律拒，防止误把"一帧"当"一小时"喂进来
            // 让积分发散。真正超长推进请循环 step / 用 step_n。
            return StepResult::Skipped(StepSkipReason::TooLarge);
        }

        match self.orbit_integration {
            OrbitIntegration::RapierForce => {
                if dt > MAX_STEP_DT {
                    // dt 介于 MAX_STEP_DT(10s) 与硬上限(30s) 之间：自动拆成
                    // 每段 ≤ MAX_STEP_DT 的子步，每子步重注入力以保精度。
                    let substeps = (dt / MAX_STEP_DT).ceil() as u32;
                    let sub_dt = dt / substeps as f64;
                    for _ in 0..substeps {
                        self.step_via_rapier_force(sub_dt);
                    }
                    StepResult::Substepped { substeps, sub_dt }
                } else {
                    self.step_via_rapier_force(dt);
                    StepResult::Stepped(dt)
                }
            }
            OrbitIntegration::Verlet => {
                self.step_via_verlet(dt);
                StepResult::Stepped(dt)
            }
        }
    }

    /// 批量推进 `n` 个步长，每步 `dt` 秒。等价于循环 `step(dt)`，但把 `dt`
    /// 合法性校验前置一次。返回累计诊断：
    /// - `Ok(())`：所有步都正常推进。
    /// - `Err(reason)`：`dt` 非法，整批未推进。
    pub fn step_n(&mut self, dt: f64, n: u32) -> Result<(), StepSkipReason> {
        if !dt.is_finite() {
            return Err(StepSkipReason::NonFinite);
        }
        if dt <= 0.0 {
            return Err(StepSkipReason::NonPositive);
        }
        if dt > 30.0 {
            return Err(StepSkipReason::TooLarge);
        }
        for _ in 0..n {
            // step 内部对合法 dt 不会再返回 Skipped，这里丢弃每步的 StepResult。
            let _ = self.step(dt);
        }
        Ok(())
    }

    /// 取所有 n-body 源（只读）。
    pub fn n_body_sources(&self) -> &[NBodySource] {
        &self.n_body_sources
    }

    /// 取所有天体引力源（只读）。
    pub fn celestials(&self) -> &[CelestialSource] {
        &self.celestials
    }

    /// 取内部 `RigidBodySet`（只读）——供外部诊断/快照用（如
    /// [`crate::integrator::snapshot_source_positions`] 需要遍历体位置）。
    pub fn bodies(&self) -> &RigidBodySet {
        &self.bodies
    }

    /// 取 n-body 互引力的软化平方项（m²）。
    pub fn n_body_softening_sq(&self) -> f64 {
        self.n_body_softening_sq
    }

    /// 取太阳位置（光压方向参考）。
    pub fn sun_position(&self) -> Vector {
        self.sun_position
    }

    /// 旧路径：力注入 → rapier 推进。
    fn step_via_rapier_force(&mut self, dt: f64) {
        // 把 integration_parameters.dt 对齐到本次子步实际 dt；rapier.pipeline.step
        // 内部所有积分都读这个值。
        self.integration_parameters.dt = dt;
        // 0. 每步先清掉上一轮累积的 user_force。Rapier 不会自动重置
        //    add_force 累加进去的力（见 rapier `reset_forces` 文档）。
        for b in self.bodies.iter_mut() {
            if b.1.is_dynamic() {
                b.1.reset_forces(false);
                b.1.reset_torques(false);
            }
        }

        // 1. 计算并注入每体的合力（天体引力 + n-body + 环境扰动）
        self.apply_forces();

        // 2. Rapier 推进
        self.pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    /// 新路径：velocity-Verlet 显式积分轨道，rapier 仍跑碰撞/约束/姿态。
    fn step_via_verlet(&mut self, dt: f64) {
        // 轨道积分一次性把 (translation, linvel) 推进 dt。
        //
        // 关键：rapier 的 `pipeline.step` 末尾 `advance_to_final_positions` 会用
        // solver 内部（按 forces/damping）积分得到 `next_position` **覆盖**
        // `pos.position`，把 Verlet 写回的 translation 一次性抹掉。为避免这种
        // 窜改，Verlet 路径下我们不调用 `pipeline.step`，而是手写一个最小推进：
        //   1. Verlet 子步把 translation/linvel 推进 dt（写 `pos.position` 且同步
        //      `pos.next_position = pos.position`，让任何下游读 next_position 也对齐）。
        //   2. collider 跟随刚体位移更新（无 collider 时空跑）。
        //   3. 姿态/角速度按 damping 单独积分（与 rapier 的 writeback 等价的最简形式）。
        //
        // 暂不处理碰撞/关节约束求解 —— 太空场景默认不插入 collider，约束为空；
        // 若未来需要在 Verlet 路径下处理对接约束，应在此处插入一次 velocity-only
        // 的约束求解，避免 advance_to_final_positions 把 Verlet 位置覆盖。
        let substeps = self.verlet_substeps.max(1) as usize;
        let sub_dt = dt / substeps as f64;

        for _ in 0..substeps {
            self.verlet_substep(sub_dt);
        }

        // 同步 colliders 位置（若有）。rapier 的 collider 位置由
        // `collider.parent_handle → body.position` 链路派生，需要手动 propagate。
        // 这里只做最简单的"刚体位移增量覆盖 collider"——对太空场景已足够。
        self.sync_colliders_after_verlet();
    }

    /// Verlet 路径结束后的 collider 同步：
    /// rapier 的 `ColliderSet` 不会在没跑 pipeline 时自动跟随刚体。
    /// 对"挂在刚体上"的 collider（`parent` 非空），其 world 位姿 =
    /// `parent_body.position * pos_wrt_parent`；这里按这条链路重算写回。
    fn sync_colliders_after_verlet(&mut self) {
        // 先快照 (collider_handle, parent_handle, offset)，再写回，避开同时借用。
        let updates: Vec<_> = self
            .colliders
            .iter()
            .filter_map(|(h, co)| {
                co.parent().and_then(|ph| {
                    self.bodies
                        .get(ph)
                        .map(|b| (h, ph, b.position(), co.position_wrt_parent().copied()))
                })
            })
            .collect();
        for (handle, _parent, parent_pos, offset) in updates {
            if let Some(co) = self.colliders.get_mut(handle) {
                // world = parent_world * offset
                let world = parent_pos * offset.unwrap_or_default();
                co.set_position(world);
            }
        }
    }

    /// 一次 Verlet 子步：对所有动态刚体做 velocity-Verlet 推进。
    fn verlet_substep(&mut self, dt: f64) {
        // 1. 收集动态体快照 (handle, pos, vel, mass) —— 不可在迭代 bodies 时
        //    可变借用，故先快照。
        let snapshot: Vec<(RigidBodyHandle, Vector, Vector, f64)> = self
            .bodies
            .iter()
            .filter(|(_, b)| b.is_dynamic())
            .map(|(h, b)| (h, b.translation(), b.linvel(), b.mass()))
            .collect();

        // 2. n-body 源位置快照（按 arena index O(1) 查）
        let source_positions =
            crate::integrator::snapshot_source_positions(&self.bodies, &self.n_body_sources);

        // 3. 构造本子步共享的 AccelContext —— 所有体共享一份只读环境（天体源、
        //    n-body 源、源位置快照、软化平方、中心天体、太阳位置），闭包按引用
        //    capture，不再每体 to_vec 克隆（旧实现每体 O(M) 复制，n 体子步共
        //    O(n·M)；现在 O(M) 一次性）。
        let ctx = crate::integrator::AccelContext {
            celestials: &self.celestials,
            n_body_sources: &self.n_body_sources,
            source_positions: &source_positions,
            softening_sq: self.n_body_softening_sq,
            central_body: self.central_body,
            sun_position: self.sun_position,
        };

        // 4. 算每体的 a0（旧位置处加速度）。per-body 的 perturbation 在此取一份
        //    owned（Copy）即可，handle/mass 直接带值。
        let tasks: Vec<(RigidBodyHandle, Vector, f64, Option<PerturbationConfig>)> = snapshot
            .iter()
            .map(|&(handle, pos, vel, mass)| {
                let perturbation = self.perturbation_for(handle).copied();
                let a0 = crate::integrator::total_acceleration(
                    pos,
                    vel,
                    mass,
                    handle,
                    &ctx,
                    perturbation.as_ref(),
                );
                (handle, a0, mass, perturbation)
            })
            .collect();

        // 5. 不可变借用结束（tasks 不再持 self 任一字段），现在可变写回 bodies。
        //    verlet_step 内部的 a1 评估复用同一 ctx（按引用），无额外克隆。
        for (handle, a0, mass, perturbation) in tasks {
            if let Some(body) = self.bodies.get_mut(handle) {
                crate::integrator::verlet_step(body, a0, &ctx, mass, handle, perturbation, dt);
            }
        }
    }

    fn apply_forces(&mut self) {
        // 收集动态刚体 (handle, position, velocity, mass) 以避免在迭代 bodies 时
        // 同时可变借用。
        let snapshot: Vec<(RigidBodyHandle, Vector, Vector, f64)> = self
            .bodies
            .iter()
            .filter(|(_, b)| b.is_dynamic())
            .map(|(h, b)| (h, b.translation(), b.linvel(), b.mass()))
            .collect();

        // n-body 源位置快照：按 arena index 直查 O(1)，替代旧的
        // `Vec<(handle,pos)> + find` 的 O(n²) 路径（与 integrator 模块共用同一 helper）。
        let source_positions =
            crate::integrator::snapshot_source_positions(&self.bodies, &self.n_body_sources);
        let source_pos_lookup = |h: RigidBodyHandle| -> Vector {
            let idx = h.into_raw_parts().0 as usize;
            source_positions.get(idx).copied().unwrap_or(Vector::ZERO)
        };

        for (handle, pos, vel, mass) in snapshot {
            let mut total_force = Vector::ZERO;

            // 天体引力：加速度 × 质量
            for src in &self.celestials {
                let accel = celestial_acceleration(pos, src);
                total_force += accel * mass;
            }

            // n-body 互引力
            if !self.n_body_sources.is_empty() {
                let accel = n_body_acceleration(
                    pos,
                    &self.n_body_sources,
                    handle,
                    source_pos_lookup,
                    self.n_body_softening_sq,
                );
                total_force += accel * mass;
            }

            // 环境扰动
            if let Some(cfg) = self.perturbation_for(handle) {
                if cfg.enable_drag {
                    if let Some(central) = self.central_body {
                        let altitude = pos.length() - central.equatorial_radius;
                        let density = crate::perturbation::atmosphere_density_at(central, altitude);
                        if density > 0.0 {
                            let atmosphere_vel = angular_velocity_of(central).cross3(pos);
                            if let Some(f) = atmospheric_drag_force(
                                vel,
                                atmosphere_vel,
                                density,
                                cfg.drag_coefficient,
                                cfg.area,
                                mass,
                            ) {
                                total_force += f;
                            }
                        }
                    }
                }
                if cfg.enable_solar && cfg.optical_area > 0.0 {
                    let sun_to_body = pos - self.sun_position;
                    let r = sun_to_body.length();
                    let sun_dir = if r > 1e-9 {
                        -sun_to_body / r
                    } else {
                        Vector::ZERO
                    };
                    total_force += solar_pressure_force(
                        sun_to_body,
                        sun_dir,
                        cfg.optical_area,
                        cfg.reflectivity,
                        AU,
                    );
                }
            }

            if let Some(body) = self.bodies.get_mut(handle) {
                if total_force != Vector::ZERO {
                    body.add_force(total_force, true);
                }
            }
        }
    }

    fn perturbation_for(&self, handle: RigidBodyHandle) -> Option<&PerturbationConfig> {
        let idx = handle.into_raw_parts().0 as usize;
        self.perturbations.get(idx).and_then(|c| c.as_ref())
    }
}

impl Clone for CosmosWorld {
    /// 深拷贝整个物理状态。`PhysicsPipeline` 不实现 `Clone`——它是无状态
    /// 工作对象（每次 `step` 重建临时结构），克隆用 `::new()` 恢复。
    fn clone(&self) -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: self.gravity,
            integration_parameters: self.integration_parameters,
            islands: self.islands.clone(),
            broad_phase: self.broad_phase.clone(),
            narrow_phase: self.narrow_phase.clone(),
            bodies: self.bodies.clone(),
            colliders: self.colliders.clone(),
            impulse_joints: self.impulse_joints.clone(),
            multibody_joints: self.multibody_joints.clone(),
            ccd_solver: self.ccd_solver.clone(),
            celestials: self.celestials.clone(),
            n_body_sources: self.n_body_sources.clone(),
            n_body_softening_sq: self.n_body_softening_sq,
            central_body: self.central_body,
            perturbations: self.perturbations.clone(),
            sun_position: self.sun_position,
            orbit_integration: self.orbit_integration,
            verlet_substeps: self.verlet_substeps,
        }
    }
}

/// 由天体自转速率与位置近似出大气的惯性系速度 `ω × r`。
/// 这里用叉乘；`^` 在 nalgebra 上也是叉乘别名，但为清晰用显式实现。
fn angular_velocity_of(body: &mps_formula::celestial_data::CelestialBody) -> Vector {
    // 简化：假设自转轴沿 +z，速率 = rotation_rate。
    // 真实模型可后续细化；当前足以给出赤道大气速度的方向。
    Vector::new(0.0, 0.0, body.rotation_rate)
}

// 显式实现 ω × r，避免依赖 nalgebra 的 `^` 运算符可读性。
trait CrossR {
    fn cross3(self, other: Vector) -> Vector;
}
impl CrossR for Vector {
    fn cross3(self, o: Vector) -> Vector {
        Vector::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
}
