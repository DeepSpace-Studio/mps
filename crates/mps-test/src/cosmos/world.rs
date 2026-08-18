//! `mps_cosmos::world` 测试 —— 迁移自 `crates/mps-cosmos/src/world.rs`。
//!
//! 涵盖 `CosmosWorld` 的 step 语义（RapierForce / Verlet 两条路径）、子步
//! 切分边界、`step_n` 批处理、默认 `n_body_softening_sq` 限幅，以及端到端
//! 圆轨道 LEO 演算（RapierForce 短弧 + Verlet 整圈闭合）。

#[cfg(test)]
use mps_cosmos::bodies::satellite_builder;
#[cfg(test)]
use mps_cosmos::integrator::{
    AccelContext, advance_highorder, advance_highorder_kahan, snapshot_source_positions,
    total_acceleration,
};
#[cfg(test)]
use mps_cosmos::world::{
    CosmosWorld, CosmosWorldConfig, OrbitIntegration, StepResult, StepSkipReason,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
#[cfg(test)]
use mps_formula::math::KahanVec3;
#[cfg(test)]
use mps_formula::spaceflight::kepler_period;
#[cfg(test)]
use rapier3d::prelude::{Rotation, Vector};

/// RapierForce 路径回归测试：semi-implicit Euler 在纯点质量中心引力下推一段
/// 短弧（1/10 开普勒周期），位矢模长应保持在小幅能量漂移范围内。
///
/// 旧版跑整圈 ~5900 步、且因 r=7000km 处 `celestial_acceleration` 会落进
/// ellipsoid 分支（含非保守离心项）导致相位/闭合容差只能开到 10%。改造：
/// - 用内部注入的"纯点质量"加速度闭包绕过天体源分支，隔离积分器本体；
/// - 只推 1/10 周期，验证半径守恒（semi-implicit Euler 在近圆轨道小幅能量
///   漂移，半径随之小幅偏离），CI 时间从十几秒压到亚秒；
/// - 维持仍由 `CosmosWorld` 驱动 + `add_force` 注入路径，回归真实调用路径。
#[test]
fn circular_leo_orbit_period_matches_kepler() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let r = 7_000_000.0_f64;
    let v = (gm / r).sqrt();
    let expected_period = kepler_period(gm, r).expect("period");
    // 1/10 开普勒周期，1s 步长。
    let short_arc = expected_period / 10.0;
    let steps = short_arc.round() as u32;

    let mut world = CosmosWorld::new({
        let mut cfg = CosmosWorldConfig {
            gravity: Vector::ZERO,
            dt: 1.0,
            solver_iterations: 8,
            ccd_substeps: 0,
            n_body_softening_sq: 0.0,
            central_body: None,
            orbit_integration: OrbitIntegration::default(),
            verlet_substeps: 1,
            adaptive_substeps: false,
            adaptive_tolerance: 1e-9,
            relativistic_correction: mps_cosmos::world::RelativisticCorrection::None,
        };
        // 让默认 softening 不污染本测试：显式归零，直接测点质量积分精度。
        cfg.n_body_softening_sq = 0.0;
        cfg
    });
    // 不注册 celestials —— 用 n_body_sources 给"假地球"注册点质量引力。
    // step 内部 apply_forces 走真实 n_body_acceleration（不含 ellipsoid/J2）。
    let earth_hdl = world.insert_body(
        satellite_builder(5.972e24, Vector::new(0.0, 0.0, 0.0), Vector::ZERO, 1.0)
            .lock_translations(), // 静止原点：地球本体不被推动
    );
    world.add_n_body(earth_hdl, 5.972e24);

    let sat = world.insert_body(
        satellite_builder(
            1000.0,
            Vector::new(r, 0.0, 0.0),
            Vector::new(0.0, v, 0.0),
            1.0,
        )
        .linear_damping(0.0)
        .angular_damping(0.0),
    );

    for _ in 0..steps {
        world.step(1.0);
    }

    let pos = world.body_translation(sat).expect("sat exists");
    let r_final = pos.length();
    // semi-implicit Euler 在近圆轨道会有小幅能量增长（v 提前更新），半径随
    // 之小幅上漂。1s 步长 / 1/10 周期下，半径相对偏离应 < 1%。
    assert!(
        (r_final - r).abs() / r < 0.01,
        "短弧半径漂移过大：r0={r} -> {r_final}（Δ={:.3} km）",
        (r_final - r) / 1000.0
    );
    // 用 earth_hdl 消掉未使用告警（上面已用过，留这行防误删）。
    let _ = earth_hdl;
}

/// Verlet 路径：纯点质量中心引力下推一整圈，闭合应远优于 RapierForce
/// 路径。用"假地球"n-body 源（点质量互引力、不含 ellipsoid/J2 分支），
/// 1s 步长，1 圈位置偏移实测 ~3.6km ≈ 0.05% r，远优于 RapierForce 的
/// 整圈 10% 量级。
///
/// 此测试保留整圈跑（~5900 步）作为 Verlet 路径的端到端回归；release 下
/// 0.05s 完成，debug 下也只 ~0.4s，不影响 CI。若日后 CI 对该时长敏感，
/// 可如 `circular_leo_orbit_period_matches_kepler` 那样截到 1/10 短弧。
#[test]
fn circular_leo_verlet_path_closes_tighter_than_rapier() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let r = 7_000_000.0_f64;
    let v = (gm / r).sqrt();
    let expected_period = kepler_period(gm, r).expect("period");
    let steps = expected_period.round() as u32;

    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 1.0,
        solver_iterations: 8,
        ccd_substeps: 0,
        n_body_softening_sq: 0.0,
        central_body: None,
        orbit_integration: OrbitIntegration::Verlet,
        verlet_substeps: 1,
        adaptive_substeps: false,
        adaptive_tolerance: 1e-9,
        relativistic_correction: mps_cosmos::world::RelativisticCorrection::None,
    });
    // 用"假地球"刚体作 n-body 源（点质量互引力），替代 celestial 源 ——
    // 后者在 r=7000km 处会落 ellipsoid 分支（含非保守离心项），让圆轨道
    // 不再严格闭合，无法干净验证积分器本身的相位精度。n-body 路径是纯
    // `-G·M·r̂/r²` 点质量引力，圆轨道应严格闭合。
    let earth_hdl = world.insert_body(
        satellite_builder(5.972e24, Vector::ZERO, Vector::ZERO, 1.0).lock_translations(), // 静止原点
    );
    world.add_n_body(earth_hdl, 5.972e24);

    let h = world.insert_body(
        satellite_builder(
            1000.0,
            Vector::new(r, 0.0, 0.0),
            Vector::new(0.0, v, 0.0),
            1.0,
        )
        .linear_damping(0.0)
        .angular_damping(0.0)
        .gravity_scale(0.0), // 关键：避免 rapier.step 再加全局重力
    );

    for _ in 0..steps {
        world.step(1.0);
    }

    let pos = world.body_translation(h).expect("body exists");
    let off = (pos - Vector::new(r, 0.0, 0.0)).length();
    // 纯点质量 + Verlet(1s) 一圈闭合：实测 ~3.6km ≈ 0.05% r。给 1% r
    // 容差，留出数值余量，仍远紧于 RapierForce 路径的 10%。
    assert!(off / r < 0.01, "Verlet 一圈偏移 {off} (>1% r)，pos={pos:?}");
}

