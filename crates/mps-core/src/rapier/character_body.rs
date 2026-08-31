//! End-to-end **character body** — a kinematic rigid body driven by a
//! [`KinematicCharacterController`]. This is the "third body type" alongside the
//! existing rigid bodies and soft bodies: a capsule/ball collider that walks,
//! slides, autosteps and snaps to the ground, with its resolved position written
//! back to a kinematic rigid body every step so it can push other bodies and be
//! queried like any other body.
//!
//! This layer reuses the collision query construction already in `controller.rs`
//! and just binds a controller to a kinematic body. No fork changes.

use rapier3d::control::{
    CharacterAutostep, CharacterCollision, CharacterLength, KinematicCharacterController,
};
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
    /// Collisions captured by the most recent `character_body_move` (cleared each
    /// call, populated by the move's collision callback). Read back via
    /// `character_body_collision_count` / `character_body_get_collision` and fed to
    /// `character_body_solve_impulses`.
    pub collisions: Vec<CharacterCollision>,
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
                collisions: Vec::new(),
            },
        );
        clear_error();
        id
    })
}

/// Change a character body's collision shape after creation. The new shape is
/// used by subsequent `character_body_move` calls (the controller shape-casts
/// the shape directly, so no world collider is rebuilt). Useful for Minecraft
/// style avatars that change hitbox (e.g. sneaking shrinks the box).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`; `shape` must be a
/// valid [`ShapeDesc`] (finite params).
#[unsafe(no_mangle)]
pub extern "C" fn character_body_set_shape(
    world: *mut WorldHandle,
    id: u32,
    shape: ShapeDesc,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !shape_desc_valid(shape) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "character_body_set_shape: invalid shape",
            );
            return Bool::FALSE;
        }
        match world.inner.character_bodies.get_mut(&id) {
            Some(cb) => {
                cb.shape = shape;
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(ERR_NOT_FOUND, "character_body_set_shape: unknown id");
                Bool::FALSE
            }
        }
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
        let mut collected: Vec<CharacterCollision> = Vec::new();
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
                |collision| collected.push(collision),
            )
        };
        // Store this step's collisions for read-back / impulse solving.
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .collisions = collected;

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

/// Reliable "is the character standing on something" check for Minecraft-style
/// jump logic. This fork's `is_grounded` classifies a capsule resting on a flat
/// floor as `sliding_down_slope` (see `is_grounded_at_contact_manifold`'s normal
/// convention), so it alone is NOT a good jump gate. This helper ORs `grounded`
/// with `is_sliding_down_slope` and additionally excludes the case where the
/// character is moving strongly upward (i.e. already jumping), giving a stable
/// on-ground signal the caller can gate jumps on.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_is_on_ground(world: *const WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        match world.inner.character_bodies.get(&id) {
            Some(cb) => {
                let touching = cb.last_movement.grounded.0 != 0
                    || cb.last_movement.is_sliding_down_slope.0 != 0;
                // Exclude an active upward jump (vertical speed well above 0).
                let rising = cb.last_movement.translation.y > 0.05;
                Bool::from(touching && !rising)
            }
            None => {
                set_error(ERR_NOT_FOUND, "character_body_is_on_ground: unknown id");
                Bool::FALSE
            }
        }
    })
}

/// Number of collisions captured by the most recent `character_body_move`. Use
/// this with [`character_body_get_collision`] to inspect what the character hit
/// (e.g. to apply custom push forces or build a contact-reporting system).
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_collision_count(world: *const WorldHandle, id: u32) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        match world.inner.character_bodies.get(&id) {
            Some(cb) => cb.collisions.len() as u32,
            None => {
                set_error(ERR_NOT_FOUND, "character_body_collision_count: unknown id");
                0
            }
        }
    })
}

