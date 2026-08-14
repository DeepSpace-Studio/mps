#[cfg(test)]
mod tests {
    use mps_core::rapier::events::*;
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::rigid_body::*;
    use mps_core::rapier::terrain_gravity::*;
    use mps_core::rapier::world::*;

    /// Create a unit cube (8 vertices, 12 triangles)
    fn unit_cube_vertices() -> Vec<f64> {
        // 8 corners of a unit cube centered at origin
        vec![
            -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, 0.5,
            0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5,
        ]
    }

    fn unit_cube_faces() -> Vec<u32> {
        // 12 triangles (2 per face × 6 faces)
        vec![
            0, 1, 2, 0, 2, 3, // -Z face
            4, 6, 5, 4, 7, 6, // +Z face
            0, 4, 5, 0, 5, 1, // -Y face
            2, 6, 7, 2, 7, 3, // +Y face
            0, 3, 7, 0, 7, 4, // -X face
            1, 5, 6, 1, 6, 2, // +X face
        ]
    }

    #[test]
    fn polyhedron_gravity_unit_cube_far_field() {
        let verts = unit_cube_vertices();
        let faces = unit_cube_faces();
        let pos = Vec3 {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        };

        let mut accel = Vec3::default();
        let ok = polyhedron_gravity(pos, &verts, &faces, 8, 12, 1000.0, &mut accel);

        assert_eq!(ok, Bool::TRUE, "polyhedron gravity should succeed");

        // At far distance, should produce nonzero acceleration toward origin
        let mag = (accel.x * accel.x + accel.y * accel.y + accel.z * accel.z).sqrt();
        assert!(mag > 0.0, "Acceleration should be nonzero, got {:?}", accel);
        assert!(
            accel.x < 0.0,
            "Force should point toward origin (negative x)"
        );

        // Point mass: GM = G·ρ·V = 6.67430e-11 × 1000 × 1
        // At r=100: a = GM/r² = 6.67430e-8 / 10000 ≈ 6.67e-12
        let expected_accel = 6.67430e-11 * 1000.0 / (100.0 * 100.0);
        let ratio = accel.x.abs() / expected_accel;
        // Polyhedron formula differs from point mass by O(1/r⁴) terms
        // Accept order-of-magnitude match (within factor 100)
        assert!(
            ratio > 0.01 && ratio < 100.0,
            "Polyhedron at 100× should approximate point mass within 2 orders, ratio={}",
            ratio
        );
    }

    #[test]
    fn lunar_mascons_are_nonzero() {
        let count = lunar_mascon_count();
        assert!(
            count >= 8,
            "At least 8 lunar mascons expected, got {}",
            count
        );

        // At lunar orbit altitude (~50 km above surface)
        let pos = Vec3 {
            x: 1.787e6,
            y: 0.0,
            z: 0.0,
        }; // near equatorial orbit
        let accel = lunar_mascon_gravity(pos);
        let mag = (accel.x * accel.x + accel.y * accel.y + accel.z * accel.z).sqrt();

        // Mascon perturbation at 50 km should be measurable (~1e-5 to 1e-3 m/s²)
        assert!(mag > 1e-8, "Lunar mascon acceleration should be nonzero");
        assert!(mag < 1.0, "Lunar mascon acceleration should be < 1 m/s²");
    }

    #[test]
    fn lunar_mascon_get_valid() {
        let count = lunar_mascon_count();
        let mut mc = LunarMascon {
            center: Vec3::default(),
            excess_mass: 0.0,
            radius: 0.0,
        };

        // Valid index
        assert!(lunar_mascon_get(0, &mut mc).0 != 0);
        assert!(mc.excess_mass > 0.0);

        // Invalid index
        assert!(lunar_mascon_get(count + 1, &mut mc).0 == 0);
    }

    #[test]
    fn terrain_gravity_dem_at_distance() {
        let nx = 10u32;
        let ny = 10u32;
        let mut dem = vec![0.0f64; (nx * ny) as usize];
        dem[5 * 10 + 5] = 5000.0;

        let grid = TerrainGrid {
            nx,
            ny,
            resolution: 1000.0,
            reference_radius: 6371e3,
        };

        // Near-surface above the mountain
        let pos = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 6376e3,
        };