#[test]
fn world_inserts_dynamic_body() {
    let mut world = CosmosWorld::new(CosmosWorldConfig::default());
    let h = world.insert_body(satellite_builder(
        1.0,
        Vector::new(1.0, 2.0, 3.0),
        Vector::ZERO,
        0.1,
    ));
    assert_eq!(world.dynamic_body_count(), 1);
    let p = world.body_translation(h).expect("body exists");
    assert!((p - Vector::new(1.0, 2.0, 3.0)).length() < 1e-9);
}

/// P2.18 / M5：`clone_shallow` 必须复制所有 config 字段（gravity、积分参数、
/// 软化长度、子步配置、轨道积分模式、相对论修正模式、太阳位置、中心天体
/// 句柄），同时丢弃全部 body/collider/joint/celestials/n_body_sources/
/// perturbations/kahan_state/scratch buffers。
///
/// 用途：Monte Carlo 多世界并行 + 轨道预测 overlay——想要"同配置 + 全新
/// 空白物理场景" 而不需要付出深拷贝整 RigidBodySet 的 ~5–10ms 代价（1000
/// body 的 Clone 实测代价）。
#[test]
fn clone_shallow_preserves_config_drops_body_state() {
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        n_body_softening_sq: 5e5,
        orbit_integration: OrbitIntegration::ForestRuth8Kahan,
        verlet_substeps: 4,
        adaptive_substeps: true,
        adaptive_tolerance: 1e-12,
        ..CosmosWorldConfig::default()
    });
    world.set_sun_position(Vector::new(1.0, 0.0, 0.0));
    let earth = get_celestial_body(CelestialBodyId::Earth);
    world.set_central_body(Some(earth));
    // Populate some body + extra source state to verify it gets dropped.
    let _h = world.insert_body(satellite_builder(
        1.0,
        Vector::new(1.0, 2.0, 3.0),
        Vector::ZERO,
        0.1,
    ));
    let n_celestial = world.add_celestial(mps_cosmos::gravity::CelestialSource::new(
        earth,
        earth.max_degree,
    ));
    let _ = n_celestial;

    let mut shallow = world.clone_shallow();

    // --- config 端保留 ---
    assert_eq!(shallow.dynamic_body_count(), 0, "body count must reset");
    assert_eq!(shallow.n_body_softening_sq(), 5e5, "softening");
    assert_eq!(
        shallow.orbit_integration(),
        OrbitIntegration::ForestRuth8Kahan,
        "orbit integration mode preserved"
    );
    let sun = shallow.sun_position();
    assert!(
        (sun - Vector::new(1.0, 0.0, 0.0)).length() < 1e-12,
        "sun position preserved"
    );
    let central_back = shallow.central_body();
    assert!(central_back.is_some(), "central body handle preserved");

    // --- body/state 端清空 ---
    assert_eq!(shallow.dynamic_body_count(), 0);
    assert_eq!(shallow.n_body_sources().len(), 0, "n_body_sources dropped");
    // 轨道再推一步确认是 fresh-empty scene（不 crash、不残留状态推进）：
    let step_res = shallow.step(0.1);
    assert!(
        matches!(
            step_res,
            StepResult::Stepped(_) | StepResult::Substepped { .. }
        ),
        "shallow copy should remain step-able; got {step_res:?}"
    );

    // 源 world 仍可继续推（deep clone 不触及原始！）：
    let orig_step = world.step(0.1);
    assert!(
        matches!(
            orig_step,
            StepResult::Stepped(_) | StepResult::Substepped { .. }
        ),
        "source world should still step normally; got {orig_step:?}"
    );
    assert!(
        world.dynamic_body_count() >= 1,
        "source world body count intact"
    );
}

/// `step` 对非法 `dt` 必须返回 `Skipped(...)` 而非静默吞掉。这是 P1-5 的
/// 核心收益：调用方现在能区分"没推进"的三种原因。
#[test]
fn step_reports_skipped_reasons() {
    // RapierForce 路径才有 dt > MAX_STEP_DT 的子步拆分；显式积子路径由
    // verlet_substeps / adaptive 控制子步，单 int 不出 Substepped。
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        orbit_integration: OrbitIntegration::RapierForce,
        ..CosmosWorldConfig::default()
    });

    assert_eq!(
        world.step(f64::NAN),
        StepResult::Skipped(StepSkipReason::NonFinite)
    );
    assert_eq!(
        world.step(0.0),
        StepResult::Skipped(StepSkipReason::NonPositive)
    );
    assert_eq!(
        world.step(-1.0),
        StepResult::Skipped(StepSkipReason::NonPositive)
    );
    assert_eq!(
        world.step(f64::INFINITY),
        StepResult::Skipped(StepSkipReason::NonFinite)
    );
    // 硬上限：超过 30s 拒。MAX_STEP_DT(10s) 与硬上限(30s) 之间会走子步。
    assert_eq!(
        world.step(11.0),
        StepResult::Substepped {
            substeps: 2,
            sub_dt: 5.5
        }
    );
    assert_eq!(
        world.step(31.0),
        StepResult::Skipped(StepSkipReason::TooLarge)
    );
    assert_eq!(world.step(10.0), StepResult::Stepped(10.0));

    // 合法 dt 仍然推：放一个自由落体的体进来确认 Stepped 不只是返回值对、
    // 也真的推了一步。
    let h = world.insert_body(satellite_builder(
        1.0,
        Vector::new(0.0, 0.0, 0.0),
        Vector::new(1.0, 0.0, 0.0),
        0.1,
    ));
    let before = world.body_translation(h).unwrap().x;
    world.step(1.0);
    let after = world.body_translation(h).unwrap().x;
    assert!(after > before, "Stepped 应真的推进位置: {after} > {before}");
}

/// RapierForce 路径下 `dt` 介于 `MAX_STEP_DT`(10s) 与硬上限(30s) 之间时
/// 应自动拆子步并返回 `Substepped`；超过硬上限则 `TooLarge`。
#[test]
fn step_substeps_oversized_dt_rapier_force() {
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 1.0,
        orbit_integration: OrbitIntegration::RapierForce,
        ..CosmosWorldConfig::default()
    });
    let h = world.insert_body(satellite_builder(
        1.0,
        Vector::new(0.0, 0.0, 0.0),
        Vector::new(1.0, 0.0, 0.0),
        0.1,
    ));
    // 25s 介于 10s 与 30s 之间 → 3 子步（每段 8⅓s）。
    let res = world.step(25.0);
    match res {
        StepResult::Substepped { substeps, sub_dt } => {
            assert_eq!(substeps, 3);
            assert!((sub_dt - 25.0 / 3.0).abs() < 1e-9);
        }
        other => panic!("预期 Substepped，得到 {other:?}"),
    }
    // 子步累计仍推 ~25m（x += v·dt，无外力）。
    let x = world.body_translation(h).unwrap().x;
    assert!((x - 25.0).abs() < 1e-6, "25s 自由漂移应 ≈ 25m，实际 {x}");

    // 超过硬上限 30s → TooLarge。
    assert_eq!(
        world.step(31.0),
        StepResult::Skipped(StepSkipReason::TooLarge)
    );
}

