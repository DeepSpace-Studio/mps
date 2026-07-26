#[cfg(test)]
mod tests {
    use mps_formula::nuclear::*;
    use std::f64::consts::LN_2;

    // ---- decay_constant ----

    #[test]
    fn decay_constant_from_half_life() {
        let lambda = decay_constant(2.0).expect("positive half-life");
        assert!((lambda - LN_2 / 2.0).abs() < 1.0e-15);
        // T½ of 1 s → λ = ln 2 ≈ 0.6931
        assert!((decay_constant(1.0).unwrap() - LN_2).abs() < 1.0e-15);
    }

    #[test]
    fn decay_constant_rejects_invalid() {
        assert!(decay_constant(0.0).is_none());
        assert!(decay_constant(-1.0).is_none());
        assert!(decay_constant(f64::NAN).is_none());
        assert!(decay_constant(f64::INFINITY).is_none());
    }

    // ---- remaining_nuclei / activity ----

    #[test]
    fn remaining_nuclei_after_one_half_life() {
        // N₀ = 1000, λ = ln2/10 (T½ = 10), t = 10 → N = 500
        let lambda = LN_2 / 10.0;
        let n = remaining_nuclei(1000.0, lambda, 10.0).expect("valid inputs");
        assert!((n - 500.0).abs() < 1.0e-9);
    }

    #[test]
    fn remaining_nuclei_edge_cases() {
        // t = 0 or λ = 0 leaves the population unchanged.
        assert_eq!(remaining_nuclei(123.0, 0.5, 0.0), Some(123.0));
        assert_eq!(remaining_nuclei(123.0, 0.0, 10.0), Some(123.0));
        // N₀ = 0 stays zero.
        assert_eq!(remaining_nuclei(0.0, 0.5, 10.0), Some(0.0));
        // Negative inputs rejected.
        assert!(remaining_nuclei(-1.0, 0.5, 1.0).is_none());
        assert!(remaining_nuclei(1.0, -0.5, 1.0).is_none());
        assert!(remaining_nuclei(1.0, 0.5, -1.0).is_none());
        assert!(remaining_nuclei(1.0, 0.5, f64::NAN).is_none());
    }

    #[test]
    fn activity_is_lambda_times_n() {
        let a = activity(0.5, 1.0e6).expect("valid inputs");
        assert_eq!(a, 5.0e5);
        assert_eq!(activity(0.0, 1.0e6), Some(0.0));
        assert_eq!(activity(0.5, 0.0), Some(0.0));
        assert!(activity(-0.1, 1.0).is_none());
        assert!(activity(0.1, -1.0).is_none());
        assert!(activity(f64::INFINITY, 1.0).is_none());
    }

    // ---- half_life / mean_lifetime ----

    #[test]
    fn half_life_roundtrips_decay_constant() {
        let lambda = decay_constant(5730.0).expect("C-14 half-life");
        let t_half = half_life(lambda).expect("positive lambda");
        assert!((t_half - 5730.0).abs() < 1.0e-9);
    }

    #[test]
    fn half_life_rejects_invalid() {
        assert!(half_life(0.0).is_none());
        assert!(half_life(-0.1).is_none());
        assert!(half_life(f64::NAN).is_none());
    }

    #[test]
    fn mean_lifetime_is_inverse_lambda() {
        assert_eq!(mean_lifetime(2.0), Some(0.5));
        assert_eq!(mean_lifetime(0.25), Some(4.0));
        assert!(mean_lifetime(0.0).is_none());
        assert!(mean_lifetime(-1.0).is_none());
    }

    // ---- Bethe–Weizsäcker binding energy ----

    #[test]
    fn binding_energy_fe56_matches_hand_computed() {
        // Fe-56: A = 56, Z = 26, N = 30 (even-even → positive pairing term).
        // volume = 15.75·56 = 882.0
        // surface = -17.80·56^(2/3) ≈ -260.547
        // coulomb = -0.711·26²/56^(1/3) ≈ -125.626
        // asymmetry = -23.70·(56-52)²/56 ≈ -6.7714
        // pairing = +11.18/√56 ≈ +1.4940
        // B ≈ 490.55 MeV (real value 492.3 MeV)
        let b = bethe_weizsaecker_binding_energy(56.0, 26.0).expect("valid nucleus");
        assert!((b - 490.55).abs() < 0.5, "B={b}");
        let per_nucleon = binding_energy_per_nucleon(56.0, 26.0).expect("valid nucleus");
        assert!((per_nucleon - 8.76).abs() < 0.02, "B/A={per_nucleon}");
        assert_eq!(per_nucleon, b / 56.0);
    }

