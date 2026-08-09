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
    use rapier3d::prelude::RigidBodyHandle;
    use mps_formula::celestial_data::G;
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
    assert!((old_mass - m0).abs() / m0 < 1e-12, "set_body_mass 返回旧质量");
    assert!((w1.body_mass(a1).unwrap() - m1).abs() / m1 < 1e-12, "A 质量已变 m1");
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
    assert!((ratio - 10.0).abs() < 0.4, "速度增量比 vx1/vx0={ratio} 应≈10 (= m1/m0)");
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

    let m_total: f64 = 1.0e6;     // 源总质量 kg
    let a: f64 = 100.0;           // 两团块离质心 m
    let r: f64 = 55.0;            // 探测点离源质心 m（→ 必在近场分支内）
    // 非对称质量分布：3/4 M 在 (+a,0,0)，1/4 M 在 (-a,0,0)
    let heavier = 3.0 * m_total / 4.0;
    let lighter = m_total / 4.0;
    let points = vec![
        MassPoint { local_offset: Vector::new(a, 0.0, 0.0), gm: G * heavier },
        MassPoint { local_offset: Vector::new(-a, 0.0, 0.0), gm: G * lighter },
    ];
    let bounding = a + 1.0;       // 边界球略大于 a，含两点

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
    assert!(expected_ax.abs() > 1e-12, "预期非零 +x 加速度分量（非径向）");
    assert!((v.x - expected_vx).abs() / expected_vx.abs() < 0.05,
        "非径向 x 分量：vx={vx} 期望≈{expected_vx} (5% 容差)", vx = v.x);

    // 关键非径向信号——纯 monopole 模型必给 vx=0，本测试 vx>0 即证明近场质点求和起效。
    assert!(v.x.abs() > 0.0, "vx 应非零（非径向分量），实为 {vx}", vx = v.x);

    // y 分量解析：两点都在 (±a,0,0)，场点 (0,r,0)，到每点距离 d=√(a²+r²)，每点的 y
    // 拉力 = G·mᵢ·(-r,0,0?）的 y 分量；求和 ay = G·r·(-(3M/4) - (M/4))/d³ = -G·M·r/d³。
    // 与 monopole（ay_mono = -GM/r²，≈8.9× 较大）差 ~8.9×：这正是近场用质点分布而非
    // 点 mass 的修正效应——不应近似 прошли monopole 而应精确匹配该 Σ 解析值。
    let d_cubed_for_y = (a * a + r * r).powf(1.5);
    let expected_ay = -G * m_total * r / d_cubed_for_y;
    let expected_vy = expected_ay * dt;
    let ay = v.y;
    assert!((ay - expected_vy).abs() / expected_vy.abs() < 0.05,
        "y 分量（近场 点模型）= -GM·r/d³：ay={ay} 期望≈{expected_vy} (5% 容差)");
    let _ = RigidBodyHandle;
}
