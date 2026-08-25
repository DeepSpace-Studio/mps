//! FFI wrapper tests for `mps_core::rapier::qphys` (the C-ABI surface that
//! wraps `mps_formula::quantum`). Mirrors the `qphys` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::qphys::*;
    use std::ptr;

    #[test]
    fn bohr_radius_and_fine_structure_are_constants() {
        let mut a0 = 0.0;
        assert_eq!(quantum_bohr_radius(&mut a0), Bool::TRUE);
        assert!(a0 > 5.0e-11 && a0 < 5.3e-11);
        let mut alpha = 0.0;
        assert_eq!(quantum_fine_structure_constant(&mut alpha), Bool::TRUE);
        assert!((alpha - 1.0 / 137.036_f64).abs() < 1e-4);
    }

    #[test]
    fn hydrogen_energy_level_ground_state() {
        let mut e = 0.0;
        // n=1 -> -13.59844 eV
        assert_eq!(quantum_hydrogen_energy_level(1, &mut e), Bool::TRUE);
        assert!((e + 13.59844).abs() < 1e-4);
        let mut e2 = 0.0;
        // n=2 -> -3.39961 eV
        assert_eq!(quantum_hydrogen_energy_level(2, &mut e2), Bool::TRUE);
        assert!((e2 + 3.39961).abs() < 1e-4);
    }

    #[test]
    fn de_broglie_wavelength_formula() {
        let mut lam = 0.0;
        // electron m=9.11e-31, v=1e6
        assert_eq!(
            quantum_de_broglie_wavelength(9.11e-31, 1.0e6, &mut lam),
            Bool::TRUE
        );
        let h = 6.62607015e-34;
        let expected = h / (9.11e-31 * 1.0e6);
        assert!((lam - expected).abs() < expected * 1e-6);
    }

    #[test]
    fn degenerate_perturbation_writes_two_outputs() {
        let mut e1 = 0.0;
        let mut e2 = 0.0;
        // diagonal h11=h22=0, off-diag h12=1 -> eigenvalues ±1
        let ok = quantum_degenerate_perturbation_2x2(0.0, 1.0, 0.0, &mut e1, &mut e2);
        assert_eq!(ok, Bool::TRUE);
        assert!((e1 + 1.0).abs() < 1e-9 || (e1 - 1.0).abs() < 1e-9);
        assert!((e1 + e2).abs() < 1e-9);
    }

    #[test]
    fn time_evolution_phase_unit_magnitude() {
        let mut re = 0.0;
        let mut im = 0.0;
        let ok = quantum_time_evolution_phase(1.0, 1.0, &mut re, &mut im);
        assert_eq!(ok, Bool::TRUE);
        // |e^{-iEt/ħ}| = 1
        assert!((re * re + im * im - 1.0).abs() < 1e-9);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(quantum_bohr_radius(ptr::null_mut()), Bool::FALSE);
        assert_eq!(
            quantum_degenerate_perturbation_2x2(0.0, 1.0, 0.0, ptr::null_mut(), &mut 0.0),
            Bool::FALSE
        );
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        // n=0 -> None for hydrogen level
        assert_eq!(quantum_hydrogen_energy_level(0, &mut out), Bool::FALSE);
    }
}