    #[test]
    fn binding_energy_pairing_ordering() {
        // Even-even nuclei are more bound than odd-odd nuclei at the same A.
        // A = 4: (Z, N) = (2, 2) even-even vs (1, 3) even-odd vs (0, 4) even-even.
        let even_even = bethe_weizsaecker_binding_energy(4.0, 2.0).unwrap();
        let even_odd = bethe_weizsaecker_binding_energy(4.0, 1.0).unwrap();
        // A = 4, Z = 2: 63.0 - 44.853 - 1.7916 + 0 + 5.59 ≈ 21.95 MeV
        assert!((even_even - 21.95).abs() < 0.1, "B={even_even}");
        assert!(even_even > even_odd);
    }

    #[test]
    fn binding_energy_rejects_invalid() {
        assert!(bethe_weizsaecker_binding_energy(0.5, 0.0).is_none()); // A < 1
        assert!(bethe_weizsaecker_binding_energy(56.0, 57.0).is_none()); // Z > A
        assert!(bethe_weizsaecker_binding_energy(56.0, -1.0).is_none()); // Z < 0
        assert!(bethe_weizsaecker_binding_energy(f64::NAN, 26.0).is_none());
        assert!(binding_energy_per_nucleon(56.0, 57.0).is_none());
    }

    // ---- reaction_q_value / fusion & fission constants ----

    #[test]
    fn reaction_q_value_dt_fusion() {
        // D (2.014102) + T (3.016049) → He-4 (4.002603) + n (1.008665)
        // Δm = 0.018883 u → Q = 0.018883 × 931.494 ≈ 17.589 MeV
        let q =
            reaction_q_value(2.014102 + 3.016049, 4.002603 + 1.008665).expect("positive masses");
        assert!((q - 17.59).abs() < 0.01, "Q={q}");
        // Consistent with the hardcoded exact D-T Q-value.
        assert!((q - dt_fusion_q_value()).abs() < 0.01);
    }

    #[test]
    fn reaction_q_value_endothermic_is_negative() {
        // Final mass exceeding initial mass gives a negative (endothermic) Q.
        let q = reaction_q_value(1.0, 1.001).expect("positive masses");
        assert!(q < 0.0);
        assert!((q - -0.931494).abs() < 1.0e-6);
        // Equal masses give Q = 0.
        assert_eq!(reaction_q_value(12.0, 12.0), Some(0.0));
        // Non-positive / non-finite masses rejected.
        assert!(reaction_q_value(0.0, 1.0).is_none());
        assert!(reaction_q_value(-1.0, 1.0).is_none());
        assert!(reaction_q_value(1.0, f64::NAN).is_none());
    }

    #[test]
    fn fusion_fission_constants_have_physical_values() {
        assert_eq!(dt_fusion_energy(), 17.6);
        assert_eq!(dd_fusion_branch1_energy(), 4.0);
        assert_eq!(dd_fusion_branch2_energy(), 3.3);
        assert_eq!(u235_fission_energy(), 200.0);
        assert_eq!(dt_fusion_q_value(), 17.59);
        // D-T releases more energy than either D-D branch.
        assert!(dt_fusion_energy() > dd_fusion_branch1_energy());
        assert!(dt_fusion_energy() > dd_fusion_branch2_energy());
    }

    // ---- bateman_abundance ----

    #[test]
    fn bateman_single_nuclide_is_plain_decay() {
        // n = 1 reduces to N₁(t) = N₁(0)·exp(-λ₁t).
        let n = bateman_abundance(1000.0, &[LN_2], 1, 1.0).expect("valid chain");
        assert!((n - 500.0).abs() < 1.0e-9);
    }

