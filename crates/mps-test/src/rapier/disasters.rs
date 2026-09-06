#[cfg(test)]
mod tests {
    use mps_core::rapier::disasters::*;
    use mps_core::rapier::ffi::{BodyStatus, Bool, Vec3};
    use mps_formula::disasters::{
        CycloneParams, ICE_DENSITY, TornadoParams, cyclone_tangential_speed,
        cyclone_tangential_speed as tangential, cyclone_wind_at, cyclostrophic_pressure_drop,
        hail_fall_speed, hail_impulse, hail_kinetic_energy, hail_mass, hail_terminal_velocity,
        tornado_wind_at, wind_drag_force,
    };

    fn point(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn cyclone_params(translation: Vec3) -> CycloneParams {
        CycloneParams {
            center: point(0.0, 0.0, 0.0),
            translation,
            max_wind: 40.0,
            radius_max_wind: 1000.0,
            spin: 1.0,
            inflow_fraction: 0.0,
            height_exponent: 0.0,
            reference_height: 10.0,
        }
    }

    // ---- formula: tropical cyclone ----

    #[test]
    fn cyclone_wind_follows_rankine_profile() {
        let p = cyclone_params(point(0.0, 0.0, 0.0));

        // At the radius of maximum wind the tangential speed equals max_wind.
        let at_rm = cyclone_wind_at(&p, point(1000.0, 10.0, 0.0));
        assert!((at_rm.z - 40.0).abs() < 1.0e-9);
        assert!((at_rm.x - 0.0).abs() < 1.0e-9);

        // Inside: linear growth (r = R_m / 2 → half wind).
        let inside = cyclone_wind_at(&p, point(500.0, 10.0, 0.0));
        assert!((inside.z - 20.0).abs() < 1.0e-9);

        // Outside: (R_m / r)^0.6 decay.
        let outside = cyclone_wind_at(&p, point(2000.0, 10.0, 0.0));
        assert!((outside.z - 40.0 * 0.5f64.powf(0.6)).abs() < 1.0e-9);

        // The profile helper agrees with the field sampling at R_m.
        assert!((tangential(40.0, 1000.0, 1000.0) - 40.0).abs() < 1.0e-9);
        assert!((cyclone_tangential_speed(40.0, 1000.0, 500.0) - 20.0).abs() < 1.0e-9);
    }

    #[test]
    fn cyclone_wind_adds_translation_and_flips_with_spin() {
        let p = cyclone_params(point(5.0, 0.0, 0.0));
        // At the eye only the storm translation remains.
        let eye = cyclone_wind_at(&p, point(0.0, 0.0, 0.0));
        assert!((eye.x - 5.0).abs() < 1.0e-9);

        // Mirrored spin reverses the tangential direction.
        let mut flipped = p;
        flipped.spin = -1.0;
        let east = cyclone_wind_at(&flipped, point(1000.0, 10.0, 0.0));
        assert!((east.z + 40.0).abs() < 1.0e-9);
    }

    #[test]
    fn cyclone_wind_scales_with_boundary_layer_profile() {
        let mut p = cyclone_params(point(0.0, 0.0, 0.0));
        p.height_exponent = 0.11;
        // At the reference height the factor is exactly 1.
        let at_ref = cyclone_wind_at(&p, point(1000.0, 10.0, 0.0));
        assert!((at_ref.z - 40.0).abs() < 1.0e-9);
        // Higher up: (z / z_ref)^α.
        let higher = cyclone_wind_at(&p, point(1000.0, 100.0, 0.0));
        assert!((higher.z - 40.0 * 10f64.powf(0.11)).abs() < 1.0e-9);
    }

    // ---- formula: tornado ----

    #[test]
    fn tornado_wind_follows_rankine_and_fades_above_top() {
        let p = TornadoParams {
            base: point(0.0, 0.0, 0.0),
            translation: point(0.0, 0.0, 0.0),
            max_wind: 80.0,
            core_radius: 60.0,
            top_height: 1500.0,
            updraft_fraction: 0.35,
            spin: 1.0,
        };

        // Core edge at ground: full tangential wind, no updraft yet.
        let edge = tornado_wind_at(&p, point(60.0, 0.0, 0.0));
        assert!((edge.z - 80.0).abs() < 1.0e-9);
        assert!(edge.y.abs() < 1.0e-9);

        // Inside the core: linear (r = R_c / 2 → half wind), mid-funnel updraft.
        let core = tornado_wind_at(&p, point(30.0, 750.0, 0.0));
        assert!((core.z - 40.0).abs() < 1.0e-9);
        assert!((core.y - 0.35 * 80.0).abs() < 1.0e-9);

        // Outside the core: R_c / r decay.
        let outside = tornado_wind_at(&p, point(120.0, 0.0, 0.0));
        assert!((outside.z - 40.0).abs() < 1.0e-9);

        // Above twice the funnel top: only the translation remains.
        let above = tornado_wind_at(&p, point(60.0, 3000.0, 0.0));
        assert!(above.z.abs() < 1.0e-9 && above.y.abs() < 1.0e-9);
    }

    #[test]
    fn tornado_pressure_drop_is_cyclostrophic() {
        assert!((cyclostrophic_pressure_drop(80.0, 1.225) - 0.5 * 1.225 * 6400.0).abs() < 1.0e-9);
    }

    // ---- formula: hail ----

    #[test]
    fn hail_terminal_velocity_matches_reference_values() {
        // 2 cm hailstone: ~20 m/s; 20 cm: ~65 m/s.
        let small = hail_terminal_velocity(0.01, ICE_DENSITY, 1.225, 0.47);
        assert!(small > 19.0 && small < 22.0, "got {small}");
        let large = hail_terminal_velocity(0.1, ICE_DENSITY, 1.225, 0.47);
        assert!(large > 60.0 && large < 70.0, "got {large}");
    }

    #[test]
    fn hail_fall_speed_approaches_terminal_velocity() {
        let vt = hail_terminal_velocity(0.01, ICE_DENSITY, 1.225, 0.47);
        // A short drop stays well below terminal velocity.
        let short = hail_fall_speed(0.01, 10.0, ICE_DENSITY, 1.225, 0.47);
        assert!(short > 10.0 && short < vt);
        // A very long drop converges to terminal velocity.
        let long = hail_fall_speed(0.01, 100.0, ICE_DENSITY, 1.225, 0.47);
        assert!(long > 19.5 && long <= vt);
    }

    #[test]
    fn hail_impact_quantities_are_consistent() {
        let mass = hail_mass(0.01, ICE_DENSITY);
        // Sphere: 4/3 π r³ ρ ≈ 3.84 g.
        assert!(mass > 0.0038 && mass < 0.0039, "got {mass}");
        let speed = 20.0;
        assert!((hail_kinetic_energy(mass, speed) - 0.5 * mass * speed * speed).abs() < 1.0e-12);
        assert!((hail_impulse(mass, speed) - mass * speed).abs() < 1.0e-12);
    }

    #[test]
    fn wind_drag_force_is_quadratic_in_speed() {
        let f1 = wind_drag_force(point(10.0, 0.0, 0.0), 1.0, 1.0, 1.0);
        let f2 = wind_drag_force(point(20.0, 0.0, 0.0), 1.0, 1.0, 1.0);
        // F = ½ ρ C_d A |v| v → 10 m/s: 50 N; 20 m/s: 200 N (4×).
        assert!((f1.x - 50.0).abs() < 1.0e-9);
        assert!((f2.x - 200.0).abs() < 1.0e-9);
        // Opposing flow flips the direction.
        let back = wind_drag_force(point(-10.0, 0.0, 0.0), 1.0, 1.0, 1.0);
        assert!((back.x + 50.0).abs() < 1.0e-9);
    }

    // ---- FFI: lifecycle ----

    #[test]
    fn registers_validates_and_removes_disasters() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // Null world is rejected.
        let mut id = u32::MAX;
        assert_eq!(
            disaster_add_typhoon(
                std::ptr::null_mut(),
                point(0.0, 0.0, 0.0),
                40.0,
                1000.0,
                point(0.0, 0.0, 0.0),
                1.0,
                0.0,
                0.0,
                0.0,
                &mut id
            ),
            1 // ERR_NULL_POINTER
        );

        assert_eq!(
            disaster_add_typhoon(
                world,
                point(0.0, 0.0, 0.0),
                40.0,
                1000.0,
                point(5.0, 0.0, 0.0),
                1.0,
                0.0,
                0.0,
                0.0,
                &mut id
            ),
            0
        );
        assert_ne!(id, u32::MAX);

        let mut tornado_id = u32::MAX;
        assert_eq!(
            disaster_add_tornado(
                world,
                point(0.0, 0.0, 0.0),
                80.0,
                60.0,
                1500.0,
                point(0.0, 0.0, 0.0),
                -1.0,
                0.35,
                &mut tornado_id
            ),
            0
        );
        assert_ne!(tornado_id, u32::MAX);
        assert_ne!(tornado_id, id);

        // Invalid arguments are rejected with ERR_INVALID_ARGUMENT.
        assert_eq!(
            disaster_add_typhoon(
                world,
                point(0.0, 0.0, 0.0),
                0.0, // max_wind must be positive
                1000.0,
                point(0.0, 0.0, 0.0),
                1.0,
                0.0,
                0.0,
                0.0,
                &mut id
            ),
            2
        );
        assert_eq!(
            disaster_add_hail(
                world,
                point(0.0, 0.0, 0.0),
                -1.0, // radius must be positive
                10.0,
                0.01,
                &mut id
            ),
            2
        );

        // Advance runs without error and moves nothing observably here.
        assert_eq!(disaster_advance(world, 1.0 / 60.0), 0);
        assert_eq!(disaster_advance(world, -1.0), 2);

        // Remove: second remove of the same id misses.
        assert_eq!(disaster_remove(world, id), 0);
        assert_eq!(disaster_remove(world, id), 3); // ERR_NOT_FOUND
        assert_eq!(disaster_remove(world, tornado_id), 0);
        assert_eq!(disaster_remove(world, 12345), 3);

        assert_eq!(disaster_clear(world), 0);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn samples_combined_wind_field() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let mut id = 0u32;
        assert_eq!(
            disaster_add_typhoon(
                world,
                point(0.0, 0.0, 0.0),
                40.0,
                1000.0,
                point(5.0, 0.0, 0.0),
                1.0,
                0.15,
                0.11,
                10.0,
                &mut id
            ),
            0
        );

        let mut wind = Vec3::default();
        assert_eq!(
            disaster_sample_wind(world, point(1000.0, 10.0, 0.0), &mut wind),
            0
        );
        assert!((wind.z - 40.0).abs() < 1.0e-9);
        assert!((wind.x - (5.0 - 0.15 * 40.0)).abs() < 1.0e-9);

        // Removing the storm zeroes the field again.
        assert_eq!(disaster_remove(world, id), 0);
        assert_eq!(
            disaster_sample_wind(world, point(1000.0, 10.0, 0.0), &mut wind),
            0
        );
        assert!(wind.z.abs() < 1.0e-12 && wind.x.abs() < 1.0e-12);

        // Null output pointer is rejected.
        assert_eq!(
            disaster_sample_wind(world, point(0.0, 0.0, 0.0), std::ptr::null_mut()),
            1
        );
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn samples_pressure_drop_from_vortices() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let mut id = 0u32;
        assert_eq!(
            disaster_add_tornado(
                world,
                point(0.0, 0.0, 0.0),
                80.0,
                60.0,
                1500.0,
                point(0.0, 0.0, 0.0),
                1.0,
                0.35,
                &mut id
            ),
            0
        );
        let mut drop = f64::MAX;
        assert_eq!(
            disaster_sample_pressure_drop(world, point(60.0, 0.0, 0.0), 1.225, &mut drop),
            0
        );
        assert!((drop - 0.5 * 1.225 * 6400.0).abs() < 1.0e-9);
        mps_core::rapier::world::world_destroy(world);
    }

