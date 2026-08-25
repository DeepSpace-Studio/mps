#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::{Bool, RigidBodyHandleRaw, Vec3, WorldHandle};
    use mps_core::rapier::soft_body::{
        soft_body_add_distance_constraint, soft_body_add_particle, soft_body_add_spring,
        soft_body_add_tetrahedron, soft_body_configure_solver, soft_body_count, soft_body_create,
        soft_body_destroy, soft_body_get_particle, soft_body_particle_count,
        soft_body_remove_particle, soft_body_set_gravity, soft_body_voxel_build, soft_chain_create,
        soft_chain_node_handles,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};
    use rapier3d::prelude::soft_body::SoftBodyId;

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

    // ── Phase 5a: general soft-body builder (arbitrary topology) ──────────────

    #[test]
    fn soft_body_builder_creates_tetra_and_preserves_volume_xpbd() {
        let world = make_world();
        assert!(!world.is_null());

        // Build a unit tetrahedron (4 particles at the canonical simplex corners).
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        );
        assert!(id != u32::MAX, "soft_body_create should succeed");

        let p0 = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p1 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p2 = soft_body_add_particle(world, id, 0.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let p3 = soft_body_add_particle(world, id, 0.0, 0.0, 1.0, 1.0, Bool::FALSE);
        for p in [p0, p1, p2, p3] {
            assert!(p != u32::MAX, "particle add should succeed");
        }

        // XPBD: distance constraints on the 6 edges + one volume tetrahedron.
        assert_eq!(
            soft_body_configure_solver(world, id, 1, 20, 0.0),
            Bool::TRUE
        );
        let edges = [(p0, p1), (p0, p2), (p0, p3), (p1, p2), (p1, p3), (p2, p3)];
        for (a, b) in edges {
            assert_eq!(
                soft_body_add_distance_constraint(world, id, a, b, 0.0),
                Bool::TRUE,
                "distance constraint add should succeed"
            );
        }
        assert_eq!(
            soft_body_add_tetrahedron(world, id, p0, p1, p2, p3),
            Bool::TRUE,
            "tetrahedron add should succeed"
        );

        // Capture rest volume, step, and confirm the XPBD volume constraint keeps
        // the signed volume within a small tolerance of the rest value.
        let rest = {
            let sb = unsafe {
                (*world)
                    .inner
                    .soft_bodies
                    .get(SoftBodyId(id))
                    .expect("soft body present")
            };
            sb.total_volume()
        };
        assert!(rest.is_finite() && rest > 0.0, "rest volume positive");

        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        let after = {
            let sb = unsafe {
                (*world)
                    .inner
                    .soft_bodies
                    .get(SoftBodyId(id))
                    .expect("soft body present")
            };
            sb.total_volume()
        };
        // 6 rigid edges + a volume constraint → the tetrahedron should stay
        // near its rest shape under gravity (volume drift bounded).
        assert!(
            (after - rest).abs() / rest < 0.2,
            "XPBD volume should be preserved (rest={rest}, after={after})"
        );
        world_destroy(world);
    }

    #[test]
    fn soft_body_builder_mass_spring_chain_falls_and_stays_bounded() {
        let world = make_world();
        assert!(!world.is_null());

        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        );
        assert!(id != u32::MAX);

        // 4 free particles in a line, pinned start (anchor).
        let p0 = soft_body_add_particle(world, id, 0.0, 5.0, 0.0, 1.0, Bool::TRUE);
        assert!(p0 != u32::MAX);
        for i in 1..4 {
            let p = soft_body_add_particle(world, id, i as f64, 5.0, 0.0, 1.0, Bool::FALSE);
            assert!(p != u32::MAX, "particle {i} add should succeed");
        }
        // Springs between adjacent particles.
        for i in 0..3 {
            assert_eq!(
                soft_body_add_spring(world, id, i, i + 1, 200.0, 5.0),
                Bool::TRUE,
                "spring {i} add should succeed"
            );
        }

        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        let sb = unsafe {
            (*world)
                .inner
                .soft_bodies
                .get(SoftBodyId(id))
                .expect("soft body present")
        };
        assert_eq!(sb.particles.len(), 4);
        for p in &sb.particles {
            assert!(p.pos.x.is_finite() && p.pos.y.is_finite() && p.pos.z.is_finite());
            assert!(p.pos.x.abs() < 50.0 && p.pos.y.abs() < 50.0 && p.pos.z.abs() < 50.0);
        }
        // Pinned particle (p0) must not have moved.
        assert!(
            (sb.particles[p0 as usize].pos.x - 0.0).abs() < 1e-9
                && (sb.particles[p0 as usize].pos.y - 5.0).abs() < 1e-9,
            "pinned particle must stay anchored"
        );
        world_destroy(world);
    }

    #[test]
    fn soft_body_builder_rejects_bad_params() {
        let world = make_world();
        assert!(!world.is_null());

        // Null world.
        assert_eq!(
            soft_body_create(
                std::ptr::null_mut(),
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ),
            u32::MAX
        );
        // Null world for add_particle.
        assert_eq!(
            soft_body_add_particle(std::ptr::null_mut(), 0, 0.0, 0.0, 0.0, 1.0, Bool::FALSE),
            u32::MAX
        );
        // Bad gravity.
        assert_eq!(
            soft_body_create(
                world,
                Vec3 {
                    x: f64::NAN,
                    y: 0.0,
                    z: 0.0
                }
            ),
            u32::MAX
        );
        // Non-positive mass.
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert!(id != u32::MAX);
        assert_eq!(
            soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 0.0, Bool::FALSE),
            u32::MAX
        );
        // Unknown id.
        assert_eq!(
            soft_body_add_spring(world, 999, 0, 1, 1.0, 1.0),
            Bool::FALSE
        );
        // Unknown id for configure.
        assert_eq!(
            soft_body_configure_solver(world, 999, 1, 10, 0.0),
            Bool::FALSE
        );
        // Bad solver_mode.
        assert_eq!(
            soft_body_configure_solver(world, id, 2, 10, 0.0),
            Bool::FALSE
        );
        // Zero iterations for XPBD.
        assert_eq!(
            soft_body_configure_solver(world, id, 1, 0, 0.0),
            Bool::FALSE
        );
        // Degenerate tetrahedron (duplicate index).
        let a = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);
        assert!(a != u32::MAX);
        assert_eq!(
            soft_body_add_tetrahedron(world, id, a, a, a, a),
            Bool::FALSE
        );
        world_destroy(world);
    }

    // ── Phase 5b: query / readback / lifecycle FFI ────────────────────────────

    #[test]
    fn soft_body_query_readback_and_lifecycle() {
        let world = make_world();
        assert!(!world.is_null());

        // No soft bodies yet.
        assert_eq!(soft_body_count(world), 0);

        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        );
        assert!(id != u32::MAX);
        assert_eq!(soft_body_count(world), 1);

        // Add 3 collinear particles.
        let p0 = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p1 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p2 = soft_body_add_particle(world, id, 2.0, 0.0, 0.0, 1.0, Bool::FALSE);
        assert_eq!(soft_body_particle_count(world, id), 3);

        // Read back p1's position (should be (1,0,0)).
        let mut pos = Vec3::default();
        let mut vel = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, id, p1, &mut pos, &mut vel),
            Bool::TRUE
        );
        assert!((pos.x - 1.0).abs() < 1e-12 && pos.y.abs() < 1e-12 && pos.z.abs() < 1e-12);
        // Out-of-bounds index → FALSE.
        assert_eq!(
            soft_body_get_particle(world, id, 99, &mut pos, &mut vel),
            Bool::FALSE
        );

        // Remove p1; the other two stay, topology indices remain valid.
        assert_eq!(soft_body_remove_particle(world, id, p1), Bool::TRUE);
        assert_eq!(soft_body_particle_count(world, id), 2);
        // Remaining particles kept their positions.
        assert_eq!(
            soft_body_get_particle(world, id, p0, &mut pos, std::ptr::null_mut()),
            Bool::TRUE
        );
        assert!((pos.x - 0.0).abs() < 1e-12);
        // p2 was index 2, after removal of index 1 it shifts to index 1.
        assert_eq!(
            soft_body_get_particle(world, id, 1, &mut pos, std::ptr::null_mut()),
            Bool::TRUE
        );
        assert!((pos.x - 2.0).abs() < 1e-12, "p2 should now be at index 1");

        // Destroy the body; count drops, other ids would stay valid (only one here).
        assert_eq!(soft_body_destroy(world, id), Bool::TRUE);
        assert_eq!(soft_body_count(world), 0);
        // Re-querying the destroyed id → unknown.
        assert_eq!(soft_body_particle_count(world, id), u32::MAX);
        // Unknown id destroy → FALSE.
        assert_eq!(soft_body_destroy(world, 999), Bool::FALSE);

        world_destroy(world);
    }

    #[test]
    fn soft_body_remove_particle_keeps_topology_valid_after_stepping() {
        let world = make_world();
        assert!(!world.is_null());

        // 2x1x1 solid grid → 2 particles + 1 spring (mass-spring chain).
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
            Bool::FALSE,
        );
        assert!(id != u32::MAX);
        assert_eq!(soft_body_particle_count(world, id), 2);

        // Step a bit, then remove particle 0. The remaining particle must still be
        // finite and the body still steppable (no dangling spring index).
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        assert_eq!(soft_body_remove_particle(world, id, 0), Bool::TRUE);
        assert_eq!(soft_body_particle_count(world, id), 1);
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        let sb = unsafe {
            (*world)
                .inner
                .soft_bodies
                .get(SoftBodyId(id))
                .expect("soft body present")
        };
        assert_eq!(sb.particles.len(), 1);
        assert!(sb.particles[0].pos.x.is_finite());

        world_destroy(world);
    }
}
