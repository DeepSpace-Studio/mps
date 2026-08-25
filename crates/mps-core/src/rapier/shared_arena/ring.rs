//! `shared_arena::ring` submodule — lock-free SPSC command ring (Java→Rust)
//! and event ring (Rust→Java), plus the `flush_events_from_handler` bridge
//! from [`crate::rapier::events::CollectingEventHandler`].
//!
//! Split out of the original 1028-line `shared_arena.rs` per OPTIMIZATION.md
//! §N5.  Memory orderings here are the load-bearing bit: command drains use
//! plain `read_unaligned` (Java has already published with `Release` on its
//! `cmd_write` index bump, and Rust reads after observing a non-zero
//! `OFF_CMD_WRITE`); event pushes use `Release` on `event_write` and `Acquire`
//! on `event_read`.  Miri should re-validate the SPSC tests after any change
//! to this file — see OPTIMIZATION.md §4 risk note.

use std::sync::atomic::Ordering;

use super::{CMD_SLOT_STRIDE, EVENT_SLOT_STRIDE, OFF_CMD_WRITE};

impl super::SharedPhysicsArena {
    // -----------------------------------------------------------------------
    // Command ring (Java writes, Rust reads)
    // -----------------------------------------------------------------------

    fn cmd_slot_ptr(&self, index: u32) -> *mut u8 {
        let wrapped = index % self.max_commands;
        unsafe {
            self.ptr
                .add(self.cmd_ring_offset + wrapped as usize * CMD_SLOT_STRIDE as usize)
        }
    }

    /// Drain all pending commands from the command ring.
    ///
    /// Returns a Vec of (cmd_type, body_index, arg0, arg1, arg2) tuples.
    /// Called at the beginning of `world_step`.
    ///
    /// Protocol: Java (sole producer) writes a command slot and then bumps the
    /// write index at header offset `OFF_CMD_WRITE`; Rust (sole consumer)
    /// drains `[0, write)` here and resets the write index to 0.
    pub fn drain_commands(&self) -> Vec<(u32, u32, f64, f64, f64)> {
        let mut commands = Vec::new();
        // Clamp to ring capacity: a buggy/overflowing producer must not make
        // us read past the command ring into the event ring.
        let write = self.header_u32(OFF_CMD_WRITE).min(self.max_commands);

        for read in 0..write {
            let slot = self.cmd_slot_ptr(read);
            let cmd_type = unsafe { (slot as *const u32).read_unaligned() };
            let body_index = unsafe { (slot.add(4) as *const u32).read_unaligned() };
            let arg0 = unsafe { (slot.add(8) as *const f64).read_unaligned() };
            let arg1 = unsafe { (slot.add(16) as *const f64).read_unaligned() };
            let arg2 = unsafe { (slot.add(24) as *const f64).read_unaligned() };

            commands.push((cmd_type, body_index, arg0, arg1, arg2));
        }

        // Reset the write index: every drained frame starts with an empty ring.
        self.set_header_u32(OFF_CMD_WRITE, 0);

        commands
    }

    // -----------------------------------------------------------------------
    // Event ring (Rust writes, Java reads)
    // -----------------------------------------------------------------------

    fn event_slot_ptr(&self, index: u32) -> *mut u8 {
        let wrapped = index % self.max_events;
        unsafe {
            self.ptr
                .add(self.event_ring_offset + wrapped as usize * EVENT_SLOT_STRIDE as usize)
        }
    }

    /// Push a collision event to the event ring.
    pub fn push_collision_event(
        &self,
        started: bool,
        collider1: u32,
        collider2: u32,
        sensor: bool,
        removed: bool,
    ) {
        let write = self.event_write.load(Ordering::Relaxed);
        let read = self.event_read.load(Ordering::Acquire);

        // Ring full check
        if write.wrapping_sub(read) >= self.max_events {
            return; // drop event (ring full)
        }

        let slot = self.event_slot_ptr(write);

        let flags: u32 = if sensor { 1 } else { 0 } | if removed { 2 } else { 0 };

        unsafe {
            (slot as *mut u32).write_unaligned(if started { 0 } else { 1 });
            (slot.add(4) as *mut u32).write_unaligned(collider1);
            (slot.add(8) as *mut u32).write_unaligned(collider2);
            (slot.add(12) as *mut u32).write_unaligned(flags);
            // Zero out force fields
            (slot.add(16) as *mut f64).write_unaligned(0.0);
            (slot.add(24) as *mut f64).write_unaligned(0.0);
            (slot.add(32) as *mut f64).write_unaligned(0.0);
            (slot.add(40) as *mut f64).write_unaligned(0.0);
            (slot.add(48) as *mut f64).write_unaligned(0.0);
            (slot.add(56) as *mut f64).write_unaligned(0.0);
        }

        self.event_write
            .store(write.wrapping_add(1), Ordering::Release);
        // Update header event count
        let count = write.wrapping_add(1).wrapping_sub(read);
        self.set_header_u32(40, count.min(self.max_events));
    }

