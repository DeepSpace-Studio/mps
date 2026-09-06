#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::{BodyStatus, Bool, Vec3, WorldHandle};
    use mps_core::rapier::volcano::*;
    use mps_formula::volcano::{
        AMBIENT_TEMPERATURE, BOMB_RADIUS, BOMB_SPEED, LAVA_TEMPERATURE, PlumeParams, ROCK_DENSITY,
        body_heating_rate, lava_bomb_impulse, lava_bomb_mass, melt_mass_loss_rate,
        plume_column_radius, plume_flow_at, plume_temperature_at,
    };

    pub(crate) fn point(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn plume() -> PlumeParams {
        PlumeParams {
            vent: point(0.0, 0.0, 0.0),
            plume_radius: 200.0,
            plume_height: 2000.0,
            max_updraft: 60.0,
            lava_temperature: LAVA_TEMPERATURE,
        }
    }

    // ---- formula: plume flow ----

    #[test]
    fn plume_flow_peaks_at_vent_axis() {
        let p = plume();
        // On the axis at the vent: full updraft, no radial component.
        let base = plume_flow_at(&p, point(0.0, 1.0, 0.0));
        assert!(
            (base.y - 60.0 * (1.0 - 1.0 / (2000.0 * 1.2))).abs() < 0.5,
            "got {base:?}"
        );
        assert!(base.x.abs() < 1.0e-9 && base.z.abs() < 1.0e-9);

        // The column widens with height.
        assert!((plume_column_radius(200.0, 0.0) - 200.0).abs() < 1.0e-9);
        assert!((plume_column_radius(200.0, 1.0) - 600.0).abs() < 1.0e-9);
    }

    #[test]
    fn plume_flow_has_radial_cap_near_the_top() {
        let p = plume();
        // One column radius off the axis at q = 1 (mushroom-cap level): the
        // flow points outward and slightly up.
        let cap = plume_flow_at(&p, point(600.0, 2000.0, 0.0));
        assert!(cap.x > 0.0, "got {cap:?}");
        assert!(cap.y > 0.0, "got {cap:?}");

        // Far above the decay height the field vanishes.
        let above = plume_flow_at(&p, point(0.0, 2600.0, 0.0));
        assert!(above.y.abs() < 1.0e-9, "got {above:?}");
    }

    #[test]
    fn plume_temperature_decays_with_height_and_distance() {
        let p = plume();
        // At the vent core: lava temperature.
        let core = plume_temperature_at(&p, point(0.0, 10.0, 0.0), AMBIENT_TEMPERATURE);
        assert!((core - LAVA_TEMPERATURE).abs() < 10.0, "got {core}");

        // Half a plume height up on the axis: clearly cooler.
        let mid = plume_temperature_at(&p, point(0.0, 1000.0, 0.0), AMBIENT_TEMPERATURE);
        assert!(mid < core - 300.0, "got {mid}");

        // Far outside the column: ambient.
        let out = plume_temperature_at(&p, point(5000.0, 10.0, 0.0), AMBIENT_TEMPERATURE);
        assert!((out - AMBIENT_TEMPERATURE).abs() < 1.0e-6, "got {out}");
    }

    // ---- formula: heat & melt ----

    #[test]
    fn heating_rate_is_newton_exchange() {
        assert!((body_heating_rate(1473.0, 293.0, 5.0) - 5.0 * 1180.0).abs() < 1.0e-9);
        // Cooler surroundings cool the body.
        assert!(body_heating_rate(293.0, 1473.0, 5.0) < 0.0);
    }

    #[test]
    fn melt_rate_is_zero_below_melt_point_and_linear_above() {
        assert_eq!(melt_mass_loss_rate(799.0, 800.0, 2.0), 0.0);
        // 2× superheat → the full base rate.
        assert!((melt_mass_loss_rate(1600.0, 800.0, 2.0) - 2.0).abs() < 1.0e-9);
        assert!(melt_mass_loss_rate(1200.0, 800.0, 2.0) < 2.0);
    }

    #[test]
    fn bomb_impulse_matches_mass_times_speed() {
        let mass = lava_bomb_mass(BOMB_RADIUS, ROCK_DENSITY);
        assert!(mass > 30.0 && mass < 45.0, "got {mass}");
        assert!((lava_bomb_impulse(mass, BOMB_SPEED) - mass * BOMB_SPEED).abs() < 1.0e-9);
    }

    // ---- FFI: lifecycle ----

    #[test]
    fn registers_validates_and_removes_volcanoes() {
        let world = mps_core::rapier::world::world_create(Vec3::default());

        // Null world is rejected.
        let mut id = u32::MAX;
        assert_eq!(
            volcano_add(
                std::ptr::null_mut(),
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                60.0,
                0.0,
                0.0,
                &mut id
            ),
            1 // ERR_NULL_POINTER
        );

        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                60.0,
                0.0, // lava_temperature 0 → basaltic default
                0.0,
                &mut id
            ),
            0
        );
        assert_ne!(id, u32::MAX);

        // Invalid geometry is rejected.
        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                0.0, // plume_radius must be positive
                2000.0,
                60.0,
                0.0,
                0.0,
                &mut id
            ),
            2 // ERR_INVALID_ARGUMENT
        );
        assert_eq!(volcano_advance(world, -1.0), 2);

        assert_eq!(volcano_remove(world, id), 0);
        assert_eq!(volcano_remove(world, id), 3); // ERR_NOT_FOUND
        assert_eq!(volcano_clear(world), 0);
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn samples_flow_and_temperature() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let mut id = u32::MAX;
        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                60.0,
                0.0,
                0.0,
                &mut id
            ),
            0
        );

        let mut flow = Vec3::default();
        assert_eq!(
            volcano_sample_flow(world, point(0.0, 10.0, 0.0), &mut flow),
            0
        );
        assert!(flow.y > 50.0, "got {flow:?}");

        let mut temp = 0.0;
        assert_eq!(
            volcano_sample_temperature(world, point(0.0, 10.0, 0.0), &mut temp),
            0
        );
        assert!((temp - LAVA_TEMPERATURE).abs() < 10.0, "got {temp}");

        // Far outside the column: ambient and still.
        assert_eq!(
            volcano_sample_temperature(world, point(5000.0, 10.0, 0.0), &mut temp),
            0
        );
        assert!((temp - AMBIENT_TEMPERATURE).abs() < 1.0e-6);

        // Null output is rejected.
        assert_eq!(
            volcano_sample_flow(world, point(0.0, 0.0, 0.0), std::ptr::null_mut()),
            1
        );
        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn body_temperature_state_lifecycle() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(0.0, 10.0, 0.0));

        // No thermal state yet.
        let mut temp = 0.0;
        assert_eq!(volcano_body_temperature(world, body, &mut temp), 3);

        // Scripted override creates the state.
        assert_eq!(volcano_set_body_temperature(world, body, 900.0), 0);
        assert_eq!(volcano_body_temperature(world, body, &mut temp), 0);
        assert!((temp - 900.0).abs() < 1.0e-9);

        // Non-finite temperatures are rejected.
        assert_eq!(volcano_set_body_temperature(world, body, f64::NAN), 2);

        mps_core::rapier::world::world_destroy(world);
    }

    // ---- FFI: heating & melting ----

    pub(crate) fn make_dynamic_body_at(world: *mut WorldHandle, pos: Vec3) -> u64 {
        let builder =
            mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Dynamic as u32);
        mps_core::rapier::rigid_body::rigid_body_builder_set_translation(builder, pos);
        mps_core::rapier::rigid_body::rigid_body_builder_set_additional_mass(builder, 1.0);
        mps_core::rapier::rigid_body::rigid_body_builder_set_can_sleep(builder, Bool::FALSE);
        let body = mps_core::rapier::rigid_body::rigid_body_builder_build(builder);
        mps_core::rapier::rigid_body::world_insert_rigid_body(world, body)
    }

    #[test]
    fn body_inside_plume_heats_and_melts() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(0.0, 10.0, 0.0));
        let outside = make_dynamic_body_at(world, point(1.0e5, 0.0, 0.0));

        let mut id = u32::MAX;
        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                0.0, // no updraft — isolate the thermal model
                0.0,
                0.0,
                &mut id
            ),
            0
        );

        // Aggressive exchange so the body reaches the melt point quickly.
        let mut melted = [0u64; 4];
        let mut melted_count = 0u32;
        let mut total_melted = 0u32;
        let mut bombs = 0u32;
        for _ in 0..400 {
            assert_eq!(volcano_advance(world, 1.0 / 60.0), 0);
            assert_eq!(
                volcano_apply(
                    world,
                    1.0 / 60.0,
                    800.0, // melt_point (K)
                    5.0,   // melt_rate
                    5.0,   // heat_exchange_rate (1/s)
                    Bool::TRUE,
                    &mut bombs,
                    melted.as_mut_ptr(),
                    melted.len() as u32,
                    &mut melted_count
                ),
                0
            );
            // The counters are per-call: accumulate across the run.
            if melted_count > 0 {
                total_melted += melted_count;
            }
            mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        }

        // The in-plume body fully melted; the cold one never did.
        assert_eq!(total_melted, 1);
        assert_eq!(melted[0], body);
        assert_eq!(bombs, 0); // ejecta disabled

        let mut temp = 0.0;
        assert_eq!(volcano_body_temperature(world, body, &mut temp), 0);
        assert!(temp > 800.0, "got {temp}");
        assert_eq!(volcano_body_temperature(world, outside, &mut temp), 0);
        assert!((temp - AMBIENT_TEMPERATURE).abs() < 1.0, "got {temp}");

        // The melted body is disabled in place.
        let disabled = unsafe {
            (*world)
                .inner
                .bodies
                .get(mps_core::rapier::ffi::unpack_rigid_body_handle(body))
                .is_none_or(|b| !b.is_enabled())
        };
        assert!(disabled);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn melting_shrinks_body_mass_progressively() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(0.0, 10.0, 0.0));

        let mut id = u32::MAX;
        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                0.0,
                0.0,
                0.0,
                &mut id
            ),
            0
        );

        // Pre-heat the body above the melt point through the state override.
        assert_eq!(volcano_set_body_temperature(world, body, 1473.0), 0);

        // Melt with a gentle rate: after ~1 s at ~4× superheat the body has
        // lost a measurable but not complete fraction of its mass.
        let mut melted = [0u64; 4];
        let mut melted_count = 0u32;
        let mut bombs = 0u32;
        for _ in 0..20 {
            assert_eq!(volcano_advance(world, 1.0 / 60.0), 0);
            assert_eq!(
                volcano_apply(
                    world,
                    1.0 / 60.0,
                    293.0 + 100.0, // melt_point 393.15 K — superheat ~1080 K
                    1.0,           // melt_rate
                    0.0,           // no further exchange — hold temperature
                    Bool::TRUE,
                    &mut bombs,
                    melted.as_mut_ptr(),
                    melted.len() as u32,
                    &mut melted_count
                ),
                0
            );
            mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        }

        let mass = mps_core::rapier::rigid_body::rigid_body_get_mass(world, body);
        assert!(mass > 0.05 && mass < 1.0, "got {mass}");
        assert_eq!(melted_count, 0); // not fully melted yet

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn ejecta_bombs_launch_bodies_upward() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let body = make_dynamic_body_at(world, point(50.0, 5.0, 0.0));

        let mut id = u32::MAX;
        // 6000 bombs/s over dt = 1/60 s → exactly 100 deterministic bombs.
        assert_eq!(
            volcano_add(
                world,
                point(0.0, 0.0, 0.0),
                200.0,
                2000.0,
                0.0,
                0.0,
                6000.0,
                &mut id
            ),
            0
        );

        assert_eq!(volcano_advance(world, 1.0 / 60.0), 0);
        let mut melted = [0u64; 4];
        let mut melted_count = 0u32;
        let mut bombs = 0u32;
        assert_eq!(
            volcano_apply(
                world,
                1.0 / 60.0,
                f64::MAX, // no melting — isolate the ejecta impulse
                0.0,
                0.0, // no heat exchange — isolate the ejecta impulse
                Bool::TRUE,
                &mut bombs,
                melted.as_mut_ptr(),
                melted.len() as u32,
                &mut melted_count
            ),
            0
        );
        assert_eq!(bombs, 100);
        assert_eq!(melted_count, 0);

        // The launch direction is up-dominant: after a step the body moves up.
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = mps_core::rapier::rigid_body::rigid_body_get_linvel(world, body);
        assert!(velocity.y > 0.0, "got {velocity:?}");

        mps_core::rapier::world::world_destroy(world);
    }
}