    #[test]
    fn bateman_two_member_chain() {
        // λ₁ = 1, λ₂ = 2, N₀ = 1, t = ln 2:
        // N₂(t) = N₀ · λ₁/(λ₂-λ₁) · (e^{-λ₁t} - e^{-λ₂t}) = 1 · (0.5 - 0.25) = 0.25
        let n = bateman_abundance(1.0, &[1.0, 2.0], 2, LN_2).expect("valid chain");
        assert!((n - 0.25).abs() < 1.0e-12, "N2={n}");
    }

    #[test]
    fn bateman_rejects_invalid() {
        assert!(bateman_abundance(1.0, &[1.0], 0, 1.0).is_none()); // n = 0
        assert!(bateman_abundance(1.0, &[1.0], 2, 1.0).is_none()); // n > chain length
        assert!(bateman_abundance(1.0, &[], 1, 1.0).is_none()); // empty chain
        assert!(bateman_abundance(-1.0, &[1.0], 1, 1.0).is_none()); // negative parent
        assert!(bateman_abundance(1.0, &[1.0], 1, -1.0).is_none()); // negative time
        // Degenerate decay constants (λ₁ = λ₂) make the denominator vanish.
        assert!(bateman_abundance(1.0, &[1.0, 1.0], 2, 1.0).is_none());
    }

    // ---- neutron_flux_sphere ----

    #[test]
    fn neutron_flux_sphere_center_and_surface() {
        // r = 0 → φ = S/(4π·D·R). With S = 4π, D = 1, R = 1 → φ = 1.
        let center =
            neutron_flux_sphere(1.0, 1.0, 4.0 * std::f64::consts::PI, 0.0).expect("valid geometry");
        assert!((center - 1.0).abs() < 1.0e-12, "center={center}");
        // r = R → sin(B·R) = sin(π) = 0 → φ = 0.
        let surface =
            neutron_flux_sphere(1.0, 1.0, 4.0 * std::f64::consts::PI, 1.0).expect("r = R");
        assert!(surface.abs() < 1.0e-12, "surface={surface}");
        // r = R/2 → sin(π/2) = 1 → φ = S/(4π·D·R)·1/(R/2) = 2 with the above values.
        let mid = neutron_flux_sphere(1.0, 1.0, 4.0 * std::f64::consts::PI, 0.5).expect("r = R/2");
        assert!((mid - 2.0).abs() < 1.0e-12, "mid={mid}");
    }

    #[test]
    fn neutron_flux_sphere_rejects_invalid() {
        assert!(neutron_flux_sphere(0.0, 1.0, 1.0, 0.0).is_none()); // R = 0
        assert!(neutron_flux_sphere(1.0, 0.0, 1.0, 0.0).is_none()); // D = 0
        assert!(neutron_flux_sphere(1.0, 1.0, -1.0, 0.0).is_none()); // negative source
        assert!(neutron_flux_sphere(1.0, 1.0, 1.0, -0.1).is_none()); // r < 0
        assert!(neutron_flux_sphere(1.0, 1.0, 1.0, 1.1).is_none()); // r > R
    }

    // ---- four_factor_formula ----

    #[test]
    fn four_factor_typical_thermal_reactor() {
        // η = 2.07, ε = 1.03, p = 0.71, f = 0.84 → k_eff ≈ 1.2716 (supercritical)
        let k = four_factor_formula(2.07, 1.03, 0.71, 0.84).expect("valid factors");
        assert!((k - 1.2716).abs() < 1.0e-4, "k={k}");
        // Critical reactor: k = 1.
        let k_crit = four_factor_formula(2.0, 1.0, 0.5, 1.0).expect("valid factors");
        assert_eq!(k_crit, 1.0);
    }

    #[test]
    fn four_factor_rejects_out_of_range() {
        assert!(four_factor_formula(0.0, 1.0, 0.5, 0.5).is_none()); // η must be > 0
        assert!(four_factor_formula(2.0, 0.0, 0.5, 0.5).is_none()); // ε must be > 0
        assert!(four_factor_formula(2.0, 1.0, -0.1, 0.5).is_none()); // p < 0
        assert!(four_factor_formula(2.0, 1.0, 1.1, 0.5).is_none()); // p > 1
        assert!(four_factor_formula(2.0, 1.0, 0.5, -0.1).is_none()); // f < 0
        assert!(four_factor_formula(2.0, 1.0, 0.5, 1.1).is_none()); // f > 1
        // p = 0 is allowed and makes k = 0.
        assert_eq!(four_factor_formula(2.0, 1.0, 0.0, 0.5), Some(0.0));
    }

