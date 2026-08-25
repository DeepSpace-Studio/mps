//! FFI wrapper tests for `mps_core::rapier::thermo` (the C-ABI surface that
//! wraps the ideal-gas / polytropic parts of `mps_formula::thermodynamics`).
//! Mirrors the `thermo` core module.
#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::Bool;
    use mps_core::rapier::thermo::*;
    use std::ptr;

    const R_GAS: f64 = 8.314462618;

    #[test]
    fn ideal_gas_pressure_writes_and_returns_true() {
        let mut out = 0.0f64;
        let ok = thermodynamics_ideal_gas_pressure(1.0, 1.0, 300.0, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        let expected = R_GAS * 300.0;
        assert!(
            (out - expected).abs() < 1.0e-6,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn ideal_gas_volume_writes() {
        let mut out = 0.0f64;
        let ok = thermodynamics_ideal_gas_volume(R_GAS * 300.0, 1.0, 300.0, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        let expected = 1.0 * R_GAS * 300.0 / (R_GAS * 300.0);
        assert!(
            (out - expected).abs() < 1.0e-6,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn ideal_gas_temperature_writes() {
        let mut out = 0.0f64;
        let ok =
            thermodynamics_ideal_gas_temperature(R_GAS * 300.0, 1.0, 1.0, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        let expected = (R_GAS * 300.0) * 1.0 / (1.0 * R_GAS);
        assert!(
            (out - expected).abs() < 1.0e-6,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn ideal_gas_pressure_rejects_zero_volume() {
        let mut out = 0.0f64;
        assert_eq!(
            thermodynamics_ideal_gas_pressure(0.0, 1.0, 300.0, &mut out as *mut f64),
            Bool::FALSE
        );
    }

    #[test]
    fn polytropic_pressure_writes() {
        let mut out = 0.0f64;
        let ok = thermodynamics_polytropic_pressure(100.0, 1.0, 2.0, 1.4, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        let expected = 100.0 * 1.0f64.powf(1.4) / 2.0f64.powf(1.4);
        assert!(
            (out - expected).abs() < 1.0e-6,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn polytropic_work_writes() {
        let mut out = 0.0f64;
        let ok = thermodynamics_polytropic_work(100.0, 1.0, 50.0, 2.0, 1.4, &mut out as *mut f64);
        assert_eq!(ok, Bool::TRUE);
        // W = (P2 V2 - P1 V1) / (1 - γ); use the same P2 passed in (50.0).
        let p1 = 100.0f64;
        let v1 = 1.0f64;
        let p2 = 50.0f64;
        let v2 = 2.0f64;
        let gamma = 1.4f64;
        let expected = (p2 * v2 - p1 * v1) / (1.0 - gamma);
        assert!(
            (out - expected).abs() < 1.0e-3,
            "out={out} expected={expected}"
        );
    }

    #[test]
    fn rejects_null_out() {
        assert_eq!(
            thermodynamics_ideal_gas_pressure(1.0, 1.0, 300.0, ptr::null_mut()),
            Bool::FALSE
        );
    }
}
