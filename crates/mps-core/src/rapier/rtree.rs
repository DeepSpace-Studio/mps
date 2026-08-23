use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard,
    set_error,
};
use crate::rapier::ffi::{AabbDesc, Bool, MAX_OUTPUT_CAPACITY, MAX_TREE_ENTRIES, RTreeHandle};
use rapier3d::geometry::Aabb;
use rapier3d::geometry::user_index::GenericAabbIndex;

/// Create an empty R-tree index.
///
/// # Safety
///
/// The returned pointer is owned by the caller and must be freed exactly once
/// with `rtree_destroy`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_create() -> *mut RTreeHandle {
    ffi_guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(RTreeHandle {
            inner: GenericAabbIndex::new(),
        }))
    })
}

/// Destroy an R-tree index created by `rtree_create`.
///
/// # Safety
///
/// `tree` must be null or a pointer returned by `rtree_create`; it must not be
/// used again after this call.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_destroy(tree: *mut RTreeHandle) {
    ffi_guard((), || {
        if tree.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(tree));
        }
    })
}

/// Remove every entry from the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_clear(tree: *mut RTreeHandle) {
    ffi_guard((), || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return;
        };
        tree.inner.clear();
        clear_error();
    })
}

/// Return the number of entries stored in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_len(tree: *const RTreeHandle) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return 0;
        };
        let len = tree.inner.len().min(u32::MAX as usize) as u32;
        clear_error();
        len
    })
}

/// Insert or overwrite the bounds of `id` in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_insert(tree: *mut RTreeHandle, id: u64, aabb: AabbDesc) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        let Some(bounds) = aabb_from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return Bool::FALSE;
        };
        if id == 0 {
            set_error(ERR_INVALID_ARGUMENT, "id must be non-zero");
            return Bool::FALSE;
        }
        if tree.inner.len() >= MAX_TREE_ENTRIES {
            set_error(ERR_CAPACITY, "tree entry capacity exceeded");
            return Bool::FALSE;
        }
        clear_error();
        Bool::from(tree.inner.insert(id, bounds))
    })
}

/// Update the bounds of an existing `id` in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_update(tree: *mut RTreeHandle, id: u64, aabb: AabbDesc) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        if !tree.inner.contains(id) {
            set_error(ERR_NOT_FOUND, "entry not found");
            return Bool::FALSE;
        }
        let Some(bounds) = aabb_from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return Bool::FALSE;
        };
        clear_error();
        Bool::from(tree.inner.insert(id, bounds))
    })
}

/// Remove `id` from the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_remove(tree: *mut RTreeHandle, id: u64) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        if !tree.inner.remove(id) {
            set_error(ERR_NOT_FOUND, "entry not found");
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Force an immediate rebuild of the tree structure.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_rebuild(tree: *mut RTreeHandle) {
    ffi_guard((), || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return;
        };
        tree.inner.rebuild();
        clear_error();
    })
}

/// Count the entries whose bounds intersect `aabb`.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_query_aabb_count(tree: *mut RTreeHandle, aabb: AabbDesc) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return 0;
        };
        let Some(bounds) = aabb_from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return 0;
        };
        let count = tree.inner.query_count(bounds);
        clear_error();
        count
    })
}

/// Write the ids of entries whose bounds intersect `aabb` into `out_ids`.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `rtree_create`, and `out_ids`
/// must point to a writable buffer of at least `capacity` `u64` elements.
#[unsafe(no_mangle)]
pub extern "C" fn rtree_query_aabb(
    tree: *mut RTreeHandle,
    aabb: AabbDesc,
    out_ids: *mut u64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return 0;
        };
        if out_ids.is_null() {
            set_error(ERR_NULL_POINTER, "output buffer is null");
            return 0;
        }
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid output capacity");
            return 0;
        }
        let Some(bounds) = aabb_from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return 0;
        };

        let out = unsafe { std::slice::from_raw_parts_mut(out_ids, capacity as usize) };
        let written = tree.inner.query(bounds, out);
        clear_error();
        written
    })
}

/// Convert an FFI `AabbDesc` into the fork-native `Aabb`, rejecting non-finite
/// or inverted bounds.
fn aabb_from_desc(desc: AabbDesc) -> Option<Aabb> {
    let mins = desc.mins;
    let maxs = desc.maxs;
    if !mins.x.is_finite()
        || !mins.y.is_finite()
        || !mins.z.is_finite()
        || !maxs.x.is_finite()
        || !maxs.y.is_finite()
        || !maxs.z.is_finite()
        || mins.x > maxs.x
        || mins.y > maxs.y
        || mins.z > maxs.z
    {
        return None;
    }

    Some(Aabb::new(
        rapier3d::math::Vector::new(mins.x, mins.y, mins.z),
        rapier3d::math::Vector::new(maxs.x, maxs.y, maxs.z),
    ))
}
