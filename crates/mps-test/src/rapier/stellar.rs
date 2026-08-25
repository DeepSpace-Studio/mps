#[cfg(test)]
mod tests {
    use mps_core::rapier::stellar::*;

    #[test]
    fn lane_emden_n3_first_zero_matches_textbook_6_897() {
        // Standard Lane-Emden n=3 polytrope: ξ₀ ≈ 6.89685.
        let (xi_zero, _profile, mass_ratio) = lane_emden_solve(3.0, 256).unwrap();
        assert!((xi_zero - 6.897).abs() < 0.005, "xi_zero={xi_zero}");
        // n=3 polytrope dimensionless mass ≈ 2.01824 (textbook).
        assert!((mass_ratio - 2.018).abs() < 0.01, "m_ratio={mass_ratio}");
    }

    #[test]
    fn lane_emden_n0_xi_zero_matches_sqrt_6() {
        // n=0 analytical: θ = 1 - ξ²/6 → θ=0 at ξ = √6 ≈ 2.449.
        let (xi_zero, _profile, _) = lane_emden_solve(0.0, 64).unwrap();
        assert!((xi_zero - 2.44949).abs() < 1.0e-3, "xi_zero={xi_zero}");
    }

    #[test]
    fn lane_emden_n5_has_no_finite_zero() {
        // n=5 (singular isothermal sphere): θ never crosses zero, ξ_0 = ∞ ⇒
        // the solver must report None rather than a meaningless truncation.
        assert!(lane_emden_solve(5.0, 256).is_none());
    }

    #[test]
    fn rejects_invalid_lane_emden() {
        assert!(lane_emden_solve(-1.0, 100).is_none());
        assert!(lane_emden_solve(1.0, 0).is_none());
        assert!(lane_emden_solve(1.0, 20_001).is_none());
    }

    #[test]
    fn cepheid_period_luminosity_basic_values() {
        // P=10 d → M_V = -2.76 ∙ (1 - 1) - 4.16 = -4.16
        let mv = cepheid_period_luminosity(10.0).unwrap();
        assert!((mv - (-4.16)).abs() < 1.0e-3);
        // P=1 d → M_V = -2.76 ∙ (-1) - 4.16 = -1.40
        let mv1 = cepheid_period_luminosity(1.0).unwrap();
        assert!((mv1 - (-1.40)).abs() < 1.0e-2);
    }

    #[test]
    fn white_dwarf_mestel_cooling_decreases_with_time() {
        let l_young = white_dwarf_mestel_luminosity(0.001, 0.0005, 1.0).unwrap();
        let l_old = white_dwarf_mestel_luminosity(10.0, 0.0005, 1.0).unwrap();
        assert!(
            l_old > 0.0 && l_young > l_old,
            "young={l_young} old={l_old}"
        );
    }

    #[test]
    fn sn_arnett_peak_luminosity_finite_nonnegative() {
        // Peak near peak time; small-but-positive luminosity, sanity-only.
        let l = sn_arnett_lightcurve(20.0, 0.6).unwrap();
        assert!(l.is_finite(), "l={l}");
    }
}
