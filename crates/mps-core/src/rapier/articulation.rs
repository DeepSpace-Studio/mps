//! Articulated bodies — kinematic chains of rigid links driven by revolute
//! joint motors (composition layer).
//!
//! An **articulation body** is a serial chain of ball rigid bodies strung along
//! a straight line, where each consecutive pair is connected by a revolute
//! impulse joint with a position motor (`RevoluteJointBuilder::motor_position`).
//! Like `soft_chain_create` / `cloth` / `rope` this module *composes* existing
//! primitives — no new physics, no SoA boundary touched:
//!
//! * **Links** — dynamic rigid bodies (ball colliders, `additional_mass`),
//!   laid out at `base + dir · i · spacing`.
//! * **Joints** — revolute about `joint_axis`, anchors at the shared boundary
//!   between neighbouring balls (`dir · spacing/2` on both local frames).
//! * **Servo springs** — the joints are *multibody* joints (rapier's
//!   `MultibodyJointSet`, not impulse joints); each carries an implicit
//!   backward-Euler joint spring toward `target[i]` (`set_spring`), which is
//!   unconditionally stable even on low-inertia links (an explicit position
//!   motor injects energy and explodes — the fork's `Multibody` docs call this
//!   out explicitly). Targets default to `0` when the caller passes fewer
//!   entries than joints; runtime retargeting via
//!   `articulation_body_set_joint_target` edits the spring's rest position.
//!
//! Everything else is inherited from the rigid-body surface: link handles
//! (`articulation_body_link_handle`) work with the existing `rigid_body_*`,
//! `aero_*`, `fluid_*` and force FFI exactly like any other rigid body.

use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, Vec3, WorldHandle, pack_rigid_body_handle, vec3_finite, vec3_to_rapier,
};
use rapier3d::dynamics::{MultibodyJointHandle, RigidBodyHandle};
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyType};

/// One articulated chain: rigid links + the revolute motor joints between them.
pub(crate) struct ArticulationBody {
    /// Chain links; index 0 is the base link.
    pub(crate) links: Vec<RigidBodyHandle>,
    /// Multibody joint `i` connects link `i` and link `i + 1`.
    pub(crate) joints: Vec<MultibodyJointHandle>,
}

/// Create an articulated chain and return its id, or `u32::MAX` on error.
///
/// `dir` is the chain direction (normalised internally), `joint_axis` the
/// local-space rotation axis of every revolute joint (must not be parallel to
/// `dir` — a perpendicular axis gives a planar arm). `target_angles` may be
/// null or shorter than `link_count − 1`; missing targets default to `0`.
///
/// # Safety
///
/// `world` must be a valid world pointer or null; `target_angles` must be null
/// or point to readable memory for `targets_len` doubles.
#[unsafe(no_mangle)]
pub extern "C" fn articulation_body_create(
    world: *mut WorldHandle,
    base: Vec3,
    dir: Vec3,
    joint_axis: Vec3,
    link_count: u32,
    link_radius: f64,
    link_mass: f64,
    target_angles: *const f64,
    targets_len: u32,
    stiffness: f64,
    damping: f64,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "articulation_body_create: world is null");
            return u32::MAX;
        };
        if !(2..=256).contains(&link_count)
            || !vec3_finite(base)
            || !vec3_finite(dir)
            || !vec3_finite(joint_axis)
            || !link_radius.is_finite()
            || link_radius <= 0.0
            || !link_mass.is_finite()
            || link_mass <= 0.0
            || !stiffness.is_finite()
            || stiffness < 0.0
            || !damping.is_finite()
            || damping < 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "articulation_body_create: bad parameters",
            );
            return u32::MAX;
        }
        let dv = vec3_to_rapier(dir);
        let dir_len = dv.length();
        if dir_len <= 1e-9 {
            set_error(ERR_INVALID_ARGUMENT, "articulation_body_create: zero dir");
            return u32::MAX;
        }
        let dir_u = dv / dir_len;
        let axis = vec3_to_rapier(joint_axis);
        let axis_len = axis.length();
        if axis_len <= 1e-9 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "articulation_body_create: zero joint axis",
            );
            return u32::MAX;
        }
        let axis_u = axis / axis_len;
        if dir_u.cross(axis_u).length() < 1e-6 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "articulation_body_create: joint axis parallel to chain dir",
            );
            return u32::MAX;
        }
        let targets: Vec<f64> = if target_angles.is_null() || targets_len == 0 {
            Vec::new()
        } else {
            let src = unsafe { std::slice::from_raw_parts(target_angles, targets_len as usize) };
            src.to_vec()
        };
        let mut finite = true;
        for t in &targets {
            finite &= t.is_finite();
        }
        if !finite {
            set_error(
                ERR_INVALID_ARGUMENT,
                "articulation_body_create: non-finite target",
            );
            return u32::MAX;
        }

        let base_pos = vec3_to_rapier(base);
        let spacing = link_radius * 2.0;
        let mut links: Vec<RigidBodyHandle> = Vec::with_capacity(link_count as usize);
        let mut joints: Vec<MultibodyJointHandle> = Vec::with_capacity(link_count as usize - 1);
        for i in 0..link_count as usize {
            let pos = base_pos + dir_u * (i as f64 * spacing);
            let rb = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .translation(pos)
                .build();
            let handle = world.inner.bodies.insert(rb);
            let col = ColliderBuilder::ball(link_radius).density(0.0).build();
            world
                .inner
                .colliders
                .insert_with_parent(col, handle, &mut world.inner.bodies);
            // Additional mass on the body (collider density 0 above).
            if let Some(body) = world.inner.bodies.get_mut(handle) {
                body.set_additional_mass(link_mass, true);
                if i == 0 {
                    // Fixed shoulder: a serial arm is anchored at its base
                    // link (the caller can instead anchor link 0 to a rigid
                    // body via joints, but a free-floating arm cannot reach a
                    // target pose under its own motor torques alone).
                    body.set_body_type(RigidBodyType::Fixed, true);
                }
            }
            if i > 0 {
                let anchor = dir_u * (spacing / 2.0);
                // Adjacent shells are exactly tangent at spawn: leave contacts
                // enabled only between non-neighbouring links (the solver does
                // not collide joint-connected bodies anyway; neighbours would).
                let joint = rapier3d::dynamics::RevoluteJointBuilder::new(axis_u)
                    .local_anchor1(anchor)
                    .local_anchor2(-anchor)
                    .contacts_enabled(false)
                    .build();
                let target = targets.get(i - 1).copied().unwrap_or(0.0);
                let Some(jh) =
                    world
                        .inner
                        .multibody_joints
                        .insert(links[i - 1], handle, joint, true)
                else {
                    set_error(
                        ERR_INVALID_ARGUMENT,
                        "articulation_body_create: joint rejected",
                    );
                    return u32::MAX;
                };
                // Implicit servo spring toward the initial target. `insert`
                // already created the MultibodyLink; set the spring on it.
                if let Some((mb, link_id)) = world.inner.multibody_joints.get_mut(jh)
                    && let Some(link) = mb.link_mut(link_id)
                {
                    // Angular X is the revolute DoF (index 3 = DIM + 0).
                    link.joint.set_spring(3, stiffness, target);
                }
                joints.push(jh);
            }
            links.push(handle);
        }

        clear_error();
        world
            .inner
            .articulations
            .insert(ArticulationBody { links, joints })
    })
}

