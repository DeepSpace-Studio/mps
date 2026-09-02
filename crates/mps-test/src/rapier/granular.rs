#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, ERR_OK, last_error_code,
    };
    use mps_core::rapier::ffi::{Bool, Vec3, WorldHandle};
    use mps_core::rapier::granular::{
        granular_add_particle, granular_create, granular_particle_count, granular_read_particles,
        granular_step,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    const SENTINEL: u32 = u32::MAX;

    fn make_world() -> *mut WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        })
    }

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    #[test]
    fn granular_create_and_step_via_world_step() {
        let world = make_world();
        let id = granular_create(world, v(0.0, -9.81, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(id, SENTINEL);
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(granular_particle_count(world, id), 0);

        // Two overlapping grains: the world_step tick must integrate them
        // apart (spring) and downward (gravity), staying finite/bounded.
        let p0 = granular_add_particle(world, id, -0.03, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.05);
        let p1 = granular_add_particle(world, id, 0.03, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.05);
        assert_eq!(p0, 0);
        assert_eq!(p1, 1);
        assert_eq!(granular_particle_count(world, id), 2);

        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }
        let mut pos = vec![Vec3::default(); 2];
        let mut vel = vec![Vec3::default(); 2];
        let n = granular_read_particles(world, id, pos.as_mut_ptr(), vel.as_mut_ptr(), 2);
        assert_eq!(n, 2);
        for p in &pos {
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
            assert!(p.y < 1.0, "grains must fall under gravity: y {}", p.y);
            assert!(p.y > -30.0);
        }
        // Overlap resolved: separation grows beyond the initial 0.06.
        let sep = ((pos[1].x - pos[0].x).powi(2)
            + (pos[1].y - pos[0].y).powi(2)
            + (pos[1].z - pos[0].z).powi(2))
        .sqrt();
        assert!(sep > 0.06, "overlapping grains must repel: sep {sep}");
        world_destroy(world);
    }

    #[test]
    fn granular_manual_step_hook_works() {
        let world = make_world();
        // Zero gravity so the manual tick is analytically checkable.
        let id = granular_create(world, v(0.0, 0.0, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(id, SENTINEL);
        granular_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.05);
        for _ in 0..60 {
            assert_eq!(granular_step(world, id, 1.0 / 60.0), Bool::TRUE);
        }
        let mut pos = vec![Vec3::default(); 1];
        granular_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), 1);
        assert!((pos[0].x - 1.0).abs() < 1e-9, "x {} != 1.0", pos[0].x);
        world_destroy(world);
    }

    #[test]
    fn granular_read_handles_short_buffer_and_null_channels() {
        let world = make_world();
        let id = granular_create(world, v(0.0, 0.0, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(id, SENTINEL);
        for i in 0..3u32 {
            granular_add_particle(world, id, i as f64, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.05);
        }
        // Short buffer: returns the real count, writes only capacity items.
        let mut pos = vec![Vec3::default(); 2];
        let n = granular_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), 2);
        assert_eq!(n, 3);
        assert_eq!((pos[0].x, pos[0].y, pos[0].z), (0.0, 0.0, 0.0));
        assert_eq!((pos[1].x, pos[1].y, pos[1].z), (1.0, 0.0, 0.0));
        // Velocity channel only.
        let mut vel = vec![Vec3::default(); 3];
        let n2 = granular_read_particles(world, id, std::ptr::null_mut(), vel.as_mut_ptr(), 3);
        assert_eq!(n2, 3);
        for x in &vel {
            assert_eq!((x.x, x.y, x.z), (0.0, 0.0, 0.0));
        }
        world_destroy(world);
    }

    #[test]
    fn granular_ffi_rejects_bad_params() {
        // Null world.
        assert_eq!(
            granular_create(
                std::ptr::null_mut(),
                v(0.0, 0.0, 0.0),
                0.05,
                800.0,
                0.5,
                0.6,
                0.4
            ),
            SENTINEL
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = make_world();
        // Bad scalar params.
        assert_eq!(
            granular_create(world, v(0.0, 0.0, 0.0), 0.0, 800.0, 0.5, 0.6, 0.4),
            SENTINEL,
            "zero radius"
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(
            granular_create(world, v(0.0, 0.0, 0.0), 0.05, -1.0, 0.5, 0.6, 0.4),
            SENTINEL,
            "negative stiffness"
        );
        assert_eq!(
            granular_create(world, v(0.0, 0.0, 0.0), 0.05, 800.0, 0.5, 0.6, 1.5),
            SENTINEL,
            "tangential damping out of range"
        );

        let id = granular_create(world, v(0.0, 0.0, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(id, SENTINEL);
        // Bad particle params.
        assert_eq!(
            granular_add_particle(world, id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.05),
            SENTINEL,
            "zero mass"
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // Unknown ids.
        assert_eq!(granular_particle_count(world, 999), SENTINEL);
        assert_eq!(granular_step(world, 999, 1.0 / 60.0), Bool::FALSE);
        // Bad dt.
        assert_eq!(granular_step(world, id, 0.0), Bool::FALSE);
        world_destroy(world);
    }
}

#[cfg(test)]
mod dig_link_tests {
    use mps_core::rapier::collider::{collider_builder_build, world_insert_collider};
    use mps_core::rapier::error::{ERR_OK, last_error_code};
    use mps_core::rapier::ffi::{Bool, Vec3, VoxelColliderMode, VoxelColliderOptions};
    use mps_core::rapier::granular::{
        granular_create, granular_get_voxel_dig_link, granular_link_voxel_dig,
        granular_particle_count, granular_read_particles,
    };
    use mps_core::rapier::voxel::collider_builder_create_voxels;
    use mps_core::rapier::voxel::collider_voxel_edit;
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    #[test]
    fn voxel_dig_spawns_grain_into_linked_body() {
        let world = world_create(v(0.0, -9.81, 0.0));
        assert!(!world.is_null());

        // 2×1×1 all-solid voxel collider occupying [0,1.0]×[0,0.5]×[0,0.5]
        // (cell size 0.5 along x).
        let sx = 2u32;
        let voxels = vec![1u8; sx as usize];
        let cb = collider_builder_create_voxels(
            voxels.as_ptr(),
            sx,
            1,
            1,
            0.5,
            0.5,
            0.5,
            v(0.0, 0.0, 0.0),
            VoxelColliderOptions {
                mode: VoxelColliderMode::Cuboids as u32,
                dynamic_body: Bool::FALSE,
                small_voxel_limit: 128,
                mesh_voxel_limit: 20_000,
            },
        );
        assert!(!cb.is_null());
        let handle = world_insert_collider(world, collider_builder_build(cb));
        assert_ne!(handle, 0u64);

        // One granular body, linked to dig spawns.
        let gid = granular_create(world, v(0.0, -9.81, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(gid, u32::MAX);
        assert_eq!(granular_link_voxel_dig(world, gid, 1.0, 0.05), Bool::TRUE);
        assert_eq!(last_error_code(), ERR_OK);
        let mut linked_body = 0u32;
        assert_eq!(
            granular_get_voxel_dig_link(
                world,
                &mut linked_body,
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            Bool::TRUE
        );
        assert_eq!(linked_body, gid);

        // Dig cell (0,0,0) — centre at (0.25, 0.25, 0.25) — must spawn a grain.
        assert_eq!(collider_voxel_edit(world, handle, 0, 0, 0, 0), Bool::TRUE);
        assert_eq!(granular_particle_count(world, gid), 1);
        let mut pos = vec![Vec3::default(); 1];
        let n = granular_read_particles(world, gid, pos.as_mut_ptr(), std::ptr::null_mut(), 1);
        assert_eq!(n, 1);
        assert_eq!((pos[0].x, pos[0].y, pos[0].z), (0.25, 0.25, 0.25));

        // The grain then falls/rolls inside the sim like any other particle.
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        let mut pos2 = vec![Vec3::default(); 1];
        granular_read_particles(world, gid, pos2.as_mut_ptr(), std::ptr::null_mut(), 1);
        assert!(pos2[0].y.is_finite());

        // Digging an already-empty cell spawns nothing (changed=false).
        assert_eq!(collider_voxel_edit(world, handle, 0, 0, 0, 0), Bool::TRUE);
        assert_eq!(granular_particle_count(world, gid), 1);

        // Unlink → further digs spawn nothing.
        assert_eq!(
            granular_link_voxel_dig(world, u32::MAX, 0.0, 0.0),
            Bool::TRUE
        );
        assert_eq!(
            granular_get_voxel_dig_link(
                world,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            Bool::FALSE
        );
        assert_eq!(collider_voxel_edit(world, handle, 1, 0, 0, 0), Bool::TRUE);
        assert_eq!(granular_particle_count(world, gid), 1);
        world_destroy(world);
    }

    #[test]
    fn granular_link_rejects_unknown_body_and_bad_grain() {
        let world = world_create(v(0.0, 0.0, 0.0));
        assert_eq!(granular_link_voxel_dig(world, 7, 1.0, 0.05), Bool::FALSE);
        let gid = granular_create(world, v(0.0, 0.0, 0.0), 0.05, 800.0, 0.5, 0.6, 0.4);
        assert_ne!(gid, u32::MAX);
        assert_eq!(granular_link_voxel_dig(world, gid, 0.0, 0.05), Bool::FALSE);
        assert_eq!(granular_link_voxel_dig(world, gid, 1.0, 0.0), Bool::FALSE);
        // Still unlinked after the failures.
        assert_eq!(
            granular_get_voxel_dig_link(
                world,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            Bool::FALSE
        );
        world_destroy(world);
    }
}
