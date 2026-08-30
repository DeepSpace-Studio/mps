//! End-to-end tests for the ray-cast vehicle controller (the "fifth body type").

#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_ex, world_insert_collider_with_parent,
    };
    use mps_core::rapier::ffi::{BodyStatus, Bool, ShapeDesc, ShapeType, Vec3};
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_set_translation,
        world_insert_rigid_body,
    };
    use mps_core::rapier::vehicle::{
        vehicle_controller_add_wheel, vehicle_controller_create, vehicle_controller_destroy,
        vehicle_controller_get_translation, vehicle_controller_set_engine_force,
        vehicle_controller_set_steering, vehicle_controller_update,
        vehicle_controller_wheel_on_ground,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn make_world() -> *mut mps_core::rapier::ffi::WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        })
    }

    /// A fixed floor cuboid whose top surface sits at y = 0.
    fn make_floor(world: *mut mps_core::rapier::ffi::WorldHandle) {
        let builder = rigid_body_builder_create(BodyStatus::Fixed as u32);
        rigid_body_builder_set_translation(
            builder,
            Vec3 {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
        );
        let body = world_insert_rigid_body(world, rigid_body_builder_build(builder));
        let shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 20.0,
            b: 0.5,
            c: 20.0,
            ..Default::default()
        };
        let collider = collider_builder_build(collider_builder_create_ex(shape));
        world_insert_collider_with_parent(world, collider, body);
    }

    /// Build a chassis (cuboid) + 4 wheels and return the vehicle id.
    fn make_car(world: *mut mps_core::rapier::ffi::WorldHandle) -> u32 {
        let chassis = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 0.3,
            c: 2.0,
            ..Default::default()
        };
        let id = vehicle_controller_create(
            world,
            chassis,
            Vec3 {
                x: 0.0,
                y: 1.5,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);
        // 4 wheels at the corners, suspension pointing down (-Y), axle along -Z.
        let corners = [
            (
                Vec3 {
                    x: 0.9,
                    y: 0.0,
                    z: 1.5,
                },
                -1.0f64,
            ),
            (
                Vec3 {
                    x: -0.9,
                    y: 0.0,
                    z: 1.5,
                },
                -1.0f64,
            ),
            (
                Vec3 {
                    x: 0.9,
                    y: 0.0,
                    z: -1.5,
                },
                -1.0f64,
            ),
            (
                Vec3 {
                    x: -0.9,
                    y: 0.0,
                    z: -1.5,
                },
                -1.0f64,
            ),
        ];
        for (conn, _) in corners.iter() {
            let idx = vehicle_controller_add_wheel(
                world,
                id,
                *conn,
                Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                0.3,
                0.4,
                24.0,
                0.8,
                0.8,
                1.5,
                0.3,
                6000.0,
                0.5,
            );
            assert_ne!(idx, u32::MAX);
        }
        id
    }

    #[test]
    fn create_and_destroy() {
        let world = make_world();
        let id = make_car(world);
        assert_eq!(vehicle_controller_destroy(world, id), Bool::TRUE);
        assert_eq!(vehicle_controller_destroy(world, id), Bool::FALSE);
        world_destroy(world);
    }

    #[test]
    fn wheels_touch_ground_after_drop() {
        let world = make_world();
        make_floor(world);
        let id = make_car(world);

        let dt = 1.0 / 60.0;
        // Let the car settle on the floor.
        for _ in 0..120 {
            world_step(world, dt);
            vehicle_controller_update(world, id, dt);
        }
        // All four wheels should be in contact with the floor.
        let mut grounded = 0u32;
        for w in 0..4 {
            if vehicle_controller_wheel_on_ground(world, id, w) == Bool::TRUE {
                grounded += 1;
            }
        }
        assert!(grounded >= 3, "expected wheels on ground, got {grounded}/4");
        vehicle_controller_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn engine_force_drives_forward() {
        let world = make_world();
        make_floor(world);
        let id = make_car(world);
        let dt = 1.0 / 60.0;

        // Settle first.
        for _ in 0..60 {
            world_step(world, dt);
            vehicle_controller_update(world, id, dt);
        }
        let start = read_pos(world, id);

        // Drive: apply engine force on all wheels, steer straight.
        for w in 0..4 {
            vehicle_controller_set_engine_force(world, id, w, 30.0);
            vehicle_controller_set_steering(world, id, w, 0.0);
        }
        for _ in 0..180 {
            world_step(world, dt);
            vehicle_controller_update(world, id, dt);
        }
        let end = read_pos(world, id);
        // Chassis should have travelled a meaningful distance along X (rapier's
        // vehicle forward axis defaults to index 0 = X).
        let travelled = (end.x - start.x).abs();
        assert!(
            travelled > 0.5,
            "vehicle should move under engine force, travelled {travelled}"
        );
        vehicle_controller_destroy(world, id);
        world_destroy(world);
    }

    fn read_pos(world: *mut mps_core::rapier::ffi::WorldHandle, id: u32) -> Vec3 {
        let mut p = Vec3::default();
        assert_eq!(
            vehicle_controller_get_translation(world, id, &mut p as *mut Vec3),
            Bool::TRUE
        );
        p
    }
}
