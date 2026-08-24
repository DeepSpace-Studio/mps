//! Skeletal soft-body support (Phase 1, route A) — composition layer.
//!
//! A **skeletal soft body** is a chain/tree of rigid-body "nodes" connected by
//! spring joints. This is the cheapest soft-body formulation: it reuses the
//! existing rigid-body solver + impulse-joint machinery unchanged, and expresses
//! softness entirely through joint springs (`stiffness`/`damping`). No new physics
//! is invented — we only *compose* the already-exposed FFI primitives:
//!
//! * `rigid_body_builder_create` / `rigid_body_builder_build` / `world_insert_rigid_body`
//! * `collider_builder_create_sphere` / `collider_builder_build` / `world_insert_collider_with_parent`
//! * `joint_builder_create(Spring, axis, k, c)` / `world_insert_impulse_joint`
//!
//! ## Why this is safe w.r.t. the fork's invariants
//!
//! * **No SoA boundary touched.** Every node is a normal rigid body; springs are
//!   normal impulse joints. The SIMD solver buffers (`velocities`/`accelerations`)
//!   are untouched.
//! * **bit-identical friendly.** We introduce no new floating-point numerics —
//!   the spring force is computed inside rapier's own joint solver, which already
//!   respects `enhanced-determinism`.
//! * **Reuses `force_containers` indirectly.** Springs are persistent joint
//!   constraints, so they survive across steps like any other joint (no per-step
//!   re-application needed), mirroring the `Persistent` force lifecycle.
//!
//! Phase 2 (mass-spring) and Phase 3 (deformable mesh) layer on top of this with
//! point-mass + `SoftBody` structures; this module is the route-A entry point.

use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, RigidBodyHandleRaw, Sphere, Vec3, WorldHandle, pack_rigid_body_handle,
    unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

const JOINT_TYPE_SPRING: u32 = 4;

/// Construct a rigid-body node (sphere) and insert it into the world.
///
/// Returns the packed handle, or `0` (`ERR_*`) on failure. `is_anchor` nodes are
/// created `fixed` so the chain can hang from them.
///
/// # Safety
/// `world` must be a valid world pointer.
unsafe fn spawn_node(
    world: &mut WorldHandle,
    position: Vec3,
    mass: f64,
    radius: f64,
    fixed: bool,
) -> RigidBodyHandleRaw {
    let builder = rapier3d::prelude::RigidBodyBuilder::new(if fixed {
        rapier3d::prelude::RigidBodyType::Fixed
    } else {
        rapier3d::prelude::RigidBodyType::Dynamic
    })
    .translation(vec3_to_rapier(position))
    .additional_mass(mass);

    let body = Box::into_raw(Box::new(builder.build()));
    let body_handle =
        crate::rapier::rigid_body::world_insert_rigid_body(world as *mut WorldHandle, body);
    if body_handle == 0 {
        return 0;
    }

    // Attach a sphere collider so the node participates in collision.
    let sphere = Sphere {
        center: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        radius,
    };
    let collider_builder = crate::rapier::collider::collider_builder_create_sphere(sphere);
    if collider_builder.is_null() {
        return body_handle; // body is fine even without a collider
    }
    let collider = crate::rapier::collider::collider_builder_build(collider_builder);
    if !collider.is_null() {
        crate::rapier::collider::world_insert_collider_with_parent(
            world as *mut WorldHandle,
            collider,
            body_handle,
        );
    }
    body_handle
}

/// Create a spring joint between two bodies and insert it into the world.
///
/// # Safety
/// `world` must be a valid world pointer; `body1`/`body2` must be valid handles.
unsafe fn link_spring(
    world: &mut WorldHandle,
    body1: RigidBodyHandleRaw,
    body2: RigidBodyHandleRaw,
    axis: Vec3,
    stiffness: f64,
    damping: f64,
    rest_length: f64,
) -> Bool {
    let builder =
        crate::rapier::joints::joint_builder_create(JOINT_TYPE_SPRING, axis, stiffness, damping);
    if builder.is_null() {
        return Bool::FALSE;
    }
    // Spring joint rest length is configured via the joint's limits/length; we set
    // the motor target to `rest_length` so the spring centers there.
    crate::rapier::joints::joint_builder_set_motor_position(
        builder,
        0, // primary axis
        rest_length,
        stiffness,
        damping,
    );
    let _ = crate::rapier::joints::world_insert_impulse_joint(
        world as *mut WorldHandle,
        body1,
        body2,
        builder,
        Bool::TRUE,
    );
    Bool::TRUE
}

