use super::core::*;
// ---------------------------------------------------------------------------
// Event callback registry — zero-FFI-roundtrip event dispatch
// ---------------------------------------------------------------------------

/// Opaque handle returned by `world_register_*_callback` — used to unregister.
pub type EventCallbackHandle = u64;

/// Callback signature: called from the Rapier physics step for each event.
/// Must be signal-safe (no Java upcalls, no locks from callback context).
pub type CollisionEventCallback = Option<
    unsafe extern "C" fn(
        world: *const std::ffi::c_void,
        event: *const CollisionEventRecord,
        user_data: *mut std::ffi::c_void,
    ),
>;

/// Callback signature for contact-force events.
pub type ContactForceEventCallback = Option<
    unsafe extern "C" fn(
        world: *const std::ffi::c_void,
        event: *const ContactForceEventRecord,
        user_data: *mut std::ffi::c_void,
    ),
>;

/// Dispatch mode for cached event delivery.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum EventDispatchMode {
    /// Events stay in the in-memory ring buffer; Java polls via existing APIs.
    #[default]
    Poll = 0,
    /// Events are dispatched through registered callbacks during `world_step`.
    Callback = 1,
    /// Events go to both the ring buffer and registered callbacks.
    Both = 2,
}

/// Pre-allocated ring buffer for zero-allocation event caching.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EventRingBufferStats {
    /// Total capacity of the ring buffer (in event records).
    pub capacity: u32,
    /// Number of events currently in the buffer.
    pub len: u32,
    /// Number of events dropped due to buffer overflow since last reset.
    pub dropped: u32,
    /// Whether the buffer has wrapped around (overwritten old events).
    pub wrapped: Bool,
}

impl Default for EventRingBufferStats {
    fn default() -> Self {
        Self {
            capacity: 0,
            len: 0,
            dropped: 0,
            wrapped: Bool::FALSE,
        }
    }
}
