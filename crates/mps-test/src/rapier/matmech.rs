//! FFI wrapper tests for `mps_core::rapier::matmech` (the C-ABI surface that
//! wraps `mps_formula::material_mechanics`). Mirrors the `matmech` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::matmech::*;
    use std::ptr;

    #[test]
    fn shear_modulus_writes_and_returns_true() {
        let mut out = 0.0f64;
        let ok = material_mechanics_shear_modulus(200.0e9, 0.3, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        let expected = 200.0e9 / (2.0 * (1.0 + 0.3));
        assert!(
            (out - expected).abs() < 1.0,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn hookes_law_uniaxial_writes_strain() {
        let mut out = 0.0f64;
        let ok = material_mechanics_hookes_law_uniaxial(100.0, 200.0, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        assert!((out - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn von_mises_yield_check_ratio() {
        let mut out = 0.0f64;
        let ok = material_mechanics_von_mises_yield_check(250.0, 250.0, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        assert!((out - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn rejects_nan_input() {
        let mut out = 0.0f64;
        assert_eq!(
            material_mechanics_shear_modulus(f64::NAN, 0.3, &mut out as *mut f64),
            Bool::FALSE
        );
        assert_eq!(out, 0.0);
    }

    #[test]
    fn rejects_null_out() {
        assert_eq!(
            material_mechanics_shear_modulus(200.0e9, 0.3, ptr::null_mut()),
            Bool::FALSE
        );
    }

    #[test]
    fn principal_stresses_ffi_writes_three_doubles() {
        let mut buf = [0.0f64; 3];
        let ok = material_mechanics_principal_stresses(
            300.0,
            100.0,
            200.0,
            0.0,
            0.0,
            0.0,
            buf.as_mut_ptr(),
        );
        assert_eq!(ok, Bool::TRUE);
        assert!((buf[0] - 300.0).abs() < 1.0e-9);
        assert!((buf[1] - 200.0).abs() < 1.0e-9);
        assert!((buf[2] - 100.0).abs() < 1.0e-9);
    }

    #[test]
    fn miners_damage_ffi_sums_ratios() {
        let ratios = [0.5f64, 0.5f64];
        let mut out = 0.0f64;
        let ok = material_mechanics_miners_damage(
            ratios.as_ptr(),
            ratios.len() as u32,
            &mut out as *mut f64,
        );
        assert_eq!(ok, Bool::TRUE);
        assert!((out - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn miners_damage_ffi_rejects_empty_and_null() {
        let mut out = 0.0f64;
        assert_eq!(
            material_mechanics_miners_damage(ptr::null(), 0, &mut out as *mut f64),
            Bool::FALSE
        );
    }

    #[test]
    fn bulk_and_lame_moduli_write() {
        let mut k = 0.0f64;
        let mut lam = 0.0f64;
        assert_eq!(
            material_mechanics_bulk_modulus(200.0e9, 0.3, &mut k as *mut f64),
            Bool::TRUE
        );
        assert_eq!(
            material_mechanics_lame_lambda(200.0e9, 0.3, &mut lam as *mut f64),
            Bool::TRUE
        );
        let k_exp = 200.0e9 / (3.0 * (1.0 - 2.0 * 0.3));
        let lam_exp = 200.0e9 * 0.3 / ((1.0 + 0.3) * (1.0 - 2.0 * 0.3));
        assert!((k - k_exp).abs() < 1.0);
        assert!((lam - lam_exp).abs() < 1.0);
    }

    #[test]
    fn fracture_and_fatigue_writes() {
        let mut kc = 0.0f64;
        let mut n = 0.0f64;
        assert_eq!(
            material_mechanics_ki_edge_crack(100.0, 0.02, &mut kc as *mut f64),
            Bool::TRUE
        );
        assert_eq!(
            material_mechanics_basquin_cycles_to_failure(100.0, 1000.0, -0.1, &mut n as *mut f64),
            Bool::TRUE
        );
        assert!(kc > 0.0);
        assert!(n > 0.0);
    }
}
