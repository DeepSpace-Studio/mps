#[cfg(test)]
mod tests {
    use mps_core::rapier::high_energy_astro::*;

    #[test]
    fn pulsar_characteristic_age_crab_matches_textbook() {
        // Crab: P=33 ms, Ṗ=4.2e-13 → τ = P/(2Ṗ) ≈ 1242 yr.
        let age = pulsar_characteristic_age(33.0, 4.2e-13).unwrap();
        assert!((age - 1242.0).abs() < 10.0, "age={age}");
    }

    #[test]
    fn rejects_negative_or_non_finite_pulsar_args() {
        assert!(pulsar_characteristic_age(-1.0, 4.2e-13).is_none());
        assert!(pulsar_characteristic_age(33.0, -1.0).is_none());
        assert!(pulsar_characteristic_age(33.0, 0.0).is_none());
    }

    #[test]
    fn pulsar_spin_down_luminosity_basic_units_check() {
        // I=1e38 kg m², P=1 s, Ṗ=1e-15 s/s
        // L = 4π²·I·Ṗ/P³ = 4π²·1e38·1e-15 / 1 = 3.947e24 W
        let l = pulsar_spin_down_luminosity(1.0e38, 1000.0, 1.0e-15).unwrap();
        // P was given in ms (1000) → P in seconds = 1.0 → 3.947e24 W
        assert!((l - 3.9478e24).abs() < 1.0e22, "l={l}");
    }

    #[test]
    fn pulsar_surface_b_field_matches_canonical_crab_value() {
        // Vacuum orthogonal rotator in SI: B² = 3μ₀c³IPṖ/(32π³R⁶), matching
        // the canonical B_s = 3.2e19·√(P·Ṗ) G.  Crab-style inputs
        // (P=33 ms, Ṗ=4.2e-13) give B ≈ 3.77e12 G = 3.77e8 T.
        let b = pulsar_surface_b_field(1.0e38, 1.0e4, 33.0, 4.2e-13).unwrap();
        assert!(b.is_finite() && b > 0.0, "b={b}");
        assert!(
            (b - 3.767e8).abs() < 1.0e6,
            "b={b} expected ≈3.77e8 T for the Crab"
        );
    }

    #[test]
    fn eddington_luminosity_basic_units() {
        // M=2e30 kg (≈1 Msun), κ=0.034 m²/kg → L_Edd ≈ 1.488e31 W
        let l = eddington_limited_luminosity(2.0e30, 0.034).unwrap();
        assert!((l - 1.476e31).abs() < 1.0e30, "l={l}");
    }
}
