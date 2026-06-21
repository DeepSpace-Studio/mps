//! Common math utilities shared across rapier modules.
//!
//! These functions replace the per-module copies of `finite`, `finite_positive`,
//! `finite_non_negative`, `write_out`, `vec3_*`, and `clamp` that were
//! previously duplicated in many files.

#![allow(dead_code)]

use crate::rapier::ffi::{Bool, Vec3};

// ---------------------------------------------------------------------------
// Scalar validation
// ---------------------------------------------------------------------------

/// Returns true when `value` is finite.
#[inline]
pub(crate) fn finite(value: f64) -> bool {
    value.is_finite()
}

/// Returns true when `value` is finite and > 0.
#[inline]
pub(crate) fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Returns true when `value` is finite and >= 0.
#[inline]
pub(crate) fn finite_non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// Clamp `value` to the closed interval [lo, hi].
#[inline]
pub(crate) fn clamp(value: f64, lo: f64, hi: f64) -> f64 {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

/// Write a value through an output pointer, returning `Bool::TRUE` on success.
pub(crate) fn write_out<T: Copy>(out: *mut T, value: T) -> Bool {
    let Some(out) = (unsafe { out.as_mut() }) else {
        crate::rapier::error::set_error(crate::rapier::error::ERR_NULL_POINTER, "output pointer is null");
        return Bool::FALSE;
    };
    *out = value;
    crate::rapier::error::clear_error();
    Bool::TRUE
}

// ---------------------------------------------------------------------------
// Vec3 arithmetic
// ---------------------------------------------------------------------------

#[inline]
pub(crate) fn vec3_add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
}

#[inline]
pub(crate) fn vec3_sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

#[inline]
pub(crate) fn vec3_scale(v: Vec3, s: f64) -> Vec3 {
    Vec3 {
        x: v.x * s,
        y: v.y * s,
        z: v.z * s,
    }
}

#[inline]
pub(crate) fn vec3_dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline]
pub(crate) fn vec3_cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

#[inline]
pub(crate) fn vec3_length_sq(v: Vec3) -> f64 {
    v.x * v.x + v.y * v.y + v.z * v.z
}

#[inline]
pub(crate) fn vec3_length(v: Vec3) -> f64 {
    vec3_length_sq(v).sqrt()
}

#[inline]
pub(crate) fn vec3_normalize(v: Vec3) -> Vec3 {
    let len = vec3_length(v);
    if len <= f64::EPSILON {
        Vec3::default()
    } else {
        vec3_scale(v, 1.0 / len)
    }
}
