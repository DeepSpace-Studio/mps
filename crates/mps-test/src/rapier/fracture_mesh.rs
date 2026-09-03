//! End-to-end tests for fracture mesh bodies (fracturable composite rigid
//! bodies built on top of `fracture.rs`).

#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
        ERR_UNSUPPORTED, last_error_code,
    };
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::fracture_mesh::{
        fracture_mesh_body_add_fatigue_damage, fracture_mesh_body_create,
        fracture_mesh_body_create_with_voronoi, fracture_mesh_body_is_fractured,
        fracture_mesh_body_remove, fracture_mesh_body_set_stress, fracture_mesh_body_set_trigger,
        fracture_mesh_body_set_trigger_stress, fracture_mesh_body_trigger,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn make_world() -> *mut WorldHandle {
        world_create(v3(0.0, -9.81, 0.0))
    }

    fn valid_material() -> FractureMaterial {
        FractureMaterial {
            youngs_modulus: 200.0e9,
            poisson_ratio: 0.3,
            fracture_toughness: 50.0e6,
            surface_energy: 10.0,
            density: 7850.0,
        }
    }

    fn valid_fragments() -> [FractureFragmentDesc; 2] {
        [
            FractureFragmentDesc {
                local_center: v3(-0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(0.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
            FractureFragmentDesc {
                local_center: v3(0.5, 0.0, 0.0),
                half_extents: v3(0.25, 0.5, 0.5),
                initial_velocity: v3(0.0, 0.0, 0.0),
                density: 1.0,
                friction: 0.5,
                restitution: 0.1,
            },
        ]
    }

    fn cuboid_shape() -> ShapeDesc {
        ShapeDesc {
            shape_type: ShapeType::Cuboid as u32,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            ..Default::default()
        }
    }

    fn make_mesh(world: *mut WorldHandle) -> u32 {
        let frags = valid_fragments();
        fracture_mesh_body_create(
            world,
            cuboid_shape(),
            v3(0.0, 10.0, 0.0),
            frags.as_ptr(),
            frags.len() as u32,
            valid_material(),
            Bool::TRUE,
        )
    }

    #[test]
    fn fracture_mesh_create_reports_intact_state() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_OK);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_create_rejects_bad_input() {
        let world = make_world();
        let frags = valid_fragments();

        // No fragments / too many fragments.
        assert_eq!(
            fracture_mesh_body_create(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                frags.as_ptr(),
                0,
                valid_material(),
                Bool::TRUE
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Null fragment pointer.
        assert_eq!(
            fracture_mesh_body_create(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                std::ptr::null(),
                1,
                valid_material(),
                Bool::TRUE
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Invalid material (poisson ratio out of range).
        let mut bad_material = valid_material();
        bad_material.poisson_ratio = 0.9;
        assert_eq!(
            fracture_mesh_body_create(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                frags.as_ptr(),
                frags.len() as u32,
                bad_material,
                Bool::TRUE
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Invalid fragment descriptor (zero half extent).
        let mut bad_frags = valid_fragments();
        bad_frags[1].half_extents = v3(0.25, 0.0, 0.5);
        assert_eq!(
            fracture_mesh_body_create(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                bad_frags.as_ptr(),
                bad_frags.len() as u32,
                valid_material(),
                Bool::TRUE
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_trigger_fractures_once() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);

        assert_eq!(fracture_mesh_body_trigger(world, id), Bool::TRUE);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::TRUE);

        // The fractured body keeps its id; a second trigger is rejected.
        assert_eq!(fracture_mesh_body_trigger(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_UNSUPPORTED);

        // Fragments are dynamic bodies — the world must step cleanly.
        for _ in 0..10 {
            world_step(world, 1.0 / 60.0);
        }
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_stress_trigger_auto_fractures() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            fracture_mesh_body_set_trigger(world, id, 1, 10.0),
            Bool::TRUE
        );
        // Below threshold: no fracture.
        assert_eq!(fracture_mesh_body_set_stress(world, id, 5.0), Bool::TRUE);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::FALSE);
        // Above threshold: auto-fracture.
        assert_eq!(fracture_mesh_body_set_stress(world, id, 12.0), Bool::TRUE);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::TRUE);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_set_trigger_stress_wraps_mode_1() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            fracture_mesh_body_set_trigger_stress(world, id, 3.0),
            Bool::TRUE
        );
        assert_eq!(fracture_mesh_body_set_stress(world, id, 3.0), Bool::TRUE);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::TRUE);

        // Invalid threshold is rejected.
        let id2 = make_mesh(world);
        assert_ne!(id2, u32::MAX);
        assert_eq!(
            fracture_mesh_body_set_trigger_stress(world, id2, 0.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_fatigue_trigger_auto_fractures() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);

        assert_eq!(
            fracture_mesh_body_set_trigger(world, id, 3, 0.0),
            Bool::TRUE
        );
        assert_eq!(
            fracture_mesh_body_add_fatigue_damage(world, id, 0.4),
            Bool::TRUE
        );
        assert_eq!(
            fracture_mesh_body_add_fatigue_damage(world, id, 0.4),
            Bool::TRUE
        );
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::FALSE);
        // Reaches 1.0 → auto-fracture.
        assert_eq!(
            fracture_mesh_body_add_fatigue_damage(world, id, 0.2),
            Bool::TRUE
        );
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::TRUE);

        // Negative damage is rejected.
        let id2 = make_mesh(world);
        assert_ne!(id2, u32::MAX);
        assert_eq!(
            fracture_mesh_body_add_fatigue_damage(world, id2, -1.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_set_trigger_rejects_bad_mode() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);
        assert_eq!(
            fracture_mesh_body_set_trigger(world, id, 7, 1.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_remove_drops_the_body() {
        let world = make_world();
        let id = make_mesh(world);
        assert_ne!(id, u32::MAX);

        assert_eq!(fracture_mesh_body_remove(world, id), Bool::TRUE);
        assert_eq!(fracture_mesh_body_remove(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        // Unknown ids report not-found.
        assert_eq!(fracture_mesh_body_is_fractured(world, 987654), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn fracture_mesh_null_world_is_rejected() {
        assert_eq!(
            fracture_mesh_body_create(
                std::ptr::null_mut(),
                cuboid_shape(),
                v3(0.0, 0.0, 0.0),
                std::ptr::null(),
                1,
                valid_material(),
                Bool::TRUE
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            fracture_mesh_body_trigger(std::ptr::null_mut(), 0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn create_with_voronoi_and_trigger_fractures_into_seeds() {
        let world = make_world();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];
        let id = fracture_mesh_body_create_with_voronoi(
            world,
            cuboid_shape(),
            v3(0.0, 10.0, 0.0),
            v3(-1.0, -1.0, -1.0),
            v3(1.0, 1.0, 1.0),
            seeds.as_ptr(),
            seeds.len() as u32,
            valid_material(),
            Bool::FALSE,
            0.0,
        );
        assert_ne!(id, u32::MAX);
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::FALSE);

        // Triggering replaces the source with one fragment per seed cell.
        assert_eq!(fracture_mesh_body_trigger(world, id), Bool::TRUE);
        assert_eq!(fracture_mesh_body_is_fractured(world, id), Bool::TRUE);
        // One-shot semantics still hold for generated fragment sets.
        assert_eq!(fracture_mesh_body_trigger(world, id), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_UNSUPPORTED);

        assert_eq!(fracture_mesh_body_remove(world, id), Bool::TRUE);
        world_destroy(world);
    }

    #[test]
    fn create_with_voronoi_rejects_bad_input() {
        let world = make_world();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];

        // Null seed pointer.
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                v3(-1.0, -1.0, -1.0),
                v3(1.0, 1.0, 1.0),
                std::ptr::null(),
                2,
                valid_material(),
                Bool::FALSE,
                0.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Zero seed count.
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                v3(-1.0, -1.0, -1.0),
                v3(1.0, 1.0, 1.0),
                seeds.as_ptr(),
                0,
                valid_material(),
                Bool::FALSE,
                0.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Inverted AABB.
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                v3(1.0, 1.0, 1.0),
                v3(-1.0, -1.0, -1.0),
                seeds.as_ptr(),
                seeds.len() as u32,
                valid_material(),
                Bool::FALSE,
                0.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Out-of-range shrink.
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                v3(-1.0, -1.0, -1.0),
                v3(1.0, 1.0, 1.0),
                seeds.as_ptr(),
                seeds.len() as u32,
                valid_material(),
                Bool::FALSE,
                0.6
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Invalid material.
        let mut bad_material = valid_material();
        bad_material.poisson_ratio = 0.9;
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                world,
                cuboid_shape(),
                v3(0.0, 10.0, 0.0),
                v3(-1.0, -1.0, -1.0),
                v3(1.0, 1.0, 1.0),
                seeds.as_ptr(),
                seeds.len() as u32,
                bad_material,
                Bool::FALSE,
                0.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Null world.
        assert_eq!(
            fracture_mesh_body_create_with_voronoi(
                std::ptr::null_mut(),
                cuboid_shape(),
                v3(0.0, 0.0, 0.0),
                v3(-1.0, -1.0, -1.0),
                v3(1.0, 1.0, 1.0),
                seeds.as_ptr(),
                seeds.len() as u32,
                valid_material(),
                Bool::FALSE,
                0.0
            ),
            u32::MAX
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        world_destroy(world);
    }
}
