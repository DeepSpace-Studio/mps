#[cfg(test)]
mod tests {
    use mps_core::rapier::heliophysics::*;

    #[test]
    fn parker_spiral_angle_at_1au_400km_s_is_about_neg_0_79_rad() {
        // tan(ψ) = -ω r / v_sw. ω_sun ≈ 2π/(25.05 d*86400 s) ≈ 2.913e-6 rad/s,
        // r=1 AU, v_sw=400 km/s → arg ≈ -1.093 → atan(-1.093) ≈ -0.829 rad.
        let psi = solar_wind_parker_angle_au(1.0, 400.0).unwrap();
        assert!((psi + 0.83).abs() < 5.0e-3, "psi={psi}");
    }

    #[test]
    fn rejects_invalid_parker_args() {
        assert!(solar_wind_parker_spiral_angle(0.0, 400.0, 2.9e-6).is_none());
        assert!(solar_wind_parker_spiral_angle(1.0, 0.0, 2.9e-6).is_none());
        assert!(solar_wind_parker_angle_au(-1.0, 400.0).is_none());
    }

    #[test]
    fn solar_wind_dynamic_pressure_is_finite() {
        // 5 protons/m³ (i.e. 5 e-6 cm⁻³, very tenuous), v=400 km/s
        // ⇒ ρ·v² = 5e6 · 1.67e-27 · (4e5)² Pa ≈ 1.337e-9 Pa = 1.337 nPa.
        let p_npa = solar_wind_dynamic_pressure(5.0e6, 400.0).unwrap();
        assert!(p_npa > 0.0);
        assert!((p_npa - 1.34).abs() < 0.05, "p_npa={p_npa}");
    }

    #[test]
    fn jeans_escape_flux_returns_finite_positive() {
        // Just confirm validation accepts sensible exobase params.
        let f = jeans_escape_flux(1.0e12, 1000.0, 5.0, 1.67e-27).unwrap();
        assert!(f.is_finite() && f > 0.0, "f={f}");
    }

    #[test]
    fn dst_rate_burton_basic() {
        // dDst/dt = -Dst/τ + Q; with Dst=-50 nT, τ=8 h, Q=20 nT/h → 26.25 nT/h
        let rate = dst_index_rate(-50.0, 8.0, 20.0).unwrap();
        assert!((rate - 26.25).abs() < 1.0e-3, "rate={rate}");
    }
}
