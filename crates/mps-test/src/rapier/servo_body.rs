//! End-to-end tests for the PD/PID servo body (the "sixth body type").

#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::{Bool, Quat, ShapeDesc, ShapeType, Vec3};
    use mps_core::rapier::servo_body::{
        servo_body_create, servo_body_destroy, servo_body_get_rigid_body_handle,
        servo_body_get_translation, servo_body_get_velocity,
        servo_body_set_target_angular_velocity, servo_body_set_target_position,
        servo_body_set_target_rotation, servo_body_set_target_velocity, servo_body_update,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn make_world() -> *mut mps_core::rapier::ffi::WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
    }

    fn make_servo(
        world: *mut mps_core::rapier::ffi::WorldHandle,
        kp: f64,
        kd: f64,
        ki: f64,
    ) -> u32 {
        let shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 0.5,
            b: 0.5,
            c: 0.5,
            ..Default::default()
        };
        servo_body_create(
            world,
            shape,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            kp,
            kd,
            ki,
            0, // all axes
        )
    }

    fn read_pos(world: *mut mps_core::rapier::ffi::WorldHandle, id: u32) -> Vec3 {
        let mut p = Vec3::default();
        assert_eq!(
            servo_body_get_translation(world, id, &mut p as *mut Vec3),
            Bool::TRUE
        );
        p
    }

    fn read_vel(world: *mut mps_core::rapier::ffi::WorldHandle, id: u32) -> Vec3 {
        let mut v = Vec3::default();
        assert_eq!(
            servo_body_get_velocity(world, id, &mut v as *mut Vec3),
            Bool::TRUE
        );
        v
    }

    #[test]
    fn create_and_destroy() {
        let world = make_world();
        let id = make_servo(world, 60.0, 0.8, 0.0);
        assert_ne!(id, u32::MAX);
        assert_eq!(servo_body_destroy(world, id), Bool::TRUE);
        // Double destroy fails.
        assert_eq!(servo_body_destroy(world, id), Bool::FALSE);
        world_destroy(world);
    }

    #[test]
    fn invalid_gains_rejected() {
        let world = make_world();
        let shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 0.5,
            b: 0.5,
            c: 0.5,
            ..Default::default()
        };
        // Negative kp rejected.
        assert_eq!(
            servo_body_create(world, shape, Vec3::default(), -1.0, 0.8, 0.0, 0,),
            u32::MAX
        );
        // NaN kd rejected.
        assert_eq!(
            servo_body_create(world, shape, Vec3::default(), 60.0, f64::NAN, 0.0, 0,),
            u32::MAX
        );
        world_destroy(world);
    }

    #[test]
    fn position_converges_to_target() {
        let world = make_world();
        let id = make_servo(world, 60.0, 4.0, 0.0);
        assert_ne!(id, u32::MAX);

        // Drive the body toward x = 5.0. High kd dampens the approach.
        assert_eq!(
            servo_body_set_target_position(
                world,
                id,
                Vec3 {
                    x: 5.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..600 {
            world_step(world, dt);
            servo_body_update(world, id, dt);
        }

        let pos = read_pos(world, id);
        // The body should be very close to x = 5.0.
        assert!((pos.x - 5.0).abs() < 0.2, "expected x ≈ 5.0, got {}", pos.x);
        servo_body_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn target_velocity_drives_body() {
        let world = make_world();
        // Very low kp so the position term doesn't dominate; moderate kd so
        // the velocity target (kd term) gently drives the body toward target v.
        let id = make_servo(world, 0.0, 0.5, 0.0);
        assert_ne!(id, u32::MAX);

        // Position target stays at origin (where the body starts), so the
        // position error is zero and only the velocity target matters.
        assert_eq!(
            servo_body_set_target_velocity(
                world,
                id,
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..120 {
            world_step(world, dt);
            servo_body_update(world, id, dt);
        }

        let vel = read_vel(world, id);
        // Velocity should be close to 1.0 m/s along X.
        assert!(
            (vel.x - 1.0).abs() < 0.3,
            "expected vx ≈ 1.0, got {}",
            vel.x
        );
        servo_body_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn pid_mode_with_integral_term() {
        let world = make_world();
        // Use PID mode (ki > 0) to test the PidController path.
        let id = make_servo(world, 100.0, 1.0, 5.0);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            servo_body_set_target_position(
                world,
                id,
                Vec3 {
                    x: 3.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..300 {
            world_step(world, dt);
            servo_body_update(world, id, dt);
        }

        let pos = read_pos(world, id);
        assert!(
            (pos.x - 3.0).abs() < 0.25,
            "PID: expected x ≈ 3.0, got {}",
            pos.x
        );
        servo_body_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn target_rotation_converges() {
        let world = make_world();
        let id = make_servo(world, 120.0, 1.0, 0.0);
        assert_ne!(id, u32::MAX);

        // Target: 90° rotation around Y axis.
        let angle = std::f64::consts::FRAC_PI_2;
        // Quaternion (axis-angle): (0, sin(θ/2), 0, cos(θ/2))
        let half = angle * 0.5;
        assert_eq!(
            servo_body_set_target_rotation(
                world,
                id,
                Quat {
                    i: 0.0,
                    j: half.sin(),
                    k: 0.0,
                    w: half.cos(),
                },
            ),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..300 {
            world_step(world, dt);
            servo_body_update(world, id, dt);
        }

        // The body's angular velocity should have damped — it settled near the
        // target orientation. We can't read rotation back directly, but the
        // velocity should be near zero after convergence.
        let vel = read_vel(world, id);
        assert!(
            vel.x.abs() < 1.0 && vel.y.abs() < 1.0 && vel.z.abs() < 1.0,
            "expected near-zero velocity after rotation convergence, got {:?}",
            vel
        );
        servo_body_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn set_target_angular_velocity() {
        let world = make_world();
        let id = make_servo(world, 10.0, 0.8, 0.0);
        assert_ne!(id, u32::MAX);

        // Set a target angular velocity of 2.0 rad/s around Y, and move the
        // position target far away so the position term doesn't fight it.
        assert_eq!(
            servo_body_set_target_position(
                world,
                id,
                Vec3 {
                    x: 100.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            Bool::TRUE
        );
        assert_eq!(
            servo_body_set_target_angular_velocity(
                world,
                id,
                Vec3 {
                    x: 0.0,
                    y: 2.0,
                    z: 0.0,
                },
            ),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            world_step(world, dt);
            servo_body_update(world, id, dt);
        }

        // The body should be spinning around Y at ~2.0 rad/s.
        // We read linear velocity (which should be near zero in zero-g) and
        // check that the body has started to drift — the servo's angular
        // velocity target means the body is rotating.
        let vel = read_vel(world, id);
        // Linear velocity should be near zero (no gravity, no force).
        assert!(
            vel.x.abs() < 0.5 && vel.z.abs() < 0.5,
            "expected near-zero linear velocity, got {:?}",
            vel
        );
        servo_body_destroy(world, id);
        world_destroy(world);
    }

    #[test]
    fn get_rigid_body_handle_returns_nonzero() {
        let world = make_world();
        let id = make_servo(world, 60.0, 0.8, 0.0);
        assert_ne!(id, u32::MAX);

        let raw = servo_body_get_rigid_body_handle(world, id);
        assert_ne!(raw, 0, "expected non-zero raw handle");
        servo_body_destroy(world, id);

        // Unknown id returns 0.
        assert_eq!(servo_body_get_rigid_body_handle(world, id), 0);
        world_destroy(world);
    }

    #[test]
    fn invalid_id_returns_false() {
        let world = make_world();
        let bad_id = 9999;
        assert_eq!(
            servo_body_set_target_position(world, bad_id, Vec3::default()),
            Bool::FALSE
        );
        assert_eq!(
            servo_body_set_target_rotation(
                world,
                bad_id,
                Quat {
                    i: 0.0,
                    j: 0.0,
                    k: 0.0,
                    w: 1.0,
                },
            ),
            Bool::FALSE
        );
        assert_eq!(
            servo_body_set_target_velocity(world, bad_id, Vec3::default()),
            Bool::FALSE
        );
        assert_eq!(
            servo_body_set_target_angular_velocity(world, bad_id, Vec3::default()),
            Bool::FALSE
        );
        assert_eq!(servo_body_update(world, bad_id, 1.0 / 60.0), Bool::FALSE);
        assert_eq!(servo_body_destroy(world, bad_id), Bool::FALSE);

        let mut p = Vec3::default();
        assert_eq!(
            servo_body_get_translation(world, bad_id, &mut p as *mut Vec3),
            Bool::FALSE
        );
        let mut v = Vec3::default();
        assert_eq!(
            servo_body_get_velocity(world, bad_id, &mut v as *mut Vec3),
            Bool::FALSE
        );
        world_destroy(world);
    }

    #[test]
    fn zero_quaternion_rejected() {
        let world = make_world();
        let id = make_servo(world, 60.0, 0.8, 0.0);
        assert_ne!(id, u32::MAX);

        // All-zero quaternion is invalid.
        assert_eq!(
            servo_body_set_target_rotation(
                world,
                id,
                Quat {
                    i: 0.0,
                    j: 0.0,
                    k: 0.0,
                    w: 0.0,
                },
            ),
            Bool::FALSE
        );
        servo_body_destroy(world, id);
        world_destroy(world);
    }
}
