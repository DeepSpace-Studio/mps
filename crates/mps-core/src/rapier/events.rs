use parking_lot::{Mutex, RwLock};
use rapier3d::geometry::{CollisionEvent, CollisionEventFlags, ContactPair, SolverFlags};
use rapier3d::prelude::{
    ColliderSet, ContactForceEvent, EventHandler, PhysicsHooks, Real, RigidBodySet, Vector,
};
use smallvec::SmallVec;
use std::cell::UnsafeCell;
use std::mem;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, ERR_UNSUPPORTED, clear_error, ffi_guard,
    set_error,
};
use crate::rapier::ffi::{
    AirDragLaw, Bool, CollisionEventRecord, ContactForceEventRecord, CoulombFrictionLaw,
    CustomPhysicsReport, EventDispatchMode, ExternalForceLaw, MAX_OUTPUT_CAPACITY,
    NewtonGravityLaw, WorldHandle, pack_collider_handle, vec3_finite, vec3_from_rapier,
    vec3_to_rapier,
};
use crate::rapier::math::KahanVec3;

const MAX_EVENT_RECORDS: usize = 16_384;

#[derive(Clone, Debug, Default)]
pub(crate) struct CustomPhysicsState {
    pub(crate) coulomb_friction: Option<CoulombFrictionLaw>,
    pub(crate) air_drag: Option<AirDragLaw>,
    pub(crate) external_force: Option<ExternalForceLaw>,
    pub(crate) newton_gravity: Option<NewtonGravityLaw>,
    pub(crate) last_report: CustomPhysicsReport,
}

#[derive(Default)]
pub(crate) struct CollectingEventHandler {
    collision_events: Mutex<Vec<CollisionEventRecord>>,
    contact_force_events: Mutex<Vec<ContactForceEventRecord>>,
    custom_physics: RwLock<CustomPhysicsState>,
    /// Lock-free producer cache — safe: single-producer (physics thread) writes, drains use atomics.
    producer_cache: UnsafeCell<ProducerCache>,
}

// SAFETY: CollectingEventHandler is Send + Sync under the following thread
// contract for the `producer_cache` UnsafeCell:
// * Producer: the cache is written (via `UnsafeCell`) only by the single
//   physics thread, and only while it is inside `pipeline.step` (the
//   `EventHandler` callbacks). Dispatch touches atomics (plus a Mutex for the
//   typed callback slots) and pushes into the `EventRing` SPSC rings.
// * Consumer: Java drains the rings from a single thread via explicit FFI
//   calls; drain reads synchronize with the producer through the
//   Release/Acquire atomics inside `EventRing`.
// * Registration and ring (re-)initialization take
//   `&mut *producer_cache.get()` and are therefore **init-time only**: calling
//   them concurrently with `world_step` or with each other is undefined
//   behavior — see the `# Safety` docs on the `world_init_*_event_ring` and
//   `world_register_*_callback` FFI functions.
unsafe impl Send for CollectingEventHandler {}
unsafe impl Sync for CollectingEventHandler {}

impl CollectingEventHandler {
    pub(crate) fn clear(&self) {
        self.collision_events.lock().clear();
        self.contact_force_events.lock().clear();
    }

    pub(crate) fn collision_event_count(&self) -> usize {
        self.collision_events.lock().len()
    }

    pub(crate) fn collision_event(&self, index: usize) -> Option<CollisionEventRecord> {
        self.collision_events.lock().get(index).copied()
    }

    pub(crate) fn collision_events(&self, out: &mut [CollisionEventRecord]) -> u32 {
        let events = self.collision_events.lock();
        let count = out.len().min(events.len());
        out[..count].copy_from_slice(&events[..count]);
        count as u32
    }

    pub(crate) fn contact_force_event_count(&self) -> usize {
        self.contact_force_events.lock().len()
    }

    pub(crate) fn contact_force_event(&self, index: usize) -> Option<ContactForceEventRecord> {
        self.contact_force_events.lock().get(index).copied()
    }

    pub(crate) fn contact_force_events(&self, out: &mut [ContactForceEventRecord]) -> u32 {
        let events = self.contact_force_events.lock();
        let count = out.len().min(events.len());
        out[..count].copy_from_slice(&events[..count]);
        count as u32
    }

    pub(crate) fn custom_physics(&self) -> CustomPhysicsState {
        self.custom_physics.read().clone()
    }

    pub(crate) fn set_last_custom_physics_report(&self, report: CustomPhysicsReport) {
        self.custom_physics.write().last_report = report;
    }
}

fn push_event<T>(events: &mut Vec<T>, event: T) {
    if events.len() < MAX_EVENT_RECORDS {
        events.push(event);
    }
}

// ---------------------------------------------------------------------------
// Lock-free ring buffer for zero-allocation event caching
// ---------------------------------------------------------------------------

use crate::rapier::ffi::{
    CollisionEventCallback, ContactForceEventCallback, EventCallbackHandle, EventRingBufferStats,
};

/// Single-producer (Rust physics thread), single-consumer (Java drain thread)
/// lock-free ring buffer for event records.
///
/// Thread contract: `push` may only be called from the single physics thread
/// while it is inside `world_step` (the `EventHandler` callbacks). `drain`,
/// `len`, `stats` and `clear` may run on the single consumer thread (the Java
/// drain thread) concurrently with `push` — the two sides synchronize through
/// the Release/Acquire atomics on the read/write cursors. The backing buffer
/// is allocated once at init time and never reallocated while stepping; ring
/// (re-)initialization concurrent with `push` is undefined behavior (see the
/// `# Safety` docs on `world_init_*_event_ring`).
pub(crate) struct EventRing<T> {
    buf: UnsafeCell<Box<[T]>>,
    write: AtomicU32,
    read: AtomicU32,
    dropped: AtomicU32,
}

// SAFETY: SPSC ring buffer — single producer (physics thread during
// `world_step`), single consumer (Java drain thread). The producer writes a
// slot and then publishes it with a Release store to `write`; the consumer
// loads `write` with Acquire before copying slots out, and publishes its
// progress with a Release store to `read` which the producer reads with
// Acquire. The `UnsafeCell` buffer is never reallocated after init, so the
// two sides never touch the same slot concurrently. `T: Send` is required
// because record copies cross from the producer thread to the consumer.
unsafe impl<T: Send> Send for EventRing<T> {}
unsafe impl<T: Send> Sync for EventRing<T> {}

