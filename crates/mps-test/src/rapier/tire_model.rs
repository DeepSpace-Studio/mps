//! End-to-end tests for the Pacejka-style tire model layered on the ray-cast
//! vehicle controller.

#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
        last_error_code,
    };
    use mps_core::rapier::ffi::{Bool, ShapeDesc, ShapeType, Vec3, WorldHandle};
    use mps_core::rapier::tire_model::{
        tire_model_create, tire_model_get_forces, tire_model_remove, tire_model_set_params,
        tire_model_update,
    };
    use mps_core::rapier::vehicle::{
        vehicle_controller_add_wheel, vehicle_controller_create, vehicle_controller_set_brake,
        vehicle_controller_set_engine_force, vehicle_controller_update,
    };
    use mps_core::rapier::world::{world_create, world_destroy};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn make_world() -> *mut WorldHandle {
        world_create(v3(0.0, -9.81, 0.0))
    }

    /// A chassis + 4 wheels (corners, suspension down -Y, axle along -Z),
    /// floating in free space — no floor, so wheels are airborne and the
    /// static-load fallback drives the tire model.
    fn make_car(world: *mut WorldHandle) -> u32 {
        let chassis = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 0.3,
            c: 2.0,
            ..Default::default()
        };
        let id = vehicle_controller_create(world, chassis, v3(0.0, 20.0, 0.0));
        assert_ne!(id, u32::MAX);
        for x in [-0.9f64, 0.9] {
            for z in [-1.5f64, 1.5] {
                let idx = vehicle_controller_add_wheel(
                    world,
                    id,
                    v3(x, 0.0, z),
                    v3(0.0, -1.0, 0.0),
                    v3(0.0, 0.0, -1.0),
                    0.3,    // suspension rest length
                    0.4,    // wheel radius
                    24.0,   // suspension stiffness
                    0.8,    // damping compression
                    0.8,    // damping relaxation
                    1.5,    // friction slip
                    0.3,    // max suspension travel
                    6000.0, // max suspension force
                    0.5,    // side friction stiffness
                );
                assert_ne!(idx, u32::MAX);
            }
        }
        id
    }

    #[test]
    fn tire_model_create_requires_vehicle() {
        let world = make_world();
        assert_eq!(tire_model_create(world, 999, 4), u32::MAX);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn tire_model_create_validates_wheel_count() {
        let world = make_world();
        let vehicle = make_car(world);
        assert_eq!(tire_model_create(world, vehicle, 0), u32::MAX);
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(tire_model_create(world, vehicle, 33), u32::MAX);
        assert_eq!(last_error_code(), ERR_CAPACITY);

        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(tire_model_remove(world, id), Bool::TRUE);
        assert_eq!(tire_model_remove(world, id), Bool::TRUE); // idempotent
        world_destroy(world);
    }

    #[test]
    fn tire_model_set_params_validates_input() {
        let world = make_world();
        let vehicle = make_car(world);
        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            tire_model_set_params(world, id, 0, 1.4, 1.3, 0.12, 0.15, 0.85, 1.2),
            Bool::TRUE
        );
        // Wheel index out of range.
        assert_eq!(
            tire_model_set_params(world, id, 4, 1.4, 1.3, 0.12, 0.15, 0.85, 1.2),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // Non-physical parameter (zero peak friction).
        assert_eq!(
            tire_model_set_params(world, id, 0, 0.0, 1.3, 0.12, 0.15, 0.85, 1.2),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // Unknown tire model.
        assert_eq!(
            tire_model_set_params(world, 987654, 0, 1.4, 1.3, 0.12, 0.15, 0.85, 1.2),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn tire_model_update_reports_zero_forces_at_rest() {
        let world = make_world();
        let vehicle = make_car(world);
        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            vehicle_controller_update(world, vehicle, 1.0 / 60.0),
            Bool::TRUE
        );
        assert_eq!(tire_model_update(world, id, 1.0 / 60.0), Bool::TRUE);

        let mut fx = 0.0;
        let mut fy = 0.0;
        for wheel in 0..4 {
            assert_eq!(
                tire_model_get_forces(world, id, wheel, &mut fx, &mut fy),
                Bool::TRUE
            );
            assert_eq!(fx, 0.0, "wheel {wheel} at rest must have no drive force");
            assert_eq!(fy, 0.0, "wheel {wheel} at rest must have no side force");
        }
        assert_eq!(last_error_code(), ERR_OK);
        world_destroy(world);
    }

    #[test]
    fn tire_model_engine_force_produces_longitudinal_force() {
        let world = make_world();
        let vehicle = make_car(world);
        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            vehicle_controller_set_engine_force(world, vehicle, 0, 1000.0),
            Bool::TRUE
        );
        assert_eq!(
            vehicle_controller_update(world, vehicle, 1.0 / 60.0),
            Bool::TRUE
        );
        assert_eq!(tire_model_update(world, id, 1.0 / 60.0), Bool::TRUE);

        let mut fx = 0.0;
        let mut fy = 0.0;
        assert_eq!(
            tire_model_get_forces(world, id, 0, &mut fx, &mut fy),
            Bool::TRUE
        );
        assert!(
            fx > 0.0,
            "a driven wheel must produce positive longitudinal force, got {fx}"
        );
        // The force must saturate at the friction-ellipse cap, i.e. stay
        // bounded by peak μ · load^α · ellipse factor (well below 1e4 here).
        assert!(fx < 1.0e4, "force must stay bounded, got {fx}");
        world_destroy(world);
    }

    #[test]
    fn tire_model_brake_locks_wheels() {
        let world = make_world();
        let vehicle = make_car(world);
        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);

        // Spin the wheels up first.
        assert_eq!(
            vehicle_controller_set_engine_force(world, vehicle, 0, 2000.0),
            Bool::TRUE
        );
        assert_eq!(
            vehicle_controller_update(world, vehicle, 1.0 / 60.0),
            Bool::TRUE
        );
        assert_eq!(tire_model_update(world, id, 1.0 / 60.0), Bool::TRUE);

        // Full brake locks them: slip ratio becomes negative (wheel slower
        // than the road) and the longitudinal force opposes the motion.
        assert_eq!(
            vehicle_controller_set_brake(world, vehicle, 0, 1.0e9),
            Bool::TRUE
        );
        assert_eq!(
            vehicle_controller_update(world, vehicle, 1.0 / 60.0),
            Bool::TRUE
        );
        assert_eq!(tire_model_update(world, id, 1.0 / 60.0), Bool::TRUE);

        let mut fx = 0.0;
        let mut fy = 0.0;
        assert_eq!(
            tire_model_get_forces(world, id, 0, &mut fx, &mut fy),
            Bool::TRUE
        );
        assert!(fx <= 0.0, "a locked wheel must not drive, got {fx}");
        world_destroy(world);
    }

    #[test]
    fn tire_model_get_forces_validates_input() {
        let world = make_world();
        let vehicle = make_car(world);
        let id = tire_model_create(world, vehicle, 4);
        assert_ne!(id, u32::MAX);
        assert_eq!(tire_model_update(world, id, 1.0 / 60.0), Bool::TRUE);

        let mut fx = 0.0;
        let mut fy = 0.0;
        // Wheel index out of range.
        assert_eq!(
            tire_model_get_forces(world, id, 4, &mut fx, &mut fy),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // Null out pointers.
        assert_eq!(
            tire_model_get_forces(world, id, 0, std::ptr::null_mut(), std::ptr::null_mut()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        // Invalid dt.
        assert_eq!(tire_model_update(world, id, 0.0), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // Unknown tire model.
        assert_eq!(tire_model_update(world, 987654, 1.0 / 60.0), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn tire_model_null_world_is_rejected() {
        assert_eq!(tire_model_create(std::ptr::null_mut(), 0, 4), u32::MAX);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            tire_model_update(std::ptr::null_mut(), 0, 1.0 / 60.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }
}
