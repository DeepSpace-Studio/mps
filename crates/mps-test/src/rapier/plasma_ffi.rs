//! FFI wrapper tests for `mps_core::rapier::plasma_ffi` (the C-ABI surface that
//! wraps `mps_formula::plasma`). Mirrors the `plasma_ffi` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::plasma_ffi::*;
    use std::ptr;

    #[test]
    fn plasma_beta_zero_field_is_invalid() {
        let mut out = 0.0;
        assert_eq!(plasma_beta(1.0e19, 1.0e6, 0.0, &mut out), Bool::FALSE);
    }

    #[test]
    fn gyrofrequency_formula() {
        let mut f = 0.0;
        let q = 1.602e-19;
        let b = 1.0;
        let m = 1.67e-27;
        assert_eq!(plasma_gyrofrequency(q, b, m, &mut f), Bool::TRUE);
        let expected = q * b / m;
        assert!((f - expected).abs() < expected * 1e-6);
    }

    #[test]
    fn larmor_radius_formula() {
        let mut r = 0.0;
        let m = 1.67e-27;
        let v = 1.0e6;
        let q = 1.602e-19;
        let b = 1.0;
        assert_eq!(plasma_larmor_radius(m, v, q, b, &mut r), Bool::TRUE);
        let expected = m * v / (q.abs() * b);
        assert!((r - expected).abs() < expected * 1e-6);
    }

    #[test]
    fn mirror_ratio_and_loss_cone() {
        let mut ratio = 0.0;
        assert_eq!(plasma_mirror_ratio(4.0, 1.0, &mut ratio), Bool::TRUE);
        assert!((ratio - 4.0).abs() < 1e-9);
        let mut angle = 0.0;
        assert_eq!(
            plasma_mirror_loss_cone_angle(4.0, 1.0, &mut angle),
            Bool::TRUE
        );
        // sin(theta) = 1/sqrt(ratio) -> theta = asin(0.5) = 30 deg
        let expected = (1.0 / 4.0_f64.sqrt()).asin();
        assert!((angle - expected).abs() < 1e-9);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(plasma_mirror_ratio(4.0, 1.0, ptr::null_mut()), Bool::FALSE);
    }
}
