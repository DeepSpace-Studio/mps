use std::collections::BTreeMap;

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard,
    set_error,
};
use crate::rapier::ffi::{
    AabbDesc, Bool, CRbTreeHandle, MAX_OUTPUT_CAPACITY, MAX_TREE_ENTRIES, Vec3,
};

#[derive(Clone, Copy, Debug)]
struct Aabb {
    mins: Vec3,
    maxs: Vec3,
}

impl Aabb {
    fn from_desc(desc: AabbDesc) -> Option<Self> {
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

        Some(Self { mins, maxs })
    }

    fn intersects(self, other: Self) -> bool {
        self.mins.x <= other.maxs.x
            && self.maxs.x >= other.mins.x
            && self.mins.y <= other.maxs.y
            && self.maxs.y >= other.mins.y
            && self.mins.z <= other.maxs.z
            && self.maxs.z >= other.mins.z
    }
}

pub(crate) struct CRbTreeIndex {
    entries: BTreeMap<u64, Aabb>,
}

impl CRbTreeIndex {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    fn insert(&mut self, id: u64, bounds: Aabb) -> bool {
        if id == 0 {
            return false;
        }
        if !self.entries.contains_key(&id) && self.entries.len() >= MAX_TREE_ENTRIES {
            return false;
        }
        self.entries.insert(id, bounds);
        true
    }

    fn query_count(&self, bounds: Aabb) -> u32 {
        self.entries
            .values()
            .filter(|entry| entry.intersects(bounds))
            .count()
            .min(u32::MAX as usize) as u32
    }

    fn query(&self, bounds: Aabb, out_ids: &mut [u64]) -> u32 {
        let mut written = 0usize;
        for (id, entry) in &self.entries {
            if written >= out_ids.len() {
                break;
            }
            if entry.intersects(bounds) {
                out_ids[written] = *id;
                written += 1;
            }
        }
        written as u32
    }
}

/// Create an empty red-black-tree AABB index.
///
/// # Safety
///
/// The returned pointer is owned by the caller and must be freed exactly once
/// with `crb_tree_destroy`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_create() -> *mut CRbTreeHandle {
    ffi_guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(CRbTreeHandle {
            inner: CRbTreeIndex::new(),
        }))
    })
}

/// Destroy an index created by `crb_tree_create`.
///
/// # Safety
///
/// `tree` must be null or a pointer returned by `crb_tree_create`; it must not
/// be used again after this call.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_destroy(tree: *mut CRbTreeHandle) {
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
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_clear(tree: *mut CRbTreeHandle) {
    ffi_guard((), || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return;
        };
        tree.inner.entries.clear();
        clear_error();
    })
}

/// Return the number of entries stored in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_len(tree: *const CRbTreeHandle) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return 0;
        };
        let len = tree.inner.entries.len().min(u32::MAX as usize) as u32;
        clear_error();
        len
    })
}

/// Insert or overwrite the bounds of `id` in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_insert(tree: *mut CRbTreeHandle, id: u64, aabb: AabbDesc) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        let Some(bounds) = Aabb::from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return Bool::FALSE;
        };
        if id == 0 {
            set_error(ERR_INVALID_ARGUMENT, "id must be non-zero");
            return Bool::FALSE;
        }
        if !tree.inner.insert(id, bounds) {
            set_error(ERR_CAPACITY, "tree entry capacity exceeded");
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Flag-returning variant of `crb_tree_insert`.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_insert_flag(tree: *mut CRbTreeHandle, id: u64, aabb: AabbDesc) -> u8 {
    ffi_guard(0, || crb_tree_insert(tree, id, aabb).0)
}

/// Update the bounds of an existing `id` in the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_update(tree: *mut CRbTreeHandle, id: u64, aabb: AabbDesc) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        if !tree.inner.entries.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "entry not found");
            return Bool::FALSE;
        }
        let Some(bounds) = Aabb::from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return Bool::FALSE;
        };
        clear_error();
        tree.inner.insert(id, bounds).into()
    })
}

/// Remove `id` from the tree.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_remove(tree: *mut CRbTreeHandle, id: u64) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(tree) = (unsafe { tree.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return Bool::FALSE;
        };
        if tree.inner.entries.remove(&id).is_none() {
            set_error(ERR_NOT_FOUND, "entry not found");
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Count the entries whose bounds intersect `aabb`.
///
/// # Safety
///
/// `tree` must be a valid pointer returned by `crb_tree_create`.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_query_aabb_count(tree: *const CRbTreeHandle, aabb: AabbDesc) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "tree is null");
            return 0;
        };
        let Some(bounds) = Aabb::from_desc(aabb) else {
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
/// `tree` must be a valid pointer returned by `crb_tree_create`, and `out_ids`
/// must point to a writable buffer of at least `capacity` `u64` elements.
#[unsafe(no_mangle)]
pub extern "C" fn crb_tree_query_aabb(
    tree: *const CRbTreeHandle,
    aabb: AabbDesc,
    out_ids: *mut u64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(tree) = (unsafe { tree.as_ref() }) else {
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
        let Some(bounds) = Aabb::from_desc(aabb) else {
            set_error(ERR_INVALID_ARGUMENT, "invalid AABB");
            return 0;
        };

        let out = unsafe { std::slice::from_raw_parts_mut(out_ids, capacity as usize) };
        let written = tree.inner.query(bounds, out);
        clear_error();
        written
    })
}
