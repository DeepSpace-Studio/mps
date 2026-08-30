//! End-to-end tests for the sensor trigger zone (the "fourth body type").

#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_ex, world_insert_collider_with_parent,
    };
    use mps_core::rapier::ffi::{
        Bool, ColliderHandleRaw, RigidBodyHandleRaw, ShapeDesc, ShapeType, Vec3,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_set_translation,
        world_insert_rigid_body,
    };
    use mps_core::rapier::sensor::{
        sensor_zone_contact_count, sensor_zone_create, sensor_zone_destroy,
        sensor_zone_get_contacts, sensor_zone_is_triggered, sensor_zone_poll,
        sensor_zone_set_enabled,
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
        let _ball = make_ball(world, 0.0, 0.0, 0.0);
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
}
