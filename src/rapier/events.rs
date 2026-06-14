use parking_lot::Mutex;
use rapier3d::geometry::{CollisionEvent, CollisionEventFlags, ContactPair, SolverFlags};
use rapier3d::prelude::{
    ColliderSet, ContactForceEvent, EventHandler, PhysicsHooks, Real, RigidBodySet,
};

use crate::rapier::error::{
    ERR_CAPACITY, ERR_NULL_POINTER, ERR_UNSUPPORTED, clear_error, set_error,
};
use crate::rapier::ffi::{
    Bool, CollisionEventRecord, ContactForceEventRecord, MAX_OUTPUT_CAPACITY, WorldHandle,
    pack_collider_handle, vec3_from_rapier,
};

const MAX_EVENT_RECORDS: usize = 16_384;

#[derive(Default)]
pub(crate) struct CollectingEventHandler {
    collision_events: Mutex<Vec<CollisionEventRecord>>,
    contact_force_events: Mutex<Vec<ContactForceEventRecord>>,
}

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
}

fn push_event<T>(events: &mut Vec<T>, event: T) {
    if events.len() < MAX_EVENT_RECORDS {
        events.push(event);
    }
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

        push_event(&mut self.collision_events.lock(), record);
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
        push_event(
            &mut self.contact_force_events.lock(),
            ContactForceEventRecord {
                collider1: pack_collider_handle(event.collider1),
                collider2: pack_collider_handle(event.collider2),
                total_force: vec3_from_rapier(event.total_force),
                total_force_magnitude: event.total_force_magnitude,
                max_force_direction: vec3_from_rapier(event.max_force_direction),
                max_force_magnitude: event.max_force_magnitude,
            },
        );
    }
}

#[derive(Default)]
pub(crate) struct CallbackPhysicsHooks {
    _private: (),
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
}

#[unsafe(no_mangle)]
pub extern "C" fn world_clear_events(world: *mut WorldHandle) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return;
    };
    world.inner.events.clear();
}

#[unsafe(no_mangle)]
pub extern "C" fn world_collision_event_count(world: *const WorldHandle) -> u32 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return 0;
    };
    world.inner.events.collision_event_count() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_collision_event(
    world: *const WorldHandle,
    index: u32,
) -> CollisionEventRecord {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return CollisionEventRecord::default();
    };
    world
        .inner
        .events
        .collision_event(index as usize)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_collision_events(
    world: *const WorldHandle,
    out_events: *mut CollisionEventRecord,
    capacity: u32,
) -> u32 {
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
}

#[unsafe(no_mangle)]
pub extern "C" fn world_contact_force_event_count(world: *const WorldHandle) -> u32 {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return 0;
    };
    world.inner.events.contact_force_event_count() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_contact_force_event(
    world: *const WorldHandle,
    index: u32,
) -> ContactForceEventRecord {
    let Some(world) = (unsafe { world.as_ref() }) else {
        return ContactForceEventRecord::default();
    };
    world
        .inner
        .events
        .contact_force_event(index as usize)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn world_get_contact_force_events(
    world: *const WorldHandle,
    out_events: *mut ContactForceEventRecord,
    capacity: u32,
) -> u32 {
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
}

#[unsafe(no_mangle)]
pub extern "C" fn world_set_contact_pair_filter_callback(
    world: *mut WorldHandle,
    _callback: usize,
    _user_data: usize,
) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "world is null");
        return;
    };
    set_error(
        ERR_UNSUPPORTED,
        "external contact pair callbacks are disabled for ABI safety",
    );
    world.inner.hooks = CallbackPhysicsHooks::default();
}

#[unsafe(no_mangle)]
pub extern "C" fn world_set_intersection_pair_filter_callback(
    world: *mut WorldHandle,
    _callback: usize,
    _user_data: usize,
) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "world is null");
        return;
    };
    set_error(
        ERR_UNSUPPORTED,
        "external intersection callbacks are disabled for ABI safety",
    );
    world.inner.hooks = CallbackPhysicsHooks::default();
}

#[unsafe(no_mangle)]
pub extern "C" fn world_clear_contact_pair_filter_callback(world: *mut WorldHandle) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return;
    };
    world.inner.hooks = CallbackPhysicsHooks::default();
}

#[unsafe(no_mangle)]
pub extern "C" fn world_clear_intersection_pair_filter_callback(world: *mut WorldHandle) {
    let Some(world) = (unsafe { world.as_mut() }) else {
        return;
    };
    world.inner.hooks = CallbackPhysicsHooks::default();
}
