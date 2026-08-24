#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::{RigidBodyHandleRaw, Vec3, WorldHandle};
    use mps_core::rapier::soft_body::{
        soft_body_set_gravity, soft_body_voxel_build, soft_chain_create, soft_chain_node_handles,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    fn make_world() -> *mut WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        })
    }

    #[test]
    fn soft_chain_creates_nodes_and_stays_bounded() {
        let world = make_world();
        assert!(!world.is_null());

        // 4-node chain along +X, spacing 1.0, first node fixed at origin,
        // spring stiffness 200 / damping 5 (soft but holds shape under gravity).
        let count = soft_chain_create(
            world,
            4,
            1.0,
            1.0,
            0.25,
            0, // no external anchor → node 0 fixed at origin
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            200.0,
            5.0,
        );
        assert_eq!(count, 4, "soft_chain_create should create 4 nodes");

        // Read back the (dynamic) node handles.
        let mut handles: Vec<RigidBodyHandleRaw> = vec![0; 8];
        let n = soft_chain_node_handles(world, handles.as_mut_ptr(), handles.len() as u32);
        assert!(n >= 3, "expected >=3 dynamic nodes (node 0 is fixed)");

        // Step the world; the chain should sag but remain finite and bounded.
        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        // Verify every dynamic node is finite and within a sane bounding box.
        for i in 0..n as usize {
            let h = handles[i];
            assert_ne!(h, 0, "node handle must be valid");
            let body = unsafe {
                (*world)
                    .inner
                    .bodies
                    .get(mps_core::rapier::ffi::unpack_rigid_body_handle(h))
            };
            let body = body.expect("node body present");
            let t = body.translation();
            assert!(t.x.is_finite() && t.y.is_finite() && t.z.is_finite());
            // The chain hangs from origin; it must not fly to infinity.
            assert!(t.x.abs() < 50.0, "x out of bounds: {}", t.x);
            assert!(t.y.abs() < 50.0, "y out of bounds: {}", t.y);
            assert!(t.z.abs() < 50.0, "z out of bounds: {}", t.z);
        }

        world_destroy(world);
    }

    #[test]
    fn soft_chain_rejects_bad_params() {
        let world = make_world();
        assert!(!world.is_null());
        // zero node count
        assert_eq!(
            soft_chain_create(
                world,
                0,
                1.0,
                1.0,
                0.25,
                0,
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0
                },
                200.0,
                5.0,
            ),
            0
        );
        // negative spacing
        assert_eq!(
            soft_chain_create(
                world,
                3,
                -1.0,
                1.0,
                0.25,
                0,
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0
                },
                200.0,
                5.0,
            ),
            0
        );
        world_destroy(world);
    }

    // ── Phase 4: voxel → soft-body + terrain-gravity coupling ──────────────────

    #[test]
    fn soft_body_voxel_build_creates_bounded_deformable() {
        let world = make_world();
        assert!(!world.is_null());

        // 3×3×3 grid, a solid 2×2×2 core in the middle (indices 0..1 on each axis).
        let sx = 3u32;
        let sy = 3u32;
        let sz = 3u32;
        let mut voxels = vec![0u8; (sx * sy * sz) as usize];
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    if x < 2 && y < 2 && z < 2 {
                        voxels
                            [x as usize + sx as usize * (z as usize + sz as usize * y as usize)] =
                            1;
                    }
                }
            }
        }
        let id = soft_body_voxel_build(
            world,
            voxels.as_ptr(),
            voxels.len() as u32,
            sx,
            sy,
            sz,
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
            80.0,
            1.0,
            mps_core::rapier::ffi::Bool::FALSE,
        );
        // SoftBodyId starts at 0, which is a valid id (not an error sentinel).
        assert!(
            id != u32::MAX,
            "voxel build should return a valid SoftBodyId"
        );

        // Step the world; the soft body should integrate to finite, bounded positions.
        for _ in 0..60 {
            world_step(world, 1.0 / 60.0);
        }
        let sb = unsafe {
            &(*world)
                .inner
                .soft_bodies
                .get(rapier3d::prelude::soft_body::SoftBodyId(id))
        };
        let sb = sb.expect("soft body present");
        assert!(sb.particles.len() >= 8, "expected >=8 solid particles");
        for p in &sb.particles {
            assert!(p.pos.x.is_finite() && p.pos.y.is_finite() && p.pos.z.is_finite());
            assert!(p.pos.x.abs() < 50.0 && p.pos.y.abs() < 50.0 && p.pos.z.abs() < 50.0);
        }
        world_destroy(world);
    }

    #[test]
    fn soft_body_set_gravity_applies() {
        let world = make_world();
        assert!(!world.is_null());

        // Minimal 2×1×1 solid grid (one spring, two particles, no boundary pinning).
        let sx = 2u32;
        let sy = 1u32;
        let sz = 1u32;
        let voxels = vec![1u8; (sx * sy * sz) as usize];
        let id = soft_body_voxel_build(
            world,
            voxels.as_ptr(),
            voxels.len() as u32,
            sx,
            sy,
            sz,
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
            50.0,
            0.5,
            mps_core::rapier::ffi::Bool::FALSE,
        );
        assert!(
            id != u32::MAX,
            "voxel build should return a valid SoftBodyId"
        );

        // Override gravity to pull along +X (terrain-gravity coupling hook).
        let ok = soft_body_set_gravity(
            world,
            id,
            Vec3 {
                x: 3.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_eq!(ok, mps_core::rapier::ffi::Bool::TRUE);
        let sb = unsafe {
            &(*world)
                .inner
                .soft_bodies
                .get(rapier3d::prelude::soft_body::SoftBodyId(id))
        };
        let sb = sb.expect("soft body present");
        assert!(
            (sb.gravity.x - 3.0).abs() < 1e-12,
            "gravity.x should be 3.0"
        );

        // Unknown id → FALSE.
        let bad = soft_body_set_gravity(
            world,
            999,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_eq!(bad, mps_core::rapier::ffi::Bool::FALSE);
        world_destroy(world);
    }
}
