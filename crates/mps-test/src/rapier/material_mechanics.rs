#[cfg(test)]
mod tests {
    use mps_formula::material_mechanics::principal_stresses;

    #[test]
    fn principal_stresses_hydrostatic_state() {
        // Pure hydrostatic pressure: σx = σy = σz = -100 MPa, no shear.
        // Every direction is principal; all principal stresses equal the mean stress.
        let (s1, s2, s3) = principal_stresses(-100.0e6, -100.0e6, -100.0e6, 0.0, 0.0, 0.0)
            .expect("hydrostatic stress state is valid");
        assert!((s1 - -100.0e6).abs() < 1.0e-3);
        assert!((s2 - -100.0e6).abs() < 1.0e-3);
        assert!((s3 - -100.0e6).abs() < 1.0e-3);

        // Zero stress tensor is a degenerate hydrostatic state.
        let (s1, s2, s3) =
            principal_stresses(0.0, 0.0, 0.0, 0.0, 0.0, 0.0).expect("zero tensor is valid");
        assert_eq!(s1, 0.0);
        assert_eq!(s2, 0.0);
        assert_eq!(s3, 0.0);
    }

    #[test]
    fn principal_stresses_diagonal_tensor() {
        // Diagonal tensor: principal stresses are the diagonal entries, sorted descending.
        let (s1, s2, s3) =
            principal_stresses(300.0, 100.0, 200.0, 0.0, 0.0, 0.0).expect("diagonal tensor");
        assert!((s1 - 300.0).abs() < 1.0e-9);
        assert!((s2 - 200.0).abs() < 1.0e-9);
        assert!((s3 - 100.0).abs() < 1.0e-9);
    }

    #[test]
    fn principal_stresses_uniaxial_with_shear() {
        // σx = 100, τxy = 30 → principal stresses 100·(1/2 ± ...) style 2D result:
        // σ = 50 ± sqrt(50² + 30²) ≈ 108.31, -8.31, and 0.
        let (s1, s2, s3) =
            principal_stresses(100.0, 0.0, 0.0, 30.0, 0.0, 0.0).expect("valid tensor");
        let hi = 50.0 + (50.0f64 * 50.0 + 30.0 * 30.0).sqrt();
        let lo = 50.0 - (50.0f64 * 50.0 + 30.0 * 30.0).sqrt();
        assert!((s1 - hi).abs() < 1.0e-9, "s1={s1} expected {hi}");
        assert!((s2 - 0.0).abs() < 1.0e-9, "s2={s2}");
        assert!((s3 - lo).abs() < 1.0e-9, "s3={s3} expected {lo}");
    }

    #[test]
    fn principal_stresses_rejects_non_finite() {
        assert!(principal_stresses(f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0).is_none());
        assert!(principal_stresses(0.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0).is_none());
        assert!(principal_stresses(0.0, 0.0, 0.0, 0.0, 0.0, f64::NEG_INFINITY).is_none());
    }
}
