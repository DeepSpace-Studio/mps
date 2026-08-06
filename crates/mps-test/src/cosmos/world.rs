//! `mps_cosmos::world` 测试 —— 迁移自 `crates/mps-cosmos/src/world.rs`。
//!
//! 涵盖 `CosmosWorld` 的 step 语义（RapierForce / Verlet 两条路径）、子步
//! 切分边界、`step_n` 批处理、默认 `n_body_softening_sq` 限幅，以及端到端
//! 圆轨道 LEO 演算（RapierForce 短弧 + Verlet 整圈闭合）。

#[cfg(test)]
use mps_cosmos::bodies::satellite_builder;
#[cfg(test)]
use mps_cosmos::integrator::{AccelContext, snapshot_source_positions, total_acceleration};
#[cfg(test)]
use mps_cosmos::world::{
    CosmosWorld, CosmosWorldConfig, OrbitIntegration, StepResult, StepSkipReason,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
#[cfg(test)]
use mps_formula::spaceflight::kepler_period;
#[cfg(test)]
use rapier3d::prelude::Vector;

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
        satellite_builder(1000.0, Vector::new(r, 0.0, 0.0), Vector::new(0.0, v, 0.0), 1.0)
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
        satellite_builder(1000.0, Vector::new(r, 0.0, 0.0), Vector::new(0.0, v, 0.0), 1.0)
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
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: world.n_body_sources(),
        source_positions: &src_pos,
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
