//! FFI wrapper tests for `mps_core::rapier::astrocalc` (the C-ABI surface that
//! wraps `mps_formula::astrophysics`). Mirrors the `astrocalc` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::astrocalc::*;
    use mps_core::rapier::ffi::Bool;
    use std::ptr;

    #[test]
    fn hill_sphere_radius_writes_value() {
        let mut out = 0.0;
        let ok = astrophysics_hill_sphere_radius(1.0e26, 1.0e22, 1.0e8, 0.0, &mut out);
        assert_eq!(ok, Bool::TRUE);
        let expected = 1.0e8 * (1.0e22_f64 / (3.0_f64 * 1.0e26_f64)).cbrt();
        assert!((out - expected).abs() < 1e-3);
    }

    #[test]
    fn hubble_velocity_and_distance_round_trip() {
        let h0: f64 = 70.0;
        let d: f64 = 1.0e9;
        let mut v = 0.0;
        assert_eq!(astrophysics_hubble_velocity(h0, d, &mut v), Bool::TRUE);
        assert!((v - h0 * d).abs() < 1e-3);
        let mut d2 = 0.0;
        assert_eq!(astrophysics_hubble_distance(v, h0, &mut d2), Bool::TRUE);
        assert!((d2 - d).abs() < 1e-3);
    }

    #[test]
    fn chandrasekhar_limits_are_constants() {
        let mut m1 = 0.0;
        assert_eq!(astrophysics_chandrasekhar_mass_limit(&mut m1), Bool::TRUE);
        assert!((m1 - 1.44).abs() < 1e-9);
        let mut m2 = 0.0;
        assert_eq!(astrophysics_chandrasekhar_mass_kg(&mut m2), Bool::TRUE);
        assert!((m2 - 2.865e30).abs() < 1e20);
    }

    #[test]
    fn roche_limit_writes_two_outputs() {
        let mut fluid = 0.0;
        let mut rigid = 0.0;
        let ok = astrophysics_roche_limit(1.0e6, 3.0e3, 3.0e3, &mut fluid, &mut rigid);
        assert_eq!(ok, Bool::TRUE);
        assert!((fluid - 2.44e6).abs() < 1.0);
        assert!((rigid - 1.26e6).abs() < 1.0);
    }

    #[test]
    fn habitable_zone_writes_two_outputs() {
        let mut inner = 0.0;
        let mut outer = 0.0;
        let ok = astrophysics_habitable_zone_boundaries(1.0, &mut inner, &mut outer);
        assert_eq!(ok, Bool::TRUE);
        assert!((inner - (1.0 / 1.1_f64).sqrt()).abs() < 1e-9);
        assert!((outer - (1.0 / 0.53_f64).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(
            astrophysics_wien_displacement(5000.0, ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(
            astrophysics_roche_limit(1.0e6, 3.0e3, 3.0e3, ptr::null_mut(), &mut 0.0),
            Bool::FALSE
        );
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        assert_eq!(
            astrophysics_hill_sphere_radius(-1.0, 1.0e22, 1.0e8, 0.0, &mut out),
            Bool::FALSE
        );
    }
}
