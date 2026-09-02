#[cfg(test)]
mod tests {
    use mps_core::rapier::articulation::{
        articulation_body_link_count, articulation_body_link_handle,
        articulation_body_set_joint_target,
    };
    use mps_core::rapier::error::{
        ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK, last_error_code,
    };
    use mps_core::rapier::ffi::{Bool, Vec3, WorldHandle};
    use mps_core::rapier::rigid_body::rigid_body_get_translation;
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    const SENTINEL: u32 = u32::MAX;

    fn make_world() -> *mut WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
    }

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn create_arm(world: *mut WorldHandle, targets: Option<&[f64]>) -> u32 {
        let (ptr, len) = match targets {
            Some(t) => (t.as_ptr(), t.len() as u32),
            None => (std::ptr::null(), 0),
        };
        articulation_body_create(
            world,
            v(0.0, 5.0, 0.0), // base
            v(1.0, 0.0, 0.0), // chain direction +X
            v(0.0, 0.0, 1.0), // joints rotate about +Z
            4,
            0.3, // link radius (spacing = 2·radius)
            1.0, // link mass
            ptr,
            len,
            2000.0, // servo spring stiffness
            30.0,   // motor damping
        )
    }

    use mps_core::rapier::articulation::articulation_body_create;

    #[test]
    fn articulation_create_builds_chain() {
        let world = make_world();
        let id = create_arm(world, None);
        assert_ne!(id, SENTINEL);
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(articulation_body_link_count(world, id), 4);

        // Link handles: valid, in chain order along +X, spacing 2·r = 0.6.
        let mut prev: Option<Vec3> = None;
        for i in 0..4u32 {
            let h = articulation_body_link_handle(world, id, i);
            assert_ne!(h, 0, "link {i} handle");
            let p = rigid_body_get_translation(world, h);
            if let Some(prev) = prev {
                let d = ((p.x - prev.x).powi(2) + (p.y - prev.y).powi(2) + (p.z - prev.z).powi(2))
                    .sqrt();
                assert!((d - 0.6).abs() < 1e-9, "link {i} spacing {d}");
            }
            prev = Some(p);
        }
        // Out-of-range link index → 0 + ERR_NOT_FOUND.
        assert_eq!(articulation_body_link_handle(world, id, 9), 0);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn articulation_motor_bends_chain_to_target() {
        let world = make_world();
        // All joints commanded to fold 90° (π/2) about +Z.
        let id = create_arm(world, Some(&[std::f64::consts::FRAC_PI_2; 3]));
        assert_ne!(id, SENTINEL);

        // Zero gravity was set at creation; let the motors converge.
        for _ in 0..600 {
            world_step(world, 1.0 / 60.0);
        }

        // The chain should fold: link 3 (end) must be significantly closer to
        // the base than the straight-pose 3·0.6 = 1.8 m (a 3-link fold at 90°
        // gives well under 1.2 m), and must have swung off the +X line.
        let h3 = articulation_body_link_handle(world, id, 3);
        let end = rigid_body_get_translation(world, h3);
        let base = v(0.0, 5.0, 0.0);
        let reach = ((end.x - base.x).powi(2) + (end.y - base.y).powi(2)).sqrt();
        assert!(
            reach < 1.5,
            "folded arm should shorten: end reach {reach} (end {end:?})"
        );
        assert!(
            end.y > 0.2 || end.x < 1.5,
            "arm should bend away from straight +X: {end:?}"
        );
        assert!(end.x.is_finite() && end.y.is_finite() && end.z.is_finite());
        world_destroy(world);
    }

    #[test]
    fn articulation_runtime_retarget_moves_joint() {
        let world = make_world();
        let id = create_arm(world, Some(&[0.0, 0.0, 0.0]));
        assert_ne!(id, SENTINEL);
        for _ in 0..60 {
            world_step(world, 1.0 / 60.0);
        }

        // Retarget the first joint to 90° at runtime.
        assert_eq!(
            articulation_body_set_joint_target(world, id, 0, std::f64::consts::FRAC_PI_2),
            Bool::TRUE
        );
        for _ in 0..600 {
            world_step(world, 1.0 / 60.0);
        }
        // Link 1 must have swung away from the base line.
        let h1 = articulation_body_link_handle(world, id, 1);
        let p1 = rigid_body_get_translation(world, h1);
        assert!(
            (p1.y.abs() > 0.15) || (p1.x < 0.45),
            "joint 1 should bend link 1: {p1:?}"
        );
        world_destroy(world);
    }

    #[test]
    fn articulation_ffi_rejects_bad_params() {
        // Null world.
        assert_eq!(
            articulation_body_create(
                std::ptr::null_mut(),
                v(0.0, 0.0, 0.0),
                v(1.0, 0.0, 0.0),
                v(0.0, 0.0, 1.0),
                3,
                0.3,
                1.0,
                std::ptr::null(),
                0,
                100.0,
                10.0,
            ),
            SENTINEL
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = make_world();
        let cases: [(u32, Vec3, Vec3, f64, f64, &str); 5] = [
            (
                1,
                v(1.0, 0.0, 0.0),
                v(0.0, 0.0, 1.0),
                0.3,
                1.0,
                "link_count < 2",
            ),
            (
                257,
                v(1.0, 0.0, 0.0),
                v(0.0, 0.0, 1.0),
                0.3,
                1.0,
                "link_count > 256",
            ),
            (
                3,
                v(1.0, 0.0, 0.0),
                v(1.0, 0.0, 0.0),
                0.3,
                1.0,
                "axis ∥ dir",
            ),
            (
                3,
                v(1.0, 0.0, 0.0),
                v(0.0, 0.0, 1.0),
                0.0,
                1.0,
                "zero radius",
            ),
            (3, v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0), 0.3, 0.0, "zero mass"),
        ];
        for (count, dir, axis, radius, mass, label) in cases {
            let id = articulation_body_create(
                world,
                v(0.0, 5.0, 0.0),
                dir,
                axis,
                count,
                radius,
                mass,
                std::ptr::null(),
                0,
                100.0,
                10.0,
            );
            assert_eq!(id, SENTINEL, "case: {label}");
            assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT, "case: {label}");
        }
        // Non-finite target.
        let nan = [f64::NAN; 2];
        assert_eq!(
            articulation_body_create(
                world,
                v(0.0, 5.0, 0.0),
                v(1.0, 0.0, 0.0),
                v(0.0, 0.0, 1.0),
                3,
                0.3,
                1.0,
                nan.as_ptr(),
                2,
                100.0,
                10.0,
            ),
            SENTINEL,
            "nan target"
        );
        // Unknown ids on queries/retarget.
        assert_eq!(articulation_body_link_count(world, 77), SENTINEL);
        assert_eq!(articulation_body_link_handle(world, 77, 0), 0);
        assert_eq!(
            articulation_body_set_joint_target(world, 77, 0, 0.5),
            Bool::FALSE
        );
        world_destroy(world);
    }
}
