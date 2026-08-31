//! End-to-end tests for the sensor trigger zone (the "fourth body type").

#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_ex, world_insert_collider_with_parent,
    };
    use mps_core::rapier::ffi::{
        BodyStatus, Bool, ColliderHandleRaw, RigidBodyHandleRaw, ShapeDesc, ShapeType, Vec3,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_set_translation,
        rigid_body_set_translation, world_insert_rigid_body,
    };
    use mps_core::rapier::sensor::{
        sensor_zone_clear, sensor_zone_consume, sensor_zone_contact_count, sensor_zone_create,
        sensor_zone_destroy, sensor_zone_get_contacts, sensor_zone_is_triggered, sensor_zone_poll,
        sensor_zone_set_edge, sensor_zone_set_enabled, sensor_zone_set_shape,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn make_world() -> *mut mps_core::rapier::ffi::WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        })
    }

    /// A dynamic ball dropped into the sensor zone.
    fn make_ball(
        world: *mut mps_core::rapier::ffi::WorldHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> RigidBodyHandleRaw {
        let builder = rigid_body_builder_create(mps_core::rapier::ffi::BodyStatus::Dynamic as u32);
        rigid_body_builder_set_translation(builder, Vec3 { x, y, z });
        let body = world_insert_rigid_body(world, rigid_body_builder_build(builder));
        let shape = ShapeDesc {
            shape_type: ShapeType::Ball as u32,
            a: 0.3,
            ..Default::default()
        };
        let collider = collider_builder_build(collider_builder_create_ex(shape));
        world_insert_collider_with_parent(world, collider, body);
        body
    }

    #[test]
    fn create_and_destroy() {
        let world = make_world();
        let shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        };
        let id = sensor_zone_create(
            world,
            shape,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);
        assert_eq!(sensor_zone_destroy(world, id), Bool::TRUE);
        assert_eq!(sensor_zone_destroy(world, id), Bool::FALSE);
        world_destroy(world);
    }

    #[test]
    fn detects_overlapping_body() {
        let world = make_world();
        // Sensor cuboid 1x1x1 centered at origin.
        let zone_shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        };
        let zone = sensor_zone_create(
            world,
            zone_shape,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(zone, u32::MAX);

        // Drop a ball near the origin so it falls into the zone.
        let _ball = make_ball(world, 0.0, 3.0, 0.0);

        let dt = 1.0 / 60.0;
        let mut triggered = false;
        for _ in 0..180 {
            world_step(world, dt);
            sensor_zone_poll(world, zone);
            if sensor_zone_contact_count(world, zone) > 0 {
                triggered = true;
                break;
            }
        }
        assert!(triggered, "sensor zone should detect the falling ball");
        assert_eq!(sensor_zone_is_triggered(world, zone), Bool::TRUE);
        assert!(sensor_zone_contact_count(world, zone) >= 1);
        world_destroy(world);
    }

    #[test]
    fn disabled_zone_ignores_overlaps() {
        let world = make_world();
        let zone_shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        };
        let zone = sensor_zone_create(
            world,
            zone_shape,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_eq!(
            sensor_zone_set_enabled(world, zone, Bool::FALSE),
            Bool::TRUE
        );
        let _ball = make_ball(world, 0.0, 0.0, 0.0);
        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            world_step(world, dt);
            sensor_zone_poll(world, zone);
        }
        // Disabled: poll is a no-op, so no contacts recorded.
        assert_eq!(sensor_zone_contact_count(world, zone), 0);
        assert_eq!(sensor_zone_is_triggered(world, zone), Bool::FALSE);
        world_destroy(world);
    }

    #[test]
    fn reports_contact_handles() {
        let world = make_world();
        let zone_shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 2.0,
            b: 2.0,
            c: 2.0,
            ..Default::default()
        };
        let zone = sensor_zone_create(
            world,
            zone_shape,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        // Floor so the ball rests at y=0.3 inside the 2x2x2 zone (otherwise it falls
        // out under gravity now that the zone no longer self-detects).
        let fl = rigid_body_builder_create(mps_core::rapier::ffi::BodyStatus::Fixed as u32);
        rigid_body_builder_set_translation(
            fl,
            Vec3 {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
        );
        let flb = world_insert_rigid_body(world, rigid_body_builder_build(fl));
        let fls = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 20.0,
            b: 0.5,
            c: 20.0,
            ..Default::default()
        };
        world_insert_collider_with_parent(
            world,
            collider_builder_build(collider_builder_create_ex(fls)),
            flb,
        );
        let _ball = make_ball(world, 0.0, 0.3, 0.0);
        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            world_step(world, dt);
            sensor_zone_poll(world, zone);
        }
        let count = sensor_zone_contact_count(world, zone);
        assert!(count >= 1);
        let mut buf: Vec<ColliderHandleRaw> = vec![0u64; count as usize];
        let written = sensor_zone_get_contacts(world, zone, buf.as_mut_ptr(), count);
        assert_eq!(written, count);
        assert!(buf.iter().all(|h| *h != 0));
        world_destroy(world);
    }

    /// `set_shape` mutates the sensor collider in place, so a fixed ball that was
    /// outside a small zone becomes detectable after growing it. The ball is fixed
    /// (no gravity) so the only collider the zone can detect is the ball itself.
    #[test]
    fn set_shape_grows_detection_volume() {
        let world = make_world();
        // Small 0.5×0.5×0.5 zone at origin.
        let small = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 0.25,
            b: 0.25,
            c: 0.25,
            ..Default::default()
        };
        let zone = sensor_zone_create(
            world,
            small,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(zone, u32::MAX);
        // A FIXED ball parked at x=0.6 — inside the BIG zone (x∈[-1,1]) but outside
        // the SMALL zone (x∈[-0.25,0.25]). Fixed => it never moves, so the only
        // thing the zone can detect is the ball.
        let fbuilder = rigid_body_builder_create(mps_core::rapier::ffi::BodyStatus::Fixed as u32);
        rigid_body_builder_set_translation(
            fbuilder,
            Vec3 {
                x: 0.6,
                y: 0.0,
                z: 0.0,
            },
        );
        let fbody = world_insert_rigid_body(world, rigid_body_builder_build(fbuilder));
        let fshape = ShapeDesc {
            shape_type: ShapeType::Ball as u32,
            a: 0.3,
            ..Default::default()
        };
        world_insert_collider_with_parent(
            world,
            collider_builder_build(collider_builder_create_ex(fshape)),
            fbody,
        );
        let dt = 1.0 / 60.0;
        for _ in 0..10 {
            world_step(world, dt);
            sensor_zone_poll(world, zone);
        }
        assert_eq!(
            sensor_zone_contact_count(world, zone),
            0,
            "ball should be outside the small zone"
        );

        // Grow the zone to 2×2×2 — the ball is now inside.
        let big = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        };
        assert_eq!(sensor_zone_set_shape(world, zone, big), Bool::TRUE);
        for _ in 0..10 {
            world_step(world, dt);
            sensor_zone_poll(world, zone);
        }
        assert!(
            sensor_zone_contact_count(world, zone) >= 1,
            "zone should detect the ball after growing"
        );
        // Unknown id returns FALSE without panicking.
        assert_eq!(sensor_zone_set_shape(world, zone + 999, big), Bool::FALSE);
        world_destroy(world);
    }

    /// In rising-edge mode, `is_triggered` is TRUE only on the poll where an
    /// overlap first appears, then FALSE on the next poll even though the ball is
    /// still inside (until the zone is emptied and re-entered).
    #[test]
    fn edge_trigger_fires_on_enter() {
        let world = make_world();
        // Fixed ball parked at x=0.6, inside the BIG zone (x in [-1,1]) but outside
        // the SMALL zone (x in [-0.25,0.25]). Fixed so it never moves.
        let fb = rigid_body_builder_create(mps_core::rapier::ffi::BodyStatus::Fixed as u32);
        rigid_body_builder_set_translation(
            fb,
            Vec3 {
                x: 0.6,
                y: 0.0,
                z: 0.0,
            },
        );
        let fbody = world_insert_rigid_body(world, rigid_body_builder_build(fb));
        world_insert_collider_with_parent(
            world,
            collider_builder_build(collider_builder_create_ex(ShapeDesc {
                shape_type: ShapeType::Ball as u32,
                a: 0.3,
                ..Default::default()
            })),
            fbody,
        );

        // Small zone at origin so the fixed ball starts OUTSIDE it.
        let small = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 0.25,
            b: 0.25,
            c: 0.25,
            ..Default::default()
        };
        let zone = sensor_zone_create(
            world,
            small,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(zone, u32::MAX);
        // Enable rising-edge mode.
        assert_eq!(sensor_zone_set_edge(world, zone, Bool::TRUE), Bool::TRUE);
        // Unknown id returns FALSE without panicking.
        assert_eq!(
            sensor_zone_set_edge(world, zone + 999, Bool::TRUE),
            Bool::FALSE
        );

        let dt = 1.0 / 60.0;
        // Prime: ball is outside the small zone -> no overlap.
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_is_triggered(world, zone),
            Bool::FALSE,
            "edge mode: nothing triggered before overlap"
        );

        // Grow the zone so the ball is now inside -> a rising edge should fire.
        let big = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        };
        assert_eq!(sensor_zone_set_shape(world, zone, big), Bool::TRUE);
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_is_triggered(world, zone),
            Bool::TRUE,
            "edge mode: should fire on the enter poll"
        );

        // Next poll: ball still inside, but it is no longer a NEW overlap -> FALSE.
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_is_triggered(world, zone),
            Bool::FALSE,
            "edge mode: should NOT retrigger while still overlapping"
        );

        // Reset to level mode: now it reports TRUE while overlapping.
        assert_eq!(sensor_zone_set_edge(world, zone, Bool::FALSE), Bool::TRUE);
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_is_triggered(world, zone),
            Bool::TRUE,
            "level mode: stays triggered while overlapping"
        );
        world_destroy(world);
    }

    /// `consume` returns TRUE exactly once per rising edge and then FALSE until the
    /// next edge; `clear` resets the sticky state so the edge can re-arm after the
    /// body leaves and re-enters the zone.
    #[test]
    fn consume_returns_one_shot_then_clear_rearms() {
        let world = make_world();
        // Start with NO body inside the zone.
        let zone = sensor_zone_create(
            world,
            ShapeDesc {
                shape_type: ShapeType::Cuboid as u32,
                a: 0.5,
                b: 0.5,
                c: 0.5,
                ..Default::default()
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let dt = 1.0 / 60.0;
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::FALSE,
            "empty zone: no edge to consume"
        );

        // Drop a ball inside; poll → fresh rising edge latches.
        let ball = rigid_body_builder_create(BodyStatus::Fixed as u32);
        rigid_body_builder_set_translation(
            ball,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let ball_body = world_insert_rigid_body(world, rigid_body_builder_build(ball));
        world_insert_collider_with_parent(
            world,
            collider_builder_build(collider_builder_create_ex(ShapeDesc {
                shape_type: ShapeType::Ball as u32,
                a: 0.1,
                ..Default::default()
            })),
            ball_body,
        );
        world_step(world, dt);
        sensor_zone_poll(world, zone);

        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::TRUE,
            "first consume sees the edge"
        );
        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::FALSE,
            "second consume sees nothing"
        );
        // Re-poll (still overlapping) must not produce a new edge/latch.
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::FALSE,
            "staying overlapped does not re-latch"
        );

        // Move the ball out; zone goes empty.
        rigid_body_set_translation(
            world,
            ball_body,
            Vec3 {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
            Bool::TRUE,
        );
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::FALSE,
            "no overlap after moving ball away"
        );

        // Re-arm from any prior sticky state, then move the ball back in → fresh edge.
        assert_eq!(sensor_zone_clear(world, zone), Bool::TRUE);
        rigid_body_set_translation(
            world,
            ball_body,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Bool::TRUE,
        );
        world_step(world, dt);
        sensor_zone_poll(world, zone);
        assert_eq!(
            sensor_zone_consume(world, zone),
            Bool::TRUE,
            "after clear + re-entry, consume sees the fresh edge"
        );

        sensor_zone_destroy(world, zone);
        world_destroy(world);
    }
}
