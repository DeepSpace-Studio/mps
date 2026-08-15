//! FFI wrapper tests for `mps_core::rapier::emag` (the C-ABI surface that
//! wraps `mps_formula::electromagnetism`). Mirrors the `emag` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::emag::*;
    use mps_core::rapier::ffi::Bool;
    use std::ptr;

    #[test]
    fn phase_velocity_and_wavelength_round_trip() {
        let n: f64 = 1.5;
        let mut v = 0.0;
        assert_eq!(electromagnetism_phase_velocity(n, &mut v), Bool::TRUE);
        assert!((v - 299_792_458.0 / n).abs() < 1e-3);
        let mut lam = 0.0;
        assert_eq!(
            electromagnetism_vacuum_wavelength(1.0e9, &mut lam),
            Bool::TRUE
        );
        assert!((lam - 299_792_458.0 / 1.0e9).abs() < 1e-3);
    }

    #[test]
    fn reflection_vswr_return_loss_consistent() {
        let zl = 75.0;
        let z0 = 50.0;
        let mut gamma = 0.0;
        assert_eq!(
            electromagnetism_reflection_coefficient(zl, z0, &mut gamma),
            Bool::TRUE
        );
        let expected_gamma = (zl - z0) / (zl + z0);
        assert!((gamma - expected_gamma).abs() < 1e-9);
        let mut vswr = 0.0;
        assert_eq!(electromagnetism_vswr(gamma, &mut vswr), Bool::TRUE);
        let expected_vswr = (1.0 + gamma.abs()) / (1.0 - gamma.abs());
        assert!((vswr - expected_vswr).abs() < 1e-9);
    }

    #[test]
    fn half_wave_dipole_directivity_constant() {
        let mut d = 0.0;
        assert_eq!(
            electromagnetism_half_wave_dipole_directivity(&mut d),
            Bool::TRUE
        );
        assert!((d - 1.64).abs() < 1e-9);
    }

    #[test]
    fn transmission_line_writes_two_outputs() {
        let mut real = 0.0;
        let mut imag = 0.0;
        // z0=50, z_load=50 (real), beta=0, length=any -> Z_in = 50 + j0
        let ok = electromagnetism_transmission_line_input_impedance(
            50.0, 50.0, 0.0, 0.0, 1.0, &mut real, &mut imag,
        );
        assert_eq!(ok, Bool::TRUE);
        assert!((real - 50.0).abs() < 1e-6);
        assert!(imag.abs() < 1e-6);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(
            electromagnetism_wave_frequency(1.0, ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(
            electromagnetism_transmission_line_input_impedance(
                50.0,
                50.0,
                0.0,
                0.0,
                1.0,
                ptr::null_mut(),
                &mut 0.0,
            ),
            Bool::FALSE
        );
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        // negative refractive index -> None
        assert_eq!(electromagnetism_phase_velocity(-1.0, &mut out), Bool::FALSE);
        // |Gamma| >= 1 -> None
        assert_eq!(electromagnetism_vswr(1.5, &mut out), Bool::FALSE);
    }
}
