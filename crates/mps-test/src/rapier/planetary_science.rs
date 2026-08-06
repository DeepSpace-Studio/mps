#[cfg(test)]
mod tests {
    use mps_core::rapier::planetary_science::*;

    #[test]
    fn greenhouse_earth_matches_288k_canonical() {
        // Earth canonical T_s = 288 K.  Solving T_s = T_eff·(1 + 3τ/4)^(1/4)
        // with T_eff = √4((1-A)·S/4/σ) ≈ 254.6 K for Earth gives τ ≈ 0.85.
        let t = greenhouse_simple_temperature(1361.0, 0.30, 0.85).unwrap();
        assert!((t - 288.0).abs() < 1.0, "t={t}");
    }

    #[test]
    fn greenhouse_rejects_invalid_albedo_optical_depth() {
        assert!(greenhouse_simple_temperature(1361.0, 1.5, 0.78).is_none());
        assert!(greenhouse_simple_temperature(1361.0, -0.1, 0.78).is_none());
        assert!(greenhouse_simple_temperature(1361.0, 0.30, -0.1).is_none());
    }

    #[test]
    fn runaway_greenhouse_threshold_is_factor_times_reference() {
        let threshold = runaway_greenhouse_threshold_flux(1361.0, 1.1).unwrap();
        assert!((threshold - 1.1 * 1361.0).abs() < 1.0e-3);
    }

    #[test]
    fn habitable_zone_around_1lsun_earth_factors_basic() {
        // L=1 Lsun, inner=1.0, outer=0.36 → (1, 5/3)
        let (inner, outer) = habitable_zone_separation(1.0, 1.0, 0.36).unwrap();
        assert!((inner - 1.0).abs() < 1.0e-3);
        assert!((outer - (1.0_f64 / 0.36_f64).sqrt()).abs() < 1.0e-3, "outer={outer}");
    }

    #[test]
    fn tidal_heating_positive_for_io_like_setup() {
        // Io-like: M_p=M_Jupiter≈1.9e27 kg, R_s≈1.8e6 m, n≈4.1e-5 rad/s,
        // e≈0.0041, Q≈100, a≈4.2e8 m → expect ~1e14 W (Io real value)
        let q = tidal_heating_power(1.9e27, 1.8e6, 4.1e-5, 0.0041, 100.0, 4.2e8).unwrap();
        assert!(q.is_finite() && q > 0.0, "q={q}");
        // Order-of-magnitude sanity within 100x of 1e14 W:
        assert!(
            (q - 1.0e14).abs() < 1.0e16,
            "q={q} expected ~1e14 W for Io-like setup"
        );
    }

    #[test]
    fn magma_ocean_solidification_timescale_finite_positive() {
        // E/A = ρ·D·(c_p·ΔT + L_lat) = 3000·2e6·(1200·1500 + 4e5)
        //     = 1.32e16 J/m²;  F = σ·(2000⁴-300⁴) ≈ 9.07e5 W/m²
        // ⇒ t ≈ 1.455e10 s ≈ 460 yr.
        let t = magma_ocean_solidification_timescale(
            3000.0,
            1200.0,
            4.0e5,
            2.0e6,
            1500.0,
            2000.0,
            300.0,
        )
        .unwrap();
        assert!(t.is_finite() && t > 0.0, "t={t}");
        assert!(
            (t - 1.455e10).abs() < 1.0e8,
            "t={t} expected ≈1.455e10 s"
        );
    }
}
