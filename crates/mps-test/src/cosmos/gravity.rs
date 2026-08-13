//! `mps_cosmos::gravity` 测试 —— 迁移自 `crates/mps-cosmos/src/gravity.rs`。

#[cfg(test)]
use mps_cosmos::gravity::{
    CelestialSource, NBodySource, celestial_acceleration, n_body_acceleration,
    point_mass_acceleration,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
#[cfg(test)]
use rapier3d::prelude::{RigidBodyHandle, Rotation, Vector};

#[test]
fn earth_surface_gravity_is_about_9_8() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let source = CelestialSource::new(earth, earth.max_degree);
    // 在赤道表面沿 +x 方向
    let pos = Vector::new(earth.equatorial_radius, 0.0, 0.0);
    let a = celestial_acceleration(pos, &source);
    // 表面几何中心重力 ≈ GM/R² ≈ 9.8 m/s²；椭球/J2 模型带来的赤道
    // 减小约 ~0.2%（离心 + 扁率效应），故用 1% 容差。
    let g = a.length();
    let expected = earth.gm / (earth.equatorial_radius * earth.equatorial_radius);
    assert!(
        (g - expected).abs() / expected < 0.01,
        "赤道表面重力 {g} 期望约 {expected} (1% 容差)"
    );
}

#[test]
fn point_mass_falls_off_as_inverse_square() {
    let gm = 1.0;
    let a1 = point_mass_acceleration(Vector::new(1.0, 0.0, 0.0), gm).length();
    let a2 = point_mass_acceleration(Vector::new(2.0, 0.0, 0.0), gm).length();
    assert!((a1 / a2 - 4.0).abs() < 1e-12); // 距离×2 → 引力 1/4
}

#[test]
fn n_body_excludes_self_and_points_toward_source() {
    let handle_self = RigidBodyHandle::from_raw_parts(u32::MAX, u32::MAX);
    let handle_other = RigidBodyHandle::from_raw_parts(1, 0);
    let sources = vec![
        NBodySource::monopole(handle_self, 1.0),
        NBodySource::monopole(handle_other, 1.0),
    ];
    let pos = Vector::new(0.0, 0.0, 0.0);
    let acc = n_body_acceleration(
        pos,
        &sources,
        handle_self,
        |h| {
            if h == handle_other {
                Vector::new(1.0, 0.0, 0.0)
            } else {
                Vector::ZERO
            }
        },
        |_| Rotation::IDENTITY,
        0.0,
    );
    // 仅 other 贡献，方向 +x，大小 GM/1² = 1
    assert!((acc.x - 1.0).abs() < 1e-12);
    assert!(acc.y.abs() < 1e-12 && acc.z.abs() < 1e-12);
}