impl<T: Copy + Default> std::fmt::Debug for EventRing<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventRing")
            .field("len", &self.len())
            .finish()
    }
}

impl<T: Copy + Default> EventRing<T> {
    fn new(capacity: u32) -> Self {
        let cap = capacity.max(1).min(MAX_OUTPUT_CAPACITY) as usize;
        Self {
            buf: UnsafeCell::new(vec![T::default(); cap].into_boxed_slice()),
            write: AtomicU32::new(0),
            read: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
        }
    }

    /// Push one event. Called from the physics thread (producer).
    fn push(&self, event: T) {
        let cap = self.buf().len() as u32;
        let w = self.write.load(Ordering::Relaxed);
        let r = self.read.load(Ordering::Acquire);
        if w.wrapping_sub(r) >= cap {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        // SAFETY: single producer — only the physics thread writes.
        unsafe {
            (*self.buf.get())[(w % cap) as usize] = event;
        }
        self.write.store(w.wrapping_add(1), Ordering::Release);
    }

    /// Drain up to `out.len()` events. Returns the number actually drained.
    fn drain(&self, out: &mut [T]) -> u32 {
        let cap = self.buf().len() as u32;
        let r = self.read.load(Ordering::Relaxed);
        let w = self.write.load(Ordering::Acquire);
        let avail = w.wrapping_sub(r).min(cap);
        let count = avail.min(out.len() as u32);
        // SAFETY: single consumer reads from indices that the producer has
        // finished writing to (Release/Acquire ordering guarantees visibility).
        let buf = unsafe { &*self.buf.get() };
        for i in 0..count {
            // u64 arithmetic: `r + i` can exceed u32::MAX on long runs and
            // would panic on overflow in debug builds.
            out[i as usize] = buf[((r as u64 + i as u64) % cap as u64) as usize];
        }
        self.read.store(r.wrapping_add(count), Ordering::Release);
        count
    }

    fn buf(&self) -> &[T] {
        unsafe { &*self.buf.get() }
    }

    fn len(&self) -> u32 {
        let w = self.write.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Relaxed);
        w.wrapping_sub(r).min(self.buf().len() as u32)
    }

    fn stats(&self) -> EventRingBufferStats {
        let cap = self.buf().len() as u32;
        let w = self.write.load(Ordering::Relaxed);
        let r = self.read.load(Ordering::Relaxed);
        let avail = w.wrapping_sub(r);
        EventRingBufferStats {
            capacity: cap,
            len: avail.min(cap),
            dropped: self.dropped.load(Ordering::Relaxed),
            wrapped: Bool::from(avail > cap),
        }
    }

    fn clear(&self) {
        let w = self.write.load(Ordering::Relaxed);
        self.read.store(w, Ordering::Release);
        self.dropped.store(0, Ordering::Relaxed);
    }
}

/// Ring buffer for `CollisionEventRecord`. See `EventRing` for the SPSC
/// thread contract.
pub(crate) type CollisionEventRing = EventRing<CollisionEventRecord>;

/// Ring buffer for `ContactForceEventRecord`. See `EventRing` for the SPSC
/// thread contract.
pub(crate) type ContactForceEventRing = EventRing<ContactForceEventRecord>;

// ---------------------------------------------------------------------------
// Callback registry — init-time registration, zero per-frame lookup
// ---------------------------------------------------------------------------

/// Typed collision-event callback function (non-null form of
/// `CollisionEventCallback`).
type CollisionEventFn = unsafe extern "C" fn(
    world: *const std::ffi::c_void,
    event: *const CollisionEventRecord,
    user_data: *mut std::ffi::c_void,
);

/// Typed contact-force-event callback function.
type ContactForceEventFn = unsafe extern "C" fn(
    world: *const std::ffi::c_void,
    event: *const ContactForceEventRecord,
    user_data: *mut std::ffi::c_void,
);

/// Convert a raw FFI `usize` into a typed collision callback.
///
/// FFI contract: `raw` must be `0` (meaning "unset") or the address of a
/// function with the exact `CollisionEventFn` signature
/// (`unsafe extern "C" fn(*const c_void, *const CollisionEventRecord, *mut c_void)`),
/// provided by the native caller and valid for the whole registration
/// lifetime. The raw-to-typed conversion happens exactly once, here, at
/// registration.
fn collision_callback_from_raw(raw: usize) -> CollisionEventCallback {
    if raw == 0 {
        None
    } else {
        // SAFETY: FFI contract (see fn docs) — the caller passed the address
        // of a function with this exact signature. A raw address cannot be
        // validated beyond the non-zero check above; a wrong signature or a
        // dangling address is caller error and invokes UB at dispatch time.
        Some(unsafe { mem::transmute::<usize, CollisionEventFn>(raw) })
    }
}

/// Convert a raw FFI `usize` into a typed contact-force callback.
///
/// Same FFI contract as `collision_callback_from_raw`: `0` means "unset",
/// otherwise `raw` must be the address of a function with the exact
/// `ContactForceEventFn` signature
/// (`unsafe extern "C" fn(*const c_void, *const ContactForceEventRecord, *mut c_void)`).
fn contact_force_callback_from_raw(raw: usize) -> ContactForceEventCallback {
    if raw == 0 {
        None
    } else {
        // SAFETY: FFI contract (see fn docs) — the caller passed the address
        // of a function with this exact signature. A raw address cannot be
        // validated beyond the non-zero check above; a wrong signature or a
        // dangling address is caller error and invokes UB at dispatch time.
        Some(unsafe { mem::transmute::<usize, ContactForceEventFn>(raw) })
    }
}

/// Registered callback + ring buffer pair for a single event type.
struct CallbackSlot<F> {
    /// Typed callback (None → unset).  Stored as the typed `Option<fn>` so the
    /// dispatch hot path never re-transmutes a raw integer.
    cb: Mutex<Option<F>>,
    user_data: std::sync::atomic::AtomicUsize, // opaque pointer, atomic
    handle: std::sync::atomic::AtomicU64,      // monotonically increasing handle
}

