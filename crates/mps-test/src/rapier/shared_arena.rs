#[cfg(test)]
mod tests {
    use mps_core::rapier::shared_arena::*;
    use mps_core::rapier::ffi::*;

    // Header offsets (bytes) — must match the layout doc in shared_arena.rs
    const OFF_BODY_COUNT: usize = 32;
    const OFF_EVENT_COUNT: usize = 40;
    const OFF_CMD_WRITE: usize = 44;
    const OFF_BODY_HANDLE_MAP: usize = 64;
    const OFF_FORCE_REPORT: usize = 72;
    const OFF_INTEGRATION_PARAMS: usize = 80;
    const OFF_FORCE_SUMMARY: usize = 88;
    const OFF_CMD_RING: usize = 96;
    const OFF_EVENT_RING: usize = 104;
    const OFF_COLLIDER_SLOTS: usize = 112;

    #[test]
    fn arena_create_and_drop() {
        let arena = SharedPhysicsArena::new(16, 32, 64, 128).expect("arena creation failed");
        assert!(!arena.as_ptr().is_null());
        let expected_size = HEADER_SIZE
            + 16 * BODY_SLOT_STRIDE as usize
            + 32 * COLLIDER_SLOT_STRIDE as usize
            + 16 * 8 // body_handle_map
            + 32 * 32 // force_report (per-type breakdown)
            + INTEGRATION_PARAMS_SIZE
            + FORCE_SUMMARY_SIZE
            + 128 * CMD_SLOT_STRIDE as usize
            + 64 * EVENT_SLOT_STRIDE as usize;
        assert_eq!(arena.size(), expected_size,
            "expected {} got {}", expected_size, arena.size());

        // Check header magic
        let magic = arena.header_u64(0);
        assert_eq!(magic, ARENA_MAGIC);

        let version = arena.header_u32(8);
        assert_eq!(version, ARENA_VERSION);

        // Region offsets are written into the header, strictly ordered, and
        // all body data starts only after the 128-byte header.
        let handle_map = arena.header_u64(OFF_BODY_HANDLE_MAP) as usize;
        let force_report = arena.header_u64(OFF_FORCE_REPORT) as usize;
        let integration = arena.header_u64(OFF_INTEGRATION_PARAMS) as usize;
        let summary = arena.header_u64(OFF_FORCE_SUMMARY) as usize;
        let cmd_ring = arena.header_u64(OFF_CMD_RING) as usize;
        let event_ring = arena.header_u64(OFF_EVENT_RING) as usize;
        let collider_slots = arena.header_u64(OFF_COLLIDER_SLOTS) as usize;

        assert_eq!(collider_slots, HEADER_SIZE + 16 * BODY_SLOT_STRIDE as usize);
        assert!(HEADER_SIZE < collider_slots);
        assert!(collider_slots < handle_map);
        assert!(handle_map < force_report);
        assert!(force_report < integration);
        assert_eq!(integration + INTEGRATION_PARAMS_SIZE, summary);
        assert_eq!(summary + FORCE_SUMMARY_SIZE, cmd_ring);
        assert!(cmd_ring < event_ring);
        assert_eq!(event_ring + 64 * EVENT_SLOT_STRIDE as usize, arena.size());
    }

