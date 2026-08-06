//! `mps_cosmos::integrator` 测试 —— 迁移自 `crates/mps-cosmos/src/integrator.rs`。
//!
//! velocity-Verlet 在**点质量中心引力**下推进一整圈应几乎闭合。用 n-body 源
//! （点质量互引力，`-GM·r̂/r²`，不走 `celestial_acceleration` 的 ellipsoid/J2
//! 分支）作为中心引力，直接验证 Verlet 自身的相位精度，排除引力模型误差。

#[cfg(test)]
use mps_cosmos::bodies::satellite_builder;
#[cfg(test)]
use mps_cosmos::gravity::{NBodySource, gm_from_mass};
#[cfg(test)]
use mps_cosmos::integrator::{
    AccelContext, snapshot_source_positions, total_acceleration, verlet_step,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
#[cfg(test)]
use mps_formula::spaceflight::kepler_period;
#[cfg(test)]
use rapier3d::prelude::{RigidBodySet, Vector};

#[test]
fn verlet_circle_orbit_closes_tight() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let r = 7_000_000.0_f64;
    let v = (gm / r).sqrt();
    let period = kepler_period(gm, r).expect("period");
    let dt = 1.0;
    let steps = (period / dt).round() as u32;

    // 用一个固定刚体作为"假地球"n-body 源（点质量互引力）。
    let mut bodies = RigidBodySet::new();
    let earth_hdl = bodies.insert(
        satellite_builder(5.972e24, Vector::ZERO, Vector::ZERO, 1.0)
            .lock_translations()
            .build(),
    );
    let sat_hdl = bodies.insert(
        satellite_builder(1000.0, Vector::new(r, 0.0, 0.0), Vector::new(0.0, v, 0.0), 1.0).build(),
    );
    let n_body_sources = vec![NBodySource {
        handle: earth_hdl,
        gm: gm_from_mass(5.972e24),
    }];

    // ctx：无 celestials，仅一个 n-body 点质量源。
    let src_pos = snapshot_source_positions(&bodies, &n_body_sources);
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: &n_body_sources,
        source_positions: &src_pos,
        softening_sq: 0.0,
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
    };

    let mut a = total_acceleration(
        bodies.get(sat_hdl).unwrap().translation(),
        bodies.get(sat_hdl).unwrap().linvel(),
        1000.0,
        sat_hdl,
        &ctx,
        None,
    );
    for _ in 0..steps {
        let body = bodies.get_mut(sat_hdl).unwrap();
        verlet_step(body, a, &ctx, 1000.0, sat_hdl, None, dt);
        // 子步内源位置快照不变（地球锁定），所以 ctx 可直接复用。
        a = total_acceleration(
            bodies.get(sat_hdl).unwrap().translation(),
            bodies.get(sat_hdl).unwrap().linvel(),
            1000.0,
            sat_hdl,
            &ctx,
            None,
        );
    }

    let body = bodies.get(sat_hdl).unwrap();
    let off = (body.translation() - Vector::new(r, 0.0, 0.0)).length();
    // Verlet(二阶) + 1s 步长 + 纯中心引力，一圈闭合 ~3.6km（相位误差 O(dt²·ω·T)）。
    // 给 0.1% r 放量级余量（7000km → 7km）。
    assert!(
        off / r < 1e-3,
        "Verlet 一圈偏移 {off} 过大 (>0.1% r)，pos={:?}",
        body.translation()
    );
    // 同时验证能量近乎保守（无漂）—— Verlet 不应有系统性能量增减。
    let e0 = 0.5 * v * v - gm / r;
    let final_v = body.linvel().length();
    let final_r = body.translation().length();
    let e1 = 0.5 * final_v * final_v - gm / final_r;
    assert!(
        (e1 - e0).abs() / e0.abs() < 1e-3,
        "Verlet 一圈能量漂移 {e1} vs {e0}"
    );
}