impl<F> Default for CallbackSlot<F> {
    fn default() -> Self {
        Self {
            cb: Mutex::new(None),
            user_data: std::sync::atomic::AtomicUsize::new(0),
            handle: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[derive(Default)]
struct ProducerCache {
    collisions: Option<CollisionEventRing>,
    contact_forces: Option<ContactForceEventRing>,
    collision_cb: CallbackSlot<CollisionEventFn>,
    contact_force_cb: CallbackSlot<ContactForceEventFn>,
    /// Raw `*const WorldHandle` of the owning world, captured at callback
    /// registration so dispatch passes the real world pointer (not null).
    world_ptr: AtomicUsize,
    dispatch_mode: std::sync::atomic::AtomicU32, // atomic: 0=Poll, 1=Callback, 2=Both
    next_handle: std::sync::atomic::AtomicU64,
}

impl ProducerCache {
    fn dispatch_mode(&self) -> EventDispatchMode {
        match self.dispatch_mode.load(Ordering::Acquire) {
            1 => EventDispatchMode::Callback,
            2 => EventDispatchMode::Both,
            _ => EventDispatchMode::Poll,
        }
    }

    fn dispatch_collision(&self, record: CollisionEventRecord) {
        let mode = self.dispatch_mode();
        match mode {
            EventDispatchMode::Poll => {} // legacy Vec handles this
            EventDispatchMode::Callback | EventDispatchMode::Both => {
                let cb = *self.collision_cb.cb.lock();
                if let Some(f) = cb {
                    let world = self.world_ptr.load(Ordering::Acquire) as *const std::ffi::c_void;
                    // SAFETY: `f` was registered as a valid `CollisionEventFn`;
                    // `world` is the owning world's handle captured at
                    // registration; `record` outlives the call.
                    unsafe {
                        f(
                            world,
                            &record as *const _,
                            self.collision_cb.user_data.load(Ordering::Acquire)
                                as *mut std::ffi::c_void,
                        );
                    }
                }
            }
        }
        if event_dispatch_has_poll(mode) {
            if let Some(ref ring) = self.collisions {
                ring.push(record);
            }
        }
    }

    fn dispatch_contact_force(&self, record: ContactForceEventRecord) {
        let mode = self.dispatch_mode();
        match mode {
            EventDispatchMode::Poll => {}
            EventDispatchMode::Callback | EventDispatchMode::Both => {
                let cb = *self.contact_force_cb.cb.lock();
                if let Some(f) = cb {
                    let world = self.world_ptr.load(Ordering::Acquire) as *const std::ffi::c_void;
                    // SAFETY: `f` was registered as a valid `ContactForceEventFn`;
                    // `world` is the owning world's handle captured at
                    // registration; `record` outlives the call.
                    unsafe {
                        f(
                            world,
                            &record as *const _,
                            self.contact_force_cb.user_data.load(Ordering::Acquire)
                                as *mut std::ffi::c_void,
                        );
                    }
                }
            }
        }
        if event_dispatch_has_poll(mode) {
            if let Some(ref ring) = self.contact_forces {
                ring.push(record);
            }
        }
    }
}

fn event_dispatch_has_poll(mode: EventDispatchMode) -> bool {
    matches!(mode, EventDispatchMode::Poll | EventDispatchMode::Both)
}

impl EventHandler for CollectingEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        let record = match event {
            CollisionEvent::Started(h1, h2, flags) => CollisionEventRecord {
                started: Bool::TRUE,
                collider1: pack_collider_handle(h1),
                collider2: pack_collider_handle(h2),
                sensor: flags.contains(CollisionEventFlags::SENSOR).into(),
                removed: flags.contains(CollisionEventFlags::REMOVED).into(),
            },
            CollisionEvent::Stopped(h1, h2, flags) => CollisionEventRecord {
                started: Bool::FALSE,
                collider1: pack_collider_handle(h1),
                collider2: pack_collider_handle(h2),
                sensor: flags.contains(CollisionEventFlags::SENSOR).into(),
                removed: flags.contains(CollisionEventFlags::REMOVED).into(),
            },
        };

        // Always push to the legacy Vec for backward compatibility.
        push_event(&mut self.collision_events.lock(), record);

        // P4 fix: lock-free dispatch via UnsafeCell (single producer during step)
        let pc = unsafe { &*self.producer_cache.get() };
        pc.dispatch_collision(record);
    }

    fn handle_contact_force_event(
        &self,
        dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);
        let record = ContactForceEventRecord {
            collider1: pack_collider_handle(event.collider1),
            collider2: pack_collider_handle(event.collider2),
            total_force: vec3_from_rapier(event.total_force),
            total_force_magnitude: event.total_force_magnitude,
            max_force_direction: vec3_from_rapier(event.max_force_direction),
            max_force_magnitude: event.max_force_magnitude,
        };

        push_event(&mut self.contact_force_events.lock(), record);

        // P4 fix: lock-free dispatch
        let pc = unsafe { &*self.producer_cache.get() };
        pc.dispatch_contact_force(record);
    }
}

#[derive(Default)]
pub(crate) struct CallbackPhysicsHooks {
    custom_physics: std::sync::Arc<CollectingEventHandler>,
}

impl CallbackPhysicsHooks {
    pub(crate) fn new(custom_physics: std::sync::Arc<CollectingEventHandler>) -> Self {
        Self { custom_physics }
    }
}

