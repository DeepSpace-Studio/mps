#[cfg(test)]
mod tests {
    use mps_core::rapier::cosmology::*;

    #[test]
    fn friedmann_low_z_matches_hubble_law() {
        // H0 = 70 km/s/Mpc, z = 0.01 → D_C ≈ c·z / H0 = 42.86 Mpc
        let d = friedmann_hubble_distance(70.0, 0.01).expect("valid args should return Some");
        let expected = 299_792.458 / 70.0 * 0.01; // (km/s → Mpc·km/s) ⇒ Mpc exactly: c·z/H0
        assert!((d - expected).abs() < 1.0e-3, "d={d} expected={expected}");
    }

    #[test]
    fn rejects_negative_redshift() {
        assert!(friedmann_hubble_distance(70.0, -0.1).is_none());
        // also rejects zero hubble constant
        assert!(friedmann_hubble_distance(0.0, 0.1).is_none());
        // NaN redshift
        assert!(friedmann_hubble_distance(70.0, f64::NAN).is_none());
    }

    #[test]
    fn luminosity_distance_factors_one_plus_z() {
        let d_l = luminosity_distance_hubble(70.0, 0.5).unwrap();
        let d_c = friedmann_hubble_distance(70.0, 0.5).unwrap();
        assert!((d_l - 1.5 * d_c).abs() < 1.0e-3);
    }

    #[test]
    fn einstein_de_sitter_age_for_h70_is_about_931_gyr_inverse_factor() {
        // t0 = 2/(3 H0) for matter-only EdS.
        // H0 = 70 km/s/Mpc → H0 ≈ 2.27e-18 1/s → t0 ≈ 2.93e17 s ≈ 9.28 Gyr.
        let t_gyr = einstein_de_sitter_age(70.0).unwrap();
        assert!((t_gyr - 9.28).abs() < 0.05, "t_gyr={t_gyr}");
    }

    #[test]
    fn hubble_flow_velocity_uses_hubble_law() {
        // v = H0 * D, with H0 in (km/s/Mpc) and D in Mpc → v in km/s directly.
        let v = hubble_flow_velocity(70.0, 100.0).unwrap();
        assert!((v - 7000.0).abs() < 1.0e-6);
    }
}
