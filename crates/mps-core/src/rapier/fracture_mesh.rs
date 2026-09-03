//! Fracture mesh bodies — dynamically fracturable composite rigid bodies.
//!
//! A **fracture mesh body** is a composite rigid body (multiple colliders under
//! one rigid body) that can fracture into separate fragments based on stress,
//! energy, or direct triggers. This module builds on top of the existing
//! `fracture.rs` mechanical computations and provides a high-level body type
//! that manages the fracture lifecycle:
//!
//! * **Pre-fracture state** — a single rigid body with multiple colliders
//!   representing the mesh geometry.
//! * **Fracture trigger** — computed from stress intensity, Griffith criterion,
//!   or direct API call.
//! * **Post-fracture state** — the original body is replaced by N independent
//!   dynamic bodies (fragments) connected by weak joints (which can break
//!   further under stress).
//!
//! This is a pure composition layer — no new physics, just lifecycle management
//! around the existing `world_replace_body_with_fracture_fragments` function.

use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyHandle};

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_UNSUPPORTED,
    clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, FractureFragmentDesc, FractureMaterial, RigidBodyHandleRaw, Vec3, WorldHandle,
    pack_rigid_body_handle, shape_from_desc, vec3_finite, vec3_to_rapier,
};

const MAX_FRACTURE_MESH_PARTS: u32 = 1024;

/// Fracture trigger mode.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FractureTrigger {
    /// Manual trigger via API call.
    Manual,
    /// Stress intensity factor exceeds threshold.
    StressIntensity { threshold: f64 },
    /// Griffith criterion (energy-based).
    Griffith { threshold: f64 },
    /// Miner fatigue damage accumulates to 1.0.
    Fatigue,
}

/// A fracture mesh body: single rigid body + metadata for potential fracture.
pub(crate) struct FractureMeshBody {
    /// The underlying rigid body (before fracture).
    pub body: RigidBodyHandle,
    /// Fracture material properties.
    pub material: FractureMaterial,
    /// Current fracture trigger mode.
    pub trigger: FractureTrigger,
    /// Pre-computed fragment descriptors (used when fracture occurs).
    pub fragments: Vec<FractureFragmentDesc>,
    /// Whether fragments should be connected by weak joints after fracture.
    pub connect_fragments: bool,
    /// Fatigue damage accumulator (0.0 to 1.0).
    pub fatigue_damage: f64,
    /// Current stress intensity (for stress-intensity trigger).
    pub current_stress_intensity: f64,
    /// Whether this body has already fractured.
    pub fractured: bool,
}

/// Shared insertion path for both creation entry points: validates the
/// fragment list, shape, and material, then inserts the source rigid body and
/// registers the mesh metadata. Returns the stable id, or `u32::MAX` (with
/// the error set).
fn insert_fracture_mesh_body(
    world: &mut WorldHandle,
    shape: crate::rapier::ffi::ShapeDesc,
    translation: Vec3,
    fragments: &[FractureFragmentDesc],
    material: FractureMaterial,
    connect_fragments: bool,
) -> u32 {
    if fragments.is_empty() || fragments.len() > MAX_FRACTURE_MESH_PARTS as usize {
        set_error(ERR_CAPACITY, "invalid fragment count");
        return u32::MAX;
    }
    if !crate::rapier::ffi::shape_desc_valid(shape) {
        set_error(ERR_INVALID_ARGUMENT, "invalid shape");
        return u32::MAX;
    }
    if !crate::rapier::fracture::material_valid(material) {
        set_error(ERR_INVALID_ARGUMENT, "invalid fracture material");
        return u32::MAX;
    }
    for frag in fragments {
        if !crate::rapier::fracture::fragment_valid(*frag) {
            set_error(ERR_INVALID_ARGUMENT, "invalid fragment descriptor");
            return u32::MAX;
        }
    }

    // Create the base rigid body
    let collider_shape = shape_from_desc(shape);
    let body = RigidBodyBuilder::dynamic()
        .translation(vec3_to_rapier(translation))
        .build();
    let body_handle = world.inner.bodies.insert(body);
    let collider = ColliderBuilder::new(collider_shape)
        .density(material.density)
        .friction(0.5)
        .restitution(0.2)
        .build();
    world
        .inner
        .colliders
        .insert_with_parent(collider, body_handle, &mut world.inner.bodies);

    let id = world.inner.fracture_mesh_next_id;
    world.inner.fracture_mesh_next_id = id.wrapping_add(1);

    world.inner.fracture_mesh_bodies.insert(
        id,
        FractureMeshBody {
            body: body_handle,
            material,
            trigger: FractureTrigger::Manual,
            fragments: fragments.to_vec(),
            connect_fragments,
            fatigue_damage: 0.0,
            current_stress_intensity: 0.0,
            fractured: false,
        },
    );

    clear_error();
    id
}

