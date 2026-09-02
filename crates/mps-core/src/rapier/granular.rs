//! Granular-body (DEM) FFI — Phase 1 of the granular roadmap.
//!
//! Thin compositional layer over the rapier fork's `GranularWorld` (see
//! `rapier/src/dynamics/granular.rs`). Each granular body is stored in
//! `PhysicsWorld.granular_bodies` keyed by its `Vec` index; the world steps
//! every cloud automatically inside `world_step` (mirroring `fluids`), so the
//! standalone `granular_step` is an optional manual-tick hook.

use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{Bool, Vec3, WorldHandle, vec3_finite};
use rapier3d::prelude::granular::{GranularParams, GranularWorld};

/// Create a DEM granular body and return its id (the `Vec` index in
/// `PhysicsWorld.granular_bodies`). Returns `u32::MAX` on error.
///
/// `gravity` is the body acceleration for every particle (typically the
/// world's gravity, so a granular pile falls like everything else).
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn granular_create(
    world: *mut WorldHandle,
    gravity: Vec3,
    particle_radius: f64,
    normal_stiffness: f64,
    normal_damping: f64,
    friction: f64,
    tangential_damping: f64,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "granular_create: world is null");
            return u32::MAX;
        };
        if !vec3_finite(gravity)
            || !particle_radius.is_finite()
            || particle_radius <= 0.0
            || !normal_stiffness.is_finite()
            || normal_stiffness <= 0.0
            || !normal_damping.is_finite()
            || normal_damping < 0.0
            || !friction.is_finite()
            || friction < 0.0
            || !tangential_damping.is_finite()
            || !(0.0..=1.0).contains(&tangential_damping)
        {
            set_error(ERR_INVALID_ARGUMENT, "granular_create: bad parameters");
            return u32::MAX;
        }
        let params = GranularParams {
            normal_stiffness,
            normal_damping,
            friction,
            tangential_damping,
            gravity: crate::rapier::ffi::vec3_to_rapier(gravity),
        };
        clear_error();
        let id = world.inner.granular_bodies.len() as u32;
        world.inner.granular_bodies.push(GranularWorld::new(params));
        id
    })
}

/// Append a particle to a granular body. Returns the particle index or
/// `u32::MAX` on error.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn granular_add_particle(
    world: *mut WorldHandle,
    id: u32,
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    mass: f64,
    radius: f64,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "granular_add_particle: world is null");
            return u32::MAX;
        };
        let pos = Vec3 { x, y, z };
        let vel = Vec3 {
            x: vx,
            y: vy,
            z: vz,
        };
        if !vec3_finite(pos)
            || !vec3_finite(vel)
            || !mass.is_finite()
            || mass <= 0.0
            || !radius.is_finite()
            || radius <= 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "granular_add_particle: bad parameters",
            );
            return u32::MAX;
        }
        let Some(g) = world.inner.granular_bodies.get_mut(id as usize) else {
            set_error(ERR_NOT_FOUND, "granular_add_particle: unknown id");
            return u32::MAX;
        };
        clear_error();
        g.add_particle(
            crate::rapier::ffi::vec3_to_rapier(pos),
            crate::rapier::ffi::vec3_to_rapier(vel),
            mass,
            radius,
        ) as u32
    })
}

/// Number of particles in a granular body. Returns `u32::MAX` for an unknown
/// id.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn granular_particle_count(world: *const WorldHandle, id: u32) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "granular_particle_count: world is null");
            return u32::MAX;
        };
        let Some(g) = world.inner.granular_bodies.get(id as usize) else {
            set_error(ERR_NOT_FOUND, "granular_particle_count: unknown id");
            return u32::MAX;
        };
        clear_error();
        g.len() as u32
    })
}

/// Batch-read granular particle positions + velocities into `out_pos` /
/// `out_vel` (each with `capacity` slots). Either out-pointer may be null to
/// skip that channel. Returns the real particle count (callers retry with a
/// bigger buffer when `capacity` is short). Null world / unknown id → `0`.
///
/// # Safety
///
/// `world` must be a valid world pointer or null; `out_pos` / `out_vel` must
/// be null or point to writable memory for `capacity` values each.
#[unsafe(no_mangle)]
pub extern "C" fn granular_read_particles(
    world: *const WorldHandle,
    id: u32,
    out_pos: *mut Vec3,
    out_vel: *mut Vec3,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "granular_read_particles: world is null");
            return 0;
        };
        let Some(g) = world.inner.granular_bodies.get(id as usize) else {
            set_error(ERR_NOT_FOUND, "granular_read_particles: unknown id");
            return 0;
        };
        let n = g.len();
        let cap = capacity as usize;
        if !out_pos.is_null() {
            let dst = unsafe { std::slice::from_raw_parts_mut(out_pos, cap.min(n)) };
            for (dst, src) in dst.iter_mut().zip(g.particles.iter()) {
                *dst = crate::rapier::ffi::vec3_from_rapier(src.pos);
            }
        }
        if !out_vel.is_null() {
            let dst = unsafe { std::slice::from_raw_parts_mut(out_vel, cap.min(n)) };
            for (dst, src) in dst.iter_mut().zip(g.particles.iter()) {
                *dst = crate::rapier::ffi::vec3_from_rapier(src.vel);
            }
        }
        clear_error();
        n as u32
    })
}

/// Manually advance one granular body by `dt`. `world_step` already ticks
/// every granular body — this is for callers that want a custom substep loop.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn granular_step(world: *mut WorldHandle, id: u32, dt: f64) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "granular_step: world is null");
            return Bool::FALSE;
        };
        if !dt.is_finite() || dt <= 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "granular_step: bad dt");
            return Bool::FALSE;
        }
        let Some(g) = world.inner.granular_bodies.get_mut(id as usize) else {
            set_error(ERR_NOT_FOUND, "granular_step: unknown id");
            return Bool::FALSE;
        };
        g.step(dt);
        clear_error();
        Bool::TRUE
    })
}