    /// Push a contact force event to the event ring.
    pub fn push_contact_force_event(
        &self,
        collider1: u32,
        collider2: u32,
        total_force_x: f64,
        total_force_y: f64,
        total_force_z: f64,
        total_force_mag: f64,
        max_force_x: f64,
        max_force_y: f64,
        _max_force_z: f64,
    ) {
        let write = self.event_write.load(Ordering::Relaxed);
        let read = self.event_read.load(Ordering::Acquire);

        if write.wrapping_sub(read) >= self.max_events {
            return;
        }

        let slot = self.event_slot_ptr(write);

        unsafe {
            (slot as *mut u32).write_unaligned(2); // ContactForce
            (slot.add(4) as *mut u32).write_unaligned(collider1);
            (slot.add(8) as *mut u32).write_unaligned(collider2);
            (slot.add(12) as *mut u32).write_unaligned(0);
            (slot.add(16) as *mut f64).write_unaligned(total_force_x);
            (slot.add(24) as *mut f64).write_unaligned(total_force_y);
            (slot.add(32) as *mut f64).write_unaligned(total_force_z);
            (slot.add(40) as *mut f64).write_unaligned(total_force_mag);
            (slot.add(48) as *mut f64).write_unaligned(max_force_x);
            (slot.add(56) as *mut f64).write_unaligned(max_force_y);
        }

        self.event_write
            .store(write.wrapping_add(1), Ordering::Release);
        let count = write.wrapping_add(1).wrapping_sub(read);
        self.set_header_u32(40, count.min(self.max_events));
    }

    /// Reset event ring (called after Java drains events).
    pub fn reset_event_ring(&self) {
        let write = self.event_write.load(Ordering::Relaxed);
        self.event_read.store(write, Ordering::Release);
        self.set_header_u32(40, 0);
    }

    // -----------------------------------------------------------------------
    // Full flush after world_step
    // -----------------------------------------------------------------------

    /// Flush collision and contact-force events from the event handler to the arena event ring.
    ///
    /// Called after `world_step` so Java can read events zero-JNI.
    pub(crate) fn flush_events_from_handler(
        &self,
        events: &std::sync::Arc<crate::rapier::events::CollectingEventHandler>,
    ) {
        let mut evt_count = 0u32;

        // Drain collision events
        let col_count = events.collision_event_count();
        for i in 0..col_count {
            if let Some(evt) = events.collision_event(i) {
                let collider1 = (evt.collider1 & 0xFFFF_FFFF) as u32;
                let collider2 = (evt.collider2 & 0xFFFF_FFFF) as u32;
                let started = evt.started.0 != 0;
                let sensor = evt.sensor.0 != 0;
                let removed = evt.removed.0 != 0;
                self.push_collision_event(started, collider1, collider2, sensor, removed);
                evt_count += 1;
            }
        }

        // Drain contact force events
        let cf_count = events.contact_force_event_count();
        for i in 0..cf_count {
            if let Some(evt) = events.contact_force_event(i) {
                let collider1 = (evt.collider1 & 0xFFFF_FFFF) as u32;
                let collider2 = (evt.collider2 & 0xFFFF_FFFF) as u32;
                self.push_contact_force_event(
                    collider1,
                    collider2,
                    evt.total_force.x,
                    evt.total_force.y,
                    evt.total_force.z,
                    evt.total_force_magnitude,
                    evt.max_force_direction.x,
                    evt.max_force_direction.y,
                    evt.max_force_direction.z,
                );
                evt_count += 1;
            }
        }

        // Update event count in header
        self.set_header_u32(40, evt_count);
    }
}