impl PhysicsHooks for CallbackPhysicsHooks {
    fn filter_contact_pair(
        &self,
        _context: &rapier3d::prelude::PairFilterContext,
    ) -> Option<SolverFlags> {
        Some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn filter_intersection_pair(&self, _context: &rapier3d::prelude::PairFilterContext) -> bool {
        true
    }

    fn modify_solver_contacts(&self, context: &mut rapier3d::prelude::ContactModificationContext) {
        let Some(law) = self.custom_physics.custom_physics().coulomb_friction else {
            return;
        };
        if law.enabled.0 == 0 {
            return;
        }

        let static_mu = law.static_coefficient.max(0.0);
        let dynamic_mu = law.dynamic_coefficient.max(0.0);
        let threshold = law.velocity_threshold.max(0.0);
        let relative_velocity = match (context.rigid_body1, context.rigid_body2) {
            (Some(rb1), Some(rb2)) => {
                let v1 = context
                    .bodies
                    .get(rb1)
                    .map(|body| body.linvel())
                    .unwrap_or(Vector::ZERO);
                let v2 = context
                    .bodies
                    .get(rb2)
                    .map(|body| body.linvel())
                    .unwrap_or(Vector::ZERO);
                v1 - v2
            }
            (Some(rb1), None) => context
                .bodies
                .get(rb1)
                .map(|body| body.linvel())
                .unwrap_or(Vector::ZERO),
            (None, Some(rb2)) => -context
                .bodies
                .get(rb2)
                .map(|body| body.linvel())
                .unwrap_or(Vector::ZERO),
            (None, None) => Vector::ZERO,
        };
        let normal_speed = relative_velocity.dot(*context.normal);
        let tangential_speed = (relative_velocity - *context.normal * normal_speed).length();
        let friction = if tangential_speed <= threshold {
            static_mu
        } else {
            dynamic_mu
        };

        for contact in context.solver_contacts.iter_mut() {
            contact.friction = friction;
        }
    }
}

fn coulomb_law_valid(law: CoulombFrictionLaw) -> bool {
    law.static_coefficient.is_finite()
        && law.dynamic_coefficient.is_finite()
        && law.velocity_threshold.is_finite()
        && law.static_coefficient >= 0.0
        && law.dynamic_coefficient >= 0.0
        && law.velocity_threshold >= 0.0
}

fn air_drag_law_valid(law: AirDragLaw) -> bool {
    vec3_finite(law.fluid_velocity)
        && law.density.is_finite()
        && law.dynamic_viscosity.is_finite()
        && law.characteristic_length.is_finite()
        && law.reference_area.is_finite()
        && law.drag_coefficient.is_finite()
        && law.reynolds_stokes_limit.is_finite()
        && law.density >= 0.0
        && law.dynamic_viscosity > 0.0
        && law.characteristic_length > 0.0
        && law.reference_area >= 0.0
        && law.drag_coefficient >= 0.0
        && law.reynolds_stokes_limit >= 0.0
}

fn external_force_law_valid(law: ExternalForceLaw) -> bool {
    vec3_finite(law.buoyancy_gravity)
        && vec3_finite(law.electric_field)
        && vec3_finite(law.magnetic_field)
        && vec3_finite(law.spring_anchor)
        && vec3_finite(law.gravity_source)
        && law.fluid_density.is_finite()
        && law.displaced_volume.is_finite()
        && law.charge.is_finite()
        && law.spring_stiffness.is_finite()
        && law.spring_damping.is_finite()
        && law.gravitational_parameter.is_finite()
        && law.fluid_density >= 0.0
        && law.displaced_volume >= 0.0
        && law.spring_stiffness >= 0.0
        && law.spring_damping >= 0.0
        && law.gravitational_parameter >= 0.0
}

/// Apply custom external forces (buoyancy, EM, spring, point gravity) using
/// the already-read physics state.  Air drag is now handled by
/// `interaction::apply_body_interactions`.
pub(crate) fn apply_custom_external_forces_with_custom(
    world: &mut crate::rapier::world::PhysicsWorld,
    custom: CustomPhysicsState,
) {
    let external_force = custom
        .external_force
        .filter(|law| law.enabled.0 != 0 && external_force_law_valid(*law));

    if external_force.is_none() {
        return;
    }

    // Pre-compute constant parts of external forces outside the body loop
    let buoyancy_force_vec = external_force
        .filter(|law| law.buoyancy_enabled.0 != 0)
        .map(|law| {
            -vec3_to_rapier(law.buoyancy_gravity) * (law.fluid_density * law.displaced_volume)
        });
    let em_electric_vec = external_force
        .filter(|law| law.electromagnetic_enabled.0 != 0)
        .map(|law| vec3_to_rapier(law.electric_field) * law.charge);
    let em_magnetic_vec = external_force
        .filter(|law| law.electromagnetic_enabled.0 != 0)
        .map(|law| vec3_to_rapier(law.magnetic_field));
    let em_charge = external_force
        .filter(|law| law.electromagnetic_enabled.0 != 0)
        .map(|law| law.charge);
    let spring_anchor = external_force
        .filter(|law| law.elastic_enabled.0 != 0)
        .map(|law| vec3_to_rapier(law.spring_anchor));
    let spring_k = external_force
        .filter(|law| law.elastic_enabled.0 != 0)
        .map(|law| law.spring_stiffness);
    let spring_d = external_force
        .filter(|law| law.elastic_enabled.0 != 0)
        .map(|law| law.spring_damping);
    let gravity_source = external_force
        .filter(|law| law.gravity_enabled.0 != 0)
        .map(|law| vec3_to_rapier(law.gravity_source));
    let grav_param = external_force
        .filter(|law| law.gravity_enabled.0 != 0)
        .map(|law| law.gravitational_parameter);

    let mut report = CustomPhysicsReport::default();
    let mut total_external = KahanVec3::default();

    for (_, body) in world.bodies.iter_mut() {
        report.body_count += 1;
        if !body.is_dynamic() {
            continue;
        }

        // --- External force (pre-computed constant parts) ---
        let mut force = Vector::ZERO;

        // Buoyancy: constant force per body
        if let Some(bf) = buoyancy_force_vec {
            force += bf;
        }

        // Electromagnetic: E-field constant, B-field × v per body
        if let (Some(ef), Some(bf), Some(q)) = (em_electric_vec, em_magnetic_vec, em_charge) {
            let magnetic = body.linvel().cross(bf);
            force += ef + magnetic * q;
        }

        // Elastic spring
        if let (Some(anchor), Some(k), Some(d)) = (spring_anchor, spring_k, spring_d) {
            let displacement = body.translation() - anchor;
            let damping = body.linvel() * d;
            force += -displacement * k - damping;
        }

        // Gravity point-mass
        if let (Some(src), Some(gp)) = (gravity_source, grav_param) {
            let offset = src - body.translation();
            let distance_squared = offset.length_squared();
            if distance_squared > 1.0e-12 {
                let mass = body.mass();
                if mass > 0.0 {
                    force += offset / distance_squared.sqrt() * (gp * mass / distance_squared);
                }
            }
        }

        if force != Vector::ZERO {
            body.add_force(force, true);
            total_external.add(vec3_from_rapier(force));
            report.external_force_body_count += 1;
        }
    }

    report.total_external_force = total_external.value();
    world.events.set_last_custom_physics_report(report);
}

pub struct PendingForce {
    pub(crate) handle: rapier3d::prelude::RigidBodyHandle,
    pub(crate) force: Vector,
    pub(crate) source: crate::rapier::forces::ForceLawType,
}

/// Facade-based wrapper: calls the original function and replays its forces
/// through the facade so the frame-log captures them with correct ForceLawType tags.
///
/// This is a temporary shim — once all force sources are registered as ForceLaw
/// impls, this function and `apply_custom_external_forces_with_custom` will be removed.
pub(crate) fn apply_custom_external_forces_with_facade(
    custom: &CustomPhysicsState,
    facade: &mut crate::rapier::forces::ForceFacade<'_>,
) {
    let Some(external_force) = custom
        .external_force
        .filter(|law| law.enabled.0 != 0 && external_force_law_valid(*law))
    else {
        return;
    };

    use crate::rapier::forces::ForceLawType;

    // Pre-compute constants
    let buoyancy_force_vec = external_force.buoyancy_enabled.0.ne(&0).then(|| {
        -vec3_to_rapier(external_force.buoyancy_gravity)
            * (external_force.fluid_density * external_force.displaced_volume)
    });
    let em_electric_vec = external_force
        .electromagnetic_enabled
        .0
        .ne(&0)
        .then(|| vec3_to_rapier(external_force.electric_field) * external_force.charge);
    let em_magnetic_vec = external_force
        .electromagnetic_enabled
        .0
        .ne(&0)
        .then(|| vec3_to_rapier(external_force.magnetic_field));
    let em_charge = external_force
        .electromagnetic_enabled
        .0
        .ne(&0)
        .then(|| external_force.charge);
    let spring_anchor = external_force
        .elastic_enabled
        .0
        .ne(&0)
        .then(|| vec3_to_rapier(external_force.spring_anchor));
    let spring_k = external_force
        .elastic_enabled
        .0
        .ne(&0)
        .then(|| external_force.spring_stiffness);
    let spring_d = external_force
        .elastic_enabled
        .0
        .ne(&0)
        .then(|| external_force.spring_damping);
    let gravity_source = external_force
        .gravity_enabled
        .0
        .ne(&0)
        .then(|| vec3_to_rapier(external_force.gravity_source));
    let grav_param = external_force
        .gravity_enabled
        .0
        .ne(&0)
        .then(|| external_force.gravitational_parameter);

    // Phase 1: compute forces (immutable body read) — use persistent buffer (P5)
    let pending = &mut facade.pending_forces;
    pending.clear();

    for (handle, body) in facade.bodies.iter() {
        if !body.is_dynamic() {
            continue;
        }

        if let Some(bf) = buoyancy_force_vec {
            pending.push(PendingForce {
                handle,
                force: bf,
                source: ForceLawType::Buoyancy,
            });
        }
        if let (Some(ef), Some(bf), Some(q)) = (em_electric_vec, em_magnetic_vec, em_charge) {
            let magnetic = body.linvel().cross(bf);
            pending.push(PendingForce {
                handle,
                force: ef + magnetic * q,
                source: ForceLawType::Electromagnetic,
            });
        }
        if let (Some(anchor), Some(k), Some(d)) = (spring_anchor, spring_k, spring_d) {
            let displacement = body.translation() - anchor;
            let damping = body.linvel() * d;
            pending.push(PendingForce {
                handle,
                force: -displacement * k - damping,
                source: ForceLawType::ElasticSpring,
            });
        }
        if let (Some(src), Some(gp)) = (gravity_source, grav_param) {
            let offset = src - body.translation();
            let distance_squared = offset.length_squared();
            if distance_squared > 1.0e-12 {
                let mass = body.mass();
                if mass > 0.0 {
                    let f = offset / distance_squared.sqrt() * (gp * mass / distance_squared);
                    pending.push(PendingForce {
                        handle,
                        force: f,
                        source: ForceLawType::PointGravity,
                    });
                }
            }
        }
    }

    // Phase 2: apply forces (mutable body write)
    // Collect pending forces into local SmallVec to release &mut on facade.pending_forces.
    let pending_work: SmallVec<[PendingForce; 128]> = (0..pending.len())
        .map(|i| PendingForce {
            handle: pending[i].handle,
            force: pending[i].force,
            source: pending[i].source,
        })
        .collect();
    pending.clear();
    for pf in pending_work {
        facade.add_force(pf.handle, pf.force, pf.source);
    }
}

/// Set (or disable) the Coulomb friction law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_coulomb_friction_law(
    world: *mut WorldHandle,
    law: CoulombFrictionLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !coulomb_law_valid(law) {
            set_error(ERR_INVALID_ARGUMENT, "invalid Coulomb friction law");
            return Bool::FALSE;
        }

