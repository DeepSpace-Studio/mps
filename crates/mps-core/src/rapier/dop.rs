use std::slice;

use rapier3d::math::Vector;

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::ColliderBuilderHandle;
use crate::rapier::ffi::convert::kdop_preset_from_raw;

// Re-export the fork-native hull types so existing callers (including the
// integration tests in `mps-test`) keep referencing `mps_core::rapier::dop::*`
// unchanged. The actual implementation now lives in `rapier3d::geometry`.
#[doc(hidden)]
pub use rapier3d::geometry::direction_hull::DirectionHull;
pub use rapier3d::geometry::direction_hull::{FdhHull, KdopHull, KdopPreset};

const MAX_RAW_POINTS: u32 = 1_000_000;
const MAX_RAW_DIRECTIONS: u32 = 4_096;

/// Read a point cloud from raw f64 triplets for the hull builders below.
fn builder_from_raw_points(points_xyz: *const f64, point_count: u32) -> Option<Vec<Vector>> {
    if points_xyz.is_null() {
        set_error(ERR_NULL_POINTER, "point input is null");
        return None;
    }
    if point_count < 4 {
        set_error(ERR_INVALID_ARGUMENT, "too few points for a hull");
        return None;
    }
    if point_count > MAX_RAW_POINTS {
        set_error(ERR_CAPACITY, "point count exceeds maximum");
        return None;
    }
    let Some(value_count) = (point_count as usize).checked_mul(3) else {
        set_error(ERR_CAPACITY, "point count is too large");
        return None;
    };
    let values = unsafe { slice::from_raw_parts(points_xyz, value_count) };
    let mut vectors = Vec::with_capacity(point_count as usize);
    for chunk in values.chunks_exact(3) {
        if !chunk[0].is_finite() || !chunk[1].is_finite() || !chunk[2].is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "non-finite point coordinate");
            return None;
        }
        vectors.push(Vector::new(chunk[0], chunk[1], chunk[2]));
    }
    Some(vectors)
}

/// Read a direction set from raw f64 triplets.
fn builder_from_raw_directions(
    directions_xyz: *const f64,
    direction_count: u32,
) -> Option<Vec<Vector>> {
    if directions_xyz.is_null() {
        set_error(ERR_NULL_POINTER, "direction input is null");
        return None;
    }
    if direction_count < 3 {
        set_error(ERR_INVALID_ARGUMENT, "too few directions for a hull");
        return None;
    }
    if direction_count > MAX_RAW_DIRECTIONS {
        set_error(ERR_CAPACITY, "direction count exceeds maximum");
        return None;
    }
    let Some(value_count) = (direction_count as usize).checked_mul(3) else {
        set_error(ERR_CAPACITY, "direction count is too large");
        return None;
    };
    let values = unsafe { slice::from_raw_parts(directions_xyz, value_count) };
    let mut vectors = Vec::with_capacity(direction_count as usize);
    for chunk in values.chunks_exact(3) {
        if !chunk[0].is_finite() || !chunk[1].is_finite() || !chunk[2].is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "non-finite direction component");
            return None;
        }
        vectors.push(Vector::new(chunk[0], chunk[1], chunk[2]));
    }
    Some(vectors)
}

/// Create a k-DOP collider builder from a point cloud.
///
/// # Safety
///
/// `points_xyz` must point to at least 3×point_count readable f64s. The
/// returned builder handle is owned by the caller and must be released
/// through the collider-builder destroy function.
#[unsafe(no_mangle)]
pub extern "C" fn collider_builder_create_kdop(
    points_xyz: *const f64,
    point_count: u32,
    preset: u32,
) -> *mut ColliderBuilderHandle {
    ffi_guard(std::ptr::null_mut(), || {
        let Some(points) = builder_from_raw_points(points_xyz, point_count) else {
            return std::ptr::null_mut();
        };

        let hull = KdopHull {
            directions: kdop_directions(kdop_preset_from_raw(preset)),
        };
        let Some(builder) = hull.build(&points) else {
            set_error(ERR_INVALID_ARGUMENT, "failed to build k-DOP hull");
            return std::ptr::null_mut();
        };

        clear_error();
        Box::into_raw(Box::new(ColliderBuilderHandle {
            inner: builder,
            voxel_source: None,
        }))
    })
}

/// Create a fixed-directions-hull (FDH) collider builder from a point cloud.
///
/// # Safety
///
/// `points_xyz` must point to at least 3×point_count readable f64s and
/// `directions_xyz` to at least 3×direction_count readable f64s. The returned
/// builder handle is owned by the caller and must be released through the
/// collider-builder destroy function.
#[unsafe(no_mangle)]
pub extern "C" fn collider_builder_create_fdh(
    points_xyz: *const f64,
    point_count: u32,
    directions_xyz: *const f64,
    direction_count: u32,
) -> *mut ColliderBuilderHandle {
    ffi_guard(std::ptr::null_mut(), || {
        let Some(points) = builder_from_raw_points(points_xyz, point_count) else {
            return std::ptr::null_mut();
        };
        let Some(directions) = builder_from_raw_directions(directions_xyz, direction_count) else {
            return std::ptr::null_mut();
        };

        let hull = FdhHull {
            directions: &directions,
        };
        let Some(builder) = hull.build(&points) else {
            set_error(ERR_INVALID_ARGUMENT, "failed to build FDH hull");
            return std::ptr::null_mut();
        };

        clear_error();
        Box::into_raw(Box::new(ColliderBuilderHandle {
            inner: builder,
            voxel_source: None,
        }))
    })
}

/// Re-export of the fork-native k-DOP direction preset selector, kept under the
/// historical `mps_core::rapier::dop` path so callers (and tests) that named
/// `kdop_directions` continue to resolve.
pub fn kdop_directions(preset: KdopPreset) -> Vec<Vector> {
    rapier3d::geometry::direction_hull::kdop_directions(preset)
}