/// Rapier handle of chain link `index` (0 = base), for use with the existing
/// `rigid_body_*` / force FFI. Returns `0` on error.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn articulation_body_link_handle(
    world: *const WorldHandle,
    id: u32,
    link_index: u32,
) -> u64 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(
                ERR_NULL_POINTER,
                "articulation_body_link_handle: world is null",
            );
            return 0;
        };
        let Some(body) = world.inner.articulations.get(id) else {
            set_error(ERR_NOT_FOUND, "articulation_body_link_handle: unknown id");
            return 0;
        };
        match body.links.get(link_index as usize) {
            Some(h) => {
                clear_error();
                pack_rigid_body_handle(*h)
            }
            None => {
                set_error(
                    ERR_NOT_FOUND,
                    "articulation_body_link_handle: bad link index",
                );
                0
            }
        }
    })
}

/// Number of links in an articulation. Returns `u32::MAX` for an unknown id.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn articulation_body_link_count(world: *const WorldHandle, id: u32) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(
                ERR_NULL_POINTER,
                "articulation_body_link_count: world is null",
            );
            return u32::MAX;
        };
        let Some(body) = world.inner.articulations.get(id) else {
            set_error(ERR_NOT_FOUND, "articulation_body_link_count: unknown id");
            return u32::MAX;
        };
        clear_error();
        body.links.len() as u32
    })
}

/// Retarget joint `joint_index`'s position motor at runtime (0-based, joint `i`
/// drives link `i` relative to link `i-1`). Reuses the gains stored at
/// creation. The whole chain is woken up so the new target takes effect
/// immediately. Returns `Bool::TRUE` on success.
///
/// # Safety
///
/// `world` must be a valid world pointer or null.
#[unsafe(no_mangle)]
pub extern "C" fn articulation_body_set_joint_target(
    world: *mut WorldHandle,
    id: u32,
    joint_index: u32,
    target_angle: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "articulation_body_set_joint_target: world is null",
            );
            return Bool::FALSE;
        };
        if !target_angle.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "articulation_body_set_joint_target: bad angle",
            );
            return Bool::FALSE;
        }
        let Some(body) = world.inner.articulations.get_mut(id) else {
            set_error(
                ERR_NOT_FOUND,
                "articulation_body_set_joint_target: unknown id",
            );
            return Bool::FALSE;
        };
        let Some(jh) = body.joints.get(joint_index as usize).copied() else {
            set_error(
                ERR_NOT_FOUND,
                "articulation_body_set_joint_target: bad joint index",
            );
            return Bool::FALSE;
        };
        let Some((mb, link_id)) = world.inner.multibody_joints.get_mut(jh) else {
            set_error(
                ERR_NOT_FOUND,
                "articulation_body_set_joint_target: joint gone",
            );
            return Bool::FALSE;
        };
        if let Some(link) = mb.link_mut(link_id) {
            // Keep the stiffness; move the spring's rest position (= target).
            let (k, _) = link.joint.spring(3);
            link.joint.set_spring(3, k, target_angle);
        }
        clear_error();
        Bool::TRUE
    })
}
