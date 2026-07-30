//! `mps_cosmos::bodies` 测试 —— 迁移自 `crates/mps-cosmos/src/bodies.rs`。

use mps_cosmos::bodies::{fixed_body_builder, satellite_builder};
use rapier3d::prelude::{ColliderSet, Vector};

#[test]
fn satellite_builder_sets_mass_and_kinematics() {
    let builder = satellite_builder(
        1000.0,
        Vector::new(7e6, 0.0, 0.0),
        Vector::new(0.0, 7800.0, 0.0),
        1.0,
    );
    let mut body = builder.build();
    // build() 只把 additional_mass_properties 暂存到 additional_local_mprops，
    // 并未并入 local_mprops；RigidBody::mass() 走的是 local_mprops.inv_mass。
    // 所以在第一次 step（或显式 recompute）之前，mass() 读不到 1000.0。
    // 这里显式重算一次以验证 builder 设置真的进入了质量属性。
    body.recompute_mass_properties_from_colliders(&ColliderSet::new());
    assert!((body.mass() - 1000.0).abs() < 1e-6);
    assert!((body.translation().x - 7e6).abs() < 1e-6);
    assert!((body.linvel().y - 7800.0).abs() < 1e-6);
    assert!(body.is_dynamic());
}

#[test]
fn fixed_builder_is_not_dynamic() {
    let body = fixed_body_builder(Vector::new(1.0, 2.0, 3.0)).build();
    assert!(body.is_fixed());
}