        world.inner.events.custom_physics.write().coulomb_friction =
            if law.enabled.0 != 0 { Some(law) } else { None };
        clear_error();
        Bool::TRUE
    })
}

/// `u8`-returning variant of `world_set_coulomb_friction_law`.
///
/// # Safety
///
/// Same contract as `world_set_coulomb_friction_law`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_coulomb_friction_law_flag(
    world: *mut WorldHandle,
    law: CoulombFrictionLaw,
) -> u8 {
    ffi_guard(0, || world_set_coulomb_friction_law(world, law).0)
}

/// Clear the Coulomb friction law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_coulomb_friction_law(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.events.custom_physics.write().coulomb_friction = None;
        clear_error();
    })
}

/// Read the current Coulomb friction law into `out_law`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_law` must point to writable memory for one `CoulombFrictionLaw`.
/// Null pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_coulomb_friction_law(
    world: *const WorldHandle,
    out_law: *mut CoulombFrictionLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_law) = (unsafe { out_law.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "Coulomb friction output is null");
            return Bool::FALSE;
        };

        *out_law = world
            .inner
            .events
            .custom_physics()
            .coulomb_friction
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

/// Set (or disable) the air drag law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_air_drag_law(world: *mut WorldHandle, law: AirDragLaw) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !air_drag_law_valid(law) {
            set_error(ERR_INVALID_ARGUMENT, "invalid air drag law");
            return Bool::FALSE;
        }

        world.inner.events.custom_physics.write().air_drag =
            if law.enabled.0 != 0 { Some(law) } else { None };

        // Also register into the ForceRegistry for the new dispatch path.
        // P8: single traversal to remove existing AirDrag laws, then register the new one.
        {
            world
                .inner
                .force_registry
                .unregister_by_type(crate::rapier::forces::ForceLawType::AirDrag);
            if law.enabled.0 != 0 {
                let drag_law = crate::rapier::interaction::AirDragForceLaw {
                    fluid_velocity: vec3_to_rapier(law.fluid_velocity),
                    density: law.density,
                    dynamic_viscosity: law.dynamic_viscosity,
                    characteristic_length: law.characteristic_length,
                    reference_area: law.reference_area,
                    drag_coefficient: law.drag_coefficient,
                    reynolds_stokes_limit: law.reynolds_stokes_limit,
                    enabled: true,
                };
                world.inner.force_registry.register(Box::new(drag_law));
            }
        }

        clear_error();
        Bool::TRUE
    })
}