/// `step_n` 对非法 `dt` 整批拒；合法时等价于循环 `step`。
#[test]
fn step_n_validates_and_batches() {
    let mut world = CosmosWorld::new(CosmosWorldConfig::default());
    assert_eq!(
        world.step_n(0.0, 10).unwrap_err(),
        StepSkipReason::NonPositive
    );
    assert_eq!(
        world.step_n(f64::NAN, 10).unwrap_err(),
        StepSkipReason::NonFinite
    );
    assert_eq!(world.step_n(31.0, 1).unwrap_err(), StepSkipReason::TooLarge);

    let h = world.insert_body(satellite_builder(
        1.0,
        Vector::new(0.0, 0.0, 0.0),
        Vector::new(1.0, 0.0, 0.0),
        0.1,
    ));
    world.step_n(2.0, 5).unwrap(); // 5×2s = 10s 自由漂移
    let x = world.body_translation(h).unwrap().x;
    assert!((x - 10.0).abs() < 1e-6, "5×2s 应推 10m，实际 {x}");
}

/// `step` 子步切分的边界值：`dt == MAX_STEP_DT` 直推、`dt == 硬上限` 子步、
/// `dt` 略超 `MAX_STEP_DT` 时 ceil 子步数正确。
#[test]
fn step_substep_boundary_values() {
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 1.0,
        orbit_integration: OrbitIntegration::RapierForce,
        ..CosmosWorldConfig::default()
    });
    world.insert_body(satellite_builder(
        1.0,
        Vector::ZERO,
        Vector::new(1.0, 0.0, 0.0),
        0.1,
    ));

    // dt == MAX_STEP_DT (10.0)：恰好不超，应直推 Stepped(10.0)。
    assert_eq!(world.step(10.0), StepResult::Stepped(10.0));

    // dt == 硬上限 (30.0)：恰好不超 30，应走子步 3×10s。
    assert_eq!(
        world.step(30.0),
        StepResult::Substepped {
            substeps: 3,
            sub_dt: 10.0
        }
    );

    // dt 略超 MAX_STEP_DT (11.0)：ceil(11/10)=2 子步，每步 5.5s。
    assert_eq!(
        world.step(11.0),
        StepResult::Substepped {
            substeps: 2,
            sub_dt: 5.5
        }
    );

    // dt == 20.0：恰好 2 子步，每步 10.0。
    assert_eq!(
        world.step(20.0),
        StepResult::Substepped {
            substeps: 2,
            sub_dt: 10.0
        }
    );

    // dt 略超硬上限：f64 下 30.0+EPSILON 仍舍入回 30.0，用 30.0001 验证拒。
    assert_eq!(
        world.step(30.0001),
        StepResult::Skipped(StepSkipReason::TooLarge)
    );
}

/// 默认 `n_body_softening_sq` 应为 `1e3`，且两体近距离时加速度有界不发散。
#[test]
fn default_softening_bounds_close_encounter() {
    // 默认配置应带 1e3 m² 软化。
    let cfg = CosmosWorldConfig::default();
    assert_eq!(cfg.n_body_softening_sq, 1e3);

    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 0.1,
        orbit_integration: OrbitIntegration::Verlet,
        n_body_softening_sq: CosmosWorldConfig::default().n_body_softening_sq,
        ..CosmosWorldConfig::default()
    });
    // 两个 1t 体，相距 1m（远小于软化长度 31.6m）：软化后引力应有界。
    let a = world.insert_body(satellite_builder(
        1000.0,
        Vector::new(0.0, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));
    let b = world.insert_body(satellite_builder(
        1000.0,
        Vector::new(1.0, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));
    world.add_n_body(a, 1000.0);
    world.add_n_body(b, 1000.0);

    // 取 b 的初始加速度：软化平方 1e3 加在分母，1m 距离下加速度应远小于
    // 无软化时的 G·m / r² ≈ 6.67e-11·1000 / 1 = 6.67e-8 m/s²。有软化时
    // dist_sq + softening ≈ 1001，加速度量级再降 ~1000 倍。
    let src_pos = snapshot_source_positions(world.bodies(), world.n_body_sources());
    // 纯 monopole 源（无 points）不读 rotation，用 identity 切片填充。
    let n_bodies = world.bodies().len();
    let src_rot: Vec<Rotation> = (0..n_bodies).map(|_| Rotation::IDENTITY).collect();
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: world.n_body_sources(),
        source_positions: &src_pos,
        source_rotations: &src_rot,
        softening_sq: world.n_body_softening_sq(),
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
    };
    let acc = total_acceleration(
        Vector::new(1.0, 0.0, 0.0),
        Vector::ZERO,
        1000.0,
        b,
        &ctx,
        None,
    );
    // 软化后 |a| 应 ≪ 无软化值 6.67e-8。给 1e-9 上限（~1.5% 无软化值）。
    assert!(acc.length() < 1e-9, "软化后近距加速度 {acc:?} 未被有效限幅");
}

