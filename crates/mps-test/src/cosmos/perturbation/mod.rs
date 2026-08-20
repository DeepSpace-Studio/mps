//! `mps_cosmos::perturbation` 测试 —— 迁移自 `crates/mps-cosmos/src/perturbation.rs`。

#[cfg(test)]
use mps_cosmos::perturbation::{
    atmosphere_density_at, atmospheric_drag_force, eclipse_attenuation, solar_pressure_force,
    tidal_torque,
};
#[cfg(test)]
use mps_formula::celestial_data::{AU, CelestialBodyId, SUN_EQ_RADIUS, get_celestial_body};
#[cfg(test)]
use rapier3d::prelude::Vector;

#[test]
fn drag_opposes_velocity_and_scales_with_speed_squared() {
    let v = Vector::new(7800.0, 0.0, 0.0); // 典型低轨速度
    let f1 = atmospheric_drag_force(v, Vector::ZERO, 1e-12, 2.2, 10.0, 1000.0)
        .expect("valid drag input");
    // 阻力方向应与速度相反
    assert!(f1.x < 0.0);
    // 速度×2 → 阻力×4
    let f2 = atmospheric_drag_force(2.0 * v, Vector::ZERO, 1e-12, 2.2, 10.0, 1000.0)
        .expect("valid drag input");
    assert!((f2.x / f1.x - 4.0).abs() < 1e-9);
}

#[test]
fn earth_atmosphere_density_decays_with_altitude() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let rho0 = atmosphere_density_at(earth, 0.0);
    let rho1 = atmosphere_density_at(earth, earth.scale_height);
    assert!(rho0 > 0.0);
    assert!((rho0 / rho1 - std::f64::consts::E).abs() < 1e-9); // 升高一个标高 → 衰减为 1/e
}

#[test]
fn solar_pressure_falls_off_inverse_square_at_au() {
    let sun_dir = Vector::new(-1.0, 0.0, 0.0); // 指向太阳
    let f_at_au = solar_pressure_force(Vector::new(AU, 0.0, 0.0), sun_dir, 1.0, 1.0, AU);
    let f_at_2au = solar_pressure_force(Vector::new(2.0 * AU, 0.0, 0.0), sun_dir, 1.0, 1.0, AU);
    assert!((f_at_au.length() / f_at_2au.length() - 4.0).abs() < 1e-9);
    // 方向指向 +x（远离太阳为正光压方向 −sun_dir = +x）
    assert!(f_at_au.x > 0.0);
}

/// 日食衰减纯函数：遮挡体在原点、太阳在 +x 轴。被照体在轴线上遮挡体背后 → 本影因子 0。
#[test]
fn eclipse_umbra_factor_is_zero() {
    let occ_r = 6.371e6; // 地球半径
    let sun_r = SUN_EQ_RADIUS; // ~6.957e8
    let d_sun = 1.496e11; // 1 AU
    let sun_pos = Vector::new(d_sun, 0.0, 0.0);
    // 被照体在轴线上、遮挡体（原点）正后方 1e8 m：轴向 x>0 且横向 d_perp≈0 → 本影。
    let pos = Vector::new(-1.0e8, 0.0, 0.0);
    let att = eclipse_attenuation(pos, sun_pos, occ_r, sun_r);
    assert!((att - 0.0).abs() < 1e-12, "umbra att={att}");
}

/// 半影内：横向偏移介于本影/半影外缘之间 → 因子 (0,1)。
#[test]
fn eclipse_penumbra_factor_between_zero_and_one() {
    let occ_r = 6.371e6;
    let sun_r = SUN_EQ_RADIUS;
    let d_sun = 1.496e11;
    let sun_pos = Vector::new(d_sun, 0.0, 0.0);
    let x = 1.0e9; // 遮挡体背后 1e9 m（仍在轴附近，远未到本影末端 ~1.4e9）
    let r_umbra = occ_r + (occ_r - sun_r) * x / d_sun;
    let r_pen = occ_r + (occ_r + sun_r) * x / d_sun;
    let d_perp = 0.5 * (r_umbra + r_pen); // 半影正中
    let pos = Vector::new(-x, d_perp, 0.0);
    let att = eclipse_attenuation(pos, sun_pos, occ_r, sun_r);
    assert!(att > 0.0 && att < 1.0, "penumbra att={att} (want 0<att<1)");
    // 横向偏移到半影外缘（刚好初遮）→ 因子趋近 1。
    let pos_out = Vector::new(-x, r_pen * 1.001, 0.0);
    let att_out = eclipse_attenuation(pos_out, sun_pos, occ_r, sun_r);
    assert!(
        (att_out - 1.0).abs() < 1e-9,
        "outside penumbra att={att_out}"
    );
}