    // ---- FFI: force application ----

    fn make_dynamic_body_at(world: *mut mps_core::rapier::ffi::WorldHandle, pos: Vec3) -> u64 {
        let builder =
            mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(builder, pos);
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(builder, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_can_sleep(builder, Bool::FALSE);
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(builder);
        mps_core::rapier::rigid_body::world_insert_rigid_body(world, body)
    }

    #[test]
    fn typhoon_pushes_bodies_with_the_wind() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(1000.0, 10.0, 0.0));

        let mut id = 0u32;
        assert_eq!(
            disaster_add_typhoon(
                world,
                point(0.0, 0.0, 0.0),
                40.0,
                1000.0,
                point(0.0, 0.0, 0.0),
                1.0,
                0.0,
                0.0,
                10.0,
                &mut id
            ),
            0
        );

        let mut body_count = 0u32;
        let mut impacts = 0u32;
        assert_eq!(
            disaster_apply_forces(
                world,
                1.0,
                1.0,
                1.0,
                1.0 / 60.0,
                Bool::TRUE,
                &mut body_count,
                &mut impacts
            ),
            0
        );
        assert_eq!(body_count, 1);
        assert_eq!(impacts, 0); // no hail cell registered

        // The tangential wind at the body points +z; after a step the body
        // must have gained velocity in that direction.
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = mps_core::rapier::rigid_body::rigid_body_get_linvel(world, body);
        assert!(velocity.z > 0.1, "got {velocity:?}");

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn tornado_drags_bodies_around_and_inward() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        // Body east of the funnel core edge.
        let body = make_dynamic_body_at(world, point(120.0, 0.0, 0.0));

