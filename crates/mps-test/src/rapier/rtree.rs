#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::rtree::*;

    fn aabb(min: f64, max: f64) -> AabbDesc {
        AabbDesc {
            mins: Vec3 {
                x: min,
                y: min,
                z: min,
            },
            maxs: Vec3 {
                x: max,
                y: max,
                z: max,
            },
        }
    }

    #[test]
    fn rtree_queries_intersections() {
        let tree = rtree_create();
        assert!(!tree.is_null());

        assert_eq!(rtree_insert(tree, 10, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 20, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 30, aabb(4.0, 5.0)), Bool::TRUE);

        assert_eq!(rtree_query_aabb_count(tree, aabb(0.5, 2.5)), 2);

        let mut ids = [0; 4];
        let written = rtree_query_aabb(tree, aabb(0.5, 2.5), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 2);
        assert_eq!(&ids[..2], &[10, 20]);

        rtree_destroy(tree);
    }

    #[test]
    fn rtree_update_and_remove() {
        let tree = rtree_create();

        assert_eq!(rtree_insert(tree, 7, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_update(tree, 7, aabb(10.0, 11.0)), Bool::TRUE);
        assert_eq!(rtree_query_aabb_count(tree, aabb(0.0, 1.0)), 0);
        assert_eq!(rtree_query_aabb_count(tree, aabb(10.5, 10.6)), 1);

        assert_eq!(rtree_remove(tree, 7), Bool::TRUE);
        assert_eq!(rtree_remove(tree, 7), Bool::FALSE);
        assert_eq!(rtree_len(tree), 0);

        rtree_destroy(tree);
    }

    #[test]
    fn rtree_rejects_invalid_bounds() {
        let tree = rtree_create();
        assert_eq!(
            rtree_insert(
                tree,
                1,
                AabbDesc {
                    mins: Vec3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    },
                    maxs: Vec3 {
                        x: 0.0,
                        y: 1.0,
                        z: 1.0
                    },
                }
            ),
            Bool::FALSE
        );
        assert_eq!(rtree_insert(tree, 0, aabb(0.0, 1.0)), Bool::FALSE);
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_clear_empties_tree() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        assert_eq!(rtree_len(tree), 3);
        rtree_clear(tree);
        assert_eq!(rtree_len(tree), 0);
        assert_eq!(rtree_query_aabb_count(tree, aabb(-10.0, 10.0)), 0);
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_len_reflects_inserts_and_rejects_id_zero() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        assert_eq!(rtree_len(tree), 3);
        // id 0 is reserved / rejected
        assert_eq!(rtree_insert(tree, 0, aabb(6.0, 7.0)), Bool::FALSE);
        assert_eq!(rtree_len(tree), 3);
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_duplicate_id_replaces_bounds_keeps_len() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 5, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 5, aabb(10.0, 11.0)), Bool::TRUE);
        assert_eq!(rtree_len(tree), 1);
        // the replacement bounds took effect
        assert_eq!(rtree_query_aabb_count(tree, aabb(0.0, 1.0)), 0);
        assert_eq!(rtree_query_aabb_count(tree, aabb(10.5, 10.6)), 1);
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_query_returns_matching_id_set() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 30, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 10, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 20, aabb(4.0, 5.0)), Bool::TRUE);
        // overlapping window catches all three
        let mut ids = [0u64; 4];
        let written = rtree_query_aabb(tree, aabb(-1.0, 6.0), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 3);
        let mut got: Vec<u64> = ids[..3].to_vec();
        got.sort_unstable();
        assert_eq!(got, vec![10, 20, 30]);
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_query_buffer_capped_at_capacity() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(rtree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        let mut ids = [0u64; 2];
        let written = rtree_query_aabb(tree, aabb(-1.0, 6.0), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 2);
        // untouched slot beyond capacity stays zero -> no out-of-bounds write
        rtree_destroy(tree);
    }

    #[test]
    fn rtree_query_no_match_returns_zero() {
        let tree = rtree_create();
        assert_eq!(rtree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(rtree_query_aabb_count(tree, aabb(100.0, 101.0)), 0);
        let mut ids = [0u64; 4];
        let written =
            rtree_query_aabb(tree, aabb(100.0, 101.0), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 0);
        rtree_destroy(tree);
    }
}
