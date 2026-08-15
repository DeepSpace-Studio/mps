//! FFI wrapper tests for `mps_core::rapier::rel` (the C-ABI surface that wraps
//! `mps_formula::relativity`). Mirrors the `rel` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::rel::*;
    use std::ptr;

    const G: f64 = 6.67430e-11;
    const C: f64 = 299_792_458.0;

    #[test]
    fn schwarzschild_isco_is_6m() {
        let mut out = 0.0;
        // r_isco = 6 G M / c^2
        assert_eq!(
            relativity_schwarzschild_isco(1.0e30, G, &mut out),
            Bool::TRUE
        );
        let expected = 6.0 * G * 1.0e30 / (C * C);
        assert!((out - expected).abs() < expected * 1e-9);
    }

    #[test]
    fn cosmological_redshift_formula() {
        let mut z = 0.0;
        assert_eq!(relativity_cosmological_redshift(0.5, &mut z), Bool::TRUE);
        assert!((z - 1.0).abs() < 1e-12);
        let mut z2 = 0.0;
        assert_eq!(
            relativity_redshift_from_wavelengths(2.0, 1.0, &mut z2),
            Bool::TRUE
        );
        assert!((z2 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn chirp_mass_symmetric_limit() {
        let mut mc = 0.0;
        // equal masses -> M_c = m / 2^(1/5)
        assert_eq!(relativity_chirp_mass(2.0, 2.0, &mut mc), Bool::TRUE);
        let expected = (2.0_f64 * 2.0_f64).powf(0.6) / (4.0_f64).powf(0.2);
        assert!((mc - expected).abs() < 1e-9);
    }

    #[test]
    fn kerr_horizon_radii_writes_two_outputs() {
        let mut event = 0.0;
        let mut cauchy = 0.0;
        // m = G M / c^2 ; a = spin <= m
        let m = G * 1.0e30 / (C * C);
        let ok = relativity_kerr_horizon_radii(1.0e30, m * 0.5, G, &mut event, &mut cauchy);
        assert_eq!(ok, Bool::TRUE);
        // event = m + sqrt(m^2 - a^2), cauchy = m - sqrt(...)
        let expect_event = m + (m * m - (m * 0.5).powi(2)).sqrt();
        let expect_cauchy = m - (m * m - (m * 0.5).powi(2)).sqrt();
        assert!((event - expect_event).abs() < 1e-3);
        assert!((cauchy - expect_cauchy).abs() < 1e-3);
    }

    #[test]
    fn reissner_nordstrom_horizons_writes_two_outputs() {
        let mut outer = 0.0;
        let mut inner = 0.0;
        let m = G * 1.0e30 / (C * C);
        let ok = relativity_reissner_nordstrom_horizons(1.0e30, 0.0, G, &mut outer, &mut inner);
        assert_eq!(ok, Bool::TRUE);
        // Q=0 -> Schwarzschild: outer = 2m, inner = 0
        assert!((outer - 2.0 * m).abs() < 1e-3);
        assert!(inner.abs() < 1e-3);
    }

    #[test]
    fn doppler_longitudinal_approaching_vs_receding() {
        let mut up = 0.0;
        assert_eq!(
            relativity_relativistic_doppler_longitudinal(1.0e9, 1.0e7, Bool::TRUE, &mut up),
            Bool::TRUE
        );
        let mut down = 0.0;
        assert_eq!(
            relativity_relativistic_doppler_longitudinal(1.0e9, 1.0e7, Bool::FALSE, &mut down),
            Bool::TRUE
        );
        assert!(up > 1.0e9 && down < 1.0e9);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(
            relativity_schwarzschild_isco(1.0e30, G, ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(
            relativity_kerr_horizon_radii(1.0e30, 1.0, G, ptr::null_mut(), &mut 0.0),
            Bool::FALSE
        );
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        // scale_factor <= 0 -> None
        assert_eq!(relativity_cosmological_redshift(0.0, &mut out), Bool::FALSE);
    }
}
