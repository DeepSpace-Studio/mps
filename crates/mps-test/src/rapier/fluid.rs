#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK, last_error_code,
    };
    use mps_core::rapier::ffi::BodyStatus;
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::fluid::*;

    fn water() -> FluidVolume {
        FluidVolume {
            center: Vec3::default(),
            half_extents: Vec3 {
                x: 10.0,
                y: 10.0,
                z: 10.0,
            },
            density: 1000.0,
            linear_drag: 2.0,
            quadratic_drag: 0.5,
            angular_drag: 0.2,
            flow_velocity: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            gravity: Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        }
    }

    #[test]
    fn estimates_buoyancy_and_drag() {
        let mut report = FluidForceReport::default();
        assert_eq!(
            fluid_estimate_aabb_forces(
                water(),
                Vec3::default(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Vec3::default(),
                Vec3::default(),
                &mut report,
            ),
            Bool::TRUE
        );
        assert_eq!(report.submerged_fraction, 1.0);
        assert!(report.buoyancy_force.y > 0.0);
        assert!(report.drag_force.x > 0.0);
    }

    #[test]
    fn applies_fluid_force_to_body() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let builder =
            mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(builder, 1.0);
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(builder);
        let handle = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);
        let mut report = FluidForceReport::default();

        assert_eq!(
            fluid_apply_aabb_forces(
                world,
                handle,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                &mut report,
            ),
            Bool::TRUE
        );
        assert!(report.total_force.y > 0.0);
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = mps_core::rapier::rigid_body::rigid_body_get_linvel(world, handle);
        assert!(velocity.y > 0.0);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn navier_stokes_sph_and_bernoulli_formulas_work() {
        let mut ns = NavierStokesReport::default();
        assert_eq!(
            fluid_navier_stokes_simplified_step(
                Vec3::default(),
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 2.0,
                    z: 0.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 3.0,
                },
                Vec3 {
                    x: 0.0,
                    y: -9.81,
                    z: 0.0,
                },
                2.0,
                0.5,
                0.1,
                &mut ns,
            ),
            Bool::TRUE
        );
        assert!(ns.total_acceleration.x < 0.0);
        assert!(ns.total_acceleration.y < 0.0);
        assert!(ns.total_acceleration.z > 0.0);

        let particles = [
            SphParticle {
                position: Vec3::default(),
                velocity: Vec3::default(),
                mass: 1.0,
                density: 1000.0,
                pressure: 10.0,
            },
            SphParticle {
                position: Vec3 {
                    x: 0.25,
                    y: 0.0,
                    z: 0.0,
                },
                velocity: Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                mass: 1.0,
                density: 1000.0,
                pressure: 20.0,
            },
        ];
        let mut density = 0.0;
        assert_eq!(
            fluid_sph_estimate_density(
                Vec3::default(),
                particles.as_ptr(),
                particles.len() as u32,
                1.0,
                &mut density,
            ),
            Bool::TRUE
        );
        assert!(density > 0.0);

        let mut sph = SphForceReport::default();
        assert_eq!(
            fluid_sph_estimate_forces(
                particles[0],
                particles.as_ptr(),
                particles.len() as u32,
                1.0,
                3.0,
                1000.0,
                0.1,
                0.05,
                &mut sph,
            ),
            Bool::TRUE
        );
        assert!(sph.total_force.x.is_finite());

        let pressure = fluid_bernoulli_pressure(200_000.0, 1000.0, 10.0, 9.81, 2.0);
        assert!(pressure < 200_000.0);
        let mut bernoulli = BernoulliReport::default();
        assert_eq!(
            fluid_bernoulli_report(pressure, 1000.0, 10.0, 9.81, 2.0, &mut bernoulli),
            Bool::TRUE
        );
        assert!(bernoulli.dynamic_pressure > 0.0);
        assert!(bernoulli.total_head > 0.0);
    }

    fn invalid_fluid() -> FluidVolume {
        let mut fluid = water();
        fluid.density = f64::NAN;
        fluid
    }

    fn world_with_body() -> (*mut WorldHandle, RigidBodyHandleRaw) {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let builder =
            mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(builder, 1.0);
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(builder);
        let handle = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);
        (world, handle)
    }

    #[test]
    fn estimate_aabb_forces_rejects_invalid_parameters() {
        let mut report = FluidForceReport::default();
        assert_eq!(
            fluid_estimate_aabb_forces(
                invalid_fluid(),
                Vec3::default(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Vec3::default(),
                Vec3::default(),
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_estimate_aabb_forces(
                water(),
                Vec3::default(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                0.0,
                Vec3::default(),
                Vec3::default(),
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn estimate_aabb_forces_allows_null_output_and_reports_ok() {
        assert_eq!(
            fluid_estimate_aabb_forces(
                water(),
                Vec3::default(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Vec3::default(),
                Vec3::default(),
                std::ptr::null_mut(),
            ),
            Bool::TRUE
        );
        assert_eq!(last_error_code(), ERR_OK);
    }

    #[test]
    fn apply_aabb_forces_null_world_reports_null_pointer() {
        assert_eq!(
            fluid_apply_aabb_forces(
                std::ptr::null_mut(),
                0,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn apply_aabb_forces_unknown_body_reports_not_found() {
        let (world, _) = world_with_body();
        assert_eq!(
            fluid_apply_aabb_forces(
                world,
                u64::MAX,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn apply_aabb_forces_invalid_fluid_reports_invalid_argument() {
        let (world, handle) = world_with_body();
        assert_eq!(
            fluid_apply_aabb_forces(
                world,
                handle,
                invalid_fluid(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn apply_aabb_forces_flag_mirrors_bool_result() {
        assert_eq!(
            fluid_apply_aabb_forces_flag(
                std::ptr::null_mut(),
                0,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                std::ptr::null_mut(),
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let (world, handle) = world_with_body();
        let mut report = FluidForceReport::default();
        assert_eq!(
            fluid_apply_aabb_forces_flag(
                world,
                handle,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                &mut report,
            ),
            1
        );
        assert_eq!(last_error_code(), ERR_OK);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn navier_stokes_rejects_invalid_arguments_and_null_output() {
        let mut report = NavierStokesReport::default();
        assert_eq!(
            fluid_navier_stokes_simplified_step(
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                0.0,
                0.5,
                0.1,
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_navier_stokes_simplified_step(
                Vec3 {
                    x: f64::NAN,
                    y: 0.0,
                    z: 0.0,
                },
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                2.0,
                0.5,
                0.1,
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_navier_stokes_simplified_step(
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                Vec3::default(),
                2.0,
                0.5,
                0.1,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn sph_scalar_kernels_return_nan_for_invalid_input() {
        assert!(fluid_sph_poly6_kernel(-1.0, 1.0).is_nan());
        assert!(fluid_sph_poly6_kernel(0.5, 0.0).is_nan());
        assert!(fluid_sph_viscosity_laplacian(-1.0, 1.0).is_nan());
        assert!(fluid_sph_viscosity_laplacian(0.5, f64::NAN).is_nan());
    }

    #[test]
    fn sph_spiky_gradient_rejects_bad_input_and_null_output() {
        let mut gradient = Vec3::default();
        assert_eq!(
            fluid_sph_spiky_gradient(
                Vec3 {
                    x: f64::NAN,
                    y: 0.0,
                    z: 0.0,
                },
                1.0,
                &mut gradient,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_spiky_gradient(
                Vec3 {
                    x: 0.5,
                    y: 0.0,
                    z: 0.0,
                },
                0.0,
                &mut gradient,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_spiky_gradient(
                Vec3 {
                    x: 0.5,
                    y: 0.0,
                    z: 0.0,
                },
                1.0,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn sph_estimate_density_rejects_bad_input_and_null_pointers() {
        let particle = SphParticle {
            position: Vec3::default(),
            velocity: Vec3::default(),
            mass: 1.0,
            density: 1000.0,
            pressure: 10.0,
        };
        let particles = [particle];
        let mut density = 0.0;

        assert_eq!(
            fluid_sph_estimate_density(Vec3::default(), std::ptr::null(), 1, 1.0, &mut density,),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert_eq!(
            fluid_sph_estimate_density(
                Vec3 {
                    x: f64::NAN,
                    y: 0.0,
                    z: 0.0,
                },
                particles.as_ptr(),
                1,
                1.0,
                &mut density,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_estimate_density(Vec3::default(), particles.as_ptr(), 1, 0.0, &mut density),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_estimate_density(
                Vec3::default(),
                particles.as_ptr(),
                1,
                1.0,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn sph_estimate_density_allows_zero_particles() {
        let particles = [SphParticle {
            position: Vec3::default(),
            velocity: Vec3::default(),
            mass: 1.0,
            density: 1000.0,
            pressure: 10.0,
        }];
        let mut density = 1.0;
        assert_eq!(
            fluid_sph_estimate_density(Vec3::default(), particles.as_ptr(), 0, 1.0, &mut density,),
            Bool::TRUE
        );
        assert_eq!(density, 0.0);
        assert_eq!(last_error_code(), ERR_OK);
    }

    #[test]
    fn sph_estimate_forces_rejects_bad_input_and_null_pointers() {
        let particle = SphParticle {
            position: Vec3::default(),
            velocity: Vec3::default(),
            mass: 1.0,
            density: 1000.0,
            pressure: 10.0,
        };
        let particles = [particle];
        let mut report = SphForceReport::default();

        assert_eq!(
            fluid_sph_estimate_forces(
                particle,
                std::ptr::null(),
                1,
                1.0,
                3.0,
                1000.0,
                0.1,
                0.05,
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let massless = SphParticle {
            mass: 0.0,
            ..particle
        };
        assert_eq!(
            fluid_sph_estimate_forces(
                massless,
                particles.as_ptr(),
                1,
                1.0,
                3.0,
                1000.0,
                0.1,
                0.05,
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_estimate_forces(
                particle,
                particles.as_ptr(),
                1,
                0.0,
                3.0,
                1000.0,
                0.1,
                0.05,
                &mut report,
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_sph_estimate_forces(
                particle,
                particles.as_ptr(),
                1,
                1.0,
                3.0,
                1000.0,
                0.1,
                0.05,
                std::ptr::null_mut(),
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn bernoulli_pressure_returns_nan_for_invalid_input() {
        assert!(fluid_bernoulli_pressure(200_000.0, 0.0, 10.0, 9.81, 2.0).is_nan());
        assert!(fluid_bernoulli_pressure(200_000.0, 1000.0, -1.0, 9.81, 2.0).is_nan());
        assert!(fluid_bernoulli_pressure(f64::NAN, 1000.0, 10.0, 9.81, 2.0).is_nan());
    }

    #[test]
    fn bernoulli_report_rejects_invalid_arguments_and_null_output() {
        let mut report = BernoulliReport::default();
        assert_eq!(
            fluid_bernoulli_report(100_000.0, 0.0, 10.0, 9.81, 2.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_bernoulli_report(100_000.0, 1000.0, -1.0, 9.81, 2.0, &mut report),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            fluid_bernoulli_report(100_000.0, 1000.0, 10.0, 9.81, 2.0, std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }
}
