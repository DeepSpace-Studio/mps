#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{collider_builder_build, world_insert_collider_with_parent};
    use mps_core::rapier::ffi::VoxelColliderOptions;
    use mps_core::rapier::ffi::{
        BodyStatus, Bool, ColliderHandleRaw, RigidBodyHandleRaw, Vec3, WorldHandle,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::soft_body::{
        soft_body_add_distance_constraint, soft_body_add_particle, soft_body_add_spring,
        soft_body_add_tetrahedron, soft_body_build_tetra_mesh, soft_body_configure_solver,
        soft_body_count, soft_body_create, soft_body_destroy, soft_body_enable_collision,
        soft_body_get_particle, soft_body_particle_count, soft_body_read_edges,
        soft_body_read_particles, soft_body_read_tetrahedra, soft_body_remove_particle,
        soft_body_set_gravity, soft_body_voxel_build, soft_body_voxel_dig, soft_chain_create,
        soft_chain_node_handles,
    };
    use mps_core::rapier::voxel::{collider_builder_create_voxels, collider_voxel_edit};
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

    // ── Phase 5c: 生物软体（复用 Phase 3 四面体 XPBD 体积约束）────────────────

    #[test]
    fn soft_body_build_tetra_mesh_xpbd_holds_volume_under_gravity() {
        let world = make_world();
        assert!(!world.is_null());

        // Regular tetrahedron (one vertex pinned at top, three hang free).
        let particles: [Vec3; 4] = [
            Vec3 {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            }, // 0: pinned anchor
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: -0.5,
                y: 0.0,
                z: 0.866,
            },
            Vec3 {
                x: -0.5,
                y: 0.0,
                z: -0.866,
            },
        ];
        // One tetrahedron spanning all four corners.
        let tets: [u32; 4] = [0, 1, 2, 3];

        let id = soft_body_build_tetra_mesh(
            world,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
            particles.as_ptr(),
            particles.len() as u32,
            tets.as_ptr(),
            1,
            1.0, // particle mass
            0.0, // compliance 0 → rigid volume constraint
            20,  // iterations
        );
        assert!(id != u32::MAX, "tetra mesh should build");

        // Pin vertex 0 (build_tetra_mesh creates all-dynamic; replicate the
        // caller's pin step by zeroing its inverse mass — same as anvilkit does).
        unsafe {
            let sb = (*world)
                .inner
                .soft_bodies
                .get_mut(SoftBodyId(id))
                .expect("soft body present");
            assert_eq!(sb.particles.len(), 4);
            assert_eq!(sb.tetrahedra.len(), 1);
            sb.particles[0].inv_mass = 0.0;
        }

        let rest = unsafe {
            (*world)
                .inner
                .soft_bodies
                .get(SoftBodyId(id))
                .unwrap()
                .total_volume()
        };
        assert!(rest.abs() > 1e-6, "tetrahedron has non-zero rest volume");

        for _ in 0..200 {
            world_step(world, 1.0 / 60.0);
        }

        let sb = unsafe {
            (*world)
                .inner
                .soft_bodies
                .get(SoftBodyId(id))
                .expect("soft body present")
        };
        let final_vol = sb.total_volume();
        // Volume constraint (compliance 0) keeps the tetra's volume near rest
        // even as the free vertices sag under gravity — the XPBD rebound.
        let rel_err = (final_vol - rest).abs() / rest.abs();
        assert!(
            rel_err < 0.05,
            "XPBD volume constraint holds volume (rel_err={rel_err})"
        );
        // Anchor stayed put; free vertices settled at finite positions.
        assert!(sb.particles[0].pos.y.is_finite());
        for p in &sb.particles {
            assert!(p.pos.x.is_finite() && p.pos.y.is_finite() && p.pos.z.is_finite());
        }

        world_destroy(world);
    }

    #[test]
    fn soft_body_build_tetra_mesh_rejects_degenerate() {
        let world = make_world();
        assert!(!world.is_null());

        let particles: [Vec3; 4] = [
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];
        // Degenerate tetrahedron (duplicate vertex 0) → rejected.
        let bad_tets: [u32; 4] = [0, 0, 1, 2];
        assert_eq!(
            soft_body_build_tetra_mesh(
                world,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                particles.as_ptr(),
                particles.len() as u32,
                bad_tets.as_ptr(),
                1,
                1.0,
                0.0,
                20,
            ),
            u32::MAX
        );

        // Empty particle/tet arrays → rejected.
        assert_eq!(
            soft_body_build_tetra_mesh(
                world,
                Vec3::default(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                1.0,
                0.0,
                20,
            ),
            u32::MAX
        );

        world_destroy(world);
    }

    // ── Phase 5d: 区块破坏 → 软体重建联动 ──────────────────────────────────────

    #[test]
    fn soft_body_voxel_dig_removes_cell_particle_and_rebuilds_map() {
        let world = make_world();
        assert!(!world.is_null());

        // 2x1x1 solid grid → particles [0,1] (cell (0,0,0)->p0, cell (1,0,0)->p1)
        // + 1 spring p0-p1.
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

        // Dig cell (0,0,0) → removes p0, spring dropped, map rebuilt (p1 shifts to 0).
        assert_eq!(soft_body_voxel_dig(world, id, 0, 0, 0), Bool::TRUE);
        assert_eq!(soft_body_particle_count(world, id), 1);
        let sb = unsafe {
            (*world)
                .inner
                .soft_bodies
                .get(SoftBodyId(id))
                .expect("soft body present")
        };
        assert_eq!(sb.springs.len(), 0, "incident spring removed with p0");
        // The remaining particle is the old p1, now at index 0.
        let mut pos = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, id, 0, &mut pos, std::ptr::null_mut()),
            Bool::TRUE
        );
        assert!(
            (pos.x - 1.5).abs() < 1e-9,
            "remaining particle is old (1,0,0) at x=1.5"
        );

        // Dig the last cell → body empty.
        assert_eq!(soft_body_voxel_dig(world, id, 1, 0, 0), Bool::TRUE);
        assert_eq!(soft_body_particle_count(world, id), 0);

        // Re-dig an already-dug/empty cell → FALSE.
        assert_eq!(soft_body_voxel_dig(world, id, 0, 0, 0), Bool::FALSE);
        // Out-of-bounds cell → FALSE.
        assert_eq!(soft_body_voxel_dig(world, id, 9, 0, 0), Bool::FALSE);
        // Unknown id → FALSE.
        assert_eq!(soft_body_voxel_dig(world, 999, 0, 0, 0), Bool::FALSE);

        world_destroy(world);
    }

    #[test]
    fn soft_body_voxel_dig_keeps_body_steppable_after_collapse() {
        let world = make_world();
        assert!(!world.is_null());

        // 3x1x1 chain → 3 particles + 2 springs. Dig the middle cell; the body
        // must stay steppable (no dangling indices) and remaining particles finite.
        let sx = 3u32;
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
        assert_eq!(soft_body_particle_count(world, id), 3);

        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        assert_eq!(soft_body_voxel_dig(world, id, 1, 0, 0), Bool::TRUE);
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
        assert_eq!(sb.particles.len(), 2);
        for p in &sb.particles {
            assert!(p.pos.x.is_finite() && p.pos.y.is_finite() && p.pos.z.is_finite());
        }

        world_destroy(world);
    }

    // ── Phase 5f: 软体-刚体碰撞 ──────────────────────────────────────────────
    // 一个自由软体质点从地面上方下落，启用碰撞耦合后，其 proxy 球体与静态半空间
    // 地面发生接触，质点应停在地面之上（≈粒子半径），而非穿透。禁用碰撞时则继续
    // 自由下落穿过地面。
    #[test]
    fn soft_body_collision_stops_at_ground() {
        let world = make_world();

        // 静态地面：Fixed 刚体 + 法线朝上的半空间（实体在 y<0，上方为空）。
        let ground_builder =
            mps_core::rapier::rigid_body::rigid_body_builder_create(BodyStatus::Fixed as u32);
        let ground = mps_core::rapier::rigid_body::rigid_body_builder_build(ground_builder);
        let ground_handle = mps_core::rapier::rigid_body::world_insert_rigid_body(world, ground);
        let ground_collider = mps_core::rapier::collider::collider_builder_build(
            mps_core::rapier::collider::collider_builder_create_halfspace(Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }),
        );
        mps_core::rapier::collider::world_insert_collider_with_parent(
            world,
            ground_collider,
            ground_handle,
        );

        // 单个自由质点，初始在地面上方 y=2.0，无弹簧（仅受重力）。
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        );
        let _p = soft_body_add_particle(world, id, 0.0, 2.0, 0.0, 1.0, Bool::FALSE);
        assert_eq!(
            soft_body_enable_collision(world, id, 0.5, Bool::TRUE),
            Bool::TRUE
        );

        let dt = 1.0 / 60.0;
        for _ in 0..180 {
            world_step(world, dt);
        }

        let mut pos = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, id, 0, &mut pos as *mut Vec3, std::ptr::null_mut()),
            Bool::TRUE
        );
        // 地面在 y=0，粒子半径 0.5 → 静止位置应约 y≈0.5，且不得穿透到 y<0.4。
        assert!(
            pos.y > 0.4,
            "collision-coupled particle must rest above the ground, got y={}",
            pos.y
        );

        // 对比：禁用碰撞后，质点应自由穿过地面（y 继续下降）。
        assert_eq!(
            soft_body_enable_collision(world, id, 0.5, Bool::FALSE),
            Bool::TRUE
        );
        for _ in 0..60 {
            world_step(world, dt);
        }
        let mut pos2 = Vec3::default();
        assert_eq!(
            soft_body_get_particle(world, id, 0, &mut pos2 as *mut Vec3, std::ptr::null_mut()),
            Bool::TRUE
        );
        assert!(
            pos2.y < 0.4,
            "without collision coupling the particle should fall through the ground, got y={}",
            pos2.y
        );

        world_destroy(world);
    }

    /// Phase 5g: digging a voxel collider cell auto-propagates to a soft body
    /// that shares the same world-space voxelization. Build a 2×2×2 all-solid
    /// grid as BOTH a voxel collider and a soft body (identical origin/size/
    /// voxel_size). Dig cell (0,0,0) via `collider_voxel_edit`; the overlapping
    /// soft-body particle must be removed automatically (count 8 → 7) without
    /// an explicit `soft_body_voxel_dig` call.
    #[test]
    fn collider_voxel_edit_propagates_dig_to_soft_body() {
        let world = make_world();
        assert!(!world.is_null());

        // 2×2×2 all-solid grid, origin (0,0,0), uniform voxel size 1.0.
        let size: u32 = 2;
        let voxels = vec![1u8; (size * size * size) as usize];

        // Soft body built from the same grid.
        let id = soft_body_voxel_build(
            world,
            voxels.as_ptr(),
            voxels.len() as u32,
            size,
            size,
            size,
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,         // particle_mass
            100.0,       // stiffness
            5.0,         // damping
            Bool::FALSE, // pin_boundary
        );
        assert_ne!(id, u32::MAX, "soft body build failed");
        assert_eq!(
            soft_body_particle_count(world, id),
            8,
            "2x2x2 all-solid grid should yield 8 particles"
        );

        // Voxel collider sharing the identical world-space grid.
        let options = VoxelColliderOptions {
            mode: 0, // Auto
            dynamic_body: Bool::FALSE,
            small_voxel_limit: 0,
            mesh_voxel_limit: 0,
        };
        let builder = collider_builder_create_voxels(
            voxels.as_ptr(),
            size,
            size,
            size,
            1.0,
            1.0,
            1.0,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            options,
        );
        assert!(!builder.is_null(), "voxel collider builder failed");
        let collider = collider_builder_build(builder);
        assert!(!collider.is_null(), "voxel collider build failed");

        // Parent body for the collider (a fixed anchor).
        let parent = rigid_body_builder_create(BodyStatus::Fixed as u32);
        let parent_handle = world_insert_rigid_body(world, rigid_body_builder_build(parent));
        let collider_handle: ColliderHandleRaw =
            world_insert_collider_with_parent(world, collider, parent_handle);
        assert_ne!(collider_handle, 0, "voxel collider insert failed");

        // Dig cell (0,0,0) in the collider grid.
        let dug = collider_voxel_edit(world, collider_handle, 0, 0, 0, 0);
        assert_eq!(dug, Bool::TRUE, "collider_voxel_edit dig should succeed");

        // The overlapping soft-body particle at (0,0,0) must be auto-removed.
        assert_eq!(
            soft_body_particle_count(world, id),
            7,
            "digging collider cell (0,0,0) should auto-collapse the soft body by one particle"
        );

        // Re-dig the same cell: idempotent, count stays 7.
        let dug_again = collider_voxel_edit(world, collider_handle, 0, 0, 0, 0);
        assert_eq!(dug_again, Bool::TRUE);
        assert_eq!(soft_body_particle_count(world, id), 7);

        world_destroy(world);
    }

    /// Phase 5i: topology read-back for rendering. Build a tetra-mesh soft body
    /// with a known spring + distance constraint + tetra, then pull the whole
    /// state via the bulk `soft_body_read_*` FFI and check counts/consistency
    /// against `soft_body_particle_count` + the topology we inserted.
    #[test]
    fn soft_body_read_back_topology_for_rendering() {
        let world = make_world();
        assert!(!world.is_null());

        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);

        // 4 particles forming a tetrahedron.
        let p0 = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p1 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let p2 = soft_body_add_particle(world, id, 0.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let p3 = soft_body_add_particle(world, id, 0.0, 0.0, 1.0, 1.0, Bool::FALSE);
        assert_eq!((p0, p1, p2, p3), (0, 1, 2, 3));

        // 1 spring edge (a-b) + 1 distance constraint edge (c-d).
        assert_eq!(soft_body_add_spring(world, id, 0, 1, 50.0, 2.0), Bool::TRUE);
        assert_eq!(
            soft_body_add_distance_constraint(world, id, 2, 3, 10.0),
            Bool::TRUE
        );
        // 1 tetra over all four particles.
        assert_eq!(soft_body_add_tetrahedron(world, id, 0, 1, 2, 3), Bool::TRUE);

        let count = soft_body_particle_count(world, id);
        assert_eq!(count, 4);

        // ── bulk particle read ──
        let mut pos = vec![Vec3::default(); count as usize];
        let mut inv_mass = vec![0.0f64; count as usize];
        let read =
            soft_body_read_particles(world, id, pos.as_mut_ptr(), inv_mass.as_mut_ptr(), count);
        assert_eq!(read, count, "read_particles should return particle count");
        // inv_mass of 1.0 mass → 1.0.
        for im in &inv_mass {
            assert_eq!(*im, 1.0, "unpinned particle inv_mass should be 1.0");
        }
        // Position 0 should match the particle we added.
        assert!((pos[0].x).abs() < 1e-9 && (pos[0].y).abs() < 1e-9);

        // ── edges read (spring + distance constraint = 2 edges = 4 u32) ──
        let edge_count = soft_body_read_edges(world, id, std::ptr::null_mut(), 0);
        assert_eq!(edge_count, 2, "2 edges (1 spring + 1 distance constraint)");
        let mut edges = vec![0u32; (edge_count as usize) * 2];
        let read_edges = soft_body_read_edges(world, id, edges.as_mut_ptr(), edges.len() as u32);
        assert_eq!(read_edges, 2);
        // spring edge (0,1) first, then distance edge (2,3).
        assert_eq!((edges[0], edges[1]), (0, 1));
        assert_eq!((edges[2], edges[3]), (2, 3));

        // ── tetra read (1 tetra = 4 u32) ──
        let tet_count = soft_body_read_tetrahedra(world, id, std::ptr::null_mut(), 0);
        assert_eq!(tet_count, 1);
        let mut tets = vec![0u32; (tet_count as usize) * 4];
        let read_tets = soft_body_read_tetrahedra(world, id, tets.as_mut_ptr(), tets.len() as u32);
        assert_eq!(read_tets, 1);
        assert_eq!(tets, vec![0, 1, 2, 3]);

        // ── capacity clamp: requesting fewer slots than needed must not panic ──
        let tiny = soft_body_read_edges(world, id, edges.as_mut_ptr(), 2);
        assert_eq!(
            tiny, 2,
            "real edge count returned even though buffer held only 1 edge"
        );

        // ── unknown id returns 0, no panic ──
        assert_eq!(
            soft_body_read_particles(
                world,
                u32::MAX,
                pos.as_mut_ptr(),
                std::ptr::null_mut(),
                count
            ),
            0
        );
        assert_eq!(
            soft_body_read_edges(world, u32::MAX, std::ptr::null_mut(), 0),
            0
        );
        assert_eq!(
            soft_body_read_tetrahedra(world, u32::MAX, std::ptr::null_mut(), 0),
            0
        );

        world_destroy(world);
    }
}