/// 变质量 n-body 源回归：注册一个初始质量 m0 的 n-body 源刚体 A，受其引力的
/// 无质量测试体 B 每步获取的加速度应正比于 A 当前 body.mass()。验证
/// `refresh_n_body_sources` 在 step 开头把 `NBodySource.gm` 从刚体当前质量重算。
///
/// 若 gm 仍是注册时固化值（没有刷新），把 A 质量从 m0 改成 m1（10×）后 B 获取的
/// 速度增量不会随之放大；本测试通过测速度增量的比例≈10× 来捕获这种回归。
#[test]
fn n_body_source_gm_tracks_live_body_mass() {
    use mps_formula::celestial_data::G;
    use rapier3d::prelude::RigidBodyHandle;
    let r = 1000.0_f64;
    let m0 = 1.0e7_f64; // 初始源质量 (kg)
    let m1 = 1.0e8_f64; // 10× 后的源质量 (kg)
    // 用纯净 world：零 softening、零重力、Verlet 1 子步 / dt=1s。零扰动。
    let make_world = |source_mass: f64| -> (CosmosWorld, RigidBodyHandle, RigidBodyHandle) {
        let mut w = CosmosWorld::new(CosmosWorldConfig {
            gravity: Vector::ZERO,
            dt: 1.0,
            solver_iterations: 1,
            ccd_substeps: 0,
            n_body_softening_sq: 0.0,
            central_body: None,
            orbit_integration: OrbitIntegration::Verlet,
            verlet_substeps: 1,
            adaptive_substeps: false,
            adaptive_tolerance: 1e-9,
            relativistic_correction: mps_cosmos::world::RelativisticCorrection::None,
        });
        // 源 A：原点、锁平移、初始 mass=source_mass，注册为 n-body 源。
        let a = w.insert_body(
            satellite_builder(source_mass, Vector::ZERO, Vector::ZERO, 1.0)
                .lock_translations()
                .linear_damping(0.0)
                .angular_damping(0.0)
                .gravity_scale(0.0),
        );
        w.add_n_body(a, source_mass);
        // 测试体 B：(r,0,0)、mass=1、零速、无阻尼、不施引。
        let b = w.insert_body(
            satellite_builder(1.0, Vector::new(r, 0.0, 0.0), Vector::ZERO, 0.1)
                .linear_damping(0.0)
                .angular_damping(0.0)
                .gravity_scale(0.0),
        );
        (w, a, b)
    };

    // ① 源 A 质量 m0：B 在 1s 后获取 vx ≈ G·m0/r²。
    let (mut w0, _a0, b0) = make_world(m0);
    w0.step(1.0);
    let vx0 = w0.body_linvel(b0).unwrap().x;
    // B 在 +x，A 在原点，B 被吸向原点 → vx 为负；只比量级与方向不错号。
    let expected_a0 = G * m0 / (r * r);
    assert!(
        (vx0.abs() - expected_a0).abs() / expected_a0 < 0.02,
        "m0 场景：|vx|={abs0} 期望≈{expected_a0} (2% 容差)",
        abs0 = vx0.abs()
    );
    assert!(vx0 < 0.0, "B 应被吸向原点（-x），vx 实为 {vx0}");

    // ② 源 A 仍按 m0 注册，但 step 前把 A 的 body.mass() 改成 m1。
    //    若 refresh 起作用：B 获取的 vx ≈ G·m1/r² = 10·G·m0/r² → 10× vx0。
    //    若 gm 仍固化 m0：vx 与 vx0 几乎相等，ratio≈1，本断言失败。
    let (mut w1, a1, b1) = make_world(m0);
    // 通过新增的 set_body_mass 把 A 的刚体质量翻到 m1（旧注册源质量不变）。
    let old_mass = w1.set_body_mass(a1, m1).expect("A exists");
    assert!(
        (old_mass - m0).abs() / m0 < 1e-12,
        "set_body_mass 返回旧质量"
    );
    assert!(
        (w1.body_mass(a1).unwrap() - m1).abs() / m1 < 1e-12,
        "A 质量已变 m1"
    );
    w1.step(1.0);
    let vx1 = w1.body_linvel(b1).unwrap().x;
    let expected_a1 = G * m1 / (r * r);
    assert!(
        (vx1.abs() - expected_a1).abs() / expected_a1 < 0.02,
        "m1 场景：|vx|={abs1} 期望≈{expected_a1} (2% 容差)；若 gm 未刷新会≈{expected_a0}",
        abs1 = vx1.abs()
    );
    assert!(vx1 < 0.0, "B 应仍被吸向原点（-x），vx 实为 {vx1}");

    // ratio（均为负值相除得正）应近似 m1/m0 = 10。
    let ratio = vx1 / vx0;
    assert!(
        (ratio - 10.0).abs() < 0.4,
        "速度增量比 vx1/vx0={ratio} 应≈10 (= m1/m0)"
    );
}

/// 不规则 N 体近场：非对称质量分布产生**非径向**加速度。
///
/// 这是"星球不是球体"场景的核心物理路径——纯 monopole（点质量）的 n-body
/// 算法在过去永远给出指向源质心的径向加速度 `a ∝ -r̂`；改进后的离散质点模型在
/// 近场把每个质量点单独累加 `a = Σ G·mᵢ·dᵢ/|dᵢ|³`，允许引力偏离质心连线。
///
/// 自制测试星系（非太阳系）：一个"双瓣哑铃"源刚体 A——两个不相等的质量团，
/// 较重的 (3/4 M) 在 (+a, 0, 0)，较轻的 (1/4 M) 在 (-a, 0, 0)，锁定在原点不动。
/// 哑铃的长轴沿 +x，但探测体放在 (0, R, 0)（与源质心连线为 +y），纯 monopole 模型
/// 只能给出 (0, -GM/R², 0)——严格指向源质心，x 分量为 0。带离散质点的模型因两团
/// 质量不等，对探测体的 +x 与 -x 拉扯不抵消，加速度出现非零 x 分量，方向偏离源
/// 质心连线。x 分量的解析值给出（两团到 (0,R,0) 距离相等 √(a²+R²)，只剩 x 项
/// 未抵消）：
///   a_x = G · a / (a²+R²)^(3/2) · (3/4 M - 1/4 M) = G·a·(M/2) / (a²+R²)^(3/2)
#[test]
fn irregular_n_body_near_field_induces_non_radial_acceleration() {
    use mps_cosmos::gravity::MassPoint;
    use mps_formula::celestial_data::G;
    use rapier3d::prelude::RigidBodyHandle;

    let m_total: f64 = 1.0e6; // 源总质量 kg
    let a: f64 = 100.0; // 两团块离质心 m
    let r: f64 = 55.0; // 探测点离源质心 m（→ 必在近场分支内）
    // 非对称质量分布：3/4 M 在 (+a,0,0)，1/4 M 在 (-a,0,0)
    let heavier = 3.0 * m_total / 4.0;
    let lighter = m_total / 4.0;
    let points = vec![
        MassPoint {
            local_offset: Vector::new(a, 0.0, 0.0),
            gm: G * heavier,
        },
        MassPoint {
            local_offset: Vector::new(-a, 0.0, 0.0),
            gm: G * lighter,
        },
    ];
    let bounding = a + 1.0; // 边界球略大于 a，含两点

    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 0.1,
        solver_iterations: 1,
        ccd_substeps: 0,
        n_body_softening_sq: 0.0,
        central_body: None,
        orbit_integration: OrbitIntegration::Verlet,
        verlet_substeps: 1,
        adaptive_substeps: false,
        adaptive_tolerance: 1e-9,
        relativistic_correction: mps_cosmos::world::RelativisticCorrection::None,
    });
    let a_hdl = world.insert_body(
        satellite_builder(m_total, Vector::ZERO, Vector::ZERO, 1.0)
            .lock_translations()
            .linear_damping(0.0)
            .angular_damping(0.0)
            .gravity_scale(0.0),
    );
    world.add_n_body_irregular(a_hdl, m_total, points, bounding);
    let b_hdl = world.insert_body(
        satellite_builder(1.0, Vector::new(0.0, r, 0.0), Vector::ZERO, 0.1)
            .linear_damping(0.0)
            .angular_damping(0.0)
            .gravity_scale(0.0),
    );

    // 步进 dt，Verlet 半步将 v 推进 a0·dt/2；故 step 后 vx ≈ a0_x·dt/2、vy ≈ a0_y·dt/2。
    let dt: f64 = 0.1;
    world.step(dt);
    let v = world.body_linvel(b_hdl).unwrap();

    // 解析：a0_x = G·a·(heavier-lighter)/d³，d = √(a²+r²)。
    let d_cubed = (a * a + r * r).powf(1.5);
    let expected_ax = G * a * (heavier - lighter) / d_cubed;
    // velocity-Verlet 单步 v(dt)=a0·dt/2 + a1·dt/2；源锁定，dt 内位移 ≪ √(a²+r²)，
    // 故 a1≈a0，整步后 v ≈ a0·dt（不是 dt/2）。这才是 expected_vx 的正确放大系数。
    let expected_vx = expected_ax * dt;
    assert!(
        expected_ax.abs() > 1e-12,
        "预期非零 +x 加速度分量（非径向）"
    );
    assert!(
        (v.x - expected_vx).abs() / expected_vx.abs() < 0.05,
        "非径向 x 分量：vx={vx} 期望≈{expected_vx} (5% 容差)",
        vx = v.x
    );

    // 关键非径向信号——纯 monopole 模型必给 vx=0，本测试 vx>0 即证明近场质点求和起效。
    assert!(
        v.x.abs() > 0.0,
        "vx 应非零（非径向分量），实为 {vx}",
        vx = v.x
    );

    // y 分量解析：两点都在 (±a,0,0)，场点 (0,r,0)，到每点距离 d=√(a²+r²)，每点的 y
    // 拉力 = G·mᵢ·(-r,0,0?）的 y 分量；求和 ay = G·r·(-(3M/4) - (M/4))/d³ = -G·M·r/d³。
    // 与 monopole（ay_mono = -GM/r²，≈8.9× 较大）差 ~8.9×：这正是近场用质点分布而非
    // 点 mass 的修正效应——不应近似 прошли monopole 而应精确匹配该 Σ 解析值。
    let d_cubed_for_y = (a * a + r * r).powf(1.5);
    let expected_ay = -G * m_total * r / d_cubed_for_y;
    let expected_vy = expected_ay * dt;
    let ay = v.y;
    assert!(
        (ay - expected_vy).abs() / expected_vy.abs() < 0.05,
        "y 分量（近场 点模型）= -GM·r/d³：ay={ay} 期望≈{expected_vy} (5% 容差)"
    );
    let _ = RigidBodyHandle;
}