/// Read the `index`-th collision captured by the most recent `character_body_move`.
/// Returns a default (all-zero) collision if `index` is out of range.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_get_collision(
    world: *const WorldHandle,
    id: u32,
    index: u32,
) -> crate::rapier::ffi::CharacterCollision {
    ffi_guard(crate::rapier::ffi::CharacterCollision::default(), || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return crate::rapier::ffi::CharacterCollision::default();
        };
        let Some(cb) = world.inner.character_bodies.get(&id) else {
            set_error(ERR_NOT_FOUND, "character_body_get_collision: unknown id");
            return crate::rapier::ffi::CharacterCollision::default();
        };
        match cb.collisions.get(index as usize) {
            Some(c) => crate::rapier::ffi::CharacterCollision {
                collider: crate::rapier::ffi::pack_collider_handle(c.handle),
                character_translation: vec3_from_rapier(c.character_pos.translation),
                translation_applied: vec3_from_rapier(c.translation_applied),
                translation_remaining: vec3_from_rapier(c.translation_remaining),
                world_witness1: vec3_from_rapier(c.hit.witness1),
                world_witness2: vec3_from_rapier(c.hit.witness2),
                normal1: vec3_from_rapier(c.hit.normal1),
                normal2: vec3_from_rapier(c.hit.normal2),
                time_of_impact: c.hit.time_of_impact,
            },
            None => {
                set_error(
                    ERR_NOT_FOUND,
                    "character_body_get_collision: index out of range",
                );
                crate::rapier::ffi::CharacterCollision::default()
            }
        }
    })
}

/// Apply the impulses accumulated from the latest `character_body_move` to the
/// dynamic bodies the character is touching. This is how a kinematic character
/// "pushes" crates/other rigid bodies — rapier does not auto-apply them; the
/// caller must invoke this after each move that reported contacts. No fork
/// changes: it forwards to the controller's `solve_character_collision_impulses`.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_solve_impulses(
    world: *mut WorldHandle,
    id: u32,
    dt: f64,
    character_mass: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !world.inner.character_bodies.contains_key(&id) {
            set_error(ERR_NOT_FOUND, "character_body_solve_impulses: unknown id");
            return Bool::FALSE;
        }
        if !dt.is_finite() || dt <= 0.0 || !character_mass.is_finite() || character_mass < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "character_body_solve_impulses: invalid dt/mass",
            );
            return Bool::FALSE;
        }
        let _query_lock = world.inner.query_lock.write();
        let shape = shape_from_desc(world.inner.character_bodies.get(&id).unwrap().shape);
        let mut query = world.inner.broad_phase.as_query_pipeline_mut(
            world.inner.narrow_phase.query_dispatcher(),
            &mut world.inner.bodies,
            &mut world.inner.colliders,
            rapier3d::prelude::QueryFilter::default(),
        );
        // Forward the captured collisions straight to the controller (no
        // reconstruction needed — they are already rapier CharacterCollision).
        world
            .inner
            .character_bodies
            .get(&id)
            .unwrap()
            .controller
            .solve_character_collision_impulses(
                dt,
                &mut query,
                shape.as_ref(),
                character_mass,
                world
                    .inner
                    .character_bodies
                    .get(&id)
                    .unwrap()
                    .collisions
                    .iter(),
            );
        clear_error();
        Bool::TRUE
    })
}

/// Like [`character_body_move`] but additionally samples the world's registered
/// terrain gravity (polyhedron / DEM / lunar-mascon) at the character's current
/// position and folds the resulting free-fall displacement (`½·a·dt²`) into the
/// desired translation, so the character falls toward and stands on an irregular
/// small-body surface instead of floating. When no terrain-gravity law is
/// registered this is identical to `character_body_move`.
///
/// # Safety
/// `world` must be a valid pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn character_body_move_with_terrain(
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
            set_error(
                ERR_NOT_FOUND,
                "character_body_move_with_terrain: unknown id",
            );
            return EffectiveCharacterMovement::default();
        }
        if !dt.is_finite() || dt <= 0.0 || !vec3_finite(desired) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "character_body_move_with_terrain: invalid dt/desired",
            );
            return EffectiveCharacterMovement::default();
        }

        // Sample terrain gravity and fold the free-fall displacement into desired.
        let mut desired = vec3_to_rapier(desired);
        if let Some(source) = &world.inner.terrain_gravity_source {
            let accel = crate::rapier::terrain_gravity::terrain_gravity_acceleration(source, {
                let body = world.inner.character_bodies.get(&id).unwrap().body;
                vec3_from_rapier(world.inner.bodies.get(body).unwrap().translation())
            });
            desired += vec3_to_rapier(accel) * (0.5 * dt * dt);
        }

        // Delegate the resolve + kinematic write-back to the shared move path.
        let body = world.inner.character_bodies.get(&id).unwrap().body;
        let current = world.inner.bodies.get(body).unwrap().translation();
        let mut collected: Vec<CharacterCollision> = Vec::new();
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
                desired,
                |collision| collected.push(collision),
            )
        };
        world
            .inner
            .character_bodies
            .get_mut(&id)
            .unwrap()
            .collisions = collected;

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
