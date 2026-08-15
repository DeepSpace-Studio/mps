#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::quantum::*;

    #[test]
    fn wave_probability_and_normalization_work() {
        let wave = QuantumWaveFunction {
            amplitude_real: 3.0,
            amplitude_imag: 4.0,
        };
        assert_eq!(quantum_wave_probability_density(wave), 25.0);

        let mut normalized = QuantumWaveFunction::default();
        assert_eq!(quantum_wave_normalize(wave, &mut normalized), Bool::TRUE);
        assert!((quantum_wave_probability_density(normalized) - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn tunneling_probability_uses_wkb_decay() {
        let barrier = QuantumBarrier {
            particle_energy: 1.0,
            barrier_potential: 5.0,
            barrier_width: 0.5,
            particle_mass: 1.0,
            reduced_planck: 1.0,
        };
        let mut report = QuantumTunnelingReport::default();
        assert_eq!(
            quantum_rectangular_barrier_tunneling(barrier, &mut report),
            Bool::TRUE
        );
        assert!(report.decay_constant > 0.0);
        assert!(report.transmission_coefficient > 0.0);
        assert!(report.transmission_coefficient < 1.0);
        assert!(
            (report.transmission_coefficient - quantum_wkb_transmission(2.0_f64.sqrt(), 1.0)).abs()
                < 1.0e-12
        );
    }

    #[test]
    fn zero_point_energy_is_half_hbar_omega() {
        assert_eq!(quantum_zero_point_energy(4.0, 2.0), 4.0);

        let mut report = QuantumOscillatorReport::default();
        assert_eq!(
            quantum_harmonic_oscillator_report(4.0, 2.0, &mut report),
            Bool::TRUE
        );
        assert_eq!(report.zero_point_energy, 4.0);
        assert_eq!(report.first_excited_energy, 12.0);
        assert_eq!(report.level_spacing, 8.0);
    }

    #[test]
    fn photoelectric_threshold_and_kinetic() {
        // Work function of sodium ~ 3.04 eV = 4.87e-19 J → f0 = W/h ≈ 7.35e14 Hz.
        let w = 4.866e-19;
        let f0 = photoelectric_threshold(w).unwrap();
        assert!((f0 - 7.347e14).abs() / 7.347e14 < 1.0e-3, "f0={f0}");
        // Photon at 1e15 Hz: K = h·f − W > 0.
        let k = photoelectric_max_kinetic(1.0e15, w).unwrap();
        assert!(k > 0.0);
        // Below threshold: K clamps to 0.
        let k_low = photoelectric_max_kinetic(1.0e14, w).unwrap();
        assert_eq!(k_low, 0.0);
        assert!(photoelectric_max_kinetic(-1.0, w).is_none());
        assert!(photoelectric_threshold(-1.0).is_none());
    }

    #[test]
    fn compton_shift_matches_canonical() {
        // Classical electron Compton wavelength h/(m_e c) = 2.4263e-12 m,
        // so backscatter (θ=π) gives Δλ = 2·h/(m_e c) = 4.8526e-12 m.
        let dlambda_back = compton_wavelength_shift(std::f64::consts::PI).unwrap();
        assert!(
            (dlambda_back - 4.8526e-12).abs() / 4.8526e-12 < 1.0e-3,
            "{dlambda_back}"
        );
        // Forward scatter (θ=0) → zero shift.
        assert!(compton_wavelength_shift(0.0).unwrap().abs() < 1.0e-15);
        // λ' = λ + Δλ.
        let lambda1 = 1.0e-11;
        let lp = compton_scattered_wavelength(lambda1, std::f64::consts::PI).unwrap();
        assert!((lp - (lambda1 + dlambda_back)).abs() < 1.0e-20);
    }

    #[test]
    fn rabi_oscillation_returns_probability() {
        // On resonance (δ=0) at t = π/Ω: P = sin²(π/2) = 1.
        let om = 1.0e9;
        let p = rabi_oscillation_probability(om, 0.0, std::f64::consts::PI / om).unwrap();
        assert!((p - 1.0).abs() < 1.0e-12, "p={p}");
        // At t=0: P=0.
        assert!(rabi_oscillation_probability(om, 0.0, 0.0).unwrap().abs() < 1.0e-15);
        // Off-resonance peak is suppressed: 0 < P < 1.
        let p_off = rabi_oscillation_probability(om, om, std::f64::consts::FRAC_PI_2 / om).unwrap();
        assert!(p_off > 0.0 && p_off < 1.0, "p_off={p_off}");
    }

    #[test]
    fn landau_level_ground_magnetic() {
        // E_0 = ½·(eB/m)·ħ for ground state in B=1 T.
        let b = 1.0;
        let e = 1.602_176_634e-19;
        let m = 9.109_383_701_5e-31;
        let hbar = 1.054_571_817e-34;
        let e0 = landau_level(0, b, e, m).unwrap();
        let expected = 0.5 * (e * b / m) * hbar;
        assert!((e0 - expected).abs() / expected < 1.0e-9, "e0={e0}");
        // n=1 is exactly one quantum above n=0: E_1 - E_0 = S = 2·expected
        // (expected = 0.5·S).  Each step adds the full cyclotron spacing S.
        let e1 = landau_level(1, b, e, m).unwrap();
        assert!(
            (e1 - e0 - 2.0 * expected).abs() / expected < 1.0e-12,
            "e1={e1}"
        );
        assert!(landau_level(-1, b, e, m).is_none());
    }

    #[test]
    fn einstein_a_coefficient_positive_and_scales() {
        // For a fixed dipole, A ∝ ω³ → doubling frequency multiplies A by 8.
        let d = 1.0e-29;
        let a1 = einstein_a_coefficient(1.0e14, d).unwrap();
        let a2 = einstein_a_coefficient(2.0e14, d).unwrap();
        assert!(a1 > 0.0);
        assert!((a2 / a1 - 8.0).abs() / 8.0 < 1.0e-6, "ratio={}", a2 / a1);
        assert!(einstein_a_coefficient(-1.0e14, d).is_none());
    }

    #[test]
    fn clebsch_gordan_triangle_rule() {
        // j1=1, j2=1 → j3 must satisfy 0 ≤ j3 ≤ 2; with m1=1, m2=1, m3=2 allowed.
        assert_eq!(
            clebsch_gordan_allowed(1.0, 1.0, 2.0, 1.0, 1.0, 2.0).unwrap(),
            1.0
        );
        // m1+m2 ≠ m3 → forbidden.
        assert_eq!(
            clebsch_gordan_allowed(1.0, 1.0, 2.0, 1.0, 1.0, 1.0).unwrap(),
            0.0
        );
        // Outside triangle: j3=2.5 > j1+j2 → forbidden.
        assert_eq!(
            clebsch_gordan_allowed(1.0, 1.0, 2.5, 1.0, 1.0, 2.0).unwrap(),
            0.0
        );
    }
}