/// 洛希极限（公式层纯函数 + 太空层查询 API）回归。
///
/// 公式 `roche_limit` 之前一直被埋在 FFI 函数 `astro_roche_limit` 体里——没有
/// 可被 crate 内部复用的纯 Rust 入口。太空层 `CosmosWorld` 因此也无法调用它
/// （跨 crate 走 FFI 是反模式）。本测试同时覆盖两层的新增能力：
/// 1. **公式层**：`mps_formula::astrophysics::roche_limit` 与 `roche_limit_report`
///    作为纯函数被抽出后仍给出与旧 FFI 等价的数值，并保留非有限/非正的 None 语义。
/// 2. **太空层**：`CosmosWorld::roche_limit_for` 用 `set_central_body` 注册过的
///    天体反算主星密度、再喂给公式纯函数；置刚体于两侧（分别在极限内 / 外）
///    验证 `inside_fluid_limit` 与 `inside_rigid_limit`。
#[test]
fn roche_limit_formula_and_cosmos_end_to_end() {
    use mps_formula::astrophysics::{roche_limit, roche_limit_report};
    use mps_formula::ffi::Bool;
    use rapier3d::prelude::RigidBodyHandle;

    // 公式层：密度比 5.0（主星比卫星密 5 倍），主星半径 1.0
    // → ratio = 5^(1/3) ≈ 1.70998
    // → fluid ≈ 2.44 × 1.70998 = 4.172, rigid ≈ 1.26 × 1.70998 = 2.155
    let (fluid, rigid) = roche_limit(1.0, 5.0, 1.0).unwrap();
    assert!(
        (fluid - 2.44 * 5.0f64.cbrt()).abs() < 1e-12,
        "fluid {fluid}"
    );
    assert!(
        (rigid - 1.26 * 5.0f64.cbrt()).abs() < 1e-12,
        "rigid {rigid}"
    );
    assert!(fluid > rigid, "fluid > rigid (2.44 > 1.26 系数)");

    // 越界断言：在距 orbital=2.0（< rigid 2.155 < fluid 4.17）应"隔离刚体极限 +
    // 流体极限"双重判 inner 判为 true（r=2.0 同时 < rigid 与 fluid）。
    let report = roche_limit_report(1.0, 5.0, 1.0, 2.0).unwrap();
    assert_eq!(report.fluid_roche_limit, fluid);
    assert_eq!(report.rigid_roche_limit, rigid);
    assert_eq!(report.inside_fluid_limit, Bool::TRUE);
    assert_eq!(report.inside_rigid_limit, Bool::TRUE);

    // 非法输入 → None（公式层被污染不会让调用方拿到 NaN）
    assert!(roche_limit(0.0, 5.0, 1.0).is_none(), "primary_radius=0");
    assert!(roche_limit(-1.0, 5.0, 1.0).is_none(), "primary_radius<0");
    assert!(roche_limit(f64::NAN, 5.0, 1.0).is_none(), "NaN radius");
    assert!(roche_limit_report(1.0, 5.0, 1.0, -3.0).is_none(), "dist<0");
    assert!(
        roche_limit_report(1.0, 5.0, 1.0, f64::INFINITY).is_none(),
        "inf dist"
    );

    // 太空层端到端：以地球为中心天体，放一个低轨刚体位于赤道
    // 地球：GM = 3.986e14, equatorial_radius ≈ 6378137 m。可推密度约 5515 kg/m³。
    // 卫星假定为液态水状（1000 kg/m³，松散分布——典型适配流体极限模型）。
    // 流体极限将在≈1.7×地球半径处。把"刚体"放在地球表面正上方
    //（orbital = equatorial_radius，远小于 fluid ≈ 5219km/...）。
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let r_earth = earth.equatorial_radius;
    let rho_pri = (earth.gm / mps_formula::celestial_data::G)
        / ((4.0 / 3.0) * std::f64::consts::PI * r_earth.powi(3));
    // 算解析流体极限，便于稍后核验 CosmosWorld 是否复算一致
    let expected_fluid = 2.44 * r_earth * (rho_pri / 1000.0).cbrt();
    let expected_rigid = 1.26 * r_earth * (rho_pri / 1000.0).cbrt();

    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 1.0,
        orbit_integration: OrbitIntegration::Verlet,
        ..CosmosWorldConfig::default()
    });
    world.set_central_body(Some(earth));

    // (a) 刚体在 1.5×R_earth：落在刚体极限 expected_rigid (~2.21 R_earth) 与流体极限
    //     expected_fluid (~4.31 R_earth) 双侧内 → 两个 inside 都判 true。
    let inner = world.insert_body(satellite_builder(
        1.0,
        Vector::new(1.5 * r_earth, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));

    // (b) 刚体放 2×expected_fluid 处 → 远在两极限外，inside_* 都判 false。
    let outer = world.insert_body(satellite_builder(
        1.0,
        Vector::new(2.0 * expected_fluid, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));

    let rep_inner = world
        .roche_limit_for(inner, 1000.0)
        .expect("inner: central_body + body 都有效");
    let rep_outer = world
        .roche_limit_for(outer, 1000.0)
        .expect("outer: central_body + body 都有效");

    // 数值核验：流体极限与解析公式一致（通过 G/π 反算主星密度）
    let inner_fluid = rep_inner.fluid_roche_limit;
    let inner_rigid = rep_inner.rigid_roche_limit;
    assert!(
        (inner_fluid - expected_fluid).abs() / expected_fluid < 1e-9,
        "CosmosWorld 流体极限 {inner_fluid} 应≈{expected_fluid}"
    );
    assert!(
        (inner_rigid - expected_rigid).abs() / expected_rigid < 1e-9,
        "CosmosWorld 刚体极限 {inner_rigid} 应≈{expected_rigid}"
    );
    assert!(inner_rigid < inner_fluid);

    // 1.5×R_earth < expected_rigid (~2.21 R_earth) < expected_fluid (~4.31 R_earth)
    // → 在 1.5R 处双双 inside。
    assert_eq!(rep_inner.inside_fluid_limit, Bool::TRUE);
    assert_eq!(rep_inner.inside_rigid_limit, Bool::TRUE);

    // 2×expected_fluid 远在极限外
    assert_eq!(rep_outer.inside_fluid_limit, Bool::FALSE);
    assert_eq!(rep_outer.inside_rigid_limit, Bool::FALSE);

    // 未设 central_body → roche_limit_for 返回 None（无主星，无法算）
    let mut world_no_earth = CosmosWorld::new(CosmosWorldConfig::default());
    let h = world_no_earth.insert_body(satellite_builder(1.0, Vector::ZERO, Vector::ZERO, 0.1));
    assert!(
        world_no_earth.roche_limit_for(h, 1000.0).is_none(),
        "无 central_body 应返回 None"
    );

    // 非法 secondary_density → None
    assert!(world.roche_limit_for(inner, 0.0).is_none(), "density=0");
    assert!(world.roche_limit_for(inner, -1.0).is_none(), "density<0");
    assert!(
        world.roche_limit_for(inner, f64::NAN).is_none(),
        "NaN density"
    );

    // 有效但刚体不存在 → None
    let bogus = RigidBodyHandle::from_raw_parts(u32::MAX, u32::MAX);
    assert!(
        world.roche_limit_for(bogus, 1000.0).is_none(),
        "不存在的刚体"
    );

    let _ = RigidBodyHandle;
}

/// Hill 球（天体力学界中 Roche 极限的姊妹判据）回归：先 lock 公式层
/// `hill_sphere_radius` 的解析数值；再 verify `CosmosWorld::hill_radius_for`
/// 用 central_body 反算主星质量、刚体本身取质量，得到与公式一致的结果。
///
/// `r_H = a · (1 - e) · (m_sec / (3·m_pri))^(1/3)`。
#[test]
fn hill_sphere_formula_and_cosmos_world_consistent() {
    use mps_formula::astrophysics::hill_sphere_radius;
    use rapier3d::prelude::RigidBodyHandle;

    // 公式层：典型月球-地球体系
    //  M_earth ≈ 5.972e24 kg；M_moon ≈ 7.342e22 kg
    //  a = 3.844e8 m；e = 0.0549
    let (m_pri, m_sec, a, e) = (5.972e24, 7.342e22, 3.844e8, 0.0549);
    let r_h = hill_sphere_radius(m_pri, m_sec, a, e).unwrap();
    let expect = a * (1.0 - e) * (m_sec / (3.0 * m_pri)).cbrt();
    assert!(
        (r_h - expect).abs() / expect < 1e-12,
        "r_h={r_h} expect={expect}"
    );

    // 边界：e 钳到 0..=1；非法输入 → None
    assert!(hill_sphere_radius(0.0, 1.0, 1.0, 0.0).is_none(), "M_pri=0");
    assert!(hill_sphere_radius(1.0, 0.0, 1.0, 0.0).is_none(), "m_sec=0");
    assert!(hill_sphere_radius(1.0, 1.0, 0.0, 0.0).is_none(), "a=0");
    assert!(
        hill_sphere_radius(1.0, 1.0, 1.0, f64::NAN).is_none(),
        "e=NaN"
    );
    // e>1 钳置 1 → 退化到零（(1-e)=0）
    assert_eq!(hill_sphere_radius(1.0, 1.0, 1.0, 5.0).unwrap(), 0.0);
    // e<0 钳置 0 → 等价 e=0
    let r_h_neg = hill_sphere_radius(1.0, 1.0, 1.0, -0.3).unwrap();
    let r_h_zero = hill_sphere_radius(1.0, 1.0, 1.0, 0.0).unwrap();
    assert_eq!(r_h_neg, r_h_zero);

    // 太空层：地球 central_body + 1000kg 刚体在 LEO 半径 7e6 m 处
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let primary_mass = earth.gm / mps_formula::celestial_data::G;
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt: 1.0,
        ..CosmosWorldConfig::default()
    });
    world.set_central_body(Some(earth));
    let h = world.insert_body(satellite_builder(
        1000.0,
        Vector::new(7.0e6, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));
    // cosmos 不维护根数；caller 提供 a 与 e
    let cosmos_r_h = world.hill_radius_for(h, 7.0e6, 0.0).unwrap();
    let expect2 = 7.0e6 * (1000.0 / (3.0 * primary_mass)).cbrt();
    assert!(
        (cosmos_r_h - expect2).abs() / expect2 < 1e-9,
        "cosmos hill {cosmos_r_h} expect {expect2}"
    );
    // 无 central_body / 无效刚体 / 非法 a → None
    let mut world2 = CosmosWorld::new(CosmosWorldConfig::default());
    let h2 = world2.insert_body(satellite_builder(1.0, Vector::ZERO, Vector::ZERO, 0.1));
    assert!(
        world2.hill_radius_for(h2, 1e6, 0.0).is_none(),
        "无 central_body"
    );
    let bogus = RigidBodyHandle::from_raw_parts(u32::MAX, u32::MAX);
    assert!(
        world.hill_radius_for(bogus, 1e6, 0.0).is_none(),
        "bogus handle"
    );
    assert!(world.hill_radius_for(h, -1.0, 0.0).is_none(), "a<0");
}

