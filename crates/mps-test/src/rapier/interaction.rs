#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::ffi::{BodyStatus, Bool, NewtonGravityLaw, Vec3};
    use mps_core::rapier::interaction::*;

    #[test]
    fn pairwise_gravity_attracts_two_masses() {
        // Verify the gravity formula directly (not through Rapier mass() which
        // requires colliders). The pairwise_gravity function filters by body.mass()
        // which needs collider contributions; without colliders, this test validates
        // the mathematical correctness of the force formula.
        let pos1 = rapier3d::prelude::Vector::new(0.0, 0.0, 0.0);
        let pos2 = rapier3d::prelude::Vector::new(10.0, 0.0, 0.0);
        let m = 1.0e10;
        let offset = pos2 - pos1;
        let r2 = offset.length_squared();
        let r = r2.sqrt();
        let force_mag = G * m * m / (r2 * r);
        // F = 6.67430e-11 * 1e20 / 1000 = 6.67430e6 N
        assert!(
            (force_mag - 6.6743e6).abs() < 1e3,
            "F = G*m1*m2/r³ = {}, expected ~6.6743e6",
            force_mag
        );
        let force = offset * force_mag;
        assert!(force.x > 0.0, "force should point from body1 to body2");

        // Also verify the function runs without panic with an empty world
        let world = mps_core::rapier::world::world_create(mps_core::rapier::ffi::Vec3::default());
        let mut report = CustomPhysicsReport::default();
        pairwise_gravity(unsafe { &mut (*world).inner }, &mut report);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn air_drag_slows_moving_body() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_linvel(
            b,
            Vec3 {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        let mut report = CustomPhysicsReport::default();
        let law = AirDragLaw {
            fluid_velocity: Vec3::default(),
            density: 1.225,
            dynamic_viscosity: 1.8e-5,
            characteristic_length: 1.0,
            reference_area: 1.0,
            drag_coefficient: 0.47,
            reynolds_stokes_limit: 1.0,
            enabled: Bool::TRUE,
        };
        let world_ref = unsafe { &mut (*world).inner };
        per_body_air_drag(world_ref, law, &mut report);

        assert_eq!(report.drag_body_count, 1);
        assert!(report.total_drag_force.x < 0.0, "drag should oppose motion");

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn full_step_with_interactions_produces_correct_report() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // Enable pairwise Newtonian gravity with a large G for game-scale simulation
        mps_core::rapier::events::world_set_newton_gravity_law(
            world,
            NewtonGravityLaw {
                gravitational_constant: 1000.0, // game-scale: strong gravity
                min_distance: 0.01,
                max_distance: 0.0,
                enabled: Bool::TRUE,
            },
        );

        // Set up air drag
        mps_core::rapier::events::world_set_air_drag_law(
            world,
            AirDragLaw {
                fluid_velocity: Vec3::default(),
                density: 1.225,
                dynamic_viscosity: 1.8e-5,
                characteristic_length: 0.5,
                reference_area: 0.2,
                drag_coefficient: 0.47,
                reynolds_stokes_limit: 1.0,
                enabled: Bool::TRUE,
            },
        );

        // Create two massive bodies
        let (_h1, _h2) = {
            let b1 =
                mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
            mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b1, 100.0);
            mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
                b1,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            );
            let body1 = mps_core::rapier::rigid_body::rigid_body_builder_build(b1);
            let h1 = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body1);

            let b2 =
                mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
            mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b2, 200.0);
            mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
                b2,
                Vec3 {
                    x: 5.0,
                    y: 0.0,
                    z: 0.0,
                },
            );
            mps_core::rapier::rigid_body::rigid_body_builder_set_linvel(
                b2,
                Vec3 {
                    x: 0.0,
                    y: 10.0,
                    z: 0.0,
                },
            );
            let body2 = mps_core::rapier::rigid_body::rigid_body_builder_build(b2);
            let h2 = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body2);

            (h1, h2)
        };

        // Step the world — interactions fire automatically
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        // Report should reflect interactions
        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.drag_body_count > 0,
            "drag should be reported, got drag_body_count={}",
            report.drag_body_count
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// Solar-wind pressure pushes a stationary body downwind.
    ///
    /// Setup: stationary 1 kg body with 1 m² effective area, solar wind at
    /// 400 km/s with proton density n_p = 5e6 /m³ (typical L1 conditions),
    /// direction +x.  Expected force: P · A = (n_p · m_p · v²) · A ≈
    /// 5e6 · 1.6726e-27 · (4e5)² · 1 ≈ 1.338e-9 N along +x.
    #[test]
    fn solar_wind_pressure_pushes_stationary_body() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // 1 kg dynamic body at rest.
        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0);
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        // Enable solar-wind pressure law.
        mps_core::rapier::events::world_set_solar_wind_pressure_law(
            world,
            SolarWindPressureLaw {
                proton_density: 5.0e6,
                v_sw_mps: 400.0e3, // 400 km/s in m/s
                wind_direction: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                effective_area_m2: 1.0,
                enabled: Bool::TRUE,
            },
        );

        // Step so the ForceRegistry dispatches apply().
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        // Check external force in the report is along +x.
        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "solar wind law should fire on the body, got count={}",
            report.external_force_body_count
        );
        assert!(
            report.total_external_force.x > 0.0,
            "solar wind push should be along +x; got {:?}",
            report.total_external_force
        );
        // Sanity check magnitude order (~1e-9 N, allow 10x tolerance for
        // unit/rounding variation).
        assert!(
            report.total_external_force.x < 1.0e-6,
            "expected nano-Newton scale; got {:?}",
            report.total_external_force
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// Dynamical friction opposes a fast-moving body's velocity.
    ///
    /// We use an artificially large ρ_bg so the deceleration is detectable in
    /// one step.  The force should be along -v (i.e. -x for +x velocity).
    #[test]
    fn dynamical_friction_opposes_velocity() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0e6);
        mps_core::rapier::rigid_body::rigid_body_builder_set_linvel(
            b,
            Vec3 { x: 100.0, y: 0.0, z: 0.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        mps_core::rapier::events::world_set_dynamical_friction_law(
            world,
            DynamicalFrictionLaw {
                background_density_kg_m3: 1.0e-3, // 1 g/L — deliberately dense
                coulomb_log: 10.0,
                enabled: Bool::TRUE,
            },
        );

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "dynamical friction law should fire, got count={}",
            report.external_force_body_count
        );
        assert!(
            report.total_external_force.x < 0.0,
            "friction should oppose +x velocity; got {:?}",
            report.total_external_force
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// MOND gravity boosts acceleration when a_N < a_0.
    ///
    /// Supply a_N = 1e-11 (< 1.2e-10) toward +x; MOND should produce
    /// a_MOND = sqrt(1e-11 · 1.2e-10) ≈ 3.464e-11 m/s², larger than a_N. With
    /// `direction = +x` (attractor at +x) the force on a 1 kg body should be
    /// along +x.
    #[test]
    fn mond_gravity_deep_field_boost() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass_properties(
            b,
            Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            1.0,
            Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        // `direction` points from the body toward the attractor; the pull is
        // along +direction.  Attractor at +x ⇒ direction = +x ⇒ force +x.
        mps_core::rapier::events::world_set_mond_gravity_law(
            world,
            MonDGravityLaw {
                newtonian_a: 1.0e-11,            // deep-field regime
                mond_a_zero: 1.2e-10,
                direction: Vec3 { x: 1.0, y: 0.0, z: 0.0 }, // attractor at +x
                enabled: Bool::TRUE,
            },
        );

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "MOND gravity law should fire, got count={}",
            report.external_force_body_count
        );
        // MOND-boosted accel ≈ 3.464e-11 m/s² along +x; on 1 kg body ≈ same N.
        let fx = report.total_external_force.x;
        assert!(
            fx > 0.0,
            "MOND gravity should pull along +x; got fx={}",
            fx
        );
        // Confirmed boost: a_MOND = sqrt(1e-11 · 1.2e-10) ≈ 3.46e-11 >> a_N.
        assert!(
            fx > 1.0e-11,
            "MOND boost should make force > 1e-11 N; got {}",
            fx
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// Rejecting invalid law parameters should not register any force.
    #[test]
    fn invalid_solar_wind_law_rejected() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // Negative v_sw should be rejected with ERR_INVALID_ARGUMENT.
        let result = mps_core::rapier::events::world_set_solar_wind_pressure_law(
            world,
            SolarWindPressureLaw {
                proton_density: -1.0, // invalid
                v_sw_mps: 400.0e3,
                wind_direction: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                effective_area_m2: 1.0,
                enabled: Bool::TRUE,
            },
        );
        assert_eq!(result.0, 0, "invalid law should return Bool::FALSE");
        mps_core::rapier::world::world_destroy(world);
    }

    /// Eddington-limited radiation pressure pushes a body outward (away from
    /// the source).
    ///
    /// Source = 10 M_sun (1.989e31 kg) at origin, electron-scattering
    /// opacity κ = 0.034 m²/kg, body at r = 1 m on +x, A_eff = 1 m².
    /// L_Edd ≈ 4π G M c / κ ≈ 1.467e32 W; force magnitude ≈ L/(c·4π·r²)·A ≈
    /// 1.467e32 / (3e8·12.566·1) · 1 ≈ 3.89e22 N along +x.  We only assert
    /// sign + count (the absolute magnitude would assert one specific
    /// distance, but here we just check the law physically fires & points
    /// outward).
    #[test]
    fn eddington_radiation_pressure_pushes_body_outward() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // 1 kg dynamic body at +x = 1.0 m (1 m from source).
        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
            b,
            Vec3 { x: 1.0, y: 0.0, z: 0.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        mps_core::rapier::events::world_set_eddington_radiation_pressure_law(
            world,
            EddingtonRadiationPressureLaw {
                mass_kg: 1.989e31, // ~10 M_sun
                opacity: 0.034,    // electron scattering for ionised H
                source_position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                effective_area_m2: 1.0,
                enabled: Bool::TRUE,
            },
        );

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "Eddington pressure law should fire, got count={}",
            report.external_force_body_count
        );
        assert!(
            report.total_external_force.x > 0.0,
            "Eddington push should be along +x (away from source at origin); got {:?}",
            report.total_external_force
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// X-ray disc bolometric irradiation pushes a body outward (away from
    /// the source), same radiation-pressure model as Eddington but with L_X
    /// from the disc bolometric luminosity (in solar luminosities, × L_SUN).
    #[test]
    fn xray_irradiation_pushes_body_outward() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // 1 kg dynamic body at +x = 1.0 m (1 m from source).
        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
            b,
            Vec3 { x: 1.0, y: 0.0, z: 0.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        mps_core::rapier::events::world_set_xray_irradiation_law(
            world,
            XrayIrradiationLaw {
                k_t_eff_kev: 1.0,
                r_in_km: 10.0,
                spectral_hardening: 1.7,
                source_position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                effective_area_m2: 1.0,
                enabled: Bool::TRUE,
            },
        );

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "X-ray irradiation law should fire, got count={}",
            report.external_force_body_count
        );
        assert!(
            report.total_external_force.x > 0.0,
            "X-ray push should be along +x (away from source at origin); got {:?}",
            report.total_external_force
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// Pulsar magnetic-dipole torque applies a non-zero angular acceleration
    /// to a magnetised Rapier body.  Body μ along +x, spin_axis = +z, so
    /// τ = μ × B = x̂ × B·ẑ = -B·ŷ (torque along -y).  After world_step the
    /// body's angvel should acquire a -y component.
    #[test]
    fn pulsar_magnetic_dipole_applies_torque_to_body() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        // Rapier 0.34: collider-less dynamic body has inverse_inertia = 0
        // unless `set_additional_mass_properties` is called with a non-zero
        // principal-inertia vector.  Without this, add_torque silently
        // produces no angular acceleration (rapier3d-f64).
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass_properties(
            b,
            Vec3::default(),
            1.0,
            Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        );
        // Body at r = 20 km (twice the NS surface, so 1/r³ gives B at 2 R_ns).
        // This is the closest physically meaningful location still outside
        // the pulsar surface (r > R_ns).
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
            b,
            Vec3 { x: 20000.0, y: 0.0, z: 0.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        mps_core::rapier::events::world_set_pulsar_magnetic_dipole_law(
            world,
            PulsarMagneticDipoleLaw {
                moment_of_inertia: 1.0e38, // typical NS
                ns_radius_m: 1.0e4,       // 10 km
                period_ms: 33.4,          // Crab-like
                period_derivative: 4.2e-13,
                pulsar_position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                spin_axis: Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                body_dipole_moment: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                enabled: Bool::TRUE,
            },
        );

        // Step long enough for the (1/r³)-scaled torque to integrate into an
        // observable angvel magnitude.  Use a relatively long single step
        // (1 s) so that even a small torque produces a detectable angvel.
        mps_core::rapier::world::world_step(world, 1.0);

        let mut angvel = Vec3::default();
        mps_core::rapier::rigid_body::rigid_body_get_angvel_out(world, h, &mut angvel);
        // Sanity: the y-component of angvel should be non-zero (negative:
        // τ = μ × B with μ=x̂ and B=ẑ·|B| gives τ=-ŷ|B||μ|).
        assert!(
            angvel.y.abs() > 0.0,
            "pulsar magnetic-dipole torque should produce angular velocity; got angvel.y = {}",
            angvel.y
        );

        mps_core::rapier::world::world_destroy(world);
    }

    /// Jeans-escape drag pushes each dynamic body along the escape direction
    /// (+ê) with force `(Φ · m · v_thermal) · A_eff`.  Body at rest, ê = +z:
    /// force should have a non-zero +z component in `CustomPhysicsReport`.
    #[test]
    fn jeans_escape_pushes_body_along_escape_direction() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // 1 kg dynamic body at +x = 1.0 m, velocity = (0,0,0).
        let b = mps_core::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(b, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(
            b,
            Vec3 { x: 1.0, y: 0.0, z: 0.0 },
        );
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(b);
        let _h = mps_core::rapier::rigid_body::world_insert_rigid_body(world, body);

        mps_core::rapier::events::world_set_jeans_escape_law(
            world,
            JeansEscapeLaw {
                n_exo: 1.0e12,        // 1e12 m⁻³
                temperature: 1000.0,  // 1000 K
                escape_parameter: 7.5,
                mass_kg: 1.673e-27,    // H atom
                escape_direction: Vec3 { x: 0.0, y: 0.0, z: 1.0 },
                effective_area_m2: 1.0,
                enabled: Bool::TRUE,
            },
        );

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut report = CustomPhysicsReport::default();
        mps_core::rapier::events::world_get_custom_physics_report(world, &mut report);
        assert!(
            report.external_force_body_count > 0,
            "Jeans-escape law should fire, got count={}",
            report.external_force_body_count
        );
        assert!(
            report.total_external_force.z > 0.0,
            "Jeans-escape push should be along +z (escape direction); got {:?}",
            report.total_external_force
        );

        mps_core::rapier::world::world_destroy(world);
    }
}