/// `u8`-returning variant of `world_set_air_drag_law`.
///
/// # Safety
///
/// Same contract as `world_set_air_drag_law`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_air_drag_law_flag(world: *mut WorldHandle, law: AirDragLaw) -> u8 {
    ffi_guard(0, || world_set_air_drag_law(world, law).0)
}

/// Clear the air drag law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_air_drag_law(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.events.custom_physics.write().air_drag = None;
        world
            .inner
            .events
            .set_last_custom_physics_report(CustomPhysicsReport::default());
        clear_error();
    })
}

/// Read the current air drag law into `out_law`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_law` must point to writable memory for one `AirDragLaw`. Null
/// pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_air_drag_law(
    world: *const WorldHandle,
    out_law: *mut AirDragLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_law) = (unsafe { out_law.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "air drag output is null");
            return Bool::FALSE;
        };

        *out_law = world
            .inner
            .events
            .custom_physics()
            .air_drag
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

/// Set (or disable) the external force law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_external_force_law(
    world: *mut WorldHandle,
    law: ExternalForceLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !external_force_law_valid(law) {
            set_error(ERR_INVALID_ARGUMENT, "invalid external force law");
            return Bool::FALSE;
        }

        world.inner.events.custom_physics.write().external_force =
            if law.enabled.0 != 0 { Some(law) } else { None };
        clear_error();
        Bool::TRUE
    })
}

/// `u8`-returning variant of `world_set_external_force_law`.
///
/// # Safety
///
/// Same contract as `world_set_external_force_law`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_external_force_law_flag(
    world: *mut WorldHandle,
    law: ExternalForceLaw,
) -> u8 {
    ffi_guard(0, || world_set_external_force_law(world, law).0)
}

/// Clear the external force law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_external_force_law(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.events.custom_physics.write().external_force = None;
        clear_error();
    })
}

/// Read the current external force law into `out_law`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_law` must point to writable memory for one `ExternalForceLaw`. Null
/// pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_external_force_law(
    world: *const WorldHandle,
    out_law: *mut ExternalForceLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_law) = (unsafe { out_law.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "external force output is null");
            return Bool::FALSE;
        };

        *out_law = world
            .inner
            .events
            .custom_physics()
            .external_force
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

// ---------------------------------------------------------------------------
// Newton gravity law FFI
// ---------------------------------------------------------------------------

/// Set (or disable) the Newton gravity law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_newton_gravity_law(
    world: *mut WorldHandle,
    law: NewtonGravityLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !law.gravitational_constant.is_finite()
            || law.gravitational_constant < 0.0
            || !law.min_distance.is_finite()
            || law.min_distance <= 0.0
            || !law.max_distance.is_finite()
            || law.max_distance < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid Newton gravity law");
            return Bool::FALSE;
        }
        world.inner.events.custom_physics.write().newton_gravity =
            if law.enabled.0 != 0 { Some(law) } else { None };

        // Also register into the ForceRegistry.
        // P8: single traversal to remove existing NewtonianGravity laws.
        {
            use crate::rapier::forces::ForceLawType;
            world
                .inner
                .force_registry
                .unregister_by_type(ForceLawType::NewtonianGravity);
            if law.enabled.0 != 0 {
                let gravity_law = crate::rapier::interaction::NewtonianGravityForceLaw {
                    gravitational_constant: law.gravitational_constant,
                    min_distance: law.min_distance,
                    max_distance: law.max_distance,
                    enabled: true,
                };
                world.inner.force_registry.register(Box::new(gravity_law));
            }
        }

        clear_error();
        Bool::TRUE
    })
}

/// `u8`-returning variant of `world_set_newton_gravity_law`.
///
/// # Safety
///
/// Same contract as `world_set_newton_gravity_law`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_newton_gravity_law_flag(
    world: *mut WorldHandle,
    law: NewtonGravityLaw,
) -> u8 {
    ffi_guard(0, || world_set_newton_gravity_law(world, law).0)
}

/// Clear the Newton gravity law on a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_newton_gravity_law(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.events.custom_physics.write().newton_gravity = None;
        clear_error();
    })
}

/// Read the current Newton gravity law into `out_law`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_law` must point to writable memory for one `NewtonGravityLaw`. Null
/// pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_newton_gravity_law(
    world: *const WorldHandle,
    out_law: *mut NewtonGravityLaw,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_law) = (unsafe { out_law.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "Newton gravity output is null");
            return Bool::FALSE;
        };
        *out_law = world
            .inner
            .events
            .custom_physics()
            .newton_gravity
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

/// Read the last custom-physics report into `out_report`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_report` must point to writable memory for one `CustomPhysicsReport`.
/// Null pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_custom_physics_report(
    world: *const WorldHandle,
    out_report: *mut CustomPhysicsReport,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_report) = (unsafe { out_report.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "custom physics report output is null");
            return Bool::FALSE;
        };

        *out_report = world.inner.events.custom_physics().last_report;
        clear_error();
        Bool::TRUE
    })
}

/// Clear the legacy event queues of a world.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_events(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.events.clear();
    })
}

/// Number of queued collision events (legacy Vec queue).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer returns 0.
#[unsafe(no_mangle)]
pub extern "C" fn world_collision_event_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        world.inner.events.collision_event_count() as u32
    })
}

/// Read one queued collision event by index (legacy Vec queue).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer or out-of-range index returns a zeroed record.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_collision_event(
    world: *const WorldHandle,
    index: u32,
) -> CollisionEventRecord {
    ffi_guard(CollisionEventRecord::default(), || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return CollisionEventRecord::default();
        };
        world
            .inner
            .events
            .collision_event(index as usize)
            .unwrap_or_default()
    })
}

/// Copy up to `capacity` queued collision events into `out_events`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_events` must point to writable memory for `capacity`
/// `CollisionEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_collision_events(
    world: *const WorldHandle,
    out_events: *mut CollisionEventRecord,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_events.is_null() {
            set_error(ERR_NULL_POINTER, "collision event output is null");
            return 0;
        }
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid collision event output capacity");
            return 0;
        }

        clear_error();
        let out = unsafe { std::slice::from_raw_parts_mut(out_events, capacity as usize) };
        world.inner.events.collision_events(out)
    })
}