/// 太空层扰动力单元回归：直接调用 perturbation 模块的两个新力函数，
/// 对比 baseline（无扰动）确认 sign / 量级正确。比端到端跑 1000 步更稳。
#[test]
fn perturbation_solar_wind_and_dynamical_friction_unit() {
    use mps_cosmos::perturbation::{dynamical_friction_force, solar_wind_pressure_force};

    // ===== 太阳风动压 =====
    // 物体在 +x，太阳在原点 → sun_to_body = +x，风沿 +x 推（远离太阳）。
    // 静态物体（velocity=0），proton_density=5e6 n/m³, v_sw=450 m/s, area=50m²。
    // P = ρ·v² = (5e6 · m_proton)·(450)² ≈ 1.69e-9 Pa · 1e9 = 1.69 nPa → 1.69e-9 Pa
    // F = P·A = 1.69e-9 · 50 ≈ 8.45e-8 N，方向 +x。
    let f_sw = solar_wind_pressure_force(
        Vector::new(1.0e11, 0.0, 0.0),
        Vector::ZERO,
        5.0e6,
        450.0,
        50.0,
    )
    .expect("solar wind: 参数合法应返回 Some");
    assert!(f_sw.x > 0.0, "太阳风应沿 +x：F_sw.x={}", f_sw.x);
    assert!(f_sw.x.is_finite());
    // 量级核验：1.69e-9 Pa · 50 m² ≈ 8.45e-8 N
    let expect_pa = 5.0e6 * 1.6726219e-27 * 450.0f64 * 450.0;
    let expect_force = expect_pa * 50.0;
    assert!(
        (f_sw.x - expect_force).abs() / expect_force < 1e-2,
        "量级核验：F_sw.x={0} expect≈={expect_force}",
        f_sw.x,
    );

    // 顺风而行（同方向）→ 无力
    let body_w = Vector::new(1.0e11, 0.0, 0.0);
    let strong_tail =
        solar_wind_pressure_force(body_w, Vector::new(450.0, 0.0, 0.0), 5.0e6, 100.0, 50.0);
    assert!(strong_tail.is_none(), "顺风速度 ≥ 风速 → None");
    // 无效参数 → None
    assert!(solar_wind_pressure_force(body_w, Vector::ZERO, 0.0, 100.0, 50.0).is_none());
    assert!(solar_wind_pressure_force(body_w, Vector::ZERO, 5e6, 0.0, 50.0).is_none());
    assert!(solar_wind_pressure_force(body_w, Vector::ZERO, 5e6, 100.0, 0.0).is_none());

    // ===== 动力学摩擦 =====
    // 物体以 +y 方向 7000 m/s 穿过密度 1e-20 kg/m³ 的介质，lnΛ=5，质量 1000 kg。
    let vel = Vector::new(0.0, 7000.0, 0.0);
    let f_df = dynamical_friction_force(1000.0, 1.0e-20, vel, 5.0)
        .expect("dynamical friction: 参数合法应返回 Some");
    // 反速度方向 → 应是 -y 分量
    assert!(f_df.y < 0.0, "动摩擦应反速度（-y）：F_df.y={}", f_df.y);
    assert!(f_df.x.abs() < 1e-25, "纯 +y 速度 → 摩擦应仅在 -y");
    // 量级核验：a_mag = 4π G² M ρ lnΛ/v² = 4π · (6.6743e-11)² · 1000 · 1e-20 · 5 / 7000²
    let g = 6.67430e-11_f64;
    let a_mag_expect =
        4.0 * std::f64::consts::PI * g * g * 1000.0 * 1.0e-20 * 5.0 / (7000.0 * 7000.0);
    let f_mag_expect = a_mag_expect * 1000.0; // 力 = m·a_df
    assert!(
        (f_df.y.abs() - f_mag_expect).abs() / f_mag_expect < 1e-3,
        "量级：|F_df.y|={0} expect≈={f_mag_expect}",
        f_df.y.abs(),
    );
    // 速度为零 → None（公式 1/v² 发散）
    assert!(dynamical_friction_force(1000.0, 1.0e-20, Vector::ZERO, 5.0).is_none());
    // 其它非法 → None
    assert!(
        dynamical_friction_force(0.0, 1.0e-20, vel, 5.0).is_none(),
        "m=0"
    );
    assert!(
        dynamical_friction_force(1000.0, 0.0, vel, 5.0).is_none(),
        "ρ=0"
    );
    assert!(
        dynamical_friction_force(1000.0, 1.0e-20, vel, 0.0).is_none(),
        "lnΛ=0"
    );
}

