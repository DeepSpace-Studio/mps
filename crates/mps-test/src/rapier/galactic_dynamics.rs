#[cfg(test)]
mod tests {
    use mps_core::rapier::galactic_dynamics::*;

    #[test]
    fn toomre_q_solar_neighborhood_is_marginally_stable() {
        // Solar-neighborhood values in self-consistent galactic units:
        //   σ = 30 km/s (stellar radial dispersion)
        //   κ = 37 (km/s)/kpc = 0.037 (km/s)/pc  (must use pc, matching G_UNITS)
        //   Σ = 50 Msun/pc²
        // Q = σ·κ / (π·G·Σ) = 30·0.037 / (π·4.302e-3·50) ≈ 1.64,
        // the expected marginally-stable regime (Q ≈ 1.5–2).
        let q = toomre_q(30.0, 0.037, 50.0).unwrap();
        assert!(q.is_finite() && q > 0.0, "q={q}");
        assert!((q - 1.643).abs() < 0.01, "q={q} expected ≈1.64");
    }

    #[test]
    fn toomre_q_rejects_non_positive_args() {
        assert!(toomre_q(0.0, 0.037, 50.0).is_none());
        assert!(toomre_q(30.0, 0.0, 50.0).is_none());
        assert!(toomre_q(30.0, 0.037, -1.0).is_none());
    }

    #[test]
    fn chandrasekhar_friction_returns_finite_positive() {
        // G Earlier check: a_df = 4π G² M ρ ln Λ / v²
        // M = 1e9 Msun ≈ 2e39 kg, ρ ≈ 1e-22 kg/m³ (galaxy halo neighborhood),
        // v = 200 km/s = 2e5 m/s, ln Λ = 10
        let a = chandrasekhar_dynamical_friction(2.0e39, 1.0e-22, 2.0e5, 10.0).unwrap();
        assert!(a.is_finite() && a > 0.0, "a={a}");
    }

    #[test]
    fn mond_acceleration_yields_sqrt_of_newtonian_times_a0() {
        // a_N = 1e-10 m/s², a_0 = 1.2e-10 → a_MOND = sqrt(1.2e-20) = 1.0954e-10
        let a = mond_acceleration(1.0e-10, 1.2e-10).unwrap();
        assert!((a - 1.0954e-10).abs() < 1.0e-14, "a={a}");
    }

    #[test]
    fn free_fall_timescale_is_in_seconds() {
        // ρ=1e-20 kg/m³ (~1 H-atom permeable ISM) ⇒ t_ff = √(3π/(32G·ρ))
        //   = √(3π/(32·6.674e-11·1e-20))
        //   = √(4.413e29) ≈ 6.64e14 s  ≈ 2.10e7 yr
        let t = free_fall_timescale(1.0e-20).unwrap();
        assert!((t - 6.64e14).abs() < 5.0e12, "t={t}");
    }

    #[test]
    fn stromgren_radius_basic_units_check() {
        // ṅ = 1e49 s⁻¹, α_B = 2.6e-13 cm³/s = 2.6e-19 m³/s (NOT 2.6e-19, recheck):
        // 1 cm³ = 1e-6 m³ ⇒ α_B = 2.6e-19 m³/s
        let r = stromgren_radius(1.0e49, 2.6e-19, 1.0e6).unwrap();
        // r³ ≈ 3·1e49 / (4π · 2.6e-19 · (1e6)²) = 3e49/(4π·2.6e-7) = 3e49/3.27e-6
        //     ≈ 9.17e54 m³ ⇒ r ≈ 2.09e18 m (≈ 70 pc)
        assert!((r - 2.09e18).abs() < 1.0e17, "r={r}");
    }
}