/// Number of queued contact-force events (legacy Vec queue).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer returns 0.
#[unsafe(no_mangle)]
pub extern "C" fn world_contact_force_event_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        world.inner.events.contact_force_event_count() as u32
    })
}

/// Read one queued contact-force event by index (legacy Vec queue).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer or out-of-range index returns a zeroed record.
#[unsafe(no_mangle)]
pub extern "C" fn world_get_contact_force_event(
    world: *const WorldHandle,
    index: u32,
) -> ContactForceEventRecord {
    ffi_guard(ContactForceEventRecord::default(), || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return ContactForceEventRecord::default();
        };
        world
            .inner
            .events
            .contact_force_event(index as usize)
            .unwrap_or_default()
    })
}

/// Copy up to `capacity` queued contact-force events into `out_events`.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_events` must point to writable memory for `capacity`
/// `ContactForceEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
#[unsafe(no_mangle)]
pub extern "C" fn world_get_contact_force_events(
    world: *const WorldHandle,
    out_events: *mut ContactForceEventRecord,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_events.is_null() {
            set_error(ERR_NULL_POINTER, "contact force event output is null");
            return 0;
        }
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid contact force event output capacity");
            return 0;
        }

        clear_error();
        let out = unsafe { std::slice::from_raw_parts_mut(out_events, capacity as usize) };
        world.inner.events.contact_force_events(out)
    })
}

/// Disabled external contact-pair filter callback (always reports
/// `ERR_UNSUPPORTED` and reinstalls the default hooks).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_contact_pair_filter_callback(
    world: *mut WorldHandle,
    _callback: usize,
    _user_data: usize,
) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return;
        };
        set_error(
            ERR_UNSUPPORTED,
            "external contact pair callbacks are disabled for ABI safety",
        );
        world.inner.hooks = CallbackPhysicsHooks::new(world.inner.events.clone());
    })
}

/// Disabled external intersection-pair filter callback (always reports
/// `ERR_UNSUPPORTED` and reinstalls the default hooks).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer fails with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_intersection_pair_filter_callback(
    world: *mut WorldHandle,
    _callback: usize,
    _user_data: usize,
) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return;
        };
        set_error(
            ERR_UNSUPPORTED,
            "external intersection callbacks are disabled for ABI safety",
        );
        world.inner.hooks = CallbackPhysicsHooks::new(world.inner.events.clone());
    })
}

/// Reinstall the default contact-pair filter hooks.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_contact_pair_filter_callback(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.hooks = CallbackPhysicsHooks::new(world.inner.events.clone());
    })
}

/// Reinstall the default intersection-pair filter hooks.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_intersection_pair_filter_callback(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        world.inner.hooks = CallbackPhysicsHooks::new(world.inner.events.clone());
    })
}

// ---------------------------------------------------------------------------
// Event cache registry — init-time registration, zero per-frame lookup
// ---------------------------------------------------------------------------

/// Allocate a collision-event ring buffer of `capacity` records.
/// Events will be written here during `world_step` instead of (or in addition to)
/// the legacy Vec queue.  Java drains the ring buffer at its own pace.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`.
/// Init-time only: must be called before `world_step` runs on any thread and
/// with no concurrent event-ring FFI calls on the same world.  Re-initializing
/// the ring while the physics thread produces events is undefined behavior
/// (the producer cache is an `UnsafeCell`).
#[unsafe(no_mangle)]
pub extern "C" fn world_init_collision_event_ring(world: *mut WorldHandle, capacity: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid collision event ring capacity");
            return Bool::FALSE;
        }
        // SAFETY: single writer (init is called before physics starts, or on main thread)
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        pc.collisions = Some(CollisionEventRing::new(capacity));
        clear_error();
        Bool::TRUE
    })
}

/// Allocate a contact-force-event ring buffer.
///
/// # Safety
///
/// Same init-time-only contract as `world_init_collision_event_ring`.
#[unsafe(no_mangle)]
pub extern "C" fn world_init_contact_force_event_ring(
    world: *mut WorldHandle,
    capacity: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid contact force event ring capacity");
            return Bool::FALSE;
        }
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        pc.contact_forces = Some(ContactForceEventRing::new(capacity));
        clear_error();
        Bool::TRUE
    })
}

/// Drain the collision-event ring buffer into `out_events`.
/// Returns the number of events drained.  This is the **only** FFI call needed
/// per frame after init — no more count-then-allocate-then-read cycles.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_events` must point to writable memory for `capacity`
/// `CollisionEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
/// May run concurrently with `world_step` (SPSC drain), but only from a
/// single consumer thread.
#[unsafe(no_mangle)]
pub extern "C" fn world_drain_collision_event_ring(
    world: *const WorldHandle,
    out_events: *mut CollisionEventRecord,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_events.is_null() || capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid collision event drain output");
            return 0;
        }
        let out = unsafe { std::slice::from_raw_parts_mut(out_events, capacity as usize) };
        // SAFETY: reads are lock-free; ring buffer drains use atomics internally
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        let count = pc
            .collisions
            .as_ref()
            .map(|ring| ring.drain(out))
            .unwrap_or(0);
        clear_error();
        count
    })
}

/// Drain the contact-force-event ring buffer.
///
/// # Safety
///
/// Same contract as `world_drain_collision_event_ring`, with
/// `ContactForceEventRecord` output elements.
#[unsafe(no_mangle)]
pub extern "C" fn world_drain_contact_force_event_ring(
    world: *const WorldHandle,
    out_events: *mut ContactForceEventRecord,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_events.is_null() || capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid contact force event drain output");
            return 0;
        }
        let out = unsafe { std::slice::from_raw_parts_mut(out_events, capacity as usize) };
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        let count = pc
            .contact_forces
            .as_ref()
            .map(|ring| ring.drain(out))
            .unwrap_or(0);
        clear_error();
        count
    })
}

/// Get the current number of events in the collision ring buffer (cheap, no lock).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer returns 0.
#[unsafe(no_mangle)]
pub extern "C" fn world_collision_event_ring_len(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        // SAFETY: read-only access to ring buffer length (atomically loaded internally)
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        pc.collisions.as_ref().map(|ring| ring.len()).unwrap_or(0)
    })
}

