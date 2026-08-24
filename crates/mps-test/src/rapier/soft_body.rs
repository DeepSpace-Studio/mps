#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::{RigidBodyHandleRaw, Vec3, WorldHandle};
    use mps_core::rapier::soft_body::{soft_chain_create, soft_chain_node_handles};
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
}
