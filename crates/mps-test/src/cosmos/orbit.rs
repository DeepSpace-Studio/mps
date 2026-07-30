//! `mps_cosmos::orbit` 测试 —— 迁移自 `crates/mps-cosmos/src/orbit.rs`。

use mps_cosmos::orbit::{BodyState, angular_momentum_of, elements_of, energy_of};
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
use mps_formula::spaceflight::{elements_to_state, kepler_period};
use rapier3d::prelude::Vector;

#[test]
fn elements_round_trip_with_central_body() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let elements = mps_formula::ffi::OrbitalElements {
        semi_major_axis: 7_000_000.0,
        eccentricity: 0.05,
        inclination: 0.3,
        raan: 0.4,
        argument_of_periapsis: 0.5,
        true_anomaly: 0.6,
    };
    let state_ffi = elements_to_state(elements, gm).expect("convert elements");
    let state = BodyState::new(
        Vector::new(
            state_ffi.position.x,
            state_ffi.position.y,
            state_ffi.position.z,
        ),
        Vector::new(
            state_ffi.velocity.x,
            state_ffi.velocity.y,
            state_ffi.velocity.z,
        ),
    );
    let out = elements_of(state, gm).expect("recover elements");
    assert!((out.semi_major_axis - elements.semi_major_axis).abs() < 1.0);
    assert!((out.eccentricity - elements.eccentricity).abs() < 1e-9);
}

#[test]
fn circular_orbit_energy_and_period() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let r = 7_000_000.0;
    let v = (gm / r).sqrt(); // 圆轨道速度
    let state = BodyState::new(Vector::new(r, 0.0, 0.0), Vector::new(0.0, v, 0.0));
    let e = energy_of(state, gm);
    let expected_e = -gm / (2.0 * r); // 圆轨比能量
    assert!((e - expected_e).abs() / expected_e.abs() < 1e-9);
    let h = angular_momentum_of(state);
    // 角动量大小 = r·v
    assert!((h.length() - r * v).abs() / (r * v) < 1e-9);
    // 与开普勒第三定律一致
    let expected_period = kepler_period(gm, r).expect("period");
    let measured_period = 2.0 * std::f64::consts::PI * r / v;
    assert!((measured_period - expected_period).abs() / expected_period < 1e-9);
}