/// 优化一致性回归：n-body 互引力的并行 + `get_unchecked` 优化路径必须与朴素
/// 串行 reference 逐位（bit-identical）一致。
///
/// 构造 3 体互引力世界（Verlet 模式，无天体源/扰动），跑一次 `world.step(dt)`
/// （走优化后的并行 a0 预计算 + 主循环 `get_unchecked`）。然后独立用串行
/// velocity-Verlet 重算每个体的终态位置——`a0`/`a1` 都用 `total_acceleration`
/// 在**冻结的初始配置**上求（与生产路径内部 Verlet 评估一致），半踢-全漂-半踢。
/// 生产结果与 reference 必须逐位相等（不允许多于 `1e-12` 的偏差），否则说明
/// 并行化或 `get_unchecked` 重构打乱了浮点运算顺序 / 数值。
#[test]
fn n_body_optimized_matches_serial_reference_bit_identical() {
    let dt = 0.01;
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt,
        solver_iterations: 4,
        ccd_substeps: 4,
        orbit_integration: OrbitIntegration::Verlet,
        n_body_softening_sq: 0.0,
        central_body: None,
        verlet_substeps: 1,
        ..CosmosWorldConfig::default()
    });

    // 3 个体，质量/位置/速度各异，确保互引力非零且互不对称。
    let defs = [
        (
            1.0e3,
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(0.1, 0.0, 0.0),
        ),
        (
            2.0e3,
            Vector::new(3.0, 1.0, 0.0),
            Vector::new(0.0, -0.2, 0.0),
        ),
        (
            5.0e2,
            Vector::new(-2.0, 4.0, 1.0),
            Vector::new(0.0, 0.0, 0.3),
        ),
    ];
    let mut init = Vec::new();
    for (m, p, v) in defs {
        let h = world.insert_body(satellite_builder(m, p, v, 0.1));
        world.add_n_body(h, m);
        let b = world.bodies().get(h).unwrap();
        init.push((h, b.translation(), b.linvel(), m));
    }

    // 冻结初始配置：源位置快照（生产路径 `refresh_n_body_sources` 产出）。
    let src_pos = snapshot_source_positions(world.bodies(), world.n_body_sources());
    let n_bodies = world.bodies().len();
    let src_rot: Vec<Rotation> = (0..n_bodies).map(|_| Rotation::IDENTITY).collect();
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: world.n_body_sources(),
        source_positions: &src_pos,
        source_rotations: &src_rot,
        softening_sq: 0.0,
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
    };

    // 独立串行 reference：每体 velocity-Verlet（与生产 `verlet_step` 同公式）。
    let mut reference = Vec::new();
    for &(h, r0, v0, mass) in &init {
        let a0 = total_acceleration(r0, v0, mass, h, &ctx, None);
        let v_half = v0 + a0 * (0.5 * dt);
        let r1 = r0 + v_half * dt;
        let a1 = total_acceleration(r1, v_half, mass, h, &ctx, None);
        let v1 = v_half + a1 * (0.5 * dt);
        reference.push((h, r1, v1));
    }

    // 生产路径：优化后的并行 + get_unchecked。
    world.step(dt);

    for (h, r_ref, v_ref) in reference {
        let b = world.bodies().get(h).unwrap();
        let pos = b.translation();
        let vel = b.linvel();
        assert!(
            (pos.x - r_ref.x).abs() < 1e-12
                && (pos.y - r_ref.y).abs() < 1e-12
                && (pos.z - r_ref.z).abs() < 1e-12,
            "body {h:?} 优化路径位置 {pos:?} 与串行 reference {r_ref:?} 不一致"
        );
        assert!(
            (vel.x - v_ref.x).abs() < 1e-12
                && (vel.y - v_ref.y).abs() < 1e-12
                && (vel.z - v_ref.z).abs() < 1e-12,
            "body {h:?} 优化路径速度 {vel:?} 与串行 reference {v_ref:?} 不一致"
        );
    }
}