/// Create a fracture mesh body from a rigid body and pre-defined fragments.
///
/// The body is inserted into the world as a normal rigid body; the fragments
/// are stored for later use when fracture is triggered. Returns a stable id,
/// or `u32::MAX` on error.
///
/// # Safety
///
/// `world` must be a valid world pointer. `fragments` must point to
/// `fragment_count` valid descriptors.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_create(
    world: *mut WorldHandle,
    shape: crate::rapier::ffi::ShapeDesc,
    translation: Vec3,
    fragments: *const FractureFragmentDesc,
    fragment_count: u32,
    material: FractureMaterial,
    connect_fragments: Bool,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return u32::MAX;
        };
        if fragments.is_null() {
            set_error(ERR_NULL_POINTER, "fragments is null");
            return u32::MAX;
        }

        let fragments_slice =
            unsafe { std::slice::from_raw_parts(fragments, fragment_count as usize) };
        insert_fracture_mesh_body(
            world,
            shape,
            translation,
            fragments_slice,
            material,
            connect_fragments.0 != 0,
        )
    })
}

/// Create a fracture mesh body whose fragments are generated by Voronoi
/// pre-splitting.
///
/// Instead of requiring hand-authored fragment descriptors, the caller
/// supplies an AABB (in the body's local space) and a set of seed points; the
/// Voronoi cell of each seed (clipped to the AABB, bisected against every
/// other seed) is box-fitted into a `FractureFragmentDesc`. `edge_shrink` is
/// a fraction in `[0.0, 0.5)` removed from each side of every fragment's
/// half-extents so adjacent fragments start with a gap instead of
/// interpenetrating (0.0 keeps the exact cell AABB). Fragment
/// `initial_velocity` starts at zero (inherited from the source body at
/// trigger time) and `density` at 0 (inherited from the material). Duplicate
/// seeds are merged and degenerate cells skipped; at least one valid cell is
/// required. Returns a stable id, or `u32::MAX` on error.
///
/// # Safety
///
/// `world` must be a valid world pointer. `seeds` must point to
/// `seed_count` valid `Vec3`s.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_create_with_voronoi(
    world: *mut WorldHandle,
    shape: crate::rapier::ffi::ShapeDesc,
    translation: Vec3,
    aabb_min: Vec3,
    aabb_max: Vec3,
    seeds: *const Vec3,
    seed_count: u32,
    material: FractureMaterial,
    connect_fragments: Bool,
    edge_shrink: f64,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return u32::MAX;
        };
        if seeds.is_null() {
            set_error(ERR_NULL_POINTER, "seeds is null");
            return u32::MAX;
        }
        if seed_count == 0 {
            set_error(ERR_CAPACITY, "invalid seed count");
            return u32::MAX;
        }
        if !crate::rapier::ffi::shape_desc_valid(shape) {
            set_error(ERR_INVALID_ARGUMENT, "invalid shape");
            return u32::MAX;
        }
        if !crate::rapier::fracture::material_valid(material) {
            set_error(ERR_INVALID_ARGUMENT, "invalid fracture material");
            return u32::MAX;
        }
        let seeds_slice = unsafe { std::slice::from_raw_parts(seeds, seed_count as usize) };
        for seed in seeds_slice {
            if !vec3_finite(*seed) {
                set_error(ERR_INVALID_ARGUMENT, "non-finite voronoi seed");
                return u32::MAX;
            }
        }

        let template = FractureFragmentDesc {
            local_center: Vec3::default(),
            half_extents: Vec3::default(),
            initial_velocity: Vec3::default(),
            density: 0.0, // inherited from the material at trigger time
            friction: 0.5,
            restitution: 0.1,
        };
        let Some(fragments) = mps_formula::voronoi::voronoi_fragments_from_seeds(
            aabb_min,
            aabb_max,
            seeds_slice,
            template,
            edge_shrink,
        ) else {
            set_error(ERR_INVALID_ARGUMENT, "voronoi fragment generation failed");
            return u32::MAX;
        };

        insert_fracture_mesh_body(
            world,
            shape,
            translation,
            &fragments,
            material,
            connect_fragments.0 != 0,
        )
    })
}

