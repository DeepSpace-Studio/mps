//! FFI wrapper tests for `mps_core::rapier::nucphys` (the C-ABI surface that
//! wraps `mps_formula::nuclear`). Mirrors the `nucphys` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::nucphys::*;
    use std::ptr;

    #[test]
    fn decay_constant_and_half_life_round_trip() {
        let hl: f64 = 5730.0;
        let mut lambda = 0.0;
        assert_eq!(nuclear_decay_constant(hl, &mut lambda), Bool::TRUE);
        let expected = std::f64::consts::LN_2 / hl;
        assert!((lambda - expected).abs() < 1e-9);
        let mut hl2 = 0.0;
        assert_eq!(nuclear_half_life(lambda, &mut hl2), Bool::TRUE);
        assert!((hl2 - hl).abs() < 1e-6);
    }

    #[test]
    fn activity_and_remaining_consistent() {
        let lambda = std::f64::consts::LN_2 / 5730.0;
        let n0: f64 = 1.0e24;
        let mut a = 0.0;
        assert_eq!(nuclear_activity(lambda, n0, &mut a), Bool::TRUE);
        assert!((a - lambda * n0).abs() < 1.0);
        let mut rem = 0.0;
        // after one half-life, half remains
        assert_eq!(
            nuclear_remaining_nuclei(n0, lambda, 5730.0, &mut rem),
            Bool::TRUE
        );
        assert!((rem - 0.5 * n0).abs() < 1.0);
    }

    #[test]
    fn fusion_fission_constants_are_positive() {
        let mut dt = 0.0;
        assert_eq!(nuclear_dt_fusion_energy(&mut dt), Bool::TRUE);
        assert!(dt > 0.0);
        let mut u235 = 0.0;
        assert_eq!(nuclear_u235_fission_energy(&mut u235), Bool::TRUE);
        assert!(u235 > 0.0);
        let mut q = 0.0;
        assert_eq!(nuclear_dt_fusion_q_value(&mut q), Bool::TRUE);
        assert!(q > 0.0);
    }

    #[test]
    fn reaction_q_value_mass_defect() {
        // 1 u mass defect -> ~931.5 MeV
        let mut q = 0.0;
        assert_eq!(nuclear_reaction_q_value(2.0, 1.0, &mut q), Bool::TRUE);
        assert!((q - 931.494_f64).abs() < 1.0);
    }

    #[test]
    fn null_output_returns_false() {
        assert_eq!(nuclear_half_life(1.0, ptr::null_mut()), Bool::FALSE);
    }

    #[test]
    fn invalid_input_returns_false() {
        let mut out = 0.0;
        // negative half-life -> None
        assert_eq!(nuclear_decay_constant(-1.0, &mut out), Bool::FALSE);
    }
}