/// 高阶 + Kahan 积子优化一致性回归：并行预计算的高阶推进 / Kahan 推进必须和
/// 朴素串行 reference（直接调 `advance_highorder` / `advance_highorder_kahan`
/// 纯函数）逐位一致。
///
/// 构造 3 体互引力世界，分别用 `Yoshida4` / `ForestRuth8` / `Yoshida4Kahan` /
/// `ForestRuth8Kahan` 跑一次 `world.step(dt)`（走并行预计算路径），再用对应
/// 纯函数做独立串行 reference，比对每个体终态位置/速度（Kahan 还要比对累加态）。
/// 偏差须 < 1e-12。
#[cfg(test)]
fn highorder_bit_identical_helper(mode: OrbitIntegration) {
    let dt = 0.01;
    let mut world = CosmosWorld::new(CosmosWorldConfig {
        gravity: Vector::ZERO,
        dt,
        solver_iterations: 4,
        ccd_substeps: 4,
        orbit_integration: mode,
        n_body_softening_sq: 0.0,
        central_body: None,
        verlet_substeps: 1,
        ..CosmosWorldConfig::default()
    });

    let defs = [
        (
            1.0e3,
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(0.1, 0.0, 0.0),
        ),
        (
            2.0e3,
            Vector::new(3.0, 1.0, 0.0),
            Vector::new(0.0, -0.2, 0.0),
        ),
        (
            5.0e2,
            Vector::new(-2.0, 4.0, 1.0),
            Vector::new(0.0, 0.0, 0.3),
        ),
    ];
    let mut handles = Vec::new();
    let mut init = Vec::new();
    for (m, p, v) in defs {
        let h = world.insert_body(satellite_builder(m, p, v, 0.1));
        world.add_n_body(h, m);
        let b = world.bodies().get(h).unwrap();
        init.push((h, b.translation(), b.linvel(), m));
        handles.push(h);
    }

    let src_pos = snapshot_source_positions(world.bodies(), world.n_body_sources());
    let n_bodies = world.bodies().len();
    let src_rot: Vec<Rotation> = (0..n_bodies).map(|_| Rotation::IDENTITY).collect();
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: world.n_body_sources(),
        source_positions: &src_pos,
        source_rotations: &src_rot,
        softening_sq: 0.0,
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
    };

    // 串行 reference：每个体用对应纯函数推进。
    let use_kahan = matches!(
        mode,
        OrbitIntegration::Yoshida4Kahan | OrbitIntegration::ForestRuth8Kahan
    );
    let mut ref_out = Vec::new();
    for &(h, r0, v0, mass) in &init {
        if use_kahan {
            let kp = KahanVec3::new(mps_cosmos::integrator::ffi_vec3_pub(r0));
            let kv = KahanVec3::new(mps_cosmos::integrator::ffi_vec3_pub(v0));
            let (r1, v1, nkp, nkv) =
                advance_highorder_kahan(mode, r0, v0, kp, kv, mass, h, None, &ctx, dt);
            ref_out.push((h, r1, v1, Some((nkp, nkv))));
        } else {
            let (r1, v1) = advance_highorder(mode, r0, v0, mass, h, None, &ctx, dt);
            ref_out.push((h, r1, v1, None));
        }
    }

    world.step(dt);

    for (h, r_ref, v_ref, k_ref) in ref_out {
        let b = world.bodies().get(h).unwrap();
        let pos = b.translation();
        let vel = b.linvel();
        assert!(
            (pos.x - r_ref.x).abs() < 1e-12
                && (pos.y - r_ref.y).abs() < 1e-12
                && (pos.z - r_ref.z).abs() < 1e-12,
            "body {h:?} 高阶优化路径位置 {pos:?} 与串行 reference {r_ref:?} 不一致"
        );
        assert!(
            (vel.x - v_ref.x).abs() < 1e-12
                && (vel.y - v_ref.y).abs() < 1e-12
                && (vel.z - v_ref.z).abs() < 1e-12,
            "body {h:?} 高阶优化路径速度 {vel:?} 与串行 reference {v_ref:?} 不一致"
        );
        if let Some((kp_ref, kv_ref)) = k_ref {
            let idx = h.into_raw_parts().0 as usize;
            let st = world.kahan_state_debug(idx);
            assert!(
                (st.0.value().x - kp_ref.value().x).abs() < 1e-12
                    && (st.0.value().y - kp_ref.value().y).abs() < 1e-12
                    && (st.0.value().z - kp_ref.value().z).abs() < 1e-12
                    && (st.1.value().x - kv_ref.value().x).abs() < 1e-12
                    && (st.1.value().y - kv_ref.value().y).abs() < 1e-12
                    && (st.1.value().z - kv_ref.value().z).abs() < 1e-12,
                "body {h:?} Kahan 累加态与串行 reference 不一致"
            );
        }
    }
}

#[test]
fn highorder_yoshida4_matches_serial_reference() {
    highorder_bit_identical_helper(OrbitIntegration::Yoshida4);
}

#[test]
fn highorder_forest_ruth8_matches_serial_reference() {
    highorder_bit_identical_helper(OrbitIntegration::ForestRuth8);
}

#[test]
fn highorder_yoshida4_kahan_matches_serial_reference() {
    highorder_bit_identical_helper(OrbitIntegration::Yoshida4Kahan);
}

#[test]
fn highorder_forest_ruth8_kahan_matches_serial_reference() {
    highorder_bit_identical_helper(OrbitIntegration::ForestRuth8Kahan);
}