    // ---- cross sections / reaction rate ----

    #[test]
    fn macroscopic_cross_section_is_n_times_sigma() {
        assert_eq!(macroscopic_cross_section(0.05, 2.0), Some(0.1));
        assert_eq!(macroscopic_cross_section(0.0, 2.0), Some(0.0));
        assert!(macroscopic_cross_section(-1.0, 2.0).is_none());
        assert!(macroscopic_cross_section(1.0, -2.0).is_none());
        assert!(macroscopic_cross_section(f64::NAN, 2.0).is_none());
    }

    #[test]
    fn reaction_rate_is_sigma_times_flux() {
        let r = reaction_rate(0.1, 1.0e14).expect("valid inputs");
        assert!((r - 1.0e13).abs() < 1.0);
        assert_eq!(reaction_rate(0.0, 1.0e14), Some(0.0));
        assert!(reaction_rate(-0.1, 1.0e14).is_none());
        assert!(reaction_rate(0.1, -1.0).is_none());
    }

    // ---- atomic_mass_approx / specific_activity ----

    #[test]
    fn atomic_mass_approx_subtracts_binding_correction() {
        // Fe-56 with B ≈ 490.549 MeV: m ≈ 56 - 490.549/931.494 ≈ 55.4734 u
        let m = atomic_mass_approx(56.0, 490.549).expect("valid inputs");
        assert!((m - 55.4734).abs() < 1.0e-3, "m={m}");
        // Zero binding energy gives exactly A.
        assert_eq!(atomic_mass_approx(56.0, 0.0), Some(56.0));
        // Negative binding energy is accepted (only finiteness is required).
        assert!(atomic_mass_approx(56.0, -10.0).is_some());
        assert!(atomic_mass_approx(0.0, 10.0).is_none());
        assert!(atomic_mass_approx(-1.0, 10.0).is_none());
        assert!(atomic_mass_approx(56.0, f64::NAN).is_none());
    }

    #[test]
    fn specific_activity_uses_avogadro() {
        // SA = λ · N_A / A; λ = 1, A = 1 → SA = N_A
        let sa = specific_activity(1.0, 1.0).expect("valid inputs");
        assert!((sa - 6.022_140_76e23).abs() < 1.0e13);
        assert!(specific_activity(0.0, 1.0).is_none());
        assert!(specific_activity(1.0, 0.0).is_none());
        assert!(specific_activity(-1.0, 1.0).is_none());
    }

    // ---- gamma attenuation / half-value layer ----

    #[test]
    fn gamma_attenuation_beer_lambert() {
        // I₀ = 100, μ = ln 2, x = 1 → I = 50
        let i = gamma_attenuation(100.0, LN_2, 1.0).expect("valid inputs");
        assert!((i - 50.0).abs() < 1.0e-9);
        // x = 0 leaves intensity unchanged; μ = 0 disables attenuation.
        assert_eq!(gamma_attenuation(100.0, 1.0, 0.0), Some(100.0));
        assert_eq!(gamma_attenuation(100.0, 0.0, 10.0), Some(100.0));
        assert!(gamma_attenuation(-1.0, 1.0, 1.0).is_none());
        assert!(gamma_attenuation(1.0, -1.0, 1.0).is_none());
        assert!(gamma_attenuation(1.0, 1.0, -1.0).is_none());
    }

    #[test]
    fn half_value_layer_consistent_with_attenuation() {
        // HVL = ln 2 / μ; at x = HVL the intensity is exactly halved.
        let mu = 0.35;
        let hvl = half_value_layer(mu).expect("positive mu");
        assert!((hvl - LN_2 / mu).abs() < 1.0e-15);
        let i = gamma_attenuation(80.0, mu, hvl).expect("valid inputs");
        assert!((i - 40.0).abs() < 1.0e-9);
        assert!(half_value_layer(0.0).is_none());
        assert!(half_value_layer(-0.1).is_none());
        assert!(half_value_layer(f64::NAN).is_none());
    }
}
