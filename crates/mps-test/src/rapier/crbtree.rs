#[cfg(test)]
mod tests {
    use mps_core::rapier::crbtree::*;
    use mps_core::rapier::ffi::*;

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
    fn crb_tree_queries_intersections_in_id_order() {
        let tree = crb_tree_create();
        assert!(!tree.is_null());

        assert_eq!(crb_tree_insert(tree, 20, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 10, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 30, aabb(4.0, 5.0)), Bool::TRUE);

        assert_eq!(crb_tree_query_aabb_count(tree, aabb(0.5, 2.5)), 2);

        let mut ids = [0; 4];
        let written = crb_tree_query_aabb(tree, aabb(0.5, 2.5), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 2);
        assert_eq!(&ids[..2], &[10, 20]);

        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_update_remove_and_reject_invalid_bounds() {
        let tree = crb_tree_create();

        assert_eq!(crb_tree_insert(tree, 7, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_update(tree, 7, aabb(10.0, 11.0)), Bool::TRUE);
        assert_eq!(crb_tree_query_aabb_count(tree, aabb(0.0, 1.0)), 0);
        assert_eq!(crb_tree_query_aabb_count(tree, aabb(10.5, 10.6)), 1);
        assert_eq!(crb_tree_remove(tree, 7), Bool::TRUE);
        assert_eq!(crb_tree_remove(tree, 7), Bool::FALSE);
        assert_eq!(crb_tree_insert(tree, 0, aabb(0.0, 1.0)), Bool::FALSE);
        assert_eq!(
            crb_tree_insert(
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

        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_clear_empties_tree() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        assert_eq!(crb_tree_len(tree), 3);
        crb_tree_clear(tree);
        assert_eq!(crb_tree_len(tree), 0);
        assert_eq!(crb_tree_query_aabb_count(tree, aabb(-10.0, 10.0)), 0);
        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_len_and_buffer_removal() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        assert_eq!(crb_tree_len(tree), 3);
        assert_eq!(crb_tree_remove(tree, 2), Bool::TRUE);
        assert_eq!(crb_tree_len(tree), 2);
        assert_eq!(crb_tree_remove(tree, 2), Bool::FALSE);
        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_query_order_is_id_ascending() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert(tree, 30, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 10, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 20, aabb(4.0, 5.0)), Bool::TRUE);
        let mut ids = [0u64; 4];
        let written =
            crb_tree_query_aabb(tree, aabb(-1.0, 6.0), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 3);
        // BTreeMap iteration yields ids in ascending order
        assert_eq!(&ids[..3], &[10, 20, 30]);
        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_insert_flag_returns_bool_byte() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert_flag(tree, 5, aabb(0.0, 1.0)), 1);
        // invalid AABB (min > max) -> rejected, returns 0
        assert_eq!(
            crb_tree_insert_flag(
                tree,
                6,
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
            0
        );
        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_query_buffer_capped_at_capacity() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 2, aabb(2.0, 3.0)), Bool::TRUE);
        assert_eq!(crb_tree_insert(tree, 3, aabb(4.0, 5.0)), Bool::TRUE);
        let mut ids = [0u64; 2];
        let written =
            crb_tree_query_aabb(tree, aabb(-1.0, 6.0), ids.as_mut_ptr(), ids.len() as u32);
        assert_eq!(written, 2);
        crb_tree_destroy(tree);
    }

    #[test]
    fn crb_tree_update_missing_id_returns_false() {
        let tree = crb_tree_create();
        assert_eq!(crb_tree_insert(tree, 1, aabb(0.0, 1.0)), Bool::TRUE);
        assert_eq!(crb_tree_update(tree, 99, aabb(2.0, 3.0)), Bool::FALSE);
        // existing id updates fine
        assert_eq!(crb_tree_update(tree, 1, aabb(10.0, 11.0)), Bool::TRUE);
        assert_eq!(crb_tree_query_aabb_count(tree, aabb(0.0, 1.0)), 0);
        assert_eq!(crb_tree_query_aabb_count(tree, aabb(10.5, 10.6)), 1);
        crb_tree_destroy(tree);
    }
}