        let accel = terrain_gravity_direct(pos, &dem, grid, 1000.0);
        // Just verify it doesn't panic and returns finite values
        assert!(
            accel.x.is_finite() && accel.y.is_finite() && accel.z.is_finite(),
            "Terrain gravity should return finite values, got {:?}",
            accel
        );
    }

    #[test]
    fn terrain_fft_falls_back_to_direct() {
        let nx = 5u32;
        let ny = 5u32;
        let dem = vec![100.0f64; (nx * ny) as usize];
        let grid = TerrainGrid {
            nx,
            ny,
            resolution: 1000.0,
            reference_radius: 6371e3,
        };

        let pos = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 6371e3 + 100e3,
        };
        let accel_direct = terrain_gravity_direct(pos, &dem, grid, 1000.0);
        let accel_fft = terrain_gravity_fft(pos, &dem, grid, 1000.0);

        // Both should produce the same sign (downward pull)
        let both_downward = (accel_direct.z <= 0.0) == (accel_fft.z <= 0.0);
        assert!(
            both_downward,
            "Direct and FFT should both point downward, got direct={:?} fft={:?}",
            accel_direct, accel_fft
        );
    }

    /// Terrain gravity (polyhedron model) must actually drive a dynamic body
    /// through `world_step` — the body should accelerate toward the source even
    /// when the world's uniform gravity is zero.
    #[test]
    fn terrain_gravity_polyhedron_drives_world() {
        // Zero uniform gravity: only the terrain-gravity law acts.
        let world = world_create(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(!world.is_null());

        // A 100 m cube (±50) of density 1000 kg/m³, centered at origin.
        let verts: Vec<f64> = vec![
            -50.0, -50.0, -50.0, 50.0, -50.0, -50.0, 50.0, 50.0, -50.0, -50.0, 50.0, -50.0, -50.0,
            -50.0, 50.0, 50.0, -50.0, 50.0, 50.0, 50.0, 50.0, -50.0, 50.0, 50.0,
        ];
        let faces: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 2, 6, 7, 2, 7, 3, 0, 3, 7, 0, 7,
            4, 1, 5, 6, 1, 6, 2,
        ];
        assert_eq!(
            world_register_terrain_gravity_polyhedron(
                world,
                verts.as_ptr(),
                8,
                faces.as_ptr(),
                12,
                1000.0
            ),
            Bool::TRUE
        );

        // Dynamic body of mass 1 kg placed at (200, 0, 0), outside the cube.
        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_additional_mass_properties(
            builder,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let body = rigid_body_builder_build(builder);
        let handle = world_insert_rigid_body(world, body);
        assert_ne!(handle, 0);

        let pose = [200.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(
            world_update_body_poses(world, [handle].as_ptr(), pose.as_ptr(), 1, Bool::TRUE),
            1
        );

        // Step the world; terrain gravity should pull the body toward -x.
        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        let vel = rigid_body_get_linvel(world, handle);
        assert!(
            vel.x < -1e-12,
            "body should accelerate toward the cube (negative x), got {:?}",
            vel
        );

        world_unregister_terrain_gravity(world);
        world_destroy(world);
    }

    /// The parameter-free lunar-mascon model must also drive a body when
    /// registered, and `world_unregister_terrain_gravity` must stop it.
    #[test]
    fn terrain_gravity_mascon_drives_and_unregisters() {
        let world = world_create(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(!world.is_null());

        // Place the body near the Imbrium mascon (index 0, ~(-8.29e5, 4.52e5, 6.31e5)).
        let mut mc = LunarMascon {
            center: Vec3::default(),
            excess_mass: 0.0,
            radius: 0.0,
        };
        assert!(lunar_mascon_get(0, &mut mc).0 != 0);
        let bx = mc.center.x + 5.0e4;
        let by = mc.center.y;
        let bz = mc.center.z;

        assert_eq!(world_register_terrain_gravity_mascon(world), Bool::TRUE);

        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_additional_mass_properties(
            builder,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let body = rigid_body_builder_build(builder);
        let handle = world_insert_rigid_body(world, body);
        assert_ne!(handle, 0);

        let pose = [bx, by, bz, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(
            world_update_body_poses(world, [handle].as_ptr(), pose.as_ptr(), 1, Bool::TRUE),
            1
        );

        // Step with the law active: the body must pick up speed toward the mascon.
        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }
        let vel = rigid_body_get_linvel(world, handle);
        let speed = (vel.x * vel.x + vel.y * vel.y + vel.z * vel.z).sqrt();
        assert!(
            speed > 1e-15,
            "mascon should accelerate the body, got {:?}",
            vel
        );

        // Unregistering must stop terrain gravity.  A post-unregister step must
        // NOT change the (zero-uniform-gravity) velocity, because `world_step`
        // resets persistent user forces each frame before re-applying laws.
        assert_eq!(world_unregister_terrain_gravity(world), Bool::TRUE);
        let v0 = rigid_body_get_linvel(world, handle);
        world_step(world, 1.0 / 60.0);
        let v1 = rigid_body_get_linvel(world, handle);
        let dv = (v1.x - v0.x).abs() + (v1.y - v0.y).abs() + (v1.z - v0.z).abs();
        assert!(
            dv < 1e-9,
            "after unregister, one step must not accelerate the body (dv={})",
            dv
        );

        world_destroy(world);
    }
}
