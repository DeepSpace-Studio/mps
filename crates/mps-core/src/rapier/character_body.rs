//! End-to-end **character body** — a kinematic rigid body driven by a
//! [`KinematicCharacterController`]. This is the "third body type" alongside the
//! existing rigid bodies and soft bodies: a capsule/ball collider that walks,
//! slides, autosteps and snaps to the ground, with its resolved position written
//! back to a kinematic rigid body every step so it can push other bodies and be
//! queried like any other body.
//!
//! This layer reuses the collision query construction already in `controller.rs`
//! and just binds a controller to a kinematic body. No fork changes.

use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::{QueryFilter, RigidBodyBuilder, RigidBodyHandle, RigidBodyType, Vector};

use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, EffectiveCharacterMovement, Quat, ShapeDesc, Vec3, WorldHandle, isometry_from_parts,
    shape_desc_valid, shape_from_desc, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

/// A character: a controller bound to a kinematic rigid body (which owns the
/// collider built from `shape`). `last_movement` caches the most recent resolve.
#[derive(Default)]
pub(crate) struct CharacterBody {
    pub controller: KinematicCharacterController,
    pub body: RigidBodyHandle,
    pub shape: ShapeDesc,
    pub last_movement: EffectiveCharacterMovement,
}

/// Create a character body in `world` from a collider shape and an initial
/// translation. Returns a stable id, or `u32::MAX` on bad arguments. The character
/// is a `KinematicPositionBased` rigid body so its position is driven externally
/// by [`character_body_move`].
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_create(
    world: *mut WorldHandle,
    shape: ShapeDesc,
    translation: Vec3,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return u32::MAX;
        };
        if !shape_desc_valid(shape) || !vec3_finite(translation) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "character_body_create: invalid shape/translation",
            );
            return u32::MAX;
        }

        // Kinematic body we drive from the controller's resolved movement.
        // NOTE: we deliberately do NOT insert a collider for the character. This
        // fork's `KinematicCharacterController::move_shape` has no query filter
        // parameter, so a self-collider would be caught by its own shape-cast and
        // break the resolve. The character avoids the world via `move_shape`, which
        // shape-casts against the *other* colliders; external bodies pushing the
        // character is out of scope for the controller itself.
        let mut builder = RigidBodyBuilder::new(RigidBodyType::KinematicPositionBased);
        builder = builder.translation(Vector::new(translation.x, translation.y, translation.z));
        let body = world.inner.bodies.insert(builder.build());

        let id = world.inner.character_body_next_id;
        world.inner.character_body_next_id += 1;
        world.inner.character_bodies.insert(
            id,
            CharacterBody {
                controller: KinematicCharacterController::default(),
                body,
                shape,
                last_movement: EffectiveCharacterMovement::default(),
            },
        );
        clear_error();
        id
    })
}

/// Advance the character by `desired` (a desired translation for this step). The
/// controller resolves collisions/slopes/steps and the result is written back to
/// the kinematic body. Returns the effective movement (resolved translation,
/// `grounded`, `is_sliding_down_slope`).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_move(
    world: *mut WorldHandle,
    id: u32,
    desired: Vec3,
    dt: f64,
) -> EffectiveCharacterMovement {
    ffi_guard(EffectiveCharacterMovement::default(), || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return EffectiveCharacterMovement::default();
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_move: unknown id");
            return EffectiveCharacterMovement::default();
        }
        if !dt.is_finite() || dt <= 0.0 || !vec3_finite(desired) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "character_body_move: invalid dt/desired",
            );
            return EffectiveCharacterMovement::default();
        }

        // Resolve the desired movement against the world (read-only query).
        let body = world.inner.character_bodies.get(&id).unwrap().body;
        let current = world.inner.bodies.get(body).unwrap().translation();
        let movement = {
            let cb = world.inner.character_bodies.get(&id).unwrap();
            let shape = shape_from_desc(cb.shape);
            let query = world.inner.broad_phase.as_query_pipeline(
                world.inner.narrow_phase.query_dispatcher(),
                &world.inner.bodies,
                &world.inner.colliders,
                QueryFilter::default(),
            );
            cb.controller.move_shape(
                dt,
                &query,
                shape.as_ref(),
                &isometry_from_parts(vec3_from_rapier(current), Quat::default()),
                vec3_to_rapier(desired),
                |_collision| {},
            )
        };

        // `movement.translation` is the *delta* to apply this step; add it to the
        // current pose and queue it as the next kinematic translation so the
        // following `world_step` applies it and refreshes the broad phase.
        let new_pos = current + movement.translation;
        if let Some(rb) = world.inner.bodies.get_mut(body) {
            rb.set_next_kinematic_translation(new_pos);
        }

        let result = EffectiveCharacterMovement {
            translation: vec3_from_rapier(movement.translation),
            grounded: movement.grounded.into(),
            is_sliding_down_slope: movement.is_sliding_down_slope.into(),
        };
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .last_movement = result;
        clear_error();
        result
    })
}

/// Set the character's up vector (used for slope/ground semantics). Defaults to
/// world +Y. Mirrors `KinematicCharacterController::setUp`.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_up(world: *mut WorldHandle, id: u32, up: Vec3) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_set_up: unknown id");
            return Bool::FALSE;
        }
        if !vec3_finite(up) {
            set_error(ERR_INVALID_ARGUMENT, "character_body_set_up: non-finite up");
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .up = vec3_to_rapier(up);
        clear_error();
        Bool::TRUE
    })
}

/// Set the character controller's skin/offset (absolute, in metres).
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_offset_absolute(
    world: *mut WorldHandle,
    id: u32,
    offset: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(
                ERR_NOT_FOUND,
                "character_body_set_offset_absolute: unknown id",
            );
            return Bool::FALSE;
        }
        if !offset.is_finite() || offset < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "offset must be finite and non-negative",
            );
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .offset = CharacterLength::Absolute(offset);
        clear_error();
        Bool::TRUE
    })
}