    #[test]
    fn arena_rejects_excessive_capacities() {
        assert!(SharedPhysicsArena::new(MAX_ARENA_BODIES + 1, 8, 8, 8).is_none());
        assert!(SharedPhysicsArena::new(8, MAX_ARENA_COLLIDERS + 1, 8, 8).is_none());
        assert!(SharedPhysicsArena::new(8, 8, MAX_ARENA_EVENTS + 1, 8).is_none());
        assert!(SharedPhysicsArena::new(8, 8, 8, MAX_ARENA_COMMANDS + 1).is_none());
        assert!(SharedPhysicsArena::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX).is_none());
    }

    #[test]
    fn body_flush_and_readback() {
        let arena = SharedPhysicsArena::new(8, 0, 0, 0).expect("arena creation failed");

        arena.flush_body(0, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3, 0.01, 0.02, 0.03, 0, 0, 42);

        // Read back from the slot
        let slot = arena.body_slot_ptr(0);
        unsafe {
            let generation_val = (slot as *const u64).read_unaligned();
            assert!(generation_val > 0, "generation should be non-zero");
            assert_eq!(generation_val & 1, 0, "generation should be even (stable)");

            let pos_x = (slot.add(8) as *const f64).read_unaligned();
            assert!((pos_x - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn command_drain_empty() {
        let arena = SharedPhysicsArena::new(8, 0, 0, 16).expect("arena creation failed");
        let cmds = arena.drain_commands();
        assert!(cmds.is_empty());
    }

    /// Simulate the Java producer: write a command slot and bump the write
    /// index in the shared header — `drain_commands` must see it.
    #[test]
    fn command_roundtrip_via_header_write_index() {
        let arena = SharedPhysicsArena::new(8, 8, 8, 16).expect("arena creation failed");
        let cmd_ring = arena.header_u64(OFF_CMD_RING) as usize;

        // Java side: write a SetVelocity command into slot 0, then bump the
        // write index at header offset 44.
        unsafe {
            let slot = arena.as_ptr().cast_mut().add(cmd_ring);
            (slot as *mut u32).write_unaligned(3); // SetVelocity
            (slot.add(4) as *mut u32).write_unaligned(0); // body_index
            (slot.add(8) as *mut f64).write_unaligned(5.0);
            (slot.add(16) as *mut f64).write_unaligned(6.0);
            (slot.add(24) as *mut f64).write_unaligned(7.0);
            (arena.as_ptr().cast_mut().add(OFF_CMD_WRITE) as *mut u32).write_unaligned(1);
        }

        let cmds = arena.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], (3, 0, 5.0, 6.0, 7.0));

        // Write index is reset after draining; the ring is empty again.
        assert_eq!(arena.header_u32(OFF_CMD_WRITE), 0);
        assert!(arena.drain_commands().is_empty());
    }

    /// `flush_integration_params` / `flush_force_report` must write into their
    /// own layout regions — never into the header or body slot 0.
    #[test]
    fn flush_regions_do_not_clobber_header_or_body_slots() {
        let arena = SharedPhysicsArena::new(8, 8, 8, 8).expect("arena creation failed");

        // Snapshot header fields that were previously overwritten.
        let handle_map_before = arena.header_u64(OFF_BODY_HANDLE_MAP);
        let force_report_before = arena.header_u64(OFF_FORCE_REPORT);

        // Populate body slot 0 with known data.
        arena.flush_body(0, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3, 0.01, 0.02, 0.03, 0, 0, 42);
        let gen_before = unsafe { (arena.body_slot_ptr(0) as *const u64).read_unaligned() };

        let gravity = rapier3d::prelude::Vector::new(0.0, -9.81, 0.0);
        arena.flush_integration_params(1.0 / 60.0, 4, 2, &gravity);
        arena.flush_force_report(
            123.5,
            &Vec3 { x: 10.0, y: 20.0, z: 30.0 },
            &Vec3 { x: -1.0, y: -2.0, z: -3.0 },
            7,
            9,
        );

        // Header offset fields must be untouched.
        assert_eq!(arena.header_u64(OFF_BODY_HANDLE_MAP), handle_map_before);
        assert_eq!(arena.header_u64(OFF_FORCE_REPORT), force_report_before);

        // Body slot 0 must be untouched (generation not bumped, position intact).
        let slot = arena.body_slot_ptr(0);
        unsafe {
            let gen_after = (slot as *const u64).read_unaligned();
            assert_eq!(gen_after, gen_before);
            assert!(((slot.add(8) as *const f64).read_unaligned() - 1.0).abs() < 1e-10);
            assert!(((slot.add(24) as *const f64).read_unaligned() - 3.0).abs() < 1e-10);
        }

        // The values landed in their own regions instead.
        let ip = arena.header_u64(OFF_INTEGRATION_PARAMS) as usize;
        let fs = arena.header_u64(OFF_FORCE_SUMMARY) as usize;
        unsafe {
            let base = arena.as_ptr().add(ip);
            assert!(((base as *const f64).read_unaligned() - 1.0 / 60.0).abs() < 1e-12);
            assert_eq!((base.add(8) as *const u32).read_unaligned(), 4);
            assert_eq!((base.add(12) as *const u32).read_unaligned(), 2);
            assert!(((base.add(24) as *const f64).read_unaligned() + 9.81).abs() < 1e-10);

            let base = arena.as_ptr().add(fs);
            assert!(((base as *const f64).read_unaligned() - 123.5).abs() < 1e-10);
            assert!(((base.add(8) as *const f64).read_unaligned() - 10.0).abs() < 1e-10);
            assert!(((base.add(40) as *const f64).read_unaligned() + 2.0).abs() < 1e-10);
            assert_eq!((base.add(56) as *const u32).read_unaligned(), 7);
            assert_eq!((base.add(60) as *const u32).read_unaligned(), 9);
        }
    }

    #[test]
    fn event_push_and_reset() {
        let arena = SharedPhysicsArena::new(0, 0, 8, 0).expect("arena creation failed");

        arena.push_collision_event(true, 1, 2, false, false);
        arena.push_collision_event(false, 3, 4, true, true);

        // Event count in header should be 2
        let count = arena.header_u32(OFF_EVENT_COUNT);
        assert_eq!(count, 2);

        arena.reset_event_ring();
        let count = arena.header_u32(OFF_EVENT_COUNT);
        assert_eq!(count, 0);
    }

    // -----------------------------------------------------------------------
    // World-level integration: arena + world_step
    // -----------------------------------------------------------------------

    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_set_additional_mass,
        rigid_body_builder_set_translation, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{
        world_create, world_create_shared_arena, world_destroy, world_destroy_shared_arena,
        world_step,
    };

    struct ArenaView {
        ptr: *mut u8,
    }

    impl ArenaView {
        fn u32_at(&self, offset: usize) -> u32 {
            unsafe { (self.ptr.add(offset) as *const u32).read_unaligned() }
        }
        fn u64_at(&self, offset: usize) -> u64 {
            unsafe { (self.ptr.add(offset) as *const u64).read_unaligned() }
        }
        fn f64_at(&self, offset: usize) -> f64 {
            unsafe { (self.ptr.add(offset) as *const f64).read_unaligned() }
        }
        fn write_u32(&self, offset: usize, value: u32) {
            unsafe { (self.ptr.add(offset) as *mut u32).write_unaligned(value) };
        }
        fn write_f64(&self, offset: usize, value: f64) {
            unsafe { (self.ptr.add(offset) as *mut f64).write_unaligned(value) };
        }
        /// Byte offset of body slot `index`.
        fn body_slot(&self, index: usize) -> usize {
            HEADER_SIZE + index * BODY_SLOT_STRIDE as usize
        }
    }

    /// Create world + arena + one dynamic body at (1, 2, 3); step `frames`
    /// times; verify header layout fields survive every step and body slot 0
    /// carries sane generation/position data.
    #[test]
    fn world_step_preserves_arena_layout() {
        let world = world_create(Vec3 { x: 0.0, y: -9.81, z: 0.0 });
        assert!(!world.is_null());

        let mut addr = 0u64;
        let mut size = 0u64;
        assert_eq!(
            world_create_shared_arena(world, 8, 8, 64, 64, &mut addr, &mut size),
            Bool::TRUE
        );
        assert!(addr != 0 && size != 0);
        let arena = ArenaView { ptr: addr as *mut u8 };

        // Snapshot the layout fields written by `new()`.
        let handle_map = arena.u64_at(OFF_BODY_HANDLE_MAP);
        let force_report = arena.u64_at(OFF_FORCE_REPORT);
        let integration = arena.u64_at(OFF_INTEGRATION_PARAMS);
        let summary = arena.u64_at(OFF_FORCE_SUMMARY);
        let cmd_ring = arena.u64_at(OFF_CMD_RING);
        assert!(handle_map != 0 && force_report != 0);
        assert!(integration != 0 && summary != 0 && cmd_ring != 0);

        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_translation(builder, Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        // A collider-less body has zero mass; give it mass so gravity acts.
        rigid_body_builder_set_additional_mass(builder, 5.0);
        let body = rigid_body_builder_build(builder);
        assert_ne!(world_insert_rigid_body(world, body), 0);

        let dt = 1.0 / 60.0;
        for _ in 0..3 {
            world_step(world, dt);
        }

        // Header layout fields must not have been overwritten by per-frame
        // flushes (regression: flush_force_report used to clobber slot 0 and
        // flush_integration_params the offset fields).
        assert_eq!(arena.u64_at(OFF_BODY_HANDLE_MAP), handle_map);
        assert_eq!(arena.u64_at(OFF_FORCE_REPORT), force_report);
        assert_eq!(arena.u64_at(OFF_INTEGRATION_PARAMS), integration);
        assert_eq!(arena.u64_at(OFF_FORCE_SUMMARY), summary);
        assert_eq!(arena.u64_at(OFF_CMD_RING), cmd_ring);

        // Body slot 0: generation even & non-zero, position written.
        assert_eq!(arena.u32_at(OFF_BODY_COUNT), 1);
        let slot = arena.body_slot(0);
        let generation = arena.u64_at(slot);
        assert!(generation > 0 && generation & 1 == 0, "generation {generation} not stable");
        assert!((arena.f64_at(slot + 8) - 1.0).abs() < 1e-9, "pos_x drifted");
        assert!(arena.f64_at(slot + 16) < 2.0, "gravity should have pulled pos_y down");
        assert!((arena.f64_at(slot + 24) - 3.0).abs() < 1e-9, "pos_z drifted");

        // Integration params region carries the current dt/gravity.
        let ip = integration as usize;
        assert!((arena.f64_at(ip) - dt).abs() < 1e-12);
        assert!((arena.f64_at(ip + 24) + 9.81).abs() < 1e-9);

        world_destroy_shared_arena(world);
        world_destroy(world);
    }

    /// Full command round-trip through shared memory: write a SetVelocity
    /// command exactly the way the Java side does, step, and verify the
    /// command was consumed and applied.
    #[test]
    fn world_step_applies_shared_memory_command() {
        let world = world_create(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        assert!(!world.is_null());

        let mut addr = 0u64;
        let mut size = 0u64;
        assert_eq!(
            world_create_shared_arena(world, 8, 8, 64, 64, &mut addr, &mut size),
            Bool::TRUE
        );
        let arena = ArenaView { ptr: addr as *mut u8 };

        let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        rigid_body_builder_set_translation(builder, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        let body = rigid_body_builder_build(builder);
        assert_ne!(world_insert_rigid_body(world, body), 0);

        // Java side: write a SetVelocity(5, 6, 7) command for body 0 and bump
        // the write index at header offset 44.
        let cmd_ring = arena.u64_at(OFF_CMD_RING) as usize;
        arena.write_u32(cmd_ring, 3); // SetVelocity
        arena.write_u32(cmd_ring + 4, 0); // body_index
        arena.write_f64(cmd_ring + 8, 5.0);
        arena.write_f64(cmd_ring + 16, 6.0);
        arena.write_f64(cmd_ring + 24, 7.0);
        arena.write_u32(OFF_CMD_WRITE, 1);

        world_step(world, 1.0 / 60.0);

        // Command consumed: write index reset, velocity applied to body slot 0.
        assert_eq!(arena.u32_at(OFF_CMD_WRITE), 0);
        let slot = arena.body_slot(0);
        assert!((arena.f64_at(slot + 32) - 5.0).abs() < 1e-9, "vel_x not applied");
        assert!((arena.f64_at(slot + 40) - 6.0).abs() < 1e-9, "vel_y not applied");
        assert!((arena.f64_at(slot + 48) - 7.0).abs() < 1e-9, "vel_z not applied");

        world_destroy_shared_arena(world);
        world_destroy(world);
    }

    #[test]
    fn world_create_shared_arena_rejects_invalid_capacities() {
        let world = world_create(Vec3 { x: 0.0, y: -9.81, z: 0.0 });
        assert!(!world.is_null());

        let mut addr = 0u64;
        let mut size = 0u64;

        // Zero capacities (including max_colliders) are rejected.
        assert_eq!(
            world_create_shared_arena(world, 0, 8, 8, 8, &mut addr, &mut size),
            Bool::FALSE
        );
        assert_eq!(
            world_create_shared_arena(world, 8, 0, 8, 8, &mut addr, &mut size),
            Bool::FALSE
        );
        assert_eq!(
            world_create_shared_arena(world, 8, 8, 0, 8, &mut addr, &mut size),
            Bool::FALSE
        );
        assert_eq!(
            world_create_shared_arena(world, 8, 8, 8, 0, &mut addr, &mut size),
            Bool::FALSE
        );

        // Absurd capacities are rejected instead of panicking / over-allocating.
        assert_eq!(
            world_create_shared_arena(world, u32::MAX, 8, 8, 8, &mut addr, &mut size),
            Bool::FALSE
        );
        assert_eq!(addr, 0);

        // A sane request still works.
        assert_eq!(
            world_create_shared_arena(world, 8, 8, 64, 64, &mut addr, &mut size),
            Bool::TRUE
        );
        assert!(addr != 0 && size != 0);

        world_destroy_shared_arena(world);
        world_destroy(world);
    }
}
