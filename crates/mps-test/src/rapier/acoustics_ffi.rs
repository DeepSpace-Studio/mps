//! FFI wrapper tests for `mps_core::rapier::acoustics_ffi` (the C-ABI surface
//! that wraps `mps_formula::acoustics`). Mirrors the `acoustics_ffi` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::acoustics_ffi::*;
    use mps_core::rapier::ffi::Bool;
    use std::ptr;

    #[test]
    fn spreading_loss_grows_with_log_range() {
        let mut s = 0.0;
        assert_eq!(acoustics_spherical_spreading_loss(10.0, &mut s), Bool::TRUE);
        assert!((s - 20.0 * 10.0_f64.log10()).abs() < 1e-6);
        let mut c = 0.0;
        assert_eq!(
            acoustics_cylindrical_spreading_loss(10.0, &mut c),
            Bool::TRUE
        );
        assert!((c - 10.0 * 10.0_f64.log10()).abs() < 1e-6);
    }

    #[test]
    fn helmholtz_resonance_formula() {
        let mut f = 0.0;
        let c = 343.0;
        let a = 0.01;
        let v = 0.001;
        let l = 0.05;
        assert_eq!(
            acoustics_helmholtz_resonance_frequency(c, a, v, l, &mut f),
            Bool::TRUE
        );
        let expected = c / (2.0 * std::f64::consts::PI) * (a / (v * l)).sqrt();
        assert!((f - expected).abs() < 1e-3);
    }

    #[test]
    fn doppler_shift_approach_lowers_or_raises() {
        let mut up = 0.0;
        // approaching -> higher frequency
        assert_eq!(
            acoustics_doppler_shift(1000.0, 343.0, 10.0, 0.0, Bool::TRUE, &mut up),
            Bool::TRUE
        );
        assert!(up > 1000.0);
        let mut down = 0.0;
        assert_eq!(
            acoustics_doppler_shift(1000.0, 343.0, 10.0, 0.0, Bool::FALSE, &mut down),
            Bool::TRUE
        );
        assert!(down < 1000.0);
    }

    #[test]
    fn transmission_coefficient_symmetric() {
        let mut t = 0.0;
        assert_eq!(
            acoustics_transmission_coefficient(1.0, 2.0, &mut t),
            Bool::TRUE
        );
        let mut t2 = 0.0;
        assert_eq!(
            acoustics_transmission_coefficient(2.0, 1.0, &mut t2),
            Bool::TRUE
        );
        assert!((t - t2).abs() < 1e-12);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(
            acoustics_spherical_spreading_loss(10.0, ptr::null_mut()),
            Bool::FALSE
        );
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        // negative range -> None
        assert_eq!(
            acoustics_spherical_spreading_loss(-1.0, &mut out),
            Bool::FALSE
        );
    }
}
