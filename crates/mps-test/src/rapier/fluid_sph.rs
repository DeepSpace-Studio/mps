#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_halfspace,
        world_insert_collider_with_parent,
    };
    use mps_core::rapier::ffi::{BodyStatus, Bool, Vec3, WorldHandle};
    use mps_core::rapier::fluid_sph::{
        fluid_add_particle, fluid_create, fluid_enable_collision, fluid_get_particle,
        fluid_particle_count, fluid_step,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    #[test]
    fn fluid_create_and_add_particle() {
        let world: *mut WorldHandle = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        assert!(!world.is_null());
        let id = fluid_create(
            world, 0.0, -9.81, 0.0,    // gravity
            1.0,    // smoothing_radius
            100.0,  // gas_constant
            1000.0, // rest_density
            0.1,    // viscosity
            0.0,    // surface_tension
        );
        assert_ne!(id, u32::MAX, "fluid_create should succeed");
        // Empty fluid has zero particles.
        assert_eq!(fluid_particle_count(world, id), 0);
        // Add three particles.
        for k in 0..3 {
            let p = fluid_add_particle(world, id, k as f64, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
            assert_eq!(p, k, "particle index should be sequential");
        }
        assert_eq!(fluid_particle_count(world, id), 3);
        world_destroy(world);
    }

    #[test]
    fn fluid_single_particle_free_fall() {
        let world: *mut WorldHandle = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        assert!(!world.is_null());
        let id = fluid_create(world, 0.0, -9.81, 0.0, 1.0, 100.0, 1000.0, 0.0, 0.0);
        assert_ne!(id, u32::MAX);
        fluid_add_particle(world, id, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        // Step 60 × (1/60) = 1.0 s.
        for _ in 0..60 {
            fluid_step(world, id, 1.0 / 60.0);
        }
        let mut pos = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let ok = fluid_get_particle(
            world,
            id,
            0,
            &mut pos,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_eq!(ok, Bool::TRUE);
        // Semi-implicit Euler under constant gravity: y ≈ ½ g t² within O(dt).
        let expected = 0.5 * (-9.81) * 1.0_f64 * 1.0;
        assert!(
            (pos.y - expected).abs() < 0.1,
            "free-fall y={} expected≈{}",
            pos.y,
            expected
        );
        assert!(pos.y < -4.0);
        assert!(pos.x.abs() < 1e-12 && pos.z.abs() < 1e-12);
        world_destroy(world);
    }

    #[test]
    fn fluid_two_particles_repel_and_stay_symmetric() {
        let world: *mut WorldHandle = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        assert!(!world.is_null());
        // Low rest_density so two close unit-mass particles exceed it and repel.
        let id = fluid_create(world, 0.0, -9.81, 0.0, 1.0, 200.0, 2.0, 0.0, 0.0);
        assert_ne!(id, u32::MAX);
        fluid_add_particle(world, id, -0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        fluid_add_particle(world, id, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        for _ in 0..60 {
            fluid_step(world, id, 1.0 / 120.0);
        }
        let mut pa = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut pb = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        fluid_get_particle(
            world,
            id,
            0,
            &mut pa,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        fluid_get_particle(
            world,
            id,
            1,
            &mut pb,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let sep = ((pb.x - pa.x).powi(2) + (pb.y - pa.y).powi(2) + (pb.z - pa.z).powi(2)).sqrt();
        assert!(sep > 0.1, "compressed particles should repel, sep={sep}");
        // x/z centre of mass stays at origin (symmetric); y is free to fall.
        assert!(
            (((pa.x + pb.x) / 2.0).abs() < 1e-9) && (((pa.z + pb.z) / 2.0).abs() < 1e-9),
            "com x/z should stay at origin, pa={pa:?} pb={pb:?}"
        );
        world_destroy(world);
    }

    #[test]
    fn fluid_collision_stops_at_ground() {
        // Phase 2: an SPH fluid with collision coupling enabled must be blocked by
        // rigid terrain (a fixed half-space at y=0) instead of falling through it.
        let world: *mut WorldHandle = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });

        // Static ground: Fixed body + upward half-space (solid below y<0).
        let ground_builder = rigid_body_builder_create(BodyStatus::Fixed as u32);
        let ground = rigid_body_builder_build(ground_builder);
        let ground_handle = world_insert_rigid_body(world, ground);
        let ground_collider = collider_builder_build(collider_builder_create_halfspace(Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }));
        world_insert_collider_with_parent(world, ground_collider, ground_handle);

        // Fluid with a few particles dropping from y≈2 above the ground.
        let id = fluid_create(world, 0.0, -9.81, 0.0, 0.4, 200.0, 1000.0, 0.0, 0.0);
        assert_ne!(id, u32::MAX, "fluid_create should succeed");
        let positions = [
            (0.0, 2.0, 0.0),
            (0.1, 2.2, 0.0),
            (-0.1, 2.1, 0.1),
            (0.0, 1.9, -0.1),
        ];
        for (x, y, z) in positions {
            let _ = fluid_add_particle(world, id, x, y, z, 0.0, 0.0, 0.0, 1.0);
        }
        assert_eq!(
            fluid_enable_collision(world, id, 0.2, Bool::TRUE),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..180 {
            world_step(world, dt);
        }

        // Every particle must rest above the ground (y > -0.1): collision with the
        // half-space stopped it from falling through.
        let mut pos = Vec3::default();
        let mut vel = Vec3::default();
        for i in 0..positions.len() {
            assert_eq!(
                fluid_get_particle(
                    world,
                    id,
                    i as u32,
                    &mut pos as *mut Vec3,
                    &mut vel as *mut Vec3,
                    std::ptr::null_mut()
                ),
                Bool::TRUE
            );
            assert!(
                pos.y > -0.1,
                "fluid particle {i} fell through ground: y={}",
                pos.y
            );
        }
        world_destroy(world);
    }
}
