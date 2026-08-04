//! `shared_arena::header` submodule — header field accessors, flag
//! manipulation, integration-params and force-summary region writes.
//!
//! Split out of the original 1028-line `shared_arena.rs` per OPTIMIZATION.md
//! §N5.  All methods here extend [`super::SharedPhysicsArena`] and address
//! raw header bytes via the `OFF_*` constants re-exported from `super`.

use std::sync::atomic::{AtomicU32, Ordering};

impl super::SharedPhysicsArena {
    // -----------------------------------------------------------------------
    // Header accessors
    // -----------------------------------------------------------------------

    pub fn header_u32(&self, offset: usize) -> u32 {
        unsafe { (self.ptr.add(offset) as *const u32).read_unaligned() }
    }

    pub(super) fn set_header_u32(&self, offset: usize, value: u32) {
        unsafe {
            (self.ptr.add(offset) as *mut u32).write_unaligned(value);
        }
    }

    pub fn header_u64(&self, offset: usize) -> u64 {
        unsafe { (self.ptr.add(offset) as *const u64).read_unaligned() }
    }

    /// Set flags in the header atomically.
    pub fn set_flags(&self, flags: u32) {
        let ptr = unsafe { self.ptr.add(12) as *mut AtomicU32 };
        unsafe {
            (*ptr).fetch_or(flags, Ordering::Release);
        }
    }

    /// Clear flags in the header atomically.
    pub fn clear_flags(&self, flags: u32) {
        let ptr = unsafe { self.ptr.add(12) as *mut AtomicU32 };
        unsafe {
            (*ptr).fetch_and(!flags, Ordering::Release);
        }
    }

    // -----------------------------------------------------------------------
    // Integration parameters (zero-JNI read/write by Java)
    // -----------------------------------------------------------------------

    /// Flush integration parameters into the arena's integration_params region.
    pub fn flush_integration_params(
        &self,
        dt: f64,
        solver_iterations: u32,
        ccd_substeps: u32,
        gravity: &rapier3d::prelude::Vector,
    ) {
        let base = self.integration_params_offset;
        unsafe {
            (self.ptr.add(base) as *mut f64).write_unaligned(dt);
            (self.ptr.add(base + 8) as *mut u32).write_unaligned(solver_iterations);
            (self.ptr.add(base + 12) as *mut u32).write_unaligned(ccd_substeps);
            (self.ptr.add(base + 16) as *mut f64).write_unaligned(gravity.x);
            (self.ptr.add(base + 24) as *mut f64).write_unaligned(gravity.y);
            (self.ptr.add(base + 32) as *mut f64).write_unaligned(gravity.z);
        }
    }

    /// Flush the aggregate force summary into the arena's force_summary region.
    pub fn flush_force_report(
        &self,
        max_reynolds: f64,
        total_external_force: &crate::rapier::ffi::Vec3,
        total_drag_force: &crate::rapier::ffi::Vec3,
        drag_body_count: u32,
        ext_force_body_count: u32,
    ) {
        let base = self.force_summary_offset;
        unsafe {
            (self.ptr.add(base) as *mut f64).write_unaligned(max_reynolds);
            (self.ptr.add(base + 8) as *mut f64).write_unaligned(total_external_force.x);
            (self.ptr.add(base + 16) as *mut f64).write_unaligned(total_external_force.y);
            (self.ptr.add(base + 24) as *mut f64).write_unaligned(total_external_force.z);
            (self.ptr.add(base + 32) as *mut f64).write_unaligned(total_drag_force.x);
            (self.ptr.add(base + 40) as *mut f64).write_unaligned(total_drag_force.y);
            (self.ptr.add(base + 48) as *mut f64).write_unaligned(total_drag_force.z);
            (self.ptr.add(base + 56) as *mut u32).write_unaligned(drag_body_count);
            (self.ptr.add(base + 60) as *mut u32).write_unaligned(ext_force_body_count);
        }
    }
}
