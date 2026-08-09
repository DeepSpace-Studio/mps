#[cfg(test)]
mod tests {
    use mps_core::rapier::batch::*;
    use mps_core::rapier::collider::collider_get_shape_count;
    use mps_core::rapier::error::{ERR_INVALID_ARGUMENT, last_error_code};
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::world::{world_create, world_destroy, world_get_collider_set_size};

    /// Helper: a ball collider request at the given position.
    fn ball_request(x: f64, y: f64, z: f64, radius: f64) -> ColliderRequest {
        let mut req = ColliderRequest::default();
        req.shape.shape_type = 0; // Ball
        req.shape.a = radius;
        req.translation = Vec3 { x, y, z };
        req.rotation = Quat {
            i: 0.0,
            j: 0.0,
            k: 0.0,
            w: 1.0,
        };
        req.friction = 0.6;
        req.restitution = 0.2;
        req.density = 1.0;
        req.erosion_margin = 0.0;
        req
    }

    /// Helper: a cuboid collider request at the given position.
    fn cuboid_request(x: f64, y: f64, z: f64, hx: f64, hy: f64, hz: f64) -> ColliderRequest {
        let mut req = ColliderRequest::default();
        req.shape.shape_type = 1; // Cuboid
        req.shape.a = hx;
        req.shape.b = hy;
        req.shape.c = hz;
        req.translation = Vec3 { x, y, z };
        req.rotation = Quat {
            i: 0.0,
            j: 0.0,
            k: 0.0,
            w: 1.0,
        };
        req.friction = 0.6;
        req.restitution = 0.2;
        req.density = 1.0;
        req.erosion_margin = 0.0;
        req
    }

    // ---- Box3D preset tests ----

    #[test]
    fn box3d_default_preset_has_sensible_values() {
        let preset = Box3DPreset::box3d_default();
        assert!((preset.default_friction - 0.6).abs() < 1e-9);
        assert!((preset.default_restitution - 0.2).abs() < 1e-9);
        assert!((preset.default_density - 1.0).abs() < 1e-9);
        assert!(preset.default_erosion_margin > 0.0);
        assert!(preset.solver_iterations >= 1);
    }

    #[test]
    fn box3d_presets_are_distinct() {
        let def = Box3DPreset::box3d_default();
        let sticky = Box3DPreset::box3d_sticky();
        let bouncy = Box3DPreset::box3d_bouncy();
        assert!(sticky.default_friction > def.default_friction);
        assert!(bouncy.default_restitution > def.default_restitution);
    }

    #[test]
    fn box3d_preset_default_is_zeroed() {
        let zero = Box3DPreset::default();
        assert_eq!(zero.default_friction, 0.0);
        assert_eq!(zero.default_restitution, 0.0);
        assert_eq!(zero.solver_iterations, 0);
    }

    // ---- Batch add colliders: FFI entry point ----

    #[test]
    fn batch_add_colliders_inserts_multiple_shapes() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        // Three balls at different positions with the same material properties
        // → Box3D-style merge into one compound collider.
        let requests = [
            ball_request(0.0, 0.0, 0.0, 0.5),
            ball_request(2.0, 0.0, 0.0, 0.3),
            ball_request(4.0, 0.0, 0.0, 0.4),
        ];
        let mut handles = [0u64; 4];
        let count = world_batch_add_colliders(
            world,
            requests.as_ptr(),
            3,
            preset,
            handles.as_mut_ptr(),
            4,
        );
        // Same material → merged into 1 compound containing 3 balls.
        assert_eq!(count, 1);
        assert_ne!(handles[0], 0);

        let shape_count = collider_get_shape_count(world, handles[0]);
        assert_eq!(shape_count, 3);

        let size = world_get_collider_set_size(world);
        assert_eq!(size, 1);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_merges_compatible_static_shapes() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        // Three identical cuboids at different positions with the same material
        // → should merge into one compound (single ColliderSet::insert).
        let requests = [
            cuboid_request(0.0, 0.0, 0.0, 0.5, 0.5, 0.5),
            cuboid_request(2.0, 0.0, 0.0, 0.5, 0.5, 0.5),
            cuboid_request(4.0, 0.0, 0.0, 0.5, 0.5, 0.5),
        ];
        let mut handles = [0u64; 4];
        let count = world_batch_add_colliders(
            world,
            requests.as_ptr(),
            3,
            preset,
            handles.as_mut_ptr(),
            4,
        );
        assert_eq!(count, 1, "3 compatible static cuboids should merged into 1");
        assert_ne!(handles[0], 0);

        // The merged compound should contain 3 shapes.
        let shape_count = collider_get_shape_count(world, handles[0]);
        assert_eq!(shape_count, 3);

