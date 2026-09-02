//! End-to-end tests for hair / fur systems (soft-body strands attached to
//! rigid bodies).

#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_ex, world_insert_collider_with_parent,
    };
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
        ERR_UNSUPPORTED, last_error_code,
    };
    use mps_core::rapier::ffi::{BodyStatus, Bool, ShapeDesc, ShapeType, Vec3, WorldHandle};
    use mps_core::rapier::hair::{
        HairStrandDesc, hair_system_build, hair_system_create, hair_system_remove,
        hair_system_set_gravity_scale, hair_system_set_wind, hair_system_strand_soft_body,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_set_translation,
        world_insert_rigid_body,
    };
    use mps_core::rapier::soft_body::soft_body_get_particle;
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn make_world() -> *mut WorldHandle {
        world_create(v3(0.0, -9.81, 0.0))
    }

    /// A dynamic "head" body the hair hangs from (a collider gives it mass so
    /// the routed spring forces integrate cleanly), dropped high in the air.
    fn make_head(world: *mut WorldHandle, y: f64) -> u64 {
        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_translation(builder, v3(0.0, y, 0.0));
        let body = world_insert_rigid_body(world, rigid_body_builder_build(builder));
        let shape = ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 0.5,
            b: 0.5,
            c: 0.5,
            ..Default::default()
        };
        let collider = collider_builder_build(collider_builder_create_ex(shape));
        world_insert_collider_with_parent(world, collider, body);
        body
    }

    fn strand() -> HairStrandDesc {
        HairStrandDesc {
            root_local: v3(0.0, 0.25, 0.0),
            direction: v3(0.0, -1.0, 0.0),
            segment_count: 4,
            length: 1.0,
            segment_radius: 0.02,
            stiffness: 100.0,
            damping: 0.1,
            density: 1200.0,
        }
    }

    #[test]
    fn hair_system_lifecycle_works() {
        let world = make_world();
        let body = make_head(world, 10.0);

        let strands = [strand()];
        let id = hair_system_create(world, body, strands.as_ptr(), 1);
        assert_ne!(id, u32::MAX);
        assert_eq!(last_error_code(), ERR_OK);

        // Build creates the strand soft bodies.
        assert_eq!(hair_system_build(world, id), Bool::TRUE);
        assert_ne!(hair_system_strand_soft_body(world, id, 0), u32::MAX);

        // Wind + gravity scale update cleanly.
        assert_eq!(
            hair_system_set_wind(world, id, v3(1.0, 0.0, 0.0)),
            Bool::TRUE
        );
        assert_eq!(hair_system_set_gravity_scale(world, id, 0.0), Bool::TRUE);

        // Step the world with active strands.
        for _ in 0..10 {
            world_step(world, 1.0 / 60.0);
        }

        // Double build is rejected; removal cleans up.
        assert_eq!(hair_system_build(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_UNSUPPORTED);
        assert_eq!(hair_system_remove(world, id), Bool::TRUE);
        assert_eq!(hair_system_strand_soft_body(world, id, 0), u32::MAX);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn hair_system_strand_follows_the_body() {
        let world = make_world();
        let body = make_head(world, 10.0);

        let strands = [strand()];
        let id = hair_system_create(world, body, strands.as_ptr(), 1);
        assert_ne!(id, u32::MAX);
        assert_eq!(hair_system_build(world, id), Bool::TRUE);
        let soft_id = hair_system_strand_soft_body(world, id, 0);
        assert_ne!(soft_id, u32::MAX);

        // Free fall for half a second: the bound root particle must track the
        // falling body downward.
        let mut pos = Vec3::default();
        let mut vel = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, soft_id, 0, &mut pos, &mut vel),
            Bool::TRUE
        );
        let root_start_y = pos.y;
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        assert_eq!(
            soft_body_get_particle(world, soft_id, 0, &mut pos, &mut vel),
            Bool::TRUE
        );
        assert!(
            pos.y < root_start_y - 0.5,
            "root particle must follow the falling body (start {root_start_y}, now {})",
            pos.y
        );
        world_destroy(world);
    }

    #[test]
    fn hair_system_rejects_invalid_input() {
        let world = make_world();
        let body = make_head(world, 0.0);

        // Null strand pointer.
        assert_eq!(
            hair_system_create(world, body, std::ptr::null(), 1),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Strand count out of range.
        let strands = [strand()];
        assert_eq!(
            hair_system_create(world, body, strands.as_ptr(), 0),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Unknown attached body.
        assert_eq!(
            hair_system_create(world, u64::MAX, strands.as_ptr(), 1),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);

        // Bad strand descriptor (zero segments).
        let mut bad = strand();
        bad.segment_count = 0;
        let bad_strands = [bad];
        assert_eq!(
            hair_system_create(world, body, bad_strands.as_ptr(), 1),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Negative gravity scale.
        let id = hair_system_create(world, body, strands.as_ptr(), 1);
        assert_ne!(id, u32::MAX);
        assert_eq!(hair_system_set_gravity_scale(world, id, -0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Non-finite wind.
        assert_eq!(
            hair_system_set_wind(world, id, v3(f64::NAN, 0.0, 0.0)),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Build on an unknown id.
        assert_eq!(hair_system_build(world, 987654), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);

        // Remove on an unknown id.
        assert_eq!(hair_system_remove(world, 987654), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn hair_system_strand_index_bounds() {
        let world = make_world();
        let body = make_head(world, 0.0);
        let strands = [strand()];
        let id = hair_system_create(world, body, strands.as_ptr(), 1);
        assert_ne!(id, u32::MAX);

        // Before build: no strands exist yet.
        assert_eq!(hair_system_strand_soft_body(world, id, 0), u32::MAX);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(hair_system_build(world, id), Bool::TRUE);
        assert_ne!(hair_system_strand_soft_body(world, id, 0), u32::MAX);
        assert_eq!(hair_system_strand_soft_body(world, id, 1), u32::MAX);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn hair_system_null_world_is_rejected() {
        let strands = [strand()];
        assert_eq!(
            hair_system_create(std::ptr::null_mut(), 0u64, strands.as_ptr(), 1),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(hair_system_build(std::ptr::null_mut(), 0), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }
}
