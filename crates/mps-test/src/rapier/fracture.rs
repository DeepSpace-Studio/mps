#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
        last_error_code,
    };
    use mps_core::rapier::ffi::Vec3;
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::fracture::*;
    use mps_core::rapier::world::{world_create, world_destroy};
    use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    #[test]
    fn fracture_formulas_work() {
        let mut intensity = StressIntensityReport::default();
        assert_eq!(
            fracture_stress_intensity_factor(100.0, 0.01, 1.0, 10.0, &mut intensity),
            Bool::TRUE
        );
        assert!(intensity.stress_intensity > 0.0);
        assert_eq!(intensity.critical, Bool::TRUE);

        let material = FractureMaterial {
            youngs_modulus: 200.0e9,
            poisson_ratio: 0.3,
            fracture_toughness: 50.0e6,
            surface_energy: 10.0,
            density: 7850.0,
        };
        let mut griffith = GriffithReport::default();
        assert_eq!(
            fracture_griffith_criterion(1.0e6, 0.01, material, &mut griffith),
            Bool::TRUE
        );
        assert!(griffith.critical_stress > 0.0);
        assert_eq!(griffith.critical_energy_release_rate, 20.0);

        let cycles = [100.0, 50.0];
        let lives = [1000.0, 500.0];
        let mut damage = MinerDamageReport::default();
        assert_eq!(
            fracture_miner_damage(
                cycles.as_ptr(),
                lives.as_ptr(),
                cycles.len() as u32,
                &mut damage
            ),
            Bool::TRUE
        );
        assert!((damage.damage - 0.2).abs() < 1.0e-12);
        assert_eq!(damage.failed, Bool::FALSE);

        let mut sn = SnCurveReport::default();
        assert_eq!(
            fracture_sn_curve_life(50.0, 1.0e12, 3.0, 100.0, &mut sn),
            Bool::TRUE
        );
        assert_eq!(sn.infinite_life, Bool::TRUE);

        let mut energy = FractureEnergyReport::default();
        assert_eq!(
            fracture_energy_release(120.0, 10.0, 8.0, 0.0, &mut energy),
            Bool::TRUE
        );
        assert_eq!(energy.will_fracture, Bool::TRUE);
        assert_eq!(energy.fragment_kinetic_energy, 40.0);

        let mut mode = FractureModeReport::default();
        assert_eq!(
            fracture_mode_from_stress(1.0, 3.0, 2.0, &mut mode),
            Bool::TRUE
        );
        assert_eq!(mode.mode, 2);
    }

    #[test]
    fn fracture_replaces_body_with_connected_fragments() {
        let world = world_create(v3(0.0, -9.81, 0.0));
        assert!(!world.is_null());
        let world = unsafe { &mut *world };

        let source = world
            .inner
            .bodies
            .insert(RigidBodyBuilder::dynamic().build());
        world.inner.colliders.insert_with_parent(
            ColliderBuilder::cuboid(1.0, 1.0, 1.0).density(1.0).build(),
            source,
            &mut world.inner.bodies,
        );

        let fragments = [
            FractureFragmentDesc {
                local_center: v3(-0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(-1.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
            FractureFragmentDesc {
                local_center: v3(0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(1.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
        ];
        let mut bodies = [0; 2];
        let mut joints = [0; 2];
        let mut report = FractureReplaceReport::default();
        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                pack_rigid_body_handle(source),
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::TRUE,
                Bool::TRUE,
                bodies.as_mut_ptr(),
                joints.as_mut_ptr(),
                bodies.len() as u32,
                &mut report,
            ),
            Bool::TRUE
        );
        assert_eq!(report.fragment_count, 2);
        assert_eq!(report.joint_count, 1);
        assert_eq!(report.removed_source, Bool::TRUE);
        assert!(bodies.iter().all(|handle| *handle != 0));
        assert_ne!(joints[0], 0);
        assert_eq!(world.inner.bodies.len(), 2);
    }

    fn valid_material() -> FractureMaterial {
        FractureMaterial {
            youngs_modulus: 200.0e9,
            poisson_ratio: 0.3,
            fracture_toughness: 50.0e6,
            surface_energy: 10.0,
            density: 7850.0,
        }
    }

    fn valid_fragments() -> [FractureFragmentDesc; 2] {
        [
            FractureFragmentDesc {
                local_center: v3(-0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(-1.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
            FractureFragmentDesc {
                local_center: v3(0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(1.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
        ]
    }

    fn world_with_source() -> (*mut WorldHandle, RigidBodyHandleRaw) {
        let world = world_create(v3(0.0, -9.81, 0.0));
        assert!(!world.is_null());
        let source = unsafe { &mut *world }
            .inner
            .bodies
            .insert(RigidBodyBuilder::dynamic().build());
        (world, pack_rigid_body_handle(source))
    }

    #[test]
    fn stress_intensity_rejects_invalid_arguments_and_null_output() {
        let mut report = StressIntensityReport::default();
        assert_eq!(
            fracture_stress_intensity_factor(f64::NAN, 0.01, 1.0, 10.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_stress_intensity_factor(100.0, 0.0, 1.0, 10.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_stress_intensity_factor(100.0, 0.01, 1.0, -1.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_stress_intensity_factor(100.0, 0.01, 1.0, 10.0, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn griffith_rejects_invalid_material_and_null_output() {
        let mut report = GriffithReport::default();
        let mut bad_material = valid_material();
        bad_material.youngs_modulus = 0.0;
        assert_eq!(
            fracture_griffith_criterion(1.0e6, 0.01, bad_material, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let mut bad_material = valid_material();
        bad_material.poisson_ratio = 0.6;
        assert_eq!(
            fracture_griffith_criterion(1.0e6, 0.01, bad_material, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_griffith_criterion(1.0e6, -0.01, valid_material(), &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_griffith_criterion(1.0e6, 0.01, valid_material(), std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn miner_damage_rejects_bad_count_null_arrays_and_bad_data() {
        let cycles = [100.0, 50.0];
        let lives = [1000.0, 500.0];
        let mut report = MinerDamageReport::default();

        assert_eq!(
            fracture_miner_damage(cycles.as_ptr(), lives.as_ptr(), 0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        assert_eq!(
            fracture_miner_damage(cycles.as_ptr(), lives.as_ptr(), 1_000_001, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        assert_eq!(
            fracture_miner_damage(std::ptr::null(), lives.as_ptr(), 2, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert_eq!(
            fracture_miner_damage(cycles.as_ptr(), std::ptr::null(), 2, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let bad_cycles = [f64::NAN];
        assert_eq!(
            fracture_miner_damage(bad_cycles.as_ptr(), lives.as_ptr(), 1, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let zero_lives = [0.0];
        assert_eq!(
            fracture_miner_damage(cycles.as_ptr(), zero_lives.as_ptr(), 1, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_miner_damage(cycles.as_ptr(), lives.as_ptr(), 2, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn sn_curve_rejects_invalid_arguments_and_null_output() {
        let mut report = SnCurveReport::default();
        assert_eq!(
            fracture_sn_curve_life(0.0, 1.0e12, 3.0, 100.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_sn_curve_life(50.0, f64::NAN, 3.0, 100.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_sn_curve_life(50.0, 1.0e12, 3.0, -1.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_sn_curve_life(50.0, 1.0e12, 3.0, 100.0, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn energy_release_rejects_invalid_arguments_and_null_output() {
        let mut report = FractureEnergyReport::default();
        assert_eq!(
            fracture_energy_release(-1.0, 10.0, 8.0, 0.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_energy_release(120.0, 0.0, 8.0, 0.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_energy_release(120.0, 10.0, 8.0, f64::NAN, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_energy_release(120.0, 10.0, 8.0, 0.0, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn mode_from_stress_rejects_invalid_arguments_and_null_output() {
        let mut report = FractureModeReport::default();
        assert_eq!(
            fracture_mode_from_stress(-1.0, 3.0, 2.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_mode_from_stress(1.0, f64::NAN, 2.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fracture_mode_from_stress(1.0, 3.0, 2.0, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn replace_fragments_null_world_reports_null_pointer() {
        let fragments = valid_fragments();
        let mut bodies = [0; 2];
        assert_eq!(
            world_replace_body_with_fracture_fragments(
                std::ptr::null_mut(),
                0,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn replace_fragments_capacity_violations_report_capacity() {
        let (world, source) = world_with_source();
        let fragments = valid_fragments();
        let mut bodies = [0; 2];

        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                0,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                1,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        world_destroy(world);
    }

    #[test]
    fn replace_fragments_null_pointers_report_null_pointer() {
        let (world, source) = world_with_source();
        let fragments = valid_fragments();
        let mut bodies = [0; 2];
        let mut joints = [0; 2];

        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                std::ptr::null(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::TRUE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Supplying the joint buffer when connecting succeeds.
        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::TRUE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                joints.as_mut_ptr(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::TRUE
        );
        assert_eq!(last_error_code(), ERR_OK);
        world_destroy(world);
    }

    #[test]
    fn replace_fragments_unknown_source_body_reports_not_found() {
        let (world, _) = world_with_source();
        let fragments = valid_fragments();
        let mut bodies = [0; 2];
        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                u64::MAX,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn replace_fragments_invalid_fragment_reports_invalid_argument() {
        let (world, source) = world_with_source();
        let mut fragments = valid_fragments();
        fragments[0].half_extents = v3(0.0, 0.5, 0.5);
        let mut bodies = [0; 2];
        assert_eq!(
            world_replace_body_with_fracture_fragments(
                world,
                source,
                fragments.as_ptr(),
                fragments.len() as u32,
                Bool::FALSE,
                Bool::FALSE,
                bodies.as_mut_ptr(),
                std::ptr::null_mut(),
                bodies.len() as u32,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn successful_formula_calls_report_ok() {
        let mut intensity = StressIntensityReport::default();
        assert_eq!(
            fracture_stress_intensity_factor(100.0, 0.01, 1.0, 10.0, &mut intensity),
            Bool::TRUE
        );
        assert_eq!(last_error_code(), ERR_OK);
    }
}
