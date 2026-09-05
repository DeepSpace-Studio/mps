//! Unit tests for the shared stable-id metadata registry
//! (`mps-core::rapier::registry::IdRegistry`) that backs every per-body
//! manager (`character_bodies`, `sensor_zones`, `fracture_mesh_bodies`, …).

#[cfg(test)]
mod tests {
    use mps_core::rapier::registry::IdRegistry;

    #[test]
    fn ids_are_monotonic_and_never_reused() {
        let mut registry: IdRegistry<&'static str> = IdRegistry::new();
        let a = registry.insert("a");
        let b = registry.insert("b");
        let c = registry.insert("c");
        assert_eq!((a, b, c), (0, 1, 2));

        // Removing entries never recycles ids.
        assert_eq!(registry.remove(b), Some("b"));
        assert_eq!(registry.insert("d"), 3);
        assert!(registry.get(b).is_none());
        assert_eq!(registry.get(a), Some(&"a"));
        assert_eq!(registry.get(c), Some(&"c"));
        assert_eq!(registry.get(3), Some(&"d"));
    }

    #[test]
    fn get_mut_and_contains_key_work() {
        let mut registry: IdRegistry<u32> = IdRegistry::new();
        let id = registry.insert(10);
        assert!(registry.contains_key(id));
        assert!(!registry.contains_key(id + 1));
        *registry.get_mut(id).unwrap() = 42;
        assert_eq!(registry.get(id), Some(&42));
        // Removal is take-once.
        assert_eq!(registry.remove(id), Some(42));
        assert_eq!(registry.remove(id), None);
        assert!(!registry.contains_key(id));
    }

    #[test]
    fn allocation_wraps_instead_of_overflow_panicking() {
        // The pre-consolidation registries used `+= 1` (debug-build overflow
        // panic); IdRegistry must wrap like the fracture-mesh precedent.
        let mut registry: IdRegistry<u8> = IdRegistry::new();
        // Skip near the wrap point without allocating 4 billion entries.
        registry.next_id = u32::MAX;
        let last = registry.insert(0);
        assert_eq!(last, u32::MAX);
        let wrapped = registry.insert(1);
        assert_eq!(wrapped, 0);
    }
}
