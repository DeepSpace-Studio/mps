#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{
        collider_builder_build, collider_builder_create_sphere, world_insert_collider_with_parent,
    };
    use mps_core::rapier::ffi::VoxelColliderOptions;
    use mps_core::rapier::ffi::{
        BodyStatus, Bool, ColliderHandleRaw, RigidBodyHandleRaw, Sphere, Vec3, WorldHandle,
    };
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create,
        rigid_body_builder_set_additional_mass_properties, rigid_body_builder_set_linvel,
        rigid_body_builder_set_translation, rigid_body_get_translation, world_insert_rigid_body,
    };
    use mps_core::rapier::soft_body::{
        soft_body_add_bending, soft_body_add_distance_constraint, soft_body_add_particle,
        soft_body_add_spring, soft_body_add_tetrahedron, soft_body_add_triangle,
        soft_body_apply_wind, soft_body_attach_particle, soft_body_build_tetra_mesh,
        soft_body_clear_wind, soft_body_configure_solver, soft_body_count, soft_body_create,
        soft_body_destroy, soft_body_detach_particle, soft_body_enable_collision,
        soft_body_get_particle, soft_body_is_sleeping, soft_body_kinetic_energy,
        soft_body_particle_count, soft_body_read_edges, soft_body_read_particles,
        soft_body_read_tetrahedra, soft_body_read_triangles, soft_body_remove_particle,
        soft_body_set_cross_collision, soft_body_set_distance_constraint_compliance,
        soft_body_set_gravity, soft_body_set_plasticity, soft_body_set_pressure,
        soft_body_set_self_collision, soft_body_set_spring_stiffness, soft_body_set_tear_strain,
        soft_body_sleep, soft_body_total_volume, soft_body_voxel_build, soft_body_voxel_dig,
        soft_body_wake, soft_chain_create, soft_chain_node_handles,
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
        for &h in handles.iter().take(n as usize) {
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
        let _p2 = soft_body_add_particle(world, id, 2.0, 0.0, 0.0, 1.0, Bool::FALSE);
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

    // ── Phase 6: 布料拓扑（三角形面 + 结构边自动注册 + 弯曲约束）─────────────────
    // 一个 2×1 的四边形布片 = (0,0)-(1,0)-(1,1)-(0,1)：两个三角形 + 四条边。
    // 结构边由 add_triangle 自动去重注册；弯曲约束（对角线）由 add_bending 显式添加。
    #[test]
    fn soft_body_cloth_topology_and_bending() {
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

        // 4 个布料质点（pinned[0]=固定角，其余自由）。
        assert_eq!(
            soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::TRUE),
            0
        );
        assert_eq!(
            soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE),
            1
        );
        assert_eq!(
            soft_body_add_particle(world, id, 1.0, 1.0, 0.0, 1.0, Bool::FALSE),
            2
        );
        assert_eq!(
            soft_body_add_particle(world, id, 0.0, 1.0, 0.0, 1.0, Bool::FALSE),
            3
        );

        // 两个三角形组成四边形。shared edge (0,2) 由 add_triangle 去重，故
        // 结构边 = 5（(0,1)(1,2)(2,0)(2,3)(3,0)），再加 1 条弯曲边 (1,3) = 6，
        // 恰好是 4 顶点的全部 6 条边（K4）。
        assert_eq!(soft_body_add_triangle(world, id, 0, 1, 2), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 0, 2, 3), Bool::TRUE);

        // 弯曲约束：四边形另一条对角线 (1,3)（(0,2) 已是结构共享边，无需重复）。
        assert_eq!(soft_body_add_bending(world, id, 1, 3), Bool::TRUE);

        let particle_count = soft_body_particle_count(world, id);
        assert_eq!(particle_count, 4);

        // ── 三角形读回：2 个面 = 6 个 u32 ──
        let tri_count = soft_body_read_triangles(world, id, std::ptr::null_mut(), 0);
        assert_eq!(tri_count, 2, "2 triangle faces expected");
        let mut tris = vec![0u32; (tri_count as usize) * 3];
        let read_tris = soft_body_read_triangles(world, id, tris.as_mut_ptr(), tris.len() as u32);
        assert_eq!(read_tris, 2);

        // ── 边读回：5 条结构边 + 1 条弯曲边 = 6 条边 ──
        let edge_count = soft_body_read_edges(world, id, std::ptr::null_mut(), 0);
        assert_eq!(edge_count, 6, "5 structural + 1 bending edge");
        let mut edges = vec![0u32; (edge_count as usize) * 2];
        let read_edges = soft_body_read_edges(world, id, edges.as_mut_ptr(), edges.len() as u32);
        assert_eq!(read_edges, 6);
        // 归一化为无序对，验证恰好是 K4 的全部 6 条边（去重生效，无重复）。
        let mut pairs: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
        for k in 0..6usize {
            let a = edges[k * 2];
            let b = edges[k * 2 + 1];
            assert!(a != b, "degenerate edge");
            pairs.insert(if a < b { (a, b) } else { (b, a) });
        }
        let expected: std::collections::HashSet<(u32, u32)> =
            [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
                .into_iter()
                .collect();
        assert_eq!(pairs, expected, "edge set must equal all 6 edges of K4");

        // 未知 id 返回 0，不 panic。
        assert_eq!(
            soft_body_read_triangles(world, u32::MAX, std::ptr::null_mut(), 0),
            0
        );

        world_destroy(world);
    }

    // ── Phase 6: 布料在重力下的悬垂（XPBD 求解器跑通，不发散）──────────────────
    #[test]
    fn soft_body_cloth_sags_under_gravity() {
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
        assert_ne!(id, u32::MAX);

        // 6 个质点排成 3×2 网格在 y=2 平面，pin 顶部两个角。
        let _p0 = soft_body_add_particle(world, id, -1.0, 2.0, 0.0, 1.0, Bool::TRUE);
        let _p1 = soft_body_add_particle(world, id, 1.0, 2.0, 0.0, 1.0, Bool::TRUE);
        let _p2 = soft_body_add_particle(world, id, -1.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let _p3 = soft_body_add_particle(world, id, 1.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let _p4 = soft_body_add_particle(world, id, -1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let _p5 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 4 个三角面覆盖网格。
        assert_eq!(soft_body_add_triangle(world, id, 0, 1, 3), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 0, 3, 2), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 1, 5, 3), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 3, 5, 4), Bool::TRUE);

        // 启用 XPBD 求解器（布料用距离约束保形）。
        assert_eq!(
            soft_body_configure_solver(world, id, 1, 20, 0.0),
            Bool::TRUE
        );

        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        // 读回质点，确认：固定角不动、自由点有限且下垂、无 NaN/发散。
        let count = soft_body_particle_count(world, id);
        let mut pos = vec![Vec3::default(); count as usize];
        let read =
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
        assert_eq!(read, count);
        for (i, p) in pos.iter().enumerate() {
            assert!(
                p.x.is_finite() && p.y.is_finite() && p.z.is_finite(),
                "p{i} blew up"
            );
        }
        // 固定角仍锚在 y≈2。
        assert!((pos[0].y - 2.0).abs() < 1e-6, "pinned corner drifted");
        assert!((pos[1].y - 2.0).abs() < 1e-6, "pinned corner drifted");
        // 底部自由点应下垂（y 明显低于 2）。
        assert!(pos[4].y < 1.5, "bottom row should sag below 1.5");
        assert!(pos[5].y < 1.5, "bottom row should sag below 1.5");

        world_destroy(world);
    }

    // ── Phase 7: 风场把固定边的布料吹向侧向（纯外力，无新力学）─────────────────
    #[test]
    fn soft_body_wind_blows_cloth_sideways() {
        let world = make_world();
        assert!(!world.is_null());

        // 无重力，纯风：布料应被恒定 wind accel 沿 +X 推出。
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);

        // 5 个质点：仅 pin 左上角 p0，其余自由（软边以便迎风鼓起）。
        // p0(-1,2) 固定；p1(1,2) p2(-1,0) p3(1,0) p4(0,1) 自由。
        let _p0 = soft_body_add_particle(world, id, -1.0, 2.0, 0.0, 1.0, Bool::TRUE);
        let _p1 = soft_body_add_particle(world, id, 1.0, 2.0, 0.0, 1.0, Bool::FALSE);
        let _p2 = soft_body_add_particle(world, id, -1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let _p3 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let _p4 = soft_body_add_particle(world, id, 0.0, 1.0, 0.0, 1.0, Bool::FALSE);

        assert_eq!(soft_body_add_triangle(world, id, 0, 1, 4), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 1, 3, 4), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 3, 2, 4), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 2, 0, 4), Bool::TRUE);

        // 软边（compliance > 0）让布料可迎风鼓起；仅 pin 一个角 → 整片被风吹向 +X。
        assert_eq!(
            soft_body_configure_solver(world, id, 1, 20, 1e-2),
            Bool::TRUE
        );

        // 启用 +X 风（10 m/s² 恒定加速度），无 drag 以便观察确定性位移。
        assert_eq!(
            soft_body_apply_wind(
                world,
                id,
                Vec3 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0
                },
                0.0,
            ),
            Bool::TRUE
        );

        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }

        let count = soft_body_particle_count(world, id);
        let mut pos = vec![Vec3::default(); count as usize];
        let read =
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
        assert_eq!(read, count);

        // 固定角不动。
        assert!(
            (pos[0].x + 1.0).abs() < 1e-6,
            "pinned corner must stay at x=-1"
        );
        assert!(
            (pos[0].y - 2.0).abs() < 1e-6,
            "pinned corner must stay at y=2"
        );
        // 无重力纯 +X 风下，整片布以单角铰接点像旗帜一样顺风扬起：
        // 各自由质点有限、无发散，且整片质心明显顺风（x 正方向）漂移。
        let mut finite = true;
        let mut sum_x_end = 0.0;
        for (i, p) in pos.iter().enumerate() {
            if !(p.x.is_finite() && p.y.is_finite() && p.z.is_finite()) {
                finite = false;
            }
            if i != 0 {
                sum_x_end += p.x; // 跳过 pinned p0
            }
        }
        assert!(finite, "some particle blew up");
        // 自由质点初始 x 之和 = p1(1)+p2(-1)+p3(1)+p4(0) = 1.0；顺风后应 > 1.0。
        assert!(
            sum_x_end > 1.0 + 0.3,
            "sheet should drift downwind (+X): sum_x={}",
            sum_x_end
        );
        // 至少存在一个自由质点明显顺风超出初始最远 x=1.0。
        let max_x = pos[1..]
            .iter()
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(max_x > 1.2, "a free particle should be blown past x=1.2");
        // 动能应为正且有限（风吹动）。
        let ke = soft_body_kinetic_energy(world, id);
        assert!(
            ke.is_finite() && ke > 0.0,
            "kinetic energy should be positive & finite"
        );
        // clear_wind 后动力学仍可继续（不报错）。
        assert_eq!(soft_body_clear_wind(world, id), Bool::TRUE);

        world_destroy(world);
    }

    // ── Phase 7: 休眠跳过积分 + 诊断读数（能量/体积）────────────────────────
    #[test]
    fn soft_body_sleep_skips_integration_and_diagnostics() {
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
        assert_ne!(id, u32::MAX);

        // 单个自由质点，无约束。
        let _p0 = soft_body_add_particle(world, id, 0.0, 10.0, 0.0, 1.0, Bool::FALSE);

        // 初始：清醒、动能 0（静止）、体积 0（无四面体）。
        assert_eq!(soft_body_is_sleeping(world, id), Bool::FALSE);
        assert_eq!(soft_body_kinetic_energy(world, id), 0.0);
        assert_eq!(soft_body_total_volume(world, id), 0.0);

        // 让质点自由下落若干步。
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        let ke_falling = soft_body_kinetic_energy(world, id);
        assert!(
            ke_falling > 0.0,
            "falling particle should gain kinetic energy"
        );

        // 休眠：之后 step 不再改变位置。
        assert_eq!(soft_body_sleep(world, id), Bool::TRUE);
        assert_eq!(soft_body_is_sleeping(world, id), Bool::TRUE);
        let before = {
            let mut pos = vec![Vec3::default(); 1];
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), 1);
            pos[0]
        };
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        let after = {
            let mut pos = vec![Vec3::default(); 1];
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), 1);
            pos[0]
        };
        assert!(
            (before.x - after.x).abs() < 1e-12
                && (before.y - after.y).abs() < 1e-12
                && (before.z - after.z).abs() < 1e-12,
            "sleeping body must not move under step"
        );

        // 唤醒后继续下落。
        assert_eq!(soft_body_wake(world, id), Bool::TRUE);
        assert_eq!(soft_body_is_sleeping(world, id), Bool::FALSE);
        for _ in 0..10 {
            world_step(world, 1.0 / 60.0);
        }
        let woken = {
            let mut pos = vec![Vec3::default(); 1];
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), 1);
            pos[0]
        };
        assert!(woken.y < before.y, "woken particle should resume falling");

        // 未知 id 的睡眠/诊断查询返回 False/0 而不 panic。
        assert_eq!(soft_body_is_sleeping(world, u32::MAX), Bool::FALSE);
        assert_eq!(soft_body_sleep(world, u32::MAX), Bool::FALSE);
        assert_eq!(soft_body_kinetic_energy(world, u32::MAX), 0.0);

        world_destroy(world);
    }

    // ── Phase 8: 锚定软体任意质点到刚体，质点刚性跟随刚体平移──────────────────
    #[test]
    fn soft_body_attach_particle_follows_rigid_body_translation() {
        let world = make_world();
        assert!(!world.is_null());

        // 一个匀速沿 +X 移动的刚体，初位置 x=5。
        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_translation(
            builder,
            Vec3 {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
        );
        rigid_body_builder_set_linvel(
            builder,
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let body = rigid_body_builder_build(builder);
        let body_h = world_insert_rigid_body(world, body);
        assert_ne!(body_h, 0u64);

        // 软体：单个自由质点，初始在 (5,0,0)（与刚体同位置，作为绑点）。
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);
        let p0 = soft_body_add_particle(world, id, 5.0, 0.0, 0.0, 1.0, Bool::FALSE);
        assert_eq!(p0, 0);

        // 把质点 0 锚定到刚体（绑点 = 质点当前世界位置）。
        assert_eq!(
            soft_body_attach_particle(
                world,
                id,
                0,
                body_h,
                Vec3 {
                    x: 5.0,
                    y: 0.0,
                    z: 0.0
                }
            ),
            Bool::TRUE
        );

        // 步进若干步：刚体应沿 +X 移动，质点需刚性跟随。
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }

        let tx = rigid_body_get_translation(world, body_h);
        // 刚体已平移：x ≈ 5 + 30/60 = 5.5。
        assert!(
            tx.x > 5.3 && tx.x < 5.7,
            "rigid body should have moved: tx.x={}",
            tx.x
        );

        // 绑定的质点应跟随刚体（位置 ≈ 刚体翻译，有限且不发散）。
        let count = soft_body_particle_count(world, id);
        let mut pos = vec![Vec3::default(); count as usize];
        let read =
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
        assert_eq!(read, count);
        assert!(pos[0].x.is_finite() && pos[0].y.is_finite() && pos[0].z.is_finite());
        assert!(
            (pos[0].x - tx.x).abs() < 1e-6,
            "bound particle must track body translation: pos.x={} tx.x={}",
            pos[0].x,
            tx.x
        );
        assert!(
            (pos[0].y - tx.y).abs() < 1e-6,
            "bound particle must track body translation: pos.y={} tx.y={}",
            pos[0].y,
            tx.y
        );

        // 解绑后质点恢复自由（不再强制贴合刚体）。
        assert_eq!(soft_body_detach_particle(world, id, 0), Bool::TRUE);
        world_step(world, 1.0 / 60.0);
        let count2 = soft_body_particle_count(world, id);
        let mut pos2 = vec![Vec3::default(); count2 as usize];
        let _ =
            soft_body_read_particles(world, id, pos2.as_mut_ptr(), std::ptr::null_mut(), count2);
        assert!(
            pos2[0].x.is_finite(),
            "detached particle must remain finite"
        );

        world_destroy(world);
    }

    // ── Phase 8: 非法参数返回 False 而不 panic──────────────────────────────
    #[test]
    fn soft_body_attach_particle_rejects_invalid_args() {
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
        let _p0 = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 不存在的 body → False。
        assert_eq!(
            soft_body_attach_particle(
                world,
                id,
                0,
                u64::MAX,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ),
            Bool::FALSE
        );
        // 越界 particle → False。
        assert_eq!(
            soft_body_attach_particle(
                world,
                id,
                99,
                0,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ),
            Bool::FALSE
        );
        // 越界 detach particle → False。
        assert_eq!(soft_body_detach_particle(world, id, 99), Bool::FALSE);

        world_destroy(world);
    }

    // ── Phase 9: 撕裂 — 应变超阈值的边被移除、破损面被删─────────────────────────
    #[test]
    fn soft_body_tears_when_strain_exceeds_threshold() {
        let world = make_world();
        assert!(!world.is_null());

        // 无重力，纯外力把自由点往下猛拉，确保边被过度拉伸。
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);

        // 6 个质点 3×2 网格在 y=2 平面；pin 顶部两角 p0,p1，其余自由。
        let _p0 = soft_body_add_particle(world, id, -1.0, 2.0, 0.0, 1.0, Bool::TRUE);
        let _p1 = soft_body_add_particle(world, id, 1.0, 2.0, 0.0, 1.0, Bool::TRUE);
        let _p2 = soft_body_add_particle(world, id, -1.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let _p3 = soft_body_add_particle(world, id, 1.0, 1.0, 0.0, 1.0, Bool::FALSE);
        let _p4 = soft_body_add_particle(world, id, -1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        let _p5 = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
        // 4 个三角面（边自动成为 distance constraints）。
        assert_eq!(soft_body_add_triangle(world, id, 0, 1, 3), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 0, 3, 2), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 1, 5, 3), Bool::TRUE);
        assert_eq!(soft_body_add_triangle(world, id, 3, 5, 4), Bool::TRUE);
        // XPBD 求解器。
        assert_eq!(
            soft_body_configure_solver(world, id, 1, 20, 0.0),
            Bool::TRUE
        );

        // 开撕裂，阈值极低（1% 应变即断）。
        assert_eq!(soft_body_set_tear_strain(world, id, 0.01, 1), Bool::TRUE);

        // 记录初始边数（springs+distance_constraints）和面数。
        let edges0 = soft_body_read_edges(world, id, std::ptr::null_mut(), 1024);
        assert!(edges0 > 0);
        let tris0 = soft_body_read_triangles(world, id, std::ptr::null_mut(), 1024);
        assert!(tris0 > 0);

        // 极端重力把自由点往下拽，边迅速超阈值被撕裂。
        soft_body_set_gravity(
            world,
            id,
            Vec3 {
                x: 0.0,
                y: -50.0,
                z: 0.0,
            },
        );
        for _ in 0..60 {
            world_step(world, 1.0 / 60.0);
        }

        let edges1 = soft_body_read_edges(world, id, std::ptr::null_mut(), 1024);
        let tris1 = soft_body_read_triangles(world, id, std::ptr::null_mut(), 1024);
        // 撕裂后：边和面应减少（至少断开若干边）。
        assert!(
            edges1 < edges0,
            "tearing should remove over-stretched edges: {edges1} >= {edges0}"
        );
        assert!(
            tris1 < tris0,
            "torn faces should be dropped: {tris1} >= {tris0}"
        );
        // 质点仍在、有限。
        let count = soft_body_particle_count(world, id);
        let mut pos = vec![Vec3::default(); count as usize];
        let read =
            soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
        assert_eq!(read, count);
        for (i, p) in pos.iter().enumerate() {
            assert!(
                p.x.is_finite() && p.y.is_finite() && p.z.is_finite(),
                "p{i} blew up"
            );
        }

        // 关闭撕裂后不应再断边（剩余边保持稳定）。
        assert_eq!(soft_body_set_tear_strain(world, id, 0.0, 0), Bool::TRUE);
        let edges2 = soft_body_read_edges(world, id, std::ptr::null_mut(), 1024);
        for _ in 0..30 {
            world_step(world, 1.0 / 60.0);
        }
        let edges3 = soft_body_read_edges(world, id, std::ptr::null_mut(), 1024);
        assert_eq!(
            edges3, edges2,
            "disabling tearing must stop further edge loss"
        );

        world_destroy(world);
    }

    // ── Phase 9: 撕裂非法/未知参数返回 False 而不 panic─────────────────────────
    #[test]
    fn soft_body_set_tear_strain_rejects_invalid_args() {
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
        let _p = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 未知 id → False。
        assert_eq!(
            soft_body_set_tear_strain(world, u32::MAX, 0.5, 1),
            Bool::FALSE
        );
        // 非有限阈值 → False。
        assert_eq!(
            soft_body_set_tear_strain(world, id, f64::NAN, 1),
            Bool::FALSE
        );
        // 合法调用 → True，且未 panic。
        assert_eq!(soft_body_set_tear_strain(world, id, 0.3, 1), Bool::TRUE);

        world_destroy(world);
    }

    // ── Phase 10: 塑性 — 超过屈服应变的边把形变永久冻入 rest_length─────────────
    #[test]
    fn soft_body_plasticity_freezes_deformation() {
        // 观测方式：一个被弹簧从顶部锚点拽住的自由质点，在重力下垂。
        //   * 弹性（无塑性）：弹簧始终拉回，节点停在靠近原长处（下坠少）。
        //   * 塑性（creep=1, yield 很小）：弹簧一旦超屈服就永久把 rest_length 拉到
        //     当前长度，弹簧"放弃"回拉 → 节点持续下坠得更远、更深。
        // 两者都用 soft_body_read_particles 读位置（不依赖内部字段）。
        fn sag(plastic: bool) -> f64 {
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
            // 关闭世界重力，仅用软体自身重力驱动下坠。
            assert_eq!(
                soft_body_set_gravity(
                    world,
                    id,
                    Vec3 {
                        x: 0.0,
                        y: -30.0,
                        z: 0.0
                    }
                ),
                Bool::TRUE
            );
            // 锚点 p0 钉在 y=10；自由质点 p1 从 y=9 出发（初始弹簧长≈1）。
            let _a = soft_body_add_particle(world, id, 0.0, 10.0, 0.0, 1.0, Bool::TRUE);
            let _b = soft_body_add_particle(world, id, 0.0, 9.0, 0.0, 1.0, Bool::FALSE);
            assert_eq!(
                soft_body_add_spring(world, id, 0, 1, 200.0, 0.0),
                Bool::TRUE
            );
            if plastic {
                assert_eq!(
                    soft_body_set_plasticity(world, id, 0.01, 1.0, 1),
                    Bool::TRUE
                );
            }
            for _ in 0..200 {
                world_step(world, 1.0 / 60.0);
            }
            let count = soft_body_particle_count(world, id);
            let mut pos = vec![Vec3::default(); count as usize];
            let _r =
                soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
            let y_free = pos[1].y;
            world_destroy(world);
            y_free
        }

        let y_elastic = sag(false);
        let y_plastic = sag(true);
        // 塑性让弹簧"放弃回拉"，自由节点下坠更深（y 更小）。
        assert!(
            y_plastic < y_elastic - 0.5,
            "plasticity should let the node sag much further: plastic={y_plastic} elastic={y_elastic}"
        );
        assert!(y_plastic.is_finite() && y_elastic.is_finite());
    }

    // ── Phase 10: 塑性非法/未知参数返回 False 而不 panic─────────────────────────
    #[test]
    fn soft_body_set_plasticity_rejects_invalid_args() {
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
        let _p = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 未知 id → False。
        assert_eq!(
            soft_body_set_plasticity(world, u32::MAX, 0.1, 0.5, 1),
            Bool::FALSE
        );
        // 非有限参数 → False。
        assert_eq!(
            soft_body_set_plasticity(world, id, f64::NAN, 0.5, 1),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_plasticity(world, id, 0.1, f64::INFINITY, 1),
            Bool::FALSE
        );
        // 合法调用 → True，且未 panic。
        assert_eq!(
            soft_body_set_plasticity(world, id, 0.05, 1.0, 1),
            Bool::TRUE
        );
        // 关闭（enabled=0）→ True，且不再塑性化。
        assert_eq!(
            soft_body_set_plasticity(world, id, 0.05, 1.0, 0),
            Bool::TRUE
        );

        world_destroy(world);
    }

    // ── Phase 11: 充气 — 闭合三角网格被内部气压吹胀（半径增大）─────────────────
    #[test]
    fn soft_body_pressure_inflates_closed_mesh() {
        // 八面体（6 顶点、8 三角面）闭合壳，无重力，开启气压后半径应增大。
        fn mean_radius(pressure: Option<f64>) -> f64 {
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
            // 无重力。
            assert_eq!(
                soft_body_set_gravity(
                    world,
                    id,
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0
                    }
                ),
                Bool::TRUE
            );
            // 八面体顶点（±1,0,0）(0,±1,0) (0,0,±1)。
            let v = [
                (1.0, 0.0, 0.0),
                (-1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, -1.0, 0.0),
                (0.0, 0.0, 1.0),
                (0.0, 0.0, -1.0),
            ];
            for (x, y, z) in v {
                let _ = soft_body_add_particle(world, id, x, y, z, 1.0, Bool::FALSE);
            }
            // 8 个三角面（带符号顺序使法向一致）。
            let faces = [
                (0, 2, 4),
                (2, 1, 4),
                (1, 3, 4),
                (3, 0, 4),
                (0, 3, 5),
                (3, 1, 5),
                (1, 2, 5),
                (2, 0, 5),
            ];
            for (a, b, c) in faces {
                assert_eq!(soft_body_add_triangle(world, id, a, b, c), Bool::TRUE);
            }
            if let Some(p) = pressure {
                assert_eq!(soft_body_set_pressure(world, id, p), Bool::TRUE);
            }
            for _ in 0..120 {
                world_step(world, 1.0 / 60.0);
            }
            let count = soft_body_particle_count(world, id);
            let mut pos = vec![Vec3::default(); count as usize];
            let _r =
                soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
            // 质心 + 平均半径。
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for p in &pos {
                cx += p.x;
                cy += p.y;
                cz += p.z;
            }
            let n = count as f64;
            cx /= n;
            cy /= n;
            cz /= n;
            let mut r = 0.0;
            for p in &pos {
                let dx = p.x - cx;
                let dy = p.y - cy;
                let dz = p.z - cz;
                r += (dx * dx + dy * dy + dz * dz).sqrt();
            }
            r /= n;
            world_destroy(world);
            r
        }

        let r_no_pressure = mean_radius(None);
        let r_pressurized = mean_radius(Some(2.0));
        // 气压把闭合壳吹胀 → 半径明显大于静止（≈1.0）。
        assert!(
            r_pressurized > r_no_pressure + 0.1,
            "pressure should inflate the mesh: pressurized={r_pressurized} baseline={r_no_pressure}"
        );
    }

    // ── Phase 11: 气压非法/未知参数返回 False 而不 panic─────────────────────────
    #[test]
    fn soft_body_set_pressure_rejects_invalid_args() {
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
        let _p = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 未知 id → False。
        assert_eq!(soft_body_set_pressure(world, u32::MAX, 1.0), Bool::FALSE);
        // 非有限气压 → False。
        assert_eq!(soft_body_set_pressure(world, id, f64::NAN), Bool::FALSE);
        // 合法调用 → True，且未 panic。
        assert_eq!(soft_body_set_pressure(world, id, 1.5), Bool::TRUE);
        // 关闭（<=0）→ True，且不再充气。
        assert_eq!(soft_body_set_pressure(world, id, 0.0), Bool::TRUE);

        world_destroy(world);
    }

    // ── Phase 12: 自碰撞 — 紧密堆叠的自由质点保持分离(不穿透)────────────────────
    #[test]
    fn soft_body_self_collision_keeps_particles_separated() {
        // 10 个自由质点在竖直方向密集堆叠 + 重力。无自碰撞时全部塌缩到同一点;
        // 开启自碰撞(半径 r)后任意两点最小间距应 >= ~2r。
        const N: usize = 10;
        const R: f64 = 0.25;
        fn min_pair_dist(self_col: bool) -> f64 {
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
            assert_ne!(id, u32::MAX);
            for i in 0..N {
                // 竖直堆叠,间距 0.1 (< 2R),重力下会塌缩。
                let _ =
                    soft_body_add_particle(world, id, 0.0, (i as f64) * 0.1, 0.0, 1.0, Bool::FALSE);
            }
            if self_col {
                assert_eq!(soft_body_set_self_collision(world, id, R, 0.0), Bool::TRUE);
            }
            for _ in 0..200 {
                world_step(world, 1.0 / 60.0);
            }
            let count = soft_body_particle_count(world, id);
            let mut pos = vec![Vec3::default(); count as usize];
            let _r =
                soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
            let mut m = f64::MAX;
            for i in 0..count as usize {
                for j in (i + 1)..count as usize {
                    let dx = pos[i].x - pos[j].x;
                    let dy = pos[i].y - pos[j].y;
                    let dz = pos[i].z - pos[j].z;
                    let d = (dx * dx + dy * dy + dz * dz).sqrt();
                    if d < m {
                        m = d;
                    }
                }
            }
            world_destroy(world);
            m
        }

        let sep_off = min_pair_dist(false);
        let sep_on = min_pair_dist(true);
        // 无自碰撞:塌缩到几乎重合(间距远小于 2R)。
        assert!(
            sep_off < R,
            "no self-collision should let them collapse, got {sep_off}"
        );
        // 有自碰撞:最小间距应被推到接近 2R 甚至更大。
        assert!(
            sep_on >= 2.0 * R - 1e-3,
            "self-collision should keep particles >= 2R apart: sep_on={sep_on} R={R}"
        );
    }

    // ── Phase 12: 自碰撞非法/未知参数返回 False 而不 panic───────────────────────
    #[test]
    fn soft_body_set_self_collision_rejects_invalid_args() {
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
        let _p = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 未知 id → False。
        assert_eq!(
            soft_body_set_self_collision(world, u32::MAX, 0.2, 0.0),
            Bool::FALSE
        );
        // 非有限 → False。
        assert_eq!(
            soft_body_set_self_collision(world, id, f64::NAN, 0.0),
            Bool::FALSE
        );
        // radius <= 0 → False (且保持关闭)。
        assert_eq!(
            soft_body_set_self_collision(world, id, 0.0, 0.0),
            Bool::FALSE
        );
        // stiffness < 0 → False。
        assert_eq!(
            soft_body_set_self_collision(world, id, 0.2, -1.0),
            Bool::FALSE
        );
        // 合法 → True，且未 panic。
        assert_eq!(
            soft_body_set_self_collision(world, id, 0.2, 0.0),
            Bool::TRUE
        );

        world_destroy(world);
    }

    // ── Phase 13: 运行时改弹簧刚度 → 高刚度下垂更少 ──────────────────────────────
    #[test]
    fn soft_body_set_spring_stiffness_reduces_sag() {
        // MassSpring 链:固定端点 + 3 个自由点,单条弹簧(index 0)把端点连到链。
        // 低刚度下垂多,运行时改高刚度后下垂更少。
        fn max_sag(stiff: f64) -> f64 {
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
            assert_ne!(id, u32::MAX);
            let _anc = soft_body_add_particle(world, id, 0.0, 5.0, 0.0, 1.0, Bool::TRUE);
            for i in 1..4 {
                let _ = soft_body_add_particle(world, id, i as f64, 5.0, 0.0, 1.0, Bool::FALSE);
            }
            // 全链初刚度 200。
            for i in 0..3 {
                let _ = soft_body_add_spring(world, id, i, i + 1, 200.0, 5.0);
            }
            // 运行时把第 0 条弹簧改为给定刚度。
            assert_eq!(
                soft_body_set_spring_stiffness(world, id, 0, stiff),
                Bool::TRUE
            );
            for _ in 0..200 {
                world_step(world, 1.0 / 60.0);
            }
            // 读自由点最低 y(锚点在 y=5,重力向下,越软越低)。
            let count = soft_body_particle_count(world, id);
            let mut pos = vec![Vec3::default(); count as usize];
            let _r =
                soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
            let mut min_y = f64::MAX;
            for p in &pos {
                if p.y < min_y {
                    min_y = p.y;
                }
            }
            world_destroy(world);
            min_y
        }

        let sag_soft = max_sag(20.0); // 软
        let sag_stiff = max_sag(5000.0); // 硬
        assert!(
            sag_stiff > sag_soft,
            "stiffer spring should hold the chain higher: soft={sag_soft} stiff={sag_stiff}"
        );

        // 非法参数:未知 id / 越界 index / 负刚度 → False。
        let world = make_world();
        assert!(!world.is_null());
        let id = soft_body_create(world, Vec3::default());
        assert_ne!(id, u32::MAX);
        assert_eq!(
            soft_body_set_spring_stiffness(world, u32::MAX, 0, 100.0),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_spring_stiffness(world, id, 999, 100.0),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_spring_stiffness(world, id, 0, -1.0),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_spring_stiffness(world, id, 0, f64::NAN),
            Bool::FALSE
        );
        world_destroy(world);
    }

    // ── Phase 13: 运行时改 XPBD 距离约束柔度 → 高柔度拉伸更多 ───────────────────
    #[test]
    fn soft_body_set_distance_constraint_compliance_increases_stretch() {
        // XPBD 水平单杆:锚点 (0,0,0),自由点 (1,0,0),重力向下(-y)把杆往下拉→拉伸。
        // compliance=0(硬)几乎不拉伸;运行时改为高柔度后明显拉伸。用 1 次迭代让柔度可见
        // (多迭代会把约束完全收敛,抹掉柔度差异)。
        fn rod_len(compliance: f64) -> f64 {
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
            assert_ne!(id, u32::MAX);
            let _anc = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::TRUE);
            let _free = soft_body_add_particle(world, id, 1.0, 0.0, 0.0, 1.0, Bool::FALSE);
            // 单迭代:让 compliance 在稳态下可见。
            assert_eq!(soft_body_configure_solver(world, id, 1, 1, 0.0), Bool::TRUE);
            assert_eq!(
                soft_body_add_distance_constraint(world, id, 0, 1, 0.0),
                Bool::TRUE
            );
            // 运行时改柔度(初始 0 → 给定值)。
            assert_eq!(
                soft_body_set_distance_constraint_compliance(world, id, 0, compliance),
                Bool::TRUE
            );
            for _ in 0..300 {
                world_step(world, 1.0 / 60.0);
            }
            // 自由点相对锚点的欧氏距离 = 当前杆长。
            let count = soft_body_particle_count(world, id);
            let mut pos = vec![Vec3::default(); count as usize];
            let _r =
                soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), count);
            let len = ((pos[1].x - pos[0].x).powi(2)
                + (pos[1].y - pos[0].y).powi(2)
                + (pos[1].z - pos[0].z).powi(2))
            .sqrt();
            world_destroy(world);
            len
        }

        let len_rigid = rod_len(0.0);
        let len_soft = rod_len(50.0);
        assert!(
            len_soft > len_rigid + 0.01,
            "higher compliance should stretch more: rigid={len_rigid} soft={len_soft}"
        );

        // 非法参数:未知 id / 越界 index / 负柔度 → False。
        let world = make_world();
        assert!(!world.is_null());
        let id = soft_body_create(world, Vec3::default());
        assert_ne!(id, u32::MAX);
        assert_eq!(
            soft_body_set_distance_constraint_compliance(world, u32::MAX, 0, 0.0),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_distance_constraint_compliance(world, id, 999, 0.0),
            Bool::FALSE
        );
        assert_eq!(
            soft_body_set_distance_constraint_compliance(world, id, 0, -1.0),
            Bool::FALSE
        );

        world_destroy(world);
    }

    // ── Phase 14: 软软碰撞 — 两个软体的自由质点互相排斥,不穿透 ──────────────────
    #[test]
    fn soft_body_cross_collision_keeps_bodies_apart() {
        // 两个软体各一个自由质点,初始间距 0.1 (< 2R=0.5),重力为 0。无软软碰撞时保持重合;
        // 开启后它们之间的间距应被推到 >= 2R。
        const R: f64 = 0.25;
        fn min_inter_body_dist(enabled: bool) -> f64 {
            let world = make_world();
            assert!(!world.is_null());
            let id_a = soft_body_create(
                world,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            );
            let id_b = soft_body_create(
                world,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            );
            assert_ne!(id_a, u32::MAX);
            assert_ne!(id_b, u32::MAX);
            assert_ne!(
                soft_body_add_particle(world, id_a, 0.0, 0.0, 0.0, 1.0, Bool::FALSE),
                u32::MAX
            );
            assert_ne!(
                soft_body_add_particle(world, id_b, 0.1, 0.0, 0.0, 1.0, Bool::FALSE),
                u32::MAX
            );
            if enabled {
                assert_eq!(
                    soft_body_set_cross_collision(world, id_a, R, 0.0),
                    Bool::TRUE
                );
                assert_eq!(
                    soft_body_set_cross_collision(world, id_b, R, 0.0),
                    Bool::TRUE
                );
            }
            for _ in 0..150 {
                world_step(world, 1.0 / 60.0);
            }
            let ca = soft_body_particle_count(world, id_a);
            let cb = soft_body_particle_count(world, id_b);
            let mut pa = vec![Vec3::default(); ca as usize];
            let mut pb = vec![Vec3::default(); cb as usize];
            let _ =
                soft_body_read_particles(world, id_a, pa.as_mut_ptr(), std::ptr::null_mut(), ca);
            let _ =
                soft_body_read_particles(world, id_b, pb.as_mut_ptr(), std::ptr::null_mut(), cb);
            let mut m = f64::MAX;
            for a in &pa {
                for b in &pb {
                    let d =
                        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt();
                    if d < m {
                        m = d;
                    }
                }
            }
            world_destroy(world);
            m
        }

        let d_off = min_inter_body_dist(false);
        let d_on = min_inter_body_dist(true);
        // 无碰撞:几乎重合(间距远小于 2R)。
        assert!(
            d_off < R,
            "no cross-collision should let them overlap, got {d_off}"
        );
        // 有碰撞:最小间距被推到接近 2R 甚至更大。
        assert!(
            d_on >= 2.0 * R - 1e-3,
            "cross-collision should keep bodies >= 2R apart: d_on={d_on} R={R}"
        );
    }

    // ── Phase 14: 软软碰撞非法/未知参数返回 False 而不 panic ──────────────────────
    #[test]
    fn soft_body_set_cross_collision_rejects_invalid_args() {
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
        let _p = soft_body_add_particle(world, id, 0.0, 0.0, 0.0, 1.0, Bool::FALSE);

        // 未知 id → False。
        assert_eq!(
            soft_body_set_cross_collision(world, u32::MAX, 0.2, 0.0),
            Bool::FALSE
        );
        // 非有限 → False。
        assert_eq!(
            soft_body_set_cross_collision(world, id, f64::NAN, 0.0),
            Bool::FALSE
        );
        // radius <= 0 → False。
        assert_eq!(
            soft_body_set_cross_collision(world, id, 0.0, 0.0),
            Bool::FALSE
        );
        // stiffness < 0 → False。
        assert_eq!(
            soft_body_set_cross_collision(world, id, 0.2, -1.0),
            Bool::FALSE
        );
        // 合法 → True。
        assert_eq!(
            soft_body_set_cross_collision(world, id, 0.2, 0.0),
            Bool::TRUE
        );

        world_destroy(world);
    }

    // ── Phase 15: 双向耦合 — 软体(经 proxy collider)撞动态刚体,刚体被反推且软体不穿透 ──
    #[test]
    fn soft_body_two_way_coupling_pushes_rigid_body() {
        // 重动态球 R(质量 1.0)悬于 y=3.0;下方一个软体质点 P(质量 0.5, collide, 半径 0.4)从
        // y=4.5 落下砸向 R。Phase 5f 的 proxy 是 dynamic 刚体 + 读回;Phase 15 又把 proxy
        // collider 密度置 0,使 proxy 质量恰等于质点质量,动量传递对称。验证:
        //  (1) R 被 P 反推而明显下移(R.y < 起始 - eps);
        //  (2) P 没有穿透 R(P.y >= R.y - 半径和,即 P 停在 R 上方而不是穿过);
        //  (3) P 自己下落了(P.y < 起始)。
        let world = world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        });
        assert!(!world.is_null());

        // 动态刚体 R:球半径 0.5,质量 1.0,置于 (0, 3.0, 0)。
        let rb = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        assert!(!rb.is_null());
        rigid_body_builder_set_translation(
            rb,
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: 0.0,
            },
        );
        rigid_body_builder_set_additional_mass_properties(
            rb,
            Vec3::default(),
            1.0,
            Vec3::default(),
        );
        let body_ptr = rigid_body_builder_build(rb);
        assert!(!body_ptr.is_null());
        let rh = world_insert_rigid_body(world, body_ptr);
        assert_ne!(rh, 0);
        let cb = collider_builder_create_sphere(Sphere {
            center: Vec3::default(),
            radius: 0.5,
        });
        assert!(!cb.is_null());
        let cbuilt = collider_builder_build(cb);
        assert!(!cbuilt.is_null());
        let _ch: ColliderHandleRaw = world_insert_collider_with_parent(world, cbuilt, rh);

        // 软体 P:单自由质点,质量 0.5,半径 0.4,从 y=4.0 落下(距 R 顶端仅 0.1),开启 collide。
        let id = soft_body_create(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_ne!(id, u32::MAX);
        assert_ne!(
            soft_body_add_particle(world, id, 0.0, 4.0, 0.0, 0.5, Bool::FALSE),
            u32::MAX
        );
        // 软体有独立的 gravity 字段(由 world_step 复制世界重力),必须显式设定,P 才会下落。
        soft_body_set_gravity(
            world,
            id,
            Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        );
        assert_eq!(
            soft_body_enable_collision(world, id, 0.4, Bool::TRUE),
            Bool::TRUE
        );

        let r_y0 = 3.0;
        let p_y0 = 4.0;
        for _ in 0..60 {
            world_step(world, 1.0 / 60.0);
        }

        let r_pos = rigid_body_get_translation(world, rh);
        let pc = soft_body_particle_count(world, id);
        let mut pp = vec![Vec3::default(); pc as usize];
        let _ = soft_body_read_particles(world, id, pp.as_mut_ptr(), std::ptr::null_mut(), pc);
        // 单质点:取 y 最小者(即该粒子)。
        let p_y = pp.iter().map(|v| v.y).fold(f64::INFINITY, f64::min);

        // (1) P 确实下落并撞上 R(p_y 明显小于起始)。
        assert!(
            p_y < p_y0 - 0.3,
            "soft particle should have fallen onto R, p_y={p_y}"
        );
        // (2) 双向耦合:R 被 P 反推而明显下移(r_y < 起始)。
        assert!(
            r_pos.y < r_y0 - 0.2,
            "two-way coupling: rigid body must be pushed down by the soft body, r_y={}",
            r_pos.y
        );
        // (3) P 停在 R 上方而不是穿透:R 球(半径 0.5)与质点球(半径 0.4)中心距应 >= 半径和。
        //     同 x/z 时中心距 = |gap|(gap = R.y - P.y);gap 为负表示 P 在 R 上方接触。
        //     若 P 穿透到 R 下方,则 gap 会变成明显正值(>= ~0.9)。故断言 gap 不显著为正。
        let gap = r_pos.y - p_y;
        assert!(
            gap <= (0.4 + 0.5) - 0.2,
            "soft particle must rest on top of (not tunnel through) the rigid body: gap={gap} (must not be >= ~0.7)"
        );

        world_destroy(world);
    }
}
