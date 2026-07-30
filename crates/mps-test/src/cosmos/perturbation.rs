//! `mps_cosmos::perturbation` 测试 —— 迁移自 `crates/mps-cosmos/src/perturbation.rs`。

use mps_cosmos::perturbation::{atmosphere_density_at, atmospheric_drag_force, solar_pressure_force};
use mps_formula::celestial_data::{AU, CelestialBodyId, get_celestial_body};
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