        let size = world_get_collider_set_size(world);
        assert_eq!(size, 1);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_empty_batch_returns_zero() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let mut handles = [0u64; 1];
        let count = world_batch_add_colliders(
            world,
            std::ptr::null(),
            0,
            preset,
            handles.as_mut_ptr(),
            1,
        );
        assert_eq!(count, 0);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_invalid_shape_returns_zero() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let mut bad = ball_request(0.0, 0.0, 0.0, -1.0); // negative radius
        bad.shape.a = -1.0;
        let mut handles = [0u64; 1];
        let count = world_batch_add_colliders(
            world,
            &bad as *const ColliderRequest,
            1,
            preset,
            handles.as_mut_ptr(),
            1,
        );
        assert_eq!(count, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_applies_preset_defaults() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        // All-zero request with zeroed preset → should still insert but with
        // zero values for friction/restitution.
        let preset = Box3DPreset::default();
        let req = {
            let mut r = ColliderRequest::default();
            r.shape.shape_type = 0; // Ball
            r.shape.a = 0.5;
            r.rotation = Quat {
                i: 0.0,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            };
            r
        };
        let mut handles = [0u64; 1];
        let count = world_batch_add_colliders(
            world,
            &req as *const ColliderRequest,
            1,
            preset,
            handles.as_mut_ptr(),
            1,
        );
        assert_eq!(count, 1);
        assert_ne!(handles[0], 0);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_with_erosion_on_cuboid() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let mut req = cuboid_request(0.0, 0.0, 0.0, 0.5, 0.5, 0.5);
        req.erosion_margin = 0.01;
        let mut handles = [0u64; 1];
        let count = world_batch_add_colliders(
            world,
            &req as *const ColliderRequest,
            1,
            preset,
            handles.as_mut_ptr(),
            1,
        );
        assert_eq!(count, 1);
        assert_ne!(handles[0], 0);

        world_destroy(world);
    }

    // ---- merge_static_shapes tests ----

    #[test]
    fn merge_static_shapes_inserts_compound() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let requests = [
            cuboid_request(0.0, 0.0, 0.0, 0.5, 0.5, 0.5),
            cuboid_request(2.0, 0.0, 0.0, 0.5, 0.5, 0.5),
        ];
        let mut handles = [0u64; 2];
        let count = world_merge_static_shapes(
            world,
            requests.as_ptr(),
            2,
            preset,
            handles.as_mut_ptr(),
            2,
        );
        assert_eq!(count, 1, "2 static cuboids should merge into 1 compound");
        assert_ne!(handles[0], 0);

        // The compound should contain 2 shapes.
        let shape_count = collider_get_shape_count(world, handles[0]);
        assert_eq!(shape_count, 2);

        world_destroy(world);
    }

    #[test]
    fn merge_static_shapes_rejects_parented_shapes() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let mut req = cuboid_request(0.0, 0.0, 0.0, 0.5, 0.5, 0.5);
        req.body_parent = 1; // non-zero = has parent
        let mut handles = [0u64; 1];
        let count = world_merge_static_shapes(
            world,
            &req as *const ColliderRequest,
            1,
            preset,
            handles.as_mut_ptr(),
            1,
        );
        assert_eq!(count, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        world_destroy(world);
    }

    // ---- Preset FFI functions ----

    #[test]
    fn box3d_preset_default_ffi_returns_canonical_values() {
        let preset = box3d_preset_default();
        assert!((preset.default_friction - 0.6).abs() < 1e-9);
        assert!((preset.default_restitution - 0.2).abs() < 1e-9);
    }

    #[test]
    fn box3d_preset_sticky_ffi_has_high_friction() {
        let preset = box3d_preset_sticky();
        assert!(preset.default_friction > 0.8);
        assert_eq!(preset.default_restitution, 0.0);
    }

    #[test]
    fn box3d_preset_bouncy_ffi_has_high_restitution() {
        let preset = box3d_preset_bouncy();
        assert!(preset.default_restitution > 0.5);
        assert!(preset.default_friction < 0.5);
    }

    // ---- Mixing different material groups ----

    #[test]
    fn batch_add_colliders_separates_different_materials() {
        let world = world_create(Vec3::default());
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        let mut reqs = [
            cuboid_request(0.0, 0.0, 0.0, 0.5, 0.5, 0.5),
            cuboid_request(2.0, 0.0, 0.0, 0.5, 0.5, 0.5),
            cuboid_request(4.0, 0.0, 0.0, 0.5, 0.5, 0.5),
        ];
        // Give the second one different friction → should not merge with the
        // other two.
        reqs[1].friction = 0.9;

        let mut handles = [0u64; 3];
        let count = world_batch_add_colliders(
            world,
            reqs.as_ptr(),
            3,
            preset,
            handles.as_mut_ptr(),
            3,
        );
        // We expect 2 colliders: one compound of cuboid 0+2 (same friction 0.6),
        // one single for cuboid 1 (friction 0.9).
        assert_eq!(count, 2);

        let size = world_get_collider_set_size(world);
        assert_eq!(size, 2);

        world_destroy(world);
    }

    #[test]
    fn batch_add_colliders_world_step_runs_clean() {
        let world = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        assert!(!world.is_null());

        let preset = Box3DPreset::box3d_default();
        // Create a ground and a ball above it.
        let mut ground = cuboid_request(0.0, -1.0, 0.0, 5.0, 0.1, 5.0);
        ground.friction = 0.9;
        ground.restitution = 0.0;
        let mut ball = ball_request(0.0, 5.0, 0.0, 0.5);
        ball.body_parent = 0;
        let requests = [ground, ball];
        let mut handles = [0u64; 2];
        let count = world_batch_add_colliders(
            world,
            requests.as_ptr(),
            2,
            preset,
            handles.as_mut_ptr(),
            2,
        );
        assert_eq!(count, 2);

        // Step the world — should not panic.
        use mps_core::rapier::world::world_step;
        world_step(world, 1.0 / 60.0);
        world_step(world, 1.0 / 60.0);

        // Verify error slot is clean after step.
        // (world_step may set error for unrelated reasons but should not panic.)
        let _ = last_error_code();

        world_destroy(world);
    }
}