/// 无遮挡 / 默认值：被照体与太阳同侧（x<=0）→ 因子 1；零半径遮挡体 → 因子 1。
#[test]
fn eclipse_no_occlusion_factor_is_one() {
    let sun_pos = Vector::new(1.496e11, 0.0, 0.0);
    // 被照体在太阳与遮挡体之间（x<=0 轴向）→ 无阴影。
    let pos_between = Vector::new(0.5e11, 1.0e7, 0.0);
    assert!(
        (eclipse_attenuation(pos_between, sun_pos, 6.371e6, SUN_EQ_RADIUS) - 1.0).abs() < 1e-12
    );
    // 零半径遮挡体 → 退化返回 1。
    let pos_behind = Vector::new(-1.0e8, 0.0, 0.0);
    assert!((eclipse_attenuation(pos_behind, sun_pos, 0.0, SUN_EQ_RADIUS) - 1.0).abs() < 1e-12);
}

/// Hut (1981) 平衡潮力矩公式校验：方向沿轨道法向、符号 (n - ω_∥) 驱动自旋同步。
#[test]
fn tidal_torque_drives_spin_toward_synchronous() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let a = 7.0e6_f64; // 近地圆轨道半径
    // 圆轨道：位置 +x、速度 +y（逆时针，法向 +z）。
    let pos = Vector::new(a, 0.0, 0.0);
    let v = (gm / a).sqrt();
    let vel = Vector::new(0.0, v, 0.0);
    let n = (gm / a.powi(3)).sqrt();
    let k2 = 0.299_f64;
    let q = 12.0_f64;
    let r_sat = 1.0e3_f64; // 1 km 卫星

    // 欠同步：自旋沿 +z 仅为 n 的一半 → 力矩应沿 +z（加速到同步）。
    let spin_slow = Vector::new(0.0, 0.0, 0.5 * n);
    let tau_slow = tidal_torque(
        pos,
        vel,
        spin_slow,
        gm,
        earth.equatorial_radius,
        r_sat,
        k2,
        q,
    );
    assert!(
        tau_slow.z > 0.0,
        "sub-synchronous torque must speed up spin, got {:?}",
        tau_slow
    );
    assert!(
        tau_slow.x.abs() < 1e-12 && tau_slow.y.abs() < 1e-12,
        "torque axial only"
    );

    // 过同步：自旋 = 2n → 力矩应沿 -z（减速）。
    let spin_fast = Vector::new(0.0, 0.0, 2.0 * n);
    let tau_fast = tidal_torque(
        pos,
        vel,
        spin_fast,
        gm,
        earth.equatorial_radius,
        r_sat,
        k2,
        q,
    );
    assert!(
        tau_fast.z < 0.0,
        "super-synchronous torque must slow spin, got {:?}",
        tau_fast
    );

    // 同步：自旋 = n → 力矩为零（平衡潮锁定）。
    let spin_sync = Vector::new(0.0, 0.0, n);
    let tau_sync = tidal_torque(
        pos,
        vel,
        spin_sync,
        gm,
        earth.equatorial_radius,
        r_sat,
        k2,
        q,
    );
    let tau_sync_mag =
        (tau_sync.x * tau_sync.x + tau_sync.y * tau_sync.y + tau_sync.z * tau_sync.z).sqrt();
    assert!(
        tau_sync_mag < 1e-12,
        "synchronous → zero torque, got {:?}",
        tau_sync
    );

    // 强度标度：R⁵；半径翻倍 → 力矩 32×。
    let tau_r1 = tidal_torque(
        pos,
        vel,
        spin_slow,
        gm,
        earth.equatorial_radius,
        r_sat,
        k2,
        q,
    );
    let tau_r2 = tidal_torque(
        pos,
        vel,
        spin_slow,
        gm,
        earth.equatorial_radius,
        2.0 * r_sat,
        k2,
        q,
    );
    assert!(
        (tau_r2.z / tau_r1.z - 32.0).abs() < 1e-9,
        "R^5 scaling, got {}",
        tau_r2.z / tau_r1.z
    );

    // 非法输入 → 零。
    assert_eq!(
        tidal_torque(pos, vel, spin_slow, gm, earth.equatorial_radius, 0.0, k2, q),
        Vector::ZERO
    );
    assert_eq!(
        tidal_torque(
            pos,
            vel,
            spin_slow,
            gm,
            earth.equatorial_radius,
            r_sat,
            k2,
            0.0
        ),
        Vector::ZERO
    );
}