        let mut id = 0u32;
        assert_eq!(
            disaster_add_tornado(
                world,
                point(0.0, 0.0, 0.0),
                80.0,
                60.0,
                1500.0,
                point(0.0, 0.0, 0.0),
                1.0,
                0.35,
                &mut id
            ),
            0
        );

        let mut body_count = 0u32;
        let mut impacts = 0u32;
        assert_eq!(
            disaster_apply_forces(
                world,
                1.0,
                1.0,
                1.0,
                1.0 / 60.0,
                Bool::TRUE,
                &mut body_count,
                &mut impacts
            ),
            0
        );
        assert_eq!(body_count, 1);

        mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = mps_core::rapier::rigid_body::rigid_body_get_linvel(world, body);
        // Tangential wind at +x with spin=+1 points +z; the updraft also
        // lifts the body.
        assert!(velocity.z > 0.0, "got {velocity:?}");

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn hail_rains_impacts_into_bodies_under_the_cell() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(0.0, 0.0, 0.0));

        let mut id = 0u32;
        // 6000 impacts/m²/s over a 1 m² reference area at 1/60 s → exactly
        // 100 deterministic impacts per call.
        assert_eq!(
            disaster_add_hail(world, point(0.0, 0.0, 0.0), 100.0, 6000.0, 0.01, &mut id),
            0
        );

        assert_eq!(disaster_advance(world, 1.0 / 60.0), 0);

        let mut body_count = 0u32;
        let mut impacts = 0u32;
        assert_eq!(
            disaster_apply_forces(
                world,
                1.225,
                1.0,
                1.0,
                1.0 / 60.0,
                Bool::TRUE,
                &mut body_count,
                &mut impacts
            ),
            0
        );
        assert_eq!(body_count, 0); // no vortex → no wind
        assert_eq!(impacts, 100);

        // Hail transfers downward momentum.
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = mps_core::rapier::rigid_body::rigid_body_get_linvel(world, body);
        assert!(velocity.y < -1.0, "got {velocity:?}");

        // Bodies outside the cell receive nothing (remove the first body so
        // only the outside one remains under the cell check).
        assert_eq!(
            mps_core::rapier::rigid_body::world_remove_rigid_body(world, body, Bool::TRUE),
            Bool::TRUE
        );
        let outside = make_dynamic_body_at(world, point(1.0e5, 0.0, 0.0));
        let mut impacts = 0u32;
        assert_eq!(
            disaster_apply_forces(
                world,
                1.225,
                1.0,
                1.0,
                1.0 / 60.0,
                Bool::TRUE,
                &mut body_count,
                &mut impacts
            ),
            0
        );
        assert_eq!(impacts, 0);
        let _ = outside;

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn typhoon_translation_advances_the_storm() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let mut id = 0u32;
        // Storm centred at the origin moving +x at 10 m/s.
        assert_eq!(
            disaster_add_typhoon(
                world,
                point(0.0, 0.0, 0.0),
                40.0,
                100.0,
                point(10.0, 0.0, 0.0),
                1.0,
                0.0,
                0.0,
                10.0,
                &mut id
            ),
            0
        );

        let mut wind = Vec3::default();
        // A point 100 m east sees max wind before the move (r == R_m).
        assert_eq!(
            disaster_sample_wind(world, point(100.0, 10.0, 0.0), &mut wind),
            0
        );
        assert!((wind.z - 40.0).abs() < 1.0e-9);

        // After advancing 1 s the centre sits at x = 10, so that point is now
        // at r = 90 < R_m and sees less than the maximum wind.
        assert_eq!(disaster_advance(world, 1.0), 0);
        assert_eq!(
            disaster_sample_wind(world, point(100.0, 10.0, 0.0), &mut wind),
            0
        );
        assert!(wind.z < 40.0, "got {}", wind.z);

        mps_core::rapier::world::world_destroy(world);
    }
}
