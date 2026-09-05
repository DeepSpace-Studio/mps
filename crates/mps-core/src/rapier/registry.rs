//! Stable-id metadata registries — the shared shape behind every per-body
//! manager in the world (`articulations`, `character_bodies`, `sensor_zones`,
//! `vehicle_controllers`, `tire_models`, `servo_bodies`,
//! `fracture_mesh_bodies`, `hair_systems`, `rope_knots`).
//!
//! Each of those was previously a `HashMap<u32, T>` + `_next_id: u32` pair
//! with a hand-rolled create/get/remove dance per module. This consolidates
//! them into one type with two guarantees:
//!
//! * ids are monotonic and never reused (stable across a session, safe to
//!   hold on the caller side);
//! * allocation uses `wrapping_add` — the pre-consolidation code mixed
//!   `+= 1` (debug-build overflow panic after ~4 billion creations) with
//!   `wrapping_add` depending on the module; now every registry wraps.

use std::collections::HashMap;

/// A `HashMap` keyed by internally allocated, monotonically increasing ids.
///
/// `pub` (rather than `pub(crate)`) only so `mps-test` can unit-test it per
/// the module-mirror rule (OPTIMIZATION.md §8); nothing outside `mps-core`
/// consumes it, and it never appears in the generated C header.
pub struct IdRegistry<T> {
    /// Backing map. Keep private use via the methods below in production
    /// code; direct access exists for tests only.
    pub map: HashMap<u32, T>,
    /// Next id handed out by [`IdRegistry::insert`]; wraps at `u32::MAX`.
    pub next_id: u32,
}

impl<T> Default for IdRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IdRegistry<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Inserts `value` under a fresh monotonic id and returns that id.
    pub fn insert(&mut self, value: T) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.map.insert(id, value);
        id
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.map.get_mut(&id)
    }

    pub fn contains_key(&self, id: u32) -> bool {
        self.map.contains_key(&id)
    }

    /// Removes the entry, returning it. Cleanup of Rapier-side resources the
    /// entry owns is the caller's responsibility (same contract as the
    /// pre-consolidation `HashMap::remove` dance).
    pub fn remove(&mut self, id: u32) -> Option<T> {
        self.map.remove(&id)
    }
}