/// Set the character controller's skin/offset (relative, as a fraction of the
/// shape's dimensions).
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_offset_relative(
    world: *mut WorldHandle,
    id: u32,
    offset: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(
                ERR_NOT_FOUND,
                "character_body_set_offset_relative: unknown id",
            );
            return Bool::FALSE;
        }
        if !offset.is_finite() || offset < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "offset must be finite and non-negative",
            );
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .offset = CharacterLength::Relative(offset);
        clear_error();
        Bool::TRUE
    })
}

/// Enable / disable auto-stepping so the character can climb block-sized ledges
/// (e.g. a 1-metre Minecraft step). `max_height` and `min_width` are absolute
/// metres; `include_dynamic_bodies` lets the step ride on moving platforms.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_autostep(
    world: *mut WorldHandle,
    id: u32,
    enabled: Bool,
    max_height: f64,
    min_width: f64,
    include_dynamic_bodies: Bool,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_set_autostep: unknown id");
            return Bool::FALSE;
        }
        if enabled.0 != 0
            && (!max_height.is_finite()
                || !min_width.is_finite()
                || max_height < 0.0
                || min_width < 0.0)
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "autostep dimensions must be finite and non-negative",
            );
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .autostep = if enabled.0 != 0 {
            Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(max_height),
                min_width: CharacterLength::Absolute(min_width),
                include_dynamic_bodies: include_dynamic_bodies.0 != 0,
            })
        } else {
            None
        };
        clear_error();
        Bool::TRUE
    })
}

/// Enable / disable snap-to-ground so the character sticks to block surfaces
/// instead of floating a hair above them after a step.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_snap_to_ground(
    world: *mut WorldHandle,
    id: u32,
    enabled: Bool,
    distance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(
                ERR_NOT_FOUND,
                "character_body_set_snap_to_ground: unknown id",
            );
            return Bool::FALSE;
        }
        if enabled.0 != 0 && (!distance.is_finite() || distance < 0.0) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "snap distance must be finite and non-negative",
            );
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .snap_to_ground = if enabled.0 != 0 {
            Some(CharacterLength::Absolute(distance))
        } else {
            None
        };
        clear_error();
        Bool::TRUE
    })
}

/// Set the slope-climb / slope-slide angles (radians). Tune these so the
/// character climbs gentle block ramps but slides down steep ones.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_slope_angles(
    world: *mut WorldHandle,
    id: u32,
    max_climb_angle: f64,
    min_slide_angle: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_set_slope_angles: unknown id");
            return Bool::FALSE;
        }
        if !max_climb_angle.is_finite() || !min_slide_angle.is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "slope angles must be finite");
            return Bool::FALSE;
        }
        let cb = world.inner.character_bodies.get_mut(&id).unwrap();
        cb.controller.max_slope_climb_angle = max_climb_angle;
        cb.controller.min_slope_slide_angle = min_slide_angle;
        clear_error();
        Bool::TRUE
    })
}

/// Enable / disable sliding along walls/floors when the character is blocked.
/// `slide = true` gives the smooth Minecraft-style "glide along a wall" feel;
/// `slide = false` makes the character stop dead on contact.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_slide(world: *mut WorldHandle, id: u32, slide: Bool) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_set_slide: unknown id");
            return Bool::FALSE;
        }
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .controller
            .slide = slide.0 != 0;
        clear_error();
        Bool::TRUE
    })
}

/// Whether the character was on the ground during the last `character_body_move`.
/// Essential for Minecraft-style jump logic (only jump when grounded).
#[unsafe(no_mangle)]
pub extern "C" fn character_body_is_grounded(world: *const WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        match world.inner.character_bodies.get(&id) {
            Some(cb) => cb.last_movement.grounded,
            None => {
                set_error(ERR_NOT_FOUND, "character_body_is_grounded: unknown id");
                Bool::FALSE
            }
        }
    })
}

/// Whether the character was sliding down a slope during the last
/// `character_body_move`. Useful for Minecraft-style ice/slide behaviour.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_is_sliding_down_slope(world: *const WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        match world.inner.character_bodies.get(&id) {
            Some(cb) => cb.last_movement.is_sliding_down_slope,
            None => {
                set_error(
                    ERR_NOT_FOUND,
                    "character_body_is_sliding_down_slope: unknown id",
                );
                Bool::FALSE
            }
        }
    })
}

/// Destroy a character body, removing its rigid body, collider and controller
/// state. Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_destroy(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        match world.inner.character_bodies.remove(&id) {
            Some(cb) => {
                // Removing the rigid body also frees its parented collider.
                world.inner.bodies.remove(
                    cb.body,
                    &mut world.inner.islands,
                    &mut world.inner.colliders,
                    &mut world.inner.impulse_joints,
                    &mut world.inner.multibody_joints,
                    false,
                );
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(ERR_NOT_FOUND, "character_body_destroy: unknown id");
                Bool::FALSE
            }
        }
    })
}

/// Read the character body's current world-space translation (the kinematic body
/// pose driven by [`character_body_move`]). Writes into `out` when non-null.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`; `out` may be null.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_get_translation(
    world: *const WorldHandle,
    id: u32,
    out: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        match world.inner.character_bodies.get(&id) {
            Some(cb) => {
                let t = world.inner.bodies.get(cb.body).unwrap().translation();
                if !out.is_null() {
                    unsafe { *out = vec3_from_rapier(t) };
                }
                Bool::TRUE
            }
            None => {
                set_error(ERR_NOT_FOUND, "character_body_get_translation: unknown id");
                Bool::FALSE
            }
        }
    })
}