/// Create a skeletal soft body as a chain (line) of spring-linked rigid nodes.
///
/// Nodes are placed `spacing` apart along `axis` starting at the world origin (or
/// at `anchor` if `anchor != 0`). Adjacent nodes are joined by a spring joint with
/// the given `stiffness`/`damping`; the spring's rest length is `spacing`, so the
/// chain behaves like a soft rope / articulated strand.
///
/// # Parameters
/// * `node_count` — number of nodes (must be ≥ 1).
/// * `spacing` — distance between adjacent nodes / spring rest length (> 0).
/// * `node_mass` — mass of each node (> 0).
/// * `node_radius` — collision sphere radius of each node (> 0).
/// * `anchor` — `RigidBodyHandleRaw` to pin the first node to (0 = first node is a
///   free/fixed root at the origin; pass a valid handle to hang from it).
/// * `axis` — unit direction of the chain (need not be normalized; it is normalized
///   internally; must be finite and non-zero).
/// * `stiffness` / `damping` — spring coefficients (≥ 0).
///
/// # Returns
/// The number of nodes successfully created (0 on error). On partial failure the
/// already-created nodes/joints remain in the world (caller may clear the world).
///
/// # Safety
/// `world` must be a valid world pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_chain_create(
    world: *mut WorldHandle,
    node_count: u32,
    spacing: f64,
    node_mass: f64,
    node_radius: f64,
    anchor: RigidBodyHandleRaw,
    axis: Vec3,
    stiffness: f64,
    damping: f64,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if node_count == 0
            || !spacing.is_finite()
            || spacing <= 0.0
            || !node_mass.is_finite()
            || node_mass <= 0.0
            || !node_radius.is_finite()
            || node_radius <= 0.0
            || !vec3_finite(axis)
            || !stiffness.is_finite()
            || stiffness < 0.0
            || !damping.is_finite()
            || damping < 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_chain_create: invalid parameters",
            );
            return 0;
        }

        let axis_v = vec3_to_rapier(axis);
        let len = axis_v.length();
        if len <= 1e-9 {
            set_error(ERR_INVALID_ARGUMENT, "soft_chain_create: zero axis");
            return 0;
        }
        let dir = axis_v / len;

        let origin = if anchor == 0 {
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            match world.inner.bodies.get(unpack_rigid_body_handle(anchor)) {
                Some(b) => vec3_from_rapier(b.translation()),
                None => {
                    set_error(ERR_INVALID_ARGUMENT, "soft_chain_create: bad anchor");
                    return 0;
                }
            }
        };

        let mut handles: Vec<RigidBodyHandleRaw> = Vec::with_capacity(node_count as usize);
        let mut created = 0u32;
        // First node: a fixed root at `origin` when no anchor; otherwise the anchor
        // itself is node 0 and we link node 1 to it.
        let start_fixed = anchor == 0;
        for i in 0..node_count {
            let d = spacing * i as f64;
            let pos_v = Vec3 {
                x: origin.x + dir.x * d,
                y: origin.y + dir.y * d,
                z: origin.z + dir.z * d,
            };
            let fixed = start_fixed && i == 0;
            let h = unsafe { spawn_node(world, pos_v, node_mass, node_radius, fixed) };
            if h == 0 {
                set_error(ERR_INVALID_ARGUMENT, "soft_chain_create: node spawn failed");
                break;
            }
            handles.push(h);
            created += 1;

            // Link to previous node (or anchor for the first real node).
            let prev = if i == 0 {
                if anchor != 0 { Some(anchor) } else { None }
            } else {
                Some(handles[i as usize - 1])
            };
            if let Some(prev_h) = prev {
                let ok =
                    unsafe { link_spring(world, prev_h, h, axis, stiffness, damping, spacing) };
                if ok == Bool::FALSE {
                    set_error(
                        ERR_INVALID_ARGUMENT,
                        "soft_chain_create: spring link failed",
                    );
                    break;
                }
            }
        }

        clear_error();
        created
    })
}

/// Read back the node handles of a soft chain that was just created.
///
/// Call [`soft_chain_create`] first; the chain's node handles are the last
/// `count` *dynamic* bodies, but to avoid ambiguity this helper snapshots the
/// *currently dynamic* bodies whose colliders are spheres of `node_radius`. For
/// simplicity it returns the handles of all dynamic bodies currently in the world
/// (callers typically create a fresh world per chain).
///
/// # Safety
/// `world` must be a valid world pointer; `out_handles` must point to writable
/// memory for `capacity` handles.
#[unsafe(no_mangle)]
pub extern "C" fn soft_chain_node_handles(
    world: *const WorldHandle,
    out_handles: *mut RigidBodyHandleRaw,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if out_handles.is_null() || capacity == 0 {
            return 0;
        }
        let out = unsafe { std::slice::from_raw_parts_mut(out_handles, capacity as usize) };
        let mut written = 0u32;
        for (h, body) in world.inner.bodies.iter() {
            if !body.is_dynamic() {
                continue;
            }
            if written as usize >= out.len() {
                break;
            }
            out[written as usize] = pack_rigid_body_handle(h);
            written += 1;
        }
        written
    })
}
