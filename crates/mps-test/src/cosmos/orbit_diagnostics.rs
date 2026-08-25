//! `mps_cosmos::orbit_diagnostics` 测试 —— 只读轨道诊断量的公式校验。

#[cfg(test)]
use mps_cosmos::orbit_diagnostics::{
    eccentricity_vector, kozai_period, mean_motion, mean_motion_ratio,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, G, SUN_GM, get_celestial_body};
#[cfg(test)]
use rapier3d::prelude::Vector;

#[test]
fn mean_motion_earth_like_matches_one_year() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let a = 1.0 * mps_formula::celestial_data::AU; // 1 AU
    let n = mean_motion(SUN_GM, a);
    let period = 2.0 * std::f64::consts::PI / n;
    // 地球公转 ≈ 365.25 天 = 3.15576e7 s，容忍 1%。
    let year = 365.25 * 86400.0;
    assert!(
        (period / year - 1.0).abs() < 0.01,
        "period={period} year={year}"
    );
    // 引用 earth 以免 unused（GM 一致性旁证）。
    assert!(earth.gm > 0.0);
}

#[test]
fn mean_motion_ratio_two_to_one_for_a_scaling() {
    // n ∝ a^{-3/2}；a2 = a1 · 2^{+2/3}（a2>a1 → n2<n1）时 n1/n2 = 2。
    let a1 = 1.0e11;
    let a2 = a1 * 2.0f64.powf(2.0 / 3.0);
    let r = mean_motion_ratio(SUN_GM, a1, SUN_GM, a2);
    assert!((r - 2.0).abs() < 1e-9, "MMR 2:1 ratio={r}");
    // 同 a、同 GM → 1。
    assert!((mean_motion_ratio(SUN_GM, a1, SUN_GM, a1) - 1.0).abs() < 1e-12);
}

#[test]
fn eccentricity_vector_zero_for_circular() {
    // 圆轨道：任意 r、切向 v = sqrt(GM/r)，e_vec 模长应为 0。
    let gm = 1.0e14_f64;
    let r_mag = 5.0e6_f64;
    let v = (gm / r_mag).sqrt(); // 圆轨道速度
    let pos = Vector::new(r_mag, 0.0, 0.0);
    let vel = Vector::new(0.0, v, 0.0);
    let e = eccentricity_vector(pos, vel, gm);
    let e_mag = (e.x * e.x + e.y * e.y + e.z * e.z).sqrt();
    assert!(e_mag < 1e-9, "circular e={e_mag}");
}

#[test]
fn eccentricity_vector_magnitude_equals_e_at_periapsis() {
    // 椭圆 a=1e7、e=0.5，近心点处 r=a(1-e)=5e6，v 沿 +y（逆时针）。
    let gm = 1.0e14_f64;
    let a = 1.0e7_f64;
    let e_tgt = 0.5_f64;
    let r_mag = a * (1.0 - e_tgt);
    let v = ((gm * (1.0 + e_tgt)) / (a * (1.0 - e_tgt))).sqrt();
    let pos = Vector::new(r_mag, 0.0, 0.0);
    let vel = Vector::new(0.0, v, 0.0);
    let e_vec = eccentricity_vector(pos, vel, gm);
    let e_mag = (e_vec.x * e_vec.x + e_vec.y * e_vec.y + e_vec.z * e_vec.z).sqrt();
    assert!(
        (e_mag - e_tgt).abs() < 1e-9,
        "e_vec.mag={e_mag} want {e_tgt}"
    );
    // 近心点方向应为 +x。
    assert!(e_vec.x > 0.99 * e_tgt && e_vec.y.abs() < 1e-6 && e_vec.z.abs() < 1e-6);
}

#[test]
fn kozai_period_positive_and_scales_with_geometry() {
    let m_primary = 5.972e24_f64; // 地球
    let m_tertiary = 1.989e30_f64; // 太阳
    let a_in = 3.84e8_f64; // 月地距
    let a_out = 1.496e11_f64; // 日地距
    let e_out = 0.0167_f64;
    let p1 = kozai_period(a_in, a_out, e_out, m_primary, m_tertiary);
    assert!(p1 > 0.0 && p1.is_finite(), "kozai p={p1}");
    // 几何缩放：a_out 翻倍 → P_out ∝ a_out^1.5 且 a_term ∝ a_out³ → 总 ∝ a_out^4.5。
    let p2 = kozai_period(a_in, 2.0 * a_out, e_out, m_primary, m_tertiary);
    let expected_ratio = 2.0f64.powf(4.5);
    assert!(
        (p2 / p1 - expected_ratio).abs() < 1e-6,
        "kozai scaling p2/p1={} want {expected_ratio}",
        p2 / p1
    );
    // 非法输入返回 0。
    assert_eq!(kozai_period(0.0, a_out, e_out, m_primary, m_tertiary), 0.0);
    assert_eq!(kozai_period(a_in, a_out, e_out, m_primary, 0.0), 0.0);
}

#[test]
fn g_constant_sanity() {
    // G 引用，确保导入可用且量级正确。
    const { assert!(G > 6.0e-11 && G < 7.0e-11) };
}