/// Manually trigger fracture for a fracture mesh body.
///
/// The original body is replaced by its pre-defined fragments. Returns `true`
/// on success, `false` on error.
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_trigger(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        // Gather everything the fracture call needs up front so the mutable
        // borrow of `fracture_mesh_bodies` ends before the world is handed to
        // `world_replace_body_with_fracture_fragments`.
        let connect_fragments;
        let source_raw;
        let fragment_descs;
        {
            let Some(mesh_body) = world.inner.fracture_mesh_bodies.get(&id) else {
                set_error(ERR_NOT_FOUND, "fracture mesh body not found");
                return Bool::FALSE;
            };

            if mesh_body.fractured {
                set_error(ERR_UNSUPPORTED, "body already fractured");
                return Bool::FALSE;
            }

            let Some(source) = world.inner.bodies.get(mesh_body.body) else {
                set_error(ERR_NOT_FOUND, "source body missing");
                return Bool::FALSE;
            };
            let source_linvel = source.linvel();

            let mut descs: Vec<FractureFragmentDesc> =
                Vec::with_capacity(mesh_body.fragments.len());
            for frag in &mesh_body.fragments {
                let mut desc = *frag;
                // Fragments inherit the source body's linear velocity and,
                // when a descriptor leaves the density at 0, the mesh
                // material's density.
                desc.initial_velocity = Vec3 {
                    x: source_linvel.x,
                    y: source_linvel.y,
                    z: source_linvel.z,
                };
                if desc.density == 0.0 {
                    desc.density = mesh_body.material.density;
                }
                descs.push(desc);
            }

            connect_fragments = mesh_body.connect_fragments;
            source_raw = pack_rigid_body_handle(mesh_body.body);
            fragment_descs = descs;
        }

        let mut body_handles = vec![RigidBodyHandleRaw::default(); fragment_descs.len()];
        let mut joint_handles = if connect_fragments {
            Some(vec![
                crate::rapier::ffi::ImpulseJointHandleRaw::default();
                fragment_descs.len()
            ])
        } else {
            None
        };

        let result = crate::rapier::fracture::world_replace_body_with_fracture_fragments(
            world,
            source_raw,
            fragment_descs.as_ptr(),
            fragment_descs.len() as u32,
            Bool::from(connect_fragments),
            Bool::TRUE, // remove source
            body_handles.as_mut_ptr(),
            joint_handles
                .as_mut()
                .map(|j| j.as_mut_ptr())
                .unwrap_or(std::ptr::null_mut()),
            fragment_descs.len() as u32,
            std::ptr::null_mut(),
        );

        if result == Bool::TRUE {
            if let Some(mesh_body) = world.inner.fracture_mesh_bodies.get_mut(&id) {
                mesh_body.fractured = true;
            }
            clear_error();
            Bool::TRUE
        } else {
            Bool::FALSE
        }
    })
}

/// Set the fracture trigger mode for a fracture mesh body.
///
/// Trigger modes: `0` = manual (`fracture_mesh_body_trigger` only), `1` =
/// stress intensity (auto-fractures when `fracture_mesh_body_set_stress`
/// reports stress ≥ `threshold`), `2` = Griffith (energy criterion, same
/// threshold form), `3` = fatigue (auto-fractures once accumulated fatigue
/// damage reaches 1.0).
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_set_trigger(
    world: *mut WorldHandle,
    id: u32,
    mode: u32,
    threshold: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        let Some(mesh_body) = world.inner.fracture_mesh_bodies.get_mut(&id) else {
            set_error(ERR_NOT_FOUND, "fracture mesh body not found");
            return Bool::FALSE;
        };

        mesh_body.trigger = match mode {
            0 => FractureTrigger::Manual,
            1 | 2 => {
                if !threshold.is_finite() || threshold <= 0.0 {
                    set_error(ERR_INVALID_ARGUMENT, "invalid stress threshold");
                    return Bool::FALSE;
                }
                if mode == 1 {
                    FractureTrigger::StressIntensity { threshold }
                } else {
                    FractureTrigger::Griffith { threshold }
                }
            }
            3 => FractureTrigger::Fatigue,
            _ => {
                set_error(ERR_INVALID_ARGUMENT, "invalid trigger mode");
                return Bool::FALSE;
            }
        };
        clear_error();
        Bool::TRUE
    })
}