/// Get the current number of events in the contact-force ring buffer.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer returns 0.
#[unsafe(no_mangle)]
pub extern "C" fn world_contact_force_event_ring_len(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            return 0;
        };
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        pc.contact_forces
            .as_ref()
            .map(|ring| ring.len())
            .unwrap_or(0)
    })
}

/// Get ring buffer statistics (capacity, occupancy, drops, wraps).
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`;
/// `out_stats` must point to writable memory for one `EventRingBufferStats`.
/// Null pointers fail with `ERR_NULL_POINTER`.
#[unsafe(no_mangle)]
pub extern "C" fn world_collision_event_ring_stats(
    world: *const WorldHandle,
    out_stats: *mut EventRingBufferStats,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_stats) = (unsafe { out_stats.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "ring stats output is null");
            return Bool::FALSE;
        };
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        *out_stats = pc
            .collisions
            .as_ref()
            .map(|ring| ring.stats())
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

/// Get contact-force ring buffer statistics.
///
/// # Safety
///
/// Same contract as `world_collision_event_ring_stats`.
#[unsafe(no_mangle)]
pub extern "C" fn world_contact_force_event_ring_stats(
    world: *const WorldHandle,
    out_stats: *mut EventRingBufferStats,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(out_stats) = (unsafe { out_stats.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "ring stats output is null");
            return Bool::FALSE;
        };
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        *out_stats = pc
            .contact_forces
            .as_ref()
            .map(|ring| ring.stats())
            .unwrap_or_default();
        clear_error();
        Bool::TRUE
    })
}

/// Clear both ring buffers and reset drop counters.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`; a null
/// pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn world_clear_event_rings(world: *mut WorldHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        // SAFETY: clear is atomic write on the ring; safe even if Java is draining
        let pc = unsafe { &*world.inner.events.producer_cache.get() };
        if let Some(ref ring) = pc.collisions {
            ring.clear();
        }
        if let Some(ref ring) = pc.contact_forces {
            ring.clear();
        }
    })
}

/// Register a collision-event callback.
///
/// `callback` is a C function pointer (zero = unregister).
/// `user_data` is passed through unchanged to each invocation.
/// Returns an opaque handle for later unregistration.
///
/// # Safety
///
/// `world` must be a valid world pointer returned by `world_create`.
/// `callback` must be `0` ("unset") or the address of a function with the
/// exact `CollisionEventFn` signature that stays valid while registered.
/// Init-time only: must be called before `world_step` runs on any thread and
/// with no concurrent event-ring/callback FFI calls on the same world.  The
/// producer cache is an `UnsafeCell`; concurrent registration while the
/// physics thread dispatches events is undefined behavior.
#[unsafe(no_mangle)]
pub extern "C" fn world_register_collision_callback(
    world: *mut WorldHandle,
    callback: usize,
    user_data: usize,
) -> EventCallbackHandle {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        // SAFETY: callback registration is init-time, before physics starts
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        let new_handle = pc.next_handle.load(Ordering::Relaxed).wrapping_add(1);
        pc.next_handle.store(new_handle, Ordering::Release);
        *pc.collision_cb.cb.lock() = collision_callback_from_raw(callback);
        pc.collision_cb
            .user_data
            .store(user_data, Ordering::Release);
        pc.collision_cb.handle.store(new_handle, Ordering::Release);
        // Capture the real world pointer so dispatch passes it to the callback.
        pc.world_ptr
            .store(world as *const WorldHandle as usize, Ordering::Release);
        clear_error();
        new_handle
    })
}

/// Register a contact-force-event callback.
///
/// # Safety
///
/// Same init-time-only contract as `world_register_collision_callback`;
/// `callback` must be `0` ("unset") or the address of a function with the
/// exact `ContactForceEventFn` signature that stays valid while registered.
#[unsafe(no_mangle)]
pub extern "C" fn world_register_contact_force_callback(
    world: *mut WorldHandle,
    callback: usize,
    user_data: usize,
) -> EventCallbackHandle {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        let new_handle = pc.next_handle.load(Ordering::Relaxed).wrapping_add(1);
        pc.next_handle.store(new_handle, Ordering::Release);
        *pc.contact_force_cb.cb.lock() = contact_force_callback_from_raw(callback);
        pc.contact_force_cb
            .user_data
            .store(user_data, Ordering::Release);
        pc.contact_force_cb
            .handle
            .store(new_handle, Ordering::Release);
        pc.world_ptr
            .store(world as *const WorldHandle as usize, Ordering::Release);
        clear_error();
        new_handle
    })
}

/// Unregister a previously registered callback by its handle.
/// Passing 0 or an invalid handle is a no-op.
///
/// # Safety
///
/// Same init-time-only contract as `world_register_collision_callback`.
#[unsafe(no_mangle)]
pub extern "C" fn world_unregister_callback(world: *mut WorldHandle, handle: EventCallbackHandle) {
    ffi_guard((), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            return;
        };
        if handle == 0 {
            return;
        }
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        if pc.collision_cb.handle.load(Ordering::Relaxed) == handle {
            *pc.collision_cb.cb.lock() = None;
            pc.collision_cb.user_data.store(0, Ordering::Release);
            pc.collision_cb.handle.store(0, Ordering::Release);
        }
        if pc.contact_force_cb.handle.load(Ordering::Relaxed) == handle {
            *pc.contact_force_cb.cb.lock() = None;
            pc.contact_force_cb.user_data.store(0, Ordering::Release);
            pc.contact_force_cb.handle.store(0, Ordering::Release);
        }
        clear_error();
    })
}

/// Set the event dispatch mode.
///
/// - `Poll` (0): legacy Vec queue only (default).
/// - `Callback` (1): registered callbacks only.
/// - `Both` (2): ring buffer + callbacks.
///
/// # Safety
///
/// Same init-time-only contract as `world_init_collision_event_ring`.
#[unsafe(no_mangle)]
pub extern "C" fn world_set_event_dispatch_mode(world: *mut WorldHandle, mode: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let mode = match mode {
            0 => EventDispatchMode::Poll,
            1 => EventDispatchMode::Callback,
            2 => EventDispatchMode::Both,
            _ => {
                set_error(ERR_INVALID_ARGUMENT, "invalid event dispatch mode");
                return Bool::FALSE;
            }
        };
        let pc = unsafe { &mut *world.inner.events.producer_cache.get() };
        pc.dispatch_mode.store(mode as u32, Ordering::Release);
        clear_error();
        Bool::TRUE
    })
}
