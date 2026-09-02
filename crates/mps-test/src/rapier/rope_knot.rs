//! End-to-end tests for rope knot/weaving systems (per-strand soft bodies
//! with inter-strand collision proxies).

#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
        ERR_UNSUPPORTED, last_error_code,
    };
    use mps_core::rapier::ffi::{Bool, Vec3, WorldHandle};
    use mps_core::rapier::rope_knot::{
        rope_knot_build, rope_knot_create, rope_knot_remove, rope_knot_set_wind,
        rope_knot_strand_soft_body,
    };
    use mps_core::rapier::soft_body::{soft_body_get_particle, soft_body_particle_count};
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn make_world() -> *mut WorldHandle {
        world_create(v3(0.0, -9.81, 0.0))
    }

    fn make_knot(world: *mut WorldHandle, pattern: u32, strands: u32) -> u32 {
        rope_knot_create(
            world,
            pattern,
            strands,
            std::ptr::null(),
            0,
            0.05,   // radius
            100.0,  // stiffness
            0.4,    // rope-on-rope friction
            1000.0, // density
        )
    }

    fn build_between(world: *mut WorldHandle, id: u32) -> Bool {
        rope_knot_build(world, id, v3(0.0, 5.0, 0.0), v3(0.0, 5.0, 2.0))
    }

    #[test]
    fn rope_knot_all_patterns_build_and_remove() {
        let world = make_world();
        // 0 = overhand, 1 = figure-eight, 2 = square braid, 3 = round braid.
        for pattern in 0..=3 {
            let id = make_knot(world, pattern, 3);
            assert_ne!(id, u32::MAX, "pattern {pattern}");
            assert_eq!(last_error_code(), ERR_OK);

            assert_eq!(build_between(world, id), Bool::TRUE, "pattern {pattern}");
            assert_ne!(
                rope_knot_strand_soft_body(world, id, 0),
                u32::MAX,
                "pattern {pattern}"
            );

            assert_eq!(rope_knot_remove(world, id), Bool::TRUE, "pattern {pattern}");
            assert_eq!(rope_knot_remove(world, id), Bool::FALSE);
            assert_eq!(last_error_code(), ERR_NOT_FOUND);
        }
        world_destroy(world);
    }

    #[test]
    fn rope_knot_braid_owns_one_soft_body_per_strand() {
        let world = make_world();
        let id = make_knot(world, 2, 3);
        assert_ne!(id, u32::MAX);
        assert_eq!(build_between(world, id), Bool::TRUE);

        for strand in 0..3 {
            assert_ne!(rope_knot_strand_soft_body(world, id, strand), u32::MAX);
        }
        assert_eq!(rope_knot_strand_soft_body(world, id, 3), u32::MAX);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn rope_knot_custom_pattern_uses_control_points() {
        let world = make_world();
        let points = [
            v3(0.0, 5.0, 0.0),
            v3(0.5, 5.0, 0.0),
            v3(1.0, 5.0, 0.5),
            v3(1.0, 5.0, 1.5),
            v3(0.0, 5.0, 2.0),
        ];
        let id = rope_knot_create(
            world,
            4, // custom
            1,
            points.as_ptr(),
            points.len() as u32,
            0.05,
            100.0,
            0.4,
            1000.0,
        );
        assert_ne!(id, u32::MAX);
        // start/end are ignored for the custom pattern; the control points
        // define the geometry directly.
        assert_eq!(build_between(world, id), Bool::TRUE);

        let soft_id = rope_knot_strand_soft_body(world, id, 0);
        assert_ne!(soft_id, u32::MAX);
        assert_eq!(soft_body_particle_count(world, soft_id), 5);

        let mut pos = Vec3::default();
        let mut vel = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, soft_id, 2, &mut pos, &mut vel),
            Bool::TRUE
        );
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 5.0);
        assert_eq!(pos.z, 0.5);
        world_destroy(world);
    }

    #[test]
    fn rope_knot_rejects_invalid_input() {
        let world = make_world();

        // Unknown pattern.
        assert_eq!(make_knot(world, 5, 1), u32::MAX);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Strand count out of range.
        assert_eq!(make_knot(world, 0, 0), u32::MAX);
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Bad radius / stiffness / friction / density.
        for (radius, stiffness, friction, density) in [
            (0.0, 100.0, 0.4, 1000.0),
            (0.05, -1.0, 0.4, 1000.0),
            (0.05, 100.0, f64::NAN, 1000.0),
            (0.05, 100.0, 0.4, 0.0),
        ] {
            assert_eq!(
                rope_knot_create(
                    world,
                    0,
                    1,
                    std::ptr::null(),
                    0,
                    radius,
                    stiffness,
                    friction,
                    density
                ),
                u32::MAX
            );
            assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        }

        // Custom pattern without control points.
        assert_eq!(
            rope_knot_create(world, 4, 1, std::ptr::null(), 0, 0.05, 100.0, 0.4, 1000.0),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Degenerate span.
        let id = make_knot(world, 0, 1);
        assert_ne!(id, u32::MAX);
        assert_eq!(
            rope_knot_build(world, id, v3(1.0, 1.0, 1.0), v3(1.0, 1.0, 1.0)),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Non-finite span.
        assert_eq!(
            rope_knot_build(world, id, v3(0.0, 5.0, 0.0), v3(f64::INFINITY, 5.0, 2.0)),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Unknown ids.
        assert_eq!(rope_knot_remove(world, 987654), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn rope_knot_double_build_is_rejected() {
        let world = make_world();
        let id = make_knot(world, 1, 1);
        assert_ne!(id, u32::MAX);
        assert_eq!(build_between(world, id), Bool::TRUE);
        assert_eq!(build_between(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_UNSUPPORTED);
        world_destroy(world);
    }

    #[test]
    fn rope_knot_wind_and_steps_keep_particles_finite() {
        let world = make_world();
        let id = make_knot(world, 2, 3);
        assert_ne!(id, u32::MAX);
        assert_eq!(build_between(world, id), Bool::TRUE);
        assert_eq!(rope_knot_set_wind(world, id, v3(2.0, 0.0, 0.0)), Bool::TRUE);
        assert_eq!(
            rope_knot_set_wind(world, id, v3(f64::NAN, 0.0, 0.0)),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let soft_id = rope_knot_strand_soft_body(world, id, 0);
        assert_ne!(soft_id, u32::MAX);

        let mut pos = Vec3::default();
        let mut vel = Vec3::default();
        for _ in 0..20 {
            world_step(world, 1.0 / 60.0);
        }
        assert_eq!(
            soft_body_get_particle(world, soft_id, 0, &mut pos, &mut vel),
            Bool::TRUE
        );
        assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite());
        assert!(vel.x.is_finite() && vel.y.is_finite() && vel.z.is_finite());
        world_destroy(world);
    }

    #[test]
    fn rope_knot_null_world_is_rejected() {
        assert_eq!(
            rope_knot_create(
                std::ptr::null_mut(),
                0,
                1,
                std::ptr::null(),
                0,
                0.05,
                100.0,
                0.4,
                1000.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            rope_knot_build(
                std::ptr::null_mut(),
                0,
                v3(0.0, 0.0, 0.0),
                v3(1.0, 0.0, 0.0)
            ),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }
}