/// Set the fracture trigger mode to stress intensity (convenience wrapper
/// around `fracture_mesh_body_set_trigger` with mode `1`).
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_set_trigger_stress(
    world: *mut WorldHandle,
    id: u32,
    threshold: f64,
) -> Bool {
    fracture_mesh_body_set_trigger(world, id, 1, threshold)
}

/// Report the current stress intensity for a fracture mesh body.
///
/// Stores the value (readable for diagnostics via the trigger state) and
/// auto-fractures the body when the trigger mode is `StressIntensity` or
/// `Griffith` and the reported stress reaches the configured threshold.
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_set_stress(
    world: *mut WorldHandle,
    id: u32,
    stress: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        let over_threshold = {
            let Some(mesh_body) = world.inner.fracture_mesh_bodies.get_mut(&id) else {
                set_error(ERR_NOT_FOUND, "fracture mesh body not found");
                return Bool::FALSE;
            };

            if !stress.is_finite() || stress < 0.0 {
                set_error(ERR_INVALID_ARGUMENT, "invalid stress value");
                return Bool::FALSE;
            }

            mesh_body.current_stress_intensity = stress;
            match mesh_body.trigger {
                FractureTrigger::StressIntensity { threshold }
                | FractureTrigger::Griffith { threshold } => stress >= threshold,
                _ => false,
            }
        };
        if over_threshold {
            return fracture_mesh_body_trigger(world, id);
        }

        clear_error();
        Bool::TRUE
    })
}

/// Update fatigue damage for a fracture mesh body.
///
/// Accumulates fatigue damage; when damage reaches 1.0, the body fractures
/// automatically if the trigger mode is `Fatigue`.
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_add_fatigue_damage(
    world: *mut WorldHandle,
    id: u32,
    damage: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        let Some(mesh_body) = world.inner.fracture_mesh_bodies.get_mut(&id) else {
            set_error(ERR_NOT_FOUND, "fracture mesh body not found");
            return Bool::FALSE;
        };

        if !damage.is_finite() || damage < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid fatigue damage");
            return Bool::FALSE;
        }

        mesh_body.fatigue_damage = (mesh_body.fatigue_damage + damage).min(1.0);

        // Auto-fracture if fatigue threshold reached and trigger is Fatigue
        if matches!(mesh_body.trigger, FractureTrigger::Fatigue)
            && mesh_body.fatigue_damage >= 1.0
            && !mesh_body.fractured
        {
            let _ = fracture_mesh_body_trigger(world, id);
        }

        clear_error();
        Bool::TRUE
    })
}

/// Check if a fracture mesh body has fractured.
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_is_fractured(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        let Some(mesh_body) = world.inner.fracture_mesh_bodies.get(&id) else {
            set_error(ERR_NOT_FOUND, "fracture mesh body not found");
            return Bool::FALSE;
        };

        clear_error();
        Bool::from(mesh_body.fractured)
    })
}

/// Remove a fracture mesh body from the world.
///
/// If the body has not yet fractured, removes the original rigid body.
/// If already fractured, this is a no-op (fragments are independent bodies).
///
/// # Safety
///
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn fracture_mesh_body_remove(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };

        let Some(mesh_body) = world.inner.fracture_mesh_bodies.remove(&id) else {
            set_error(ERR_NOT_FOUND, "fracture mesh body not found");
            return Bool::FALSE;
        };

        if !mesh_body.fractured {
            // Remove the original body if it hasn't fractured yet
            world.inner.bodies.remove(
                mesh_body.body,
                &mut world.inner.islands,
                &mut world.inner.colliders,
                &mut world.inner.impulse_joints,
                &mut world.inner.multibody_joints,
                true,
            );
        }

        clear_error();
        Bool::TRUE
    })
}
