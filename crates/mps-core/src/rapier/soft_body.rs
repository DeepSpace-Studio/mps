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
    ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, RigidBodyHandleRaw, Sphere, Vec3, WorldHandle, pack_rigid_body_handle,
    unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};
use rapier3d::math::Vector;
use rapier3d::prelude::soft_body::{SoftBody, SoftBodyId, SoftSolver};
use rapier3d::prelude::{
    ColliderBuilder, ColliderHandle, RigidBodyBuilder, RigidBodyHandle, RigidBodyType,
};
use std::collections::HashSet;

/// Phase 5d bookkeeping: which voxel cell maps to which soft-body particle.
///
/// Built by [`soft_body_voxel_build`] alongside the body, then consulted/updated
/// by [`soft_body_voxel_dig`] so a dug-out cell can be mapped back to the exact
/// particle to remove. `map[cell_linear] == particle_index` (or `-1` if the cell
/// was empty or already dug). After each dig the map is rebuilt to mirror
/// `SoftBody::remove_particle`'s index shift (every surviving index `> removed`
/// is decremented).
pub(crate) struct VoxelSoftMeta {
    pub sx: usize,
    pub sy: usize,
    pub sz: usize,
    /// World-space origin of the soft body's voxel grid (cell (0,0,0) center is
    /// `origin + 0.5 * voxel_size`). Needed by Phase 5g to map a dug collider
    /// cell (in a different grid's coordinate space) back to this soft body's
    /// cell coordinates via world-space overlap.
    pub origin: Vec3,
    /// Uniform voxel edge length of the soft body's grid (matching the
    /// `voxel_size` argument of `soft_body_voxel_build`).
    pub voxel_size: f64,
    pub map: Vec<i64>,
}

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

// ── Phase 4: Minecraft voxel → soft-body + terrain-gravity coupling ──────────
//
// Phase 4 把软体接到 Minecraft 联动链路上：
// * `soft_body_voxel_build`：把一个 `VoxelGrid`（Minecraft 区块）转成质点-弹簧软体——
//   每个实心方块中心放一个质点，面相邻实心方块之间连弹簧（Phase 2 mass-spring 力，
//   走 `SoftBody::add_spring`；自由质点由 `SoftBody::step` 独立积分，绑定质点走
//   Phase 2 的 force_containers 力路由）。软体插入 `world.inner.soft_bodies`
//   （Phase 0b 接入点），由 `world_step` 统一推进。
// * `soft_body_set_gravity`：设置软体的每体常量加速度（地形重力耦合钩子）。调用方
//   每步用 `terrain_gravity_acceleration` 采样后写入此处，软体即吃到行星/球体引力。
//
// 区块破坏 → 软体更新：调用方先改 `VoxelGrid` 再重建软体（或后续接 Phase 2 的删边）；
// 本 MVP 提供从 voxel 生成软体的入口，破坏联动在 Minecraft 侧删 cell 后重建即可。
//
// 注意：`anvilkit` entity→SoftBody 联动（Phase 4 第三项）依赖 `anvilkit-bridge`
// feature，而该 feature 当前 `--all-features` 下有既存编译错误，故本 MVP 不含
// anvilkit 路径；voxel + 地形重力两条可独立验证，无需 anvilkit。

/// Build a mass-spring soft body from a `VoxelGrid` (Minecraft chunk).
///
/// One point-mass particle is placed at the center of every *solid* voxel
/// (`voxels[i] != 0`). Face-adjacent solid voxels are connected by a Hookean
/// spring with the given `stiffness`/`damping` and rest length equal to the
/// cell spacing along that axis. The resulting [`SoftBody`] is inserted into the
/// world's `soft_bodies` set and advanced by `world_step`.
///
/// # Parameters
/// * `voxels` — flat `size_x * size_y * size_z` array, indexing `x + size_x*(z +
///   size_z*y)`; non-zero = solid.
/// * `size_x/y/z` — grid dimensions (each > 0, product ≤ `voxels.len()`).
/// * `voxel_size` — world-space size of one cell edge (uniform; > 0).
/// * `origin` — world-space position of the (0,0,0) cell corner.
/// * `particle_mass` — mass of each solid-cell particle (> 0).
/// * `stiffness` / `damping` — spring coefficients (≥ 0).
/// * `pin_boundary` — when non-zero, particles whose cell touches the grid edge
///   are created pinned (`inv_mass = 0`), so the soft body is anchored to the
///   chunk boundary (useful for hanging terrain/structures from the world).
///
/// # Returns
/// The `SoftBodyId` (as `u32`) on success, or `0` on error (`ERR_*`).
///
/// # Safety
/// `world` must be a valid world pointer; `voxels` must point to `voxels_len`
/// readable bytes.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_voxel_build(
    world: *mut WorldHandle,
    voxels: *const u8,
    voxels_len: u32,
    size_x: u32,
    size_y: u32,
    size_z: u32,
    voxel_size: f64,
    origin: Vec3,
    particle_mass: f64,
    stiffness: f64,
    damping: f64,
    pin_boundary: Bool,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0;
        };
        if voxels.is_null()
            || voxels_len == 0
            || size_x == 0
            || size_y == 0
            || size_z == 0
            || !voxel_size.is_finite()
            || voxel_size <= 0.0
            || !particle_mass.is_finite()
            || particle_mass <= 0.0
            || !stiffness.is_finite()
            || stiffness < 0.0
            || !damping.is_finite()
            || damping < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_voxel_build: bad params");
            return 0;
        }
        let n = size_x as usize * size_y as usize * size_z as usize;
        if n > voxels_len as usize {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_voxel_build: grid exceeds voxels_len",
            );
            return 0;
        }
        let grid = unsafe { std::slice::from_raw_parts(voxels, voxels_len as usize) };

        let mut body = rapier3d::prelude::soft_body::SoftBody::new(Vector::ZERO);
        // Map (x,y,z) -> particle index; only solid cells get a particle.
        let mut index: Vec<i64> = vec![-1; n];
        let mut count = 0u32;
        for y in 0..size_y as usize {
            for z in 0..size_z as usize {
                for x in 0..size_x as usize {
                    let gx = x as f64 * voxel_size;
                    let gy = y as f64 * voxel_size;
                    let gz = z as f64 * voxel_size;
                    let pos = Vec3 {
                        x: origin.x + gx + 0.5 * voxel_size,
                        y: origin.y + gy + 0.5 * voxel_size,
                        z: origin.z + gz + 0.5 * voxel_size,
                    };
                    let solid = grid[x + size_x as usize * (z + size_z as usize * y)] != 0;
                    if !solid {
                        continue;
                    }
                    let at_boundary = pin_boundary == Bool::TRUE
                        && (x == 0
                            || y == 0
                            || z == 0
                            || x + 1 == size_x as usize
                            || y + 1 == size_y as usize
                            || z + 1 == size_z as usize);
                    let idx = if at_boundary {
                        body.add_pinned(vec3_to_rapier(pos))
                    } else {
                        let i = body.add_particle(vec3_to_rapier(pos));
                        // set mass via inverse mass (1/m)
                        body.particles[i].inv_mass = 1.0 / particle_mass;
                        i
                    };
                    index[x + size_x as usize * (z + size_z as usize * y)] = idx as i64;
                    count += 1;
                }
            }
        }
        if count == 0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_voxel_build: no solid voxels",
            );
            return 0;
        }

        // Connect face-adjacent solid cells with springs (avoid double-linking by
        // only scanning +X, +Y, +Z neighbours).
        let sx = size_x as usize;
        let sz = size_z as usize;
        let at = |x: usize, y: usize, z: usize| x + sx * (z + sz * y);
        let neighbours = [(1, 0, 0), (0, 1, 0), (0, 0, 1)];
        for y in 0..size_y as usize {
            for z in 0..sz {
                for x in 0..sx {
                    let a = index[at(x, y, z)];
                    if a < 0 {
                        continue;
                    }
                    for (dx, dy, dz) in neighbours {
                        let nx = x + dx;
                        let ny = y + dy;
                        let nz = z + dz;
                        if nx >= sx || ny >= size_y as usize || nz >= sz {
                            continue;
                        }
                        let b = index[at(nx, ny, nz)];
                        if b < 0 {
                            continue;
                        }
                        body.add_spring(a as usize, b as usize, stiffness, damping);
                    }
                }
            }
        }

        clear_error();
        let id = world.inner.soft_bodies.insert(body);
        // Phase 5d: record the voxel→particle mapping so a later dig can resolve
        // which cell maps to which particle index (and rebuild after shifts).
        world.inner.voxel_soft_meta.insert(
            id.0,
            VoxelSoftMeta {
                sx,
                sy: size_y as usize,
                sz,
                origin,
                voxel_size,
                map: index.clone(),
            },
        );
        id.0
    })
}

/// Set the per-body constant acceleration (gravity) of a soft body.
///
/// This is the terrain-gravity coupling hook: the caller samples
/// `terrain_gravity_acceleration` per step and writes the resulting vector here,
/// so a soft body falls under planetary/spherical gravity instead of the world's
/// uniform `gravity`. Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_gravity(world: *mut WorldHandle, id: u32, gravity: Vec3) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !vec3_finite(gravity) {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_set_gravity: bad gravity");
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_gravity: unknown id");
            return Bool::FALSE;
        };
        body.gravity = vec3_to_rapier(gravity);
        clear_error();
        Bool::TRUE
    })
}

// ── Phase 7: wind / air-resistance + sleeping + diagnostics ───────────────
//
// 复用 rapier 已有机制：
//   * `SoftBody::apply_wind(accel, drag)` / `clear_wind()` — 纯外力（与重力同路）。
//   * `SoftBodySet::sleep/wake/is_sleeping` — 粗粒度休眠标志（跳过 step）。
//   * `SoftBody::kinetic_energy()` / `total_volume()` — 标量诊断（能量/体积守恒）。

/// Phase 7: enable a uniform wind / air-resistance field on a soft body.
///
/// `accel` is a constant wind acceleration (`m/s²`) applied to every free
/// particle (like a sideways gravity); `drag` is a linear air-resistance
/// coefficient (`1/s`, `F_drag = -m·drag·v`). Both components must be finite.
///
/// # Returns
/// `Bool::TRUE` on success, `Bool::FALSE` on `ERR_*` (null world, bad id,
/// non-finite arguments).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_apply_wind(
    world: *mut WorldHandle,
    id: u32,
    accel: Vec3,
    drag: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !vec3_finite(accel) || !drag.is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_apply_wind: bad accel/drag");
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_apply_wind: unknown id");
            return Bool::FALSE;
        };
        body.apply_wind(vec3_to_rapier(accel), drag);
        clear_error();
        Bool::TRUE
    })
}

/// Phase 7: disable the wind field on a soft body (`None`).
///
/// # Returns
/// `Bool::TRUE` on success, `Bool::FALSE` on `ERR_*` (null world, bad id).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_clear_wind(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_clear_wind: unknown id");
            return Bool::FALSE;
        };
        body.clear_wind();
        clear_error();
        Bool::TRUE
    })
}

/// Phase 7: mark a soft body as sleeping (no further integration until woken).
///
/// # Returns
/// `Bool::TRUE` if the body existed and was put to sleep, `Bool::FALSE` on
/// `ERR_*` (null world, bad id).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_sleep(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        if !world.inner.soft_bodies.sleep(sid) {
            set_error(ERR_NOT_FOUND, "soft_body_sleep: unknown id");
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Phase 7: wake a sleeping soft body (resume integration).
///
/// # Returns
/// `Bool::TRUE` if the body existed and was woken, `Bool::FALSE` on `ERR_*`
/// (null world, bad id).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_wake(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        if !world.inner.soft_bodies.wake(sid) {
            set_error(ERR_NOT_FOUND, "soft_body_wake: unknown id");
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// Phase 7: whether a soft body is currently sleeping.
///
/// # Returns
/// `Bool::TRUE` if sleeping, `Bool::FALSE` if awake or the id is unknown /
/// world is null (and `ERR_*` is set).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_is_sleeping(world: *const WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        if world.inner.soft_bodies.get(sid).is_none() {
            set_error(ERR_NOT_FOUND, "soft_body_is_sleeping: unknown id");
            return Bool::FALSE;
        }
        // `is_sleeping` returns false for unknown ids too; we already checked.
        clear_error();
        Bool::from(world.inner.soft_bodies.is_sleeping(sid))
    })
}

/// Phase 7: total kinetic energy of a soft body's free particles (`½·m·|v|²`).
///
/// # Returns
/// The kinetic energy (finite), or `0.0` with `ERR_*` set on null world / bad id.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_kinetic_energy(world: *const WorldHandle, id: u32) -> f64 {
    ffi_guard(0.0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0.0;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_kinetic_energy: unknown id");
            return 0.0;
        };
        clear_error();
        body.kinetic_energy()
    })
}

/// Phase 7: normalized total volume of a soft body's tetrahedra
/// (sum of `|V|/|V_rest|`, so a unit-scaled, deformation-sensitive scalar).
/// For bodies with no tetrahedra this is `0.0`.
///
/// # Returns
/// The normalized volume (finite), or `0.0` with `ERR_*` set on null world /
/// bad id.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_total_volume(world: *const WorldHandle, id: u32) -> f64 {
    ffi_guard(0.0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return 0.0;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_total_volume: unknown id");
            return 0.0;
        };
        clear_error();
        body.total_volume()
    })
}

/// # Phase 8 — 锚定软体任意质点到刚体
///
/// 把 `id` 软体的第 `particle` 号质点绑定到刚体 `body`，使其刚性跟随该刚体的
/// 平移/旋转。`attach_point` 为绑点世界坐标（通常用该质点当前位置）；函数内部
/// 把它换算成刚体局部坐标存储，故跟随刚体运动时不会漂移。绑定后该质点停止本地
/// 积分，其弹簧/阻尼力改由 `SoftBodySet::write_spring_forces` 路由进刚体的
/// `force_containers`（软体拖动刚体）。
///
/// # Returns
/// `Bool::TRUE` 成功；`particle` 越界 / `body` 不存在 / world 为 null 返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效；`attach_point` 各分量需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_attach_particle(
    world: *mut WorldHandle,
    id: u32,
    particle: u32,
    body: RigidBodyHandleRaw,
    attach_point: Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !vec3_finite(attach_point) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_attach_particle: non-finite point",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_attach_particle: unknown id");
            return Bool::FALSE;
        };
        let rbh = unpack_rigid_body_handle(body);
        if world.inner.bodies.get(rbh).is_none() {
            set_error(ERR_NOT_FOUND, "soft_body_attach_particle: unknown body");
            return Bool::FALSE;
        }
        let pt = vec3_to_rapier(attach_point);
        match sb.attach_particle(particle as usize, rbh, pt, &world.inner.bodies) {
            true => {
                clear_error();
                Bool::TRUE
            }
            false => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_attach_particle: particle out of range",
                );
                Bool::FALSE
            }
        }
    })
}

/// # Phase 8 — 解除质点与刚体的锚定
///
/// 把 `id` 软体的第 `particle` 号质点从任何已绑定刚体上解绑，恢复为自由（本地积分）
/// 质点。已自由则视为成功（幂等）。
///
/// # Returns
/// `Bool::TRUE` 成功（含已自由）；`particle` 越界 / world 为 null 返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_detach_particle(
    world: *mut WorldHandle,
    id: u32,
    particle: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_detach_particle: unknown id");
            return Bool::FALSE;
        };
        match sb.detach_particle(particle as usize) {
            true => {
                clear_error();
                Bool::TRUE
            }
            false => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_detach_particle: particle out of range",
                );
                Bool::FALSE
            }
        }
    })
}

/// # Phase 9 — 设置撕裂阈值（应变阈值）
///
/// 把 `id` 软体的撕裂阈值设为 `strain_to_break`（应变 = `(|len| − rest)/rest`，
/// 即拉伸量相对静止长度的比例）。每步 `step` 开始时，任何应变超过该阈值的**结构边**
/// （XPBD distance constraint 或 MassSpring spring）会被移除；失去任一结构边的三角形面
/// 也会被删掉，使撕裂的布料停止渲染破损面。
///
/// - `enabled != 0` 且 `strain_to_break > 0`：开启撕裂（阈值 = `strain_to_break`）。
/// - `enabled == 0`：关闭撕裂（等同于 `tear_strain = None`，默认）。
/// - `strain_to_break <= 0`：视为非法，关闭撕裂（避免首步即全撕）。
///
/// # Returns
/// `Bool::TRUE` 成功（含关闭）；`id` 未知 / world 为 null 返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效；`strain_to_break` 需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_tear_strain(
    world: *mut WorldHandle,
    id: u32,
    strain_to_break: f64,
    enabled: u8,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !strain_to_break.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_tear_strain: non-finite threshold",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_tear_strain: unknown id");
            return Bool::FALSE;
        };
        let strain = if enabled != 0 && strain_to_break > 0.0 {
            Some(strain_to_break)
        } else {
            None
        };
        sb.set_tear_strain(strain);
        clear_error();
        Bool::TRUE
    })
}

/// # Phase 10 — 设置塑性参数（永久变形 / 像橡皮泥 / 记忆棉）
///
/// 把 `id` 软体的塑性设为 `PlasticityParams { yield_strain, creep }`：
/// - 任何结构边（XPBD distance constraint 或 MassSpring spring）的弹性应变幅度
///   `|(|len| − rest)/rest|` 超过 `yield_strain` 时，每步把 rest_length 朝当前长度
///   方向移动 `creep`（夹到 `[0,1]`），使变形永久"冻住"而不是回弹。
/// - `enabled != 0` 且 `yield_strain > 0`：开启塑性（threshold=yield_strain, rate=creep）。
/// - `enabled == 0`：关闭塑性（等同于 `plasticity = None`，即完全弹性，默认）。
/// - `yield_strain <= 0` 或 `creep <= 0`：视为非法，关闭塑性。
///
/// # Returns
/// `Bool::TRUE` 成功（含关闭）；`id` 未知 / world 为 null 返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效；`yield_strain` / `creep` 需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_plasticity(
    world: *mut WorldHandle,
    id: u32,
    yield_strain: f64,
    creep: f64,
    enabled: u8,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !yield_strain.is_finite() || !creep.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_plasticity: non-finite parameter",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_plasticity: unknown id");
            return Bool::FALSE;
        };
        let params = if enabled != 0 && yield_strain > 0.0 {
            Some(rapier3d::dynamics::soft_body::PlasticityParams {
                yield_strain,
                creep,
            })
        } else {
            None
        };
        sb.set_plasticity(params);
        clear_error();
        Bool::TRUE
    })
}

/// # Phase 11 — 设置内部气压（充气 / 气球模型）
///
/// 把 `id` 软体的内部气压设为 `pressure`（力/面积）。每步在 `compute_forces`（MassSpring）
/// 与 `step_xpbd`（预测步）中，对每个**闭合三角网格**的自由质点沿面法向施加向外推力
/// `F = pressure · area`，把闭合壳"吹胀"。`pressure > 0` 开启；`pressure <= 0` 视为关闭
/// （等同于 `pressure = None`，默认）。
///
/// 纯外力，与风场同构；不引入新求解器力学。需 `self.triangles` 构成闭合流形才能像真气球，
/// 开口薄片会沿单面法向鼓起。
///
/// # Returns
/// `Bool::TRUE` 成功；`id` 未知 / world 为 null 返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效；`pressure` 需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_pressure(world: *mut WorldHandle, id: u32, pressure: f64) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !pressure.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_pressure: non-finite pressure",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_pressure: unknown id");
            return Bool::FALSE;
        };
        sb.set_pressure(pressure);
        clear_error();
        Bool::TRUE
    })
}

/// # Phase 12 — 开启/关闭软体自碰撞(self-collision)
///
/// 把 `id` 软体的自碰撞设为 `radius`(粒子球半径)+ `stiffness`(XPBD 排斥约束柔度, `0`=硬)。
/// 每步求解中,任意两个自由质点中心距 `< 2*radius` 时沿连线被推开(各自视为该半径的球),
/// 但**直接结构邻居**(已有 distance_constraint 边相连的质点对)被排除,不误判为碰撞。
/// 采用均匀空间哈希做 broad-phase,在 MassSpring 与 XPBD 两条路径内逐迭代投影,纯位置约束,
/// 不引入新求解器力学。非法参数(`radius <= 0` / `stiffness < 0` / 非有限)返回 `Bool::FALSE` 且不开。
///
/// # Returns
/// `Bool::TRUE` 成功开启;`id` 未知 / world 为 null / 参数非法返回 `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`radius` / `stiffness` 需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_self_collision(
    world: *mut WorldHandle,
    id: u32,
    radius: f64,
    stiffness: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !radius.is_finite() || !stiffness.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_self_collision: non-finite radius/stiffness",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_self_collision: unknown id");
            return Bool::FALSE;
        };
        // set_self_collision rejects bad params (returns false). A valid-id rejection
        // is a parameter error -> report FALSE (mirrors soft_body_set_pressure /
        // soft_body_set_plasticity convention for invalid arguments).
        let ok = sb.set_self_collision(radius, stiffness);
        if !ok {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_self_collision: radius<=0 or stiffness<0",
            );
            return Bool::FALSE;
        }
        clear_error();
        Bool::TRUE
    })
}

/// # Phase 13 — 运行时改单条弹簧刚度
///
/// 把 `id` 软体里下标 `index`(由 `soft_body_add_spring` 返回)的弹簧刚度(Hookean `k`)改为 `stiffness`。
/// 用于构造后就地调材质异质性(例如把"骨骼"弹簧调硬、"腱"调软),无需重建拓扑。
/// `stiffness < 0` 或非有限 → 返回 `Bool::FALSE` 且不改。
///
/// # Returns
/// `Bool::TRUE` 修改成功;`id` 未知 / `index` 越界 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`stiffness` 需为有限非负值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_spring_stiffness(
    world: *mut WorldHandle,
    id: u32,
    index: u32,
    stiffness: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_spring_stiffness: unknown id");
            return Bool::FALSE;
        };
        if sb.set_spring_stiffness(index as usize, stiffness) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_spring_stiffness: bad index or negative/non-finite stiffness",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 13 — 运行时改单条 XPBD 距离约束柔度
///
/// 把 `id` 软体里下标 `index`(由 `soft_body_add_distance_constraint` 返回)的 XPBD 距离约束柔度
/// (compliance α)改为 `compliance`。XPBD 求解器逐约束读取各自柔度(见 `step_xpbd`),因此不同边
/// 可拥有不同刚度。`compliance < 0` 或非有限 → 返回 `Bool::FALSE` 且不改。
///
/// # Returns
/// `Bool::TRUE` 修改成功;`id` 未知 / `index` 越界 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`compliance` 需为有限非负值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_distance_constraint_compliance(
    world: *mut WorldHandle,
    id: u32,
    index: u32,
    compliance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_set_distance_constraint_compliance: unknown id",
            );
            return Bool::FALSE;
        };
        if sb.set_distance_constraint_compliance(index as usize, compliance) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_distance_constraint_compliance: bad index or negative/non-finite compliance",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 19 — 设置某条距离约束的「压缩」柔度(各向异性柔度)
///
/// 把 `id` 软体第 `index` 条距离约束的**压缩** XPBD 柔度 `α_c` 设为 `compression`。
/// 该约束原本只有单一 `compliance`(拉伸/压缩共用,各向同性)。本函数令其独立在
/// **压缩**(`len < rest`,被压短)时采用 `compression` 柔度——布料/泡沫可「抗拉伸但易折叠」,
/// 是标准的各向异性 XPBD 行为。`stretch` 柔度仍由 `soft_body_set_distance_constraint_compliance`
/// 控制;二者相等即回到各向同性。求解器每个迭代按当前应变符号选用对应柔度。
/// 非法参数(`index` 越界 / `compression` 为负或非有限)返回 `Bool::FALSE`。
///
/// # Returns
/// `Bool::TRUE` 成功;`id` 未知 / 约束 `index` 越界 / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`index` 须在 `[0, 约束数)`;`compression >= 0` 且有限。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_distance_constraint_compression(
    world: *mut WorldHandle,
    id: u32,
    index: u32,
    compression: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_set_distance_constraint_compression: unknown id",
            );
            return Bool::FALSE;
        };
        if sb.set_distance_constraint_compression(index as usize, compression) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_distance_constraint_compression: bad index or negative/non-finite compression",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 14 — 开启/关闭软体间的软软碰撞(soft-soft / cross-body)
///
/// 把 `id` 软体的软软碰撞设为 `radius`(粒子球半径)+ `stiffness`(XPBD 排斥约束柔度, `0`=硬)。
/// 世界级 step 结束后,任意两个**都**开启了软软碰撞的软体,其自由质点中心距 `< 2·min(ra,rb)`
/// 时沿连线被推开(各自视为该半径的球)。复用 Phase 12 的空间哈希 + XPBD 投影原语,但在
/// world 层遍历软体对。只排 inter-body 对(同体内自碰撞由 Phase 12 处理)。
/// 非法参数(`radius <= 0` / `stiffness < 0` / 非有限)返回 `Bool::FALSE` 且不开。
///
/// # Returns
/// `Bool::TRUE` 成功开启;`id` 未知 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`radius` / `stiffness` 需为有限值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_cross_collision(
    world: *mut WorldHandle,
    id: u32,
    radius: f64,
    stiffness: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !radius.is_finite() || !stiffness.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_cross_collision: non-finite radius/stiffness",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_cross_collision: unknown id");
            return Bool::FALSE;
        };
        if sb.set_cross_collision(radius, stiffness) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_cross_collision: radius<=0 or stiffness<0",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 20 — 设置自碰撞接触摩擦系数 μ(0 ≤ μ ≤ 1)
///
/// 需要先 `soft_body_set_self_collision` 开启自碰撞。μ 控制接触处切向相对速度被阻尼的比例
/// (μ=0 无摩擦, μ=1 完全消除切向滑动, Coulomb 风格)。非法参数(非有限 / 越界 / 未开启自碰撞)
/// 返回 `Bool::FALSE` 且不改动状态。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_self_collision_friction(
    world: *mut WorldHandle,
    id: u32,
    mu: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_set_self_collision_friction: unknown id",
            );
            return Bool::FALSE;
        };
        if sb.set_self_collision_friction(mu) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_self_collision_friction: μ 非有限/越界(0≤μ≤1)或未开启自碰撞",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 20 — 设置软软(跨体)碰撞接触摩擦系数 μ(0 ≤ μ ≤ 1)
///
/// 需要先 `soft_body_set_cross_collision` 开启跨体碰撞。语义同自碰撞摩擦:阻尼接触切向相对
/// 速度。实际生效的 μ 为两体 `min(μ_a, μ_b)`(任一体无摩擦则该接触无摩擦)。非法参数返回
/// `Bool::FALSE`。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_cross_collision_friction(
    world: *mut WorldHandle,
    id: u32,
    mu: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_set_cross_collision_friction: unknown id",
            );
            return Bool::FALSE;
        };
        if sb.set_cross_collision_friction(mu) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_cross_collision_friction: μ 非有限/越界(0≤μ≤1)或未开启跨体碰撞",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 16 — 开启/关闭体积守恒约束(独立柔度, 与距离求解器解耦)
///
/// 把 `id` 软体的四面体体积约束柔度设为 `compliance`(`0`=硬/不可压缩)。开启后 `step_xpbd`
/// 里每条四面体体积约束用 `α̃ = compliance / dt²` 求解 —— 与距离求解器的 compliance 无关,
/// 因此可以让边很软而体积保持硬(不可压 blob)。与 Phase 11 气压正交:气压是向外吹胀的力,
/// 本约束是把总体积拉回静止值。关闭(`clear`)后体积约束回退到全局求解器 compliance。
/// 非法参数(非有限 / 负数)返回 `Bool::FALSE` 且不开。
///
/// # Returns
/// `Bool::TRUE` 成功开启;`id` 未知 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`compliance` 需为有限非负值。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_volume_conservation(
    world: *mut WorldHandle,
    id: u32,
    compliance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !compliance.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_volume_conservation: non-finite compliance",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_set_volume_conservation: unknown id",
            );
            return Bool::FALSE;
        };
        if sb.set_volume_conservation(compliance) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_volume_conservation: negative/non-finite compliance",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 18 — 设置全局内部(结构)阻尼系数
///
/// 把 `id` 软体的 `damping` 设为 `d`。每个 step 里每个自由质点的速度乘以 `1 - d`
/// (jelly / slime 式能量耗散),与 Phase 0 的弹簧轴向阻尼、Phase 13 的逐约束柔度正交。
/// `d=0` 无阻尼;`d in [0,1)` 振荡收敛更快;`d>=1` 或非法(非有限/负数)返回 `Bool::FALSE`。
///
/// # Returns
/// `Bool::TRUE` 成功设置;`id` 未知 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;`d` 需为有限且 `0 <= d < 1`。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_damping(world: *mut WorldHandle, id: u32, d: f64) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        if !d.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_damping: non-finite damping",
            );
            return Bool::FALSE;
        }
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_damping: unknown id");
            return Bool::FALSE;
        };
        if sb.set_damping(d) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_damping: need 0 <= d < 1",
            );
            Bool::FALSE
        }
    })
}

/// # Phase 17 — 开启/关闭软体间黏连(可撕黏附 glue)
///
/// 把 `id` 软体的 `cohesion` 设为 `CohesionParams{radius, stiffness, break_distance}`。
/// 开启后,本软体与*其它*也开了 cohesion 的软体之间:自由质点彼此进入 `radius` 即被互相
/// 吸引到接触距离(`radius`),把两体黏在一起(Phase 9 撕裂的对偶)。bond 可破断:若某对
/// 已被拉到 `break_distance` 之外,本步不再吸引(胶水撕裂)。`break_distance=inf` 表示永久胶。
/// 关闭(`clear`)后不再黏连。非法参数(radius<=0 / stiffness<0 / break_distance<=radius /
/// 任一为 NaN)返回 `Bool::FALSE` 且不开;注意 `break_distance=inf` 合法(永久胶)。
///
/// # Returns
/// `Bool::TRUE` 成功开启;`id` 未知 / world 为 null / 参数非法 → `Bool::FALSE`。
///
/// # Safety
/// `world` 必须有效;参数需为有限且符合约束。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_cohesion(
    world: *mut WorldHandle,
    id: u32,
    radius: f64,
    stiffness: f64,
    break_distance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let sid = rapier3d::prelude::soft_body::SoftBodyId(id);
        let Some(sb) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_cohesion: unknown id");
            return Bool::FALSE;
        };
        if sb.set_cohesion(radius, stiffness, break_distance) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_cohesion: bad radius/stiffness/break_distance",
            );
            Bool::FALSE
        }
    })
}

// ── Phase 5a: general soft-body builder (unlock arbitrary topology) ──────────了「voxel 网格 → 软体」与「设置重力」两个高层入口，外部调用方
// （JNI/FFM/Minecraft）无法构造任意拓扑的软体（自定义质点、弹簧、XPBD 距离约束、
// 四面体体积元）也无法切求解器。本组 FFI 把 rapier `SoftBody` 的 `add_particle` /
// `add_pinned` / `add_spring` / `add_distance_constraint` / `add_tetrahedron` /
// `configure_xpbd` 暴露出来，让上层能逐点搭建软体。
//
// 惯例：成功返回真实 id / 粒子下标（可为 0，首个软体/质点下标即 0）；失败返回
// `u32::MAX`（`add_*`/`create`）或 `Bool::FALSE`（布尔类），并置 `ERR_*`。

/// Create an empty soft body in the world and return its `SoftBodyId`.
///
/// The body starts in the `MassSpring` solver; switch it to XPBD with
/// [`soft_body_configure_solver`] if you intend to use distance/tetra constraints.
///
/// # Returns
/// The `SoftBodyId` (as `u32`) on success, or `u32::MAX` on error (`ERR_*`).
///
/// # Safety
/// `world` must be a valid world pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_create(world: *mut WorldHandle, gravity: Vec3) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_create: world is null");
            return u32::MAX;
        };
        if !vec3_finite(gravity) {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_create: bad gravity");
            return u32::MAX;
        }
        clear_error();
        let body = SoftBody::new(vec3_to_rapier(gravity));
        let id = world.inner.soft_bodies.insert(body);
        id.0
    })
}

/// Clone a soft body into a new standalone body, returning the new body id.
///
/// Deep-copies the source body verbatim — particles (position/velocity/inv_mass),
/// springs, distance constraints, tetrahedra (+ rest volumes), triangles, solver
/// selection, gravity, sleeping, damping, substeps, and every optional field
/// (wind, pressure, tearing, plasticity, self/cross collision, volume
/// conservation, cohesion). The original is untouched.
///
/// The clone is intentionally **collision-decoupled** (`collide = false`): proxy
/// colliders live in the world's proxy table keyed by `SoftBodyId`, not inside the
/// body, so a copied `collide == true` would have no proxies to drive it and would
/// freeze. Call `soft_body_enable_collision` on the new id to rebuild proxies if the
/// clone needs collision response.
///
/// Returns the new body id, or `u32::MAX` if `world` is null or `id` is unknown.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_clone(world: *mut WorldHandle, id: u32) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_clone: world is null");
            return u32::MAX;
        };
        let sid = SoftBodyId(id);
        let Some(src) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_clone: unknown id");
            return u32::MAX;
        };
        let mut cloned = src.clone();
        // Standalone clone: disable collision coupling so it self-integrates (its id
        // has no proxy colliders in the world's proxy table yet).
        cloned.collide = false;
        let new_id = world.inner.soft_bodies.insert(cloned);
        clear_error();
        new_id.0
    })
}

/// Add a particle to a soft body.
///
/// * `mass` — particle mass (> 0, finite). Ignored when `pinned` is non-zero
///   (a pinned particle has infinite mass / `inv_mass = 0` and acts as an anchor).
/// * `x/y/z` — initial world position (finite).
///
/// # Returns
/// The particle index (as `u32`) on success, or `u32::MAX` on error (`ERR_*`).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_particle(
    world: *mut WorldHandle,
    id: u32,
    x: f64,
    y: f64,
    z: f64,
    mass: f64,
    pinned: Bool,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_add_particle: world is null");
            return u32::MAX;
        };
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_add_particle: bad position");
            return u32::MAX;
        }
        if pinned == Bool::FALSE && (!mass.is_finite() || mass <= 0.0) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_add_particle: mass must be > 0",
            );
            return u32::MAX;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_add_particle: unknown id");
            return u32::MAX;
        };
        let pos = Vector::new(x, y, z);
        let idx = if pinned == Bool::TRUE {
            body.add_pinned(pos)
        } else {
            let i = body.add_particle(pos);
            body.particles[i].inv_mass = 1.0 / mass;
            i
        };
        clear_error();
        idx as u32
    })
}

/// Add a Hookean spring (edge) between two particles of a soft body.
///
/// Used by the `MassSpring` solver. Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_spring(
    world: *mut WorldHandle,
    id: u32,
    a: u32,
    b: u32,
    stiffness: f64,
    damping: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_add_spring: world is null");
            return Bool::FALSE;
        };
        if !stiffness.is_finite() || stiffness < 0.0 || !damping.is_finite() || damping < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "soft_body_add_spring: bad params");
            return Bool::FALSE;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_add_spring: unknown id");
            return Bool::FALSE;
        };
        match body.add_spring(a as usize, b as usize, stiffness, damping) {
            Some(_) => {
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_add_spring: bad particle indices",
                );
                Bool::FALSE
            }
        }
    })
}

/// Add an XPBD distance constraint (edge) between two particles.
///
/// Used by the `Xpbd` solver; switch the body to XPBD first with
/// [`soft_body_configure_solver`]. Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_distance_constraint(
    world: *mut WorldHandle,
    id: u32,
    a: u32,
    b: u32,
    compliance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_add_distance_constraint: world is null",
            );
            return Bool::FALSE;
        };
        if !compliance.is_finite() || compliance < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_add_distance_constraint: bad compliance",
            );
            return Bool::FALSE;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_add_distance_constraint: unknown id",
            );
            return Bool::FALSE;
        };
        match body.add_distance_constraint(a as usize, b as usize, compliance) {
            Some(_) => {
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_add_distance_constraint: bad particle indices",
                );
                Bool::FALSE
            }
        }
    })
}

/// Add a tetrahedral volume element `[a, b, c, d]` to a soft body.
///
/// Used by the `Xpbd` solver's volume-preservation constraint; the rest
/// (reference) signed volume is cached at add time. Returns `Bool::TRUE` on
/// success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_tetrahedron(
    world: *mut WorldHandle,
    id: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_add_tetrahedron: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_add_tetrahedron: unknown id");
            return Bool::FALSE;
        };
        match body.add_tetrahedron([a, b, c, d]) {
            Some(_) => {
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_add_tetrahedron: bad/degenerate indices",
                );
                Bool::FALSE
            }
        }
    })
}

/// # Phase 21 - adaptive tetrahedral subdivision (1 -> 4 barycentric split).
///
/// Inserts one new particle at the centroid of each source tetrahedron and replaces
/// it with four sub-tetrahedra sharing that centroid. The four sub-volumes sum to the
/// parent volume, so the XPBD volume-conservation constraint (Phase 16) stays
/// consistent; the centroid is a vertex of every sub-tet, so no extra distance edges
/// are added (that would over-constrain the solve). A source tet is split only when
/// its longest edge exceeds `max_edge_len`; pass a non-finite value to subdivide all.
/// The shell topology (`triangles`) is left untouched (volumetric refinement only).
/// Returns the number of source tetrahedra actually split (0 if none qualified).
/// Unknown id or a body with no tetrahedra returns 0 with no side effect.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_subdivide_tetrahedra(
    world: *mut WorldHandle,
    id: u32,
    max_edge_len: f64,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_subdivide_tetrahedra: world is null",
            );
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_subdivide_tetrahedra: unknown id");
            return 0;
        };
        let n = body.subdivide_tetrahedra(max_edge_len);
        clear_error();
        n as u32
    })
}

/// Phase 6 — cloth: add a triangular face `[a, b, c]` to a soft body's shell
/// topology. The three structural edges are registered automatically as
/// distance constraints (rest length from current spacing); duplicate edges
/// shared with neighbouring triangles are de-duplicated inside rapier. Bending
/// is composed separately by the caller via `soft_body_add_bending` (a single
/// cross-diagonal distance constraint) — no new mechanics, fully reusing the
/// existing XPBD distance solver.
///
/// Returns `Bool::TRUE` on success. `Bool::FALSE` if the body/id is unknown, an
/// index is out of bounds or duplicated, or the face is degenerate (a zero-length
/// edge).
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_triangle(
    world: *mut WorldHandle,
    id: u32,
    a: u32,
    b: u32,
    c: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_add_triangle: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_add_triangle: unknown id");
            return Bool::FALSE;
        };
        match body.add_triangle([a, b, c]) {
            Some(_) => {
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_add_triangle: bad/degenerate indices",
                );
                Bool::FALSE
            }
        }
    })
}

/// Phase 6 — cloth: add a single bending edge between particles `p` and `q` as
/// a distance constraint (rest length from current spacing). Compose bending
/// across a quad by calling this for its two diagonals, or across a fold line by
/// linking the un-shared vertices of two adjacent triangles. Reuses the existing
/// XPBD distance solver (no new mechanics).
///
/// Returns `Bool::TRUE` on success. `Bool::FALSE` if the body/id is unknown, an
/// index is out of bounds, or the endpoints coincide.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_add_bending(world: *mut WorldHandle, id: u32, p: u32, q: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_add_bending: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_add_bending: unknown id");
            return Bool::FALSE;
        };
        match body.add_bending_constraint(p as usize, q as usize) {
            Some(_) => {
                clear_error();
                Bool::TRUE
            }
            None => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_add_bending: bad/coincident indices",
                );
                Bool::FALSE
            }
        }
    })
}

/// Switch a soft body's solver.
///
/// * `solver_mode` — `0` = `MassSpring` (Hookean springs, semi-implicit Euler);
///   `1` = `Xpbd { iterations, compliance }` (position-based distance + volume
///   constraints).
/// * `iterations` — XPBD Gauss-Seidel iterations (> 0 when `solver_mode == 1`).
/// * `compliance` — XPBD default compliance (≥ 0, finite).
///
/// Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_configure_solver(
    world: *mut WorldHandle,
    id: u32,
    solver_mode: u32,
    iterations: u32,
    compliance: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_configure_solver: world is null",
            );
            return Bool::FALSE;
        };
        if !compliance.is_finite() || compliance < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_configure_solver: bad compliance",
            );
            return Bool::FALSE;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_configure_solver: unknown id");
            return Bool::FALSE;
        };
        match solver_mode {
            0 => {
                body.solver = SoftSolver::MassSpring;
                clear_error();
                Bool::TRUE
            }
            1 => {
                if iterations == 0 {
                    set_error(
                        ERR_INVALID_ARGUMENT,
                        "soft_body_configure_solver: iterations must be > 0",
                    );
                    return Bool::FALSE;
                }
                body.configure_xpbd(iterations, compliance);
                clear_error();
                Bool::TRUE
            }
            _ => {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_configure_solver: unknown solver_mode",
                );
                Bool::FALSE
            }
        }
    })
}

// ── Phase 5c: 生物软体（复用 Phase 3 四面体 XPBD 体积约束）──────────────────
//
// `soft_body_build_tetra_mesh` 一次性灌入质点 + 四面体拓扑，自动为每个四面体
// 的 6 条边建 XPBD 距离约束（去重），并切到 Xpbd 求解器。体积约束在
// `SoftBody::step_xpbd` 内随四面体自动激活 —— 于是得到一个「史莱姆 / 水母」式
// 可压缩可回弹的软体生物，无需任何新物理。
// 该函数纯包裹 rapier 既有 API，不改动 fork。

/// Build a tetrahedral-mesh soft body from raw particle positions and tetrahedra,
/// then switch it to the XPBD solver so the volume constraints are active.
///
/// `particles` is a `particles_len`-long array of `Vec3`; `tets` is a flat array
/// of `tets_len * 4` `u32` vertex indices (`[a,b,c,d, a,b,c,d, ...]`). For every
/// tetrahedron, its 6 edges are added as XPBD distance constraints (deduplicated
/// across shared edges). Finally the body is configured with `iterations`/`compliance`.
///
/// Returns the new `SoftBodyId` (as `u32`) or `u32::MAX` on error.
///
/// # Safety
/// `world` must be a valid world pointer. `particles`/`tets` must point to arrays
/// of at least `particles_len` / `tets_len*4` elements respectively.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_build_tetra_mesh(
    world: *mut WorldHandle,
    gravity: Vec3,
    particles: *const Vec3,
    particles_len: u32,
    tets: *const u32,
    tets_len: u32,
    particle_mass: f64,
    compliance: f64,
    iterations: u32,
) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_build_tetra_mesh: world is null",
            );
            return u32::MAX;
        };
        if particles.is_null() || particles_len == 0 || tets.is_null() || tets_len == 0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_build_tetra_mesh: empty input",
            );
            return u32::MAX;
        }
        if !vec3_finite(gravity)
            || !particle_mass.is_finite()
            || particle_mass <= 0.0
            || !compliance.is_finite()
            || compliance < 0.0
            || iterations == 0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_build_tetra_mesh: bad gravity/mass/compliance/iterations",
            );
            return u32::MAX;
        }
        let plist = unsafe { std::slice::from_raw_parts(particles, particles_len as usize) };
        let tlist = unsafe { std::slice::from_raw_parts(tets, tets_len as usize * 4) };

        let mut body = SoftBody::new(vec3_to_rapier(gravity));
        // Add particles (all dynamic; caller may pin later via soft_body_add_particle
        // is not applicable to existing particles, so pins are added up-front by the
        // caller through a separate pinned-particle path if needed).
        for p in plist {
            if !vec3_finite(*p) {
                set_error(
                    ERR_INVALID_ARGUMENT,
                    "soft_body_build_tetra_mesh: non-finite particle",
                );
                return u32::MAX;
            }
            let i = body.add_particle(vec3_to_rapier(*p));
            body.particles[i].inv_mass = 1.0 / particle_mass;
        }

        // Add tetrahedra + their 6 edges as XPBD distance constraints (dedup edges).
        let mut edges: HashSet<(u32, u32)> = HashSet::new();
        let mut tet_fail = false;
        for t in tlist.as_chunks::<4>().0 {
            let tet = [t[0], t[1], t[2], t[3]];
            if body.add_tetrahedron(tet).is_none() {
                tet_fail = true;
                break;
            }
            for (a, b) in [
                (tet[0], tet[1]),
                (tet[0], tet[2]),
                (tet[0], tet[3]),
                (tet[1], tet[2]),
                (tet[1], tet[3]),
                (tet[2], tet[3]),
            ] {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                edges.insert((lo, hi));
            }
        }
        if tet_fail {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_build_tetra_mesh: bad tetrahedron indices",
            );
            return u32::MAX;
        }
        for (lo, hi) in edges {
            // compliance propagates to edges too; rest length captured from geometry.
            body.add_distance_constraint(lo as usize, hi as usize, compliance);
        }

        body.configure_xpbd(iterations, compliance);
        let id = world.inner.soft_bodies.insert(body);
        clear_error();
        id.0
    })
}

// ── Phase 5b: query / readback / lifecycle FFI (close the loop) ──────────────
//
// Phase 5a 让外部能「搭」软体；本组让外部能「查 / 读 / 删 / 毁」——
// 这是 Minecraft 联动（读回质点渲染、区块破坏删质点、实体消失毁软体）必需的闭环。
// 返回 id 类沿用 `u32::MAX` 哨兵；布尔类沿用 `Bool::FALSE`；`SoftBodyId` 经 rapier
// `SoftBodySet::remove` 走 tombstone，删除后其余 id 仍有效。

/// Number of live soft bodies in the world.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_count(world: *const WorldHandle) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_count: world is null");
            return 0;
        };
        clear_error();
        world.inner.soft_bodies.count() as u32
    })
}

/// Number of particles in a soft body. Returns `u32::MAX` for an unknown id.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_particle_count(world: *const WorldHandle, id: u32) -> u32 {
    ffi_guard(u32::MAX, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_particle_count: world is null");
            return u32::MAX;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_particle_count: unknown id");
            return u32::MAX;
        };
        clear_error();
        body.particles.len() as u32
    })
}

/// Read back a particle's position and velocity.
///
/// `out_pos` / `out_vel` must point to writable `Vec3`; either may be null to
/// skip that output. Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer; `out_pos`/`out_vel` (if non-null) must
/// point to writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_get_particle(
    world: *const WorldHandle,
    id: u32,
    index: u32,
    out_pos: *mut Vec3,
    out_vel: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_get_particle: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_get_particle: unknown id");
            return Bool::FALSE;
        };
        let i = index as usize;
        let Some(p) = body.particles.get(i) else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_get_particle: index out of bounds",
            );
            return Bool::FALSE;
        };
        if !out_pos.is_null() {
            unsafe {
                *out_pos = vec3_from_rapier(p.pos);
            }
        }
        if !out_vel.is_null() {
            unsafe {
                *out_vel = vec3_from_rapier(p.vel);
            }
        }
        clear_error();
        Bool::TRUE
    })
}

/// Remove a particle (and every spring / distance constraint / tetrahedron that
/// references it) from a soft body, keeping the remaining topology valid.
/// Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_remove_particle(world: *mut WorldHandle, id: u32, index: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_remove_particle: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_remove_particle: unknown id");
            return Bool::FALSE;
        };
        if body.remove_particle(index as usize) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_remove_particle: index out of bounds",
            );
            Bool::FALSE
        }
    })
}

/// Apply a linear impulse to a single soft-body particle.
///
/// The impulse `J = (fx, fy, fz)` changes the particle velocity by `J * inv_mass`,
/// i.e. `p.vel += J * p.inv_mass`. For collision-coupled bodies the updated velocity
/// is pushed into the particle's proxy rigid body at the next step (see the
/// soft-body/rigid-body coupling loop), so a contact reaction naturally follows; for
/// non-coupled bodies the fork integrator consumes `p.vel` directly. Pinned particles
/// (`inv_mass == 0`, e.g. anchors) are unaffected. This is the primitive for
/// grab/poke/kick interactions on a single vertex. Pure state mutation: no solver
/// structural change.
///
/// Returns `Bool::TRUE` on success, `Bool::FALSE` if `world` is null, `id` is unknown,
/// `index` is out of bounds, or any component of the impulse is non-finite.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_apply_particle_impulse(
    world: *mut WorldHandle,
    id: u32,
    index: u32,
    fx: f64,
    fy: f64,
    fz: f64,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_apply_particle_impulse: world is null",
            );
            return Bool::FALSE;
        };
        if !fx.is_finite() || !fy.is_finite() || !fz.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_apply_particle_impulse: impulse must be finite",
            );
            return Bool::FALSE;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(
                ERR_NOT_FOUND,
                "soft_body_apply_particle_impulse: unknown id",
            );
            return Bool::FALSE;
        };
        let i = index as usize;
        if i >= body.particles.len() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_apply_particle_impulse: index out of bounds",
            );
            return Bool::FALSE;
        }
        let p = &mut body.particles[i];
        let inv = p.inv_mass;
        // inv_mass == 0 (pinned) → no velocity change, but still a valid op.
        p.vel += Vector::new(fx * inv, fy * inv, fz * inv);
        clear_error();
        Bool::TRUE
    })
}

/// Read the axis-aligned bounding box (min/max corners) and centroid of a soft body.
///
/// Computes the AABB and the per-particle average position (`centroid`) from the
/// body's current particle positions. Useful for frustum culling, broad-phase
/// spatial queries, LOD, and nearest-neighbour tests against other bodies. Pure
/// read-out: does not affect the solver. Bodies with zero particles return
/// `Bool::FALSE` (the box is undefined).
///
/// Any of `out_min`/`out_max`/`out_centroid` may be null to skip that output.
///
/// Returns `Bool::TRUE` on success, `Bool::FALSE` if `world` is null, `id` is
/// unknown, or the body has no particles.
///
/// # Safety
/// `world` must be a valid world pointer; non-null output pointers must each target
/// a writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_aabb(
    world: *const WorldHandle,
    id: u32,
    out_min: *mut Vec3,
    out_max: *mut Vec3,
    out_centroid: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_aabb: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_aabb: unknown id");
            return Bool::FALSE;
        };
        if body.particles.is_empty() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_read_aabb: body has no particles",
            );
            return Bool::FALSE;
        }
        let mut min = body.particles[0].pos;
        let mut max = body.particles[0].pos;
        let mut sum = Vector::ZERO;
        for p in body.particles.iter() {
            min = min.min(p.pos);
            max = max.max(p.pos);
            sum += p.pos;
        }
        let n = body.particles.len() as f64;
        let centroid = sum / n;
        if !out_min.is_null() {
            unsafe { *out_min = vec3_from_rapier(min) };
        }
        if !out_max.is_null() {
            unsafe { *out_max = vec3_from_rapier(max) };
        }
        if !out_centroid.is_null() {
            unsafe { *out_centroid = vec3_from_rapier(centroid) };
        }
        clear_error();
        Bool::TRUE
    })
}

/// Destroy a soft body, freeing its storage. Other live `SoftBodyId`s remain
/// valid (the id slot becomes a tombstone). Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_destroy(world: *mut WorldHandle, id: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_destroy: world is null");
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        if world.inner.soft_bodies.remove(sid) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(ERR_NOT_FOUND, "soft_body_destroy: unknown id");
            Bool::FALSE
        }
    })
}

// ── Phase 5i: 拓扑读回（渲染用）────────────────────────────────────────────
//
// 三个 FFI 把软体的「当前状态 + 拓扑」一次性拷给调用方提供的缓冲区，供渲染层
// （Minecraft 端）每帧拉取重建网格，无需逐个粒子调用 `soft_body_get_particle`。
// 语义对齐 `world_dynamic_body_snapshot`：调用方先给 capacity 申请足够大的缓冲，
// FFI 写入前 `min(count, capacity)` 个元素并返回 *真实* 元素数（capacity 不足时
// 调用方据此扩容重试）。所有函数对越界/空指针都 idempotent 返回 0，不 panic。

/// 批量读回粒子：位置（world-space）+ 逆质量（0 = pinned）。
/// `out_pos` 容量需 ≥ `capacity` 个 `Vec3`；`out_inv_mass` 容量需 ≥ `capacity` 个 `f64`。
/// 任一出参为 null 即跳过该通道（只写非 null 的通道），但仍返回粒子总数。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_particles(
    world: *const WorldHandle,
    id: u32,
    out_pos: *mut Vec3,
    out_inv_mass: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_particles: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_particles: unknown id");
            return 0;
        };
        let n = body.particles.len();
        let cap = capacity as usize;
        if !out_pos.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_pos, cap) };
            for (i, p) in body.particles.iter().enumerate().take(cap) {
                slice[i] = vec3_from_rapier(p.pos);
            }
        }
        if !out_inv_mass.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_inv_mass, cap) };
            for (i, p) in body.particles.iter().enumerate().take(cap) {
                slice[i] = p.inv_mass;
            }
        }
        clear_error();
        n as u32
    })
}

/// 批量读回边（弹簧 + 距离约束合并）。每条边是 2 个 `u32` 粒子索引。
/// `out_edges` 容量需 ≥ `capacity` 个 `u32`（即 `capacity/2` 条边）。
/// 边顺序：先所有 springs，再所有 distance_constraints（与 `soft_body_read_tetrahedra`
/// 配合可让渲染层区分软/硬边，若需要）。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_edges(
    world: *const WorldHandle,
    id: u32,
    out_edges: *mut u32,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_edges: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_edges: unknown id");
            return 0;
        };
        let total_edges = body.springs.len() + body.distance_constraints.len();
        let cap = capacity as usize;
        if !out_edges.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_edges, cap) };
            let mut w = 0usize;
            for s in body.springs.iter() {
                if w + 2 > cap {
                    break;
                }
                slice[w] = s.a as u32;
                slice[w + 1] = s.b as u32;
                w += 2;
            }
            for d in body.distance_constraints.iter() {
                if w + 2 > cap {
                    break;
                }
                slice[w] = d.a as u32;
                slice[w + 1] = d.b as u32;
                w += 2;
            }
        }
        clear_error();
        total_edges as u32
    })
}

/// 批量读回四面体（XPBD 体积约束单元）。每个四面体是 4 个 `u32` 粒子索引。
/// `out_tets` 容量需 ≥ `capacity` 个 `u32`（即 `capacity/4` 个四面体）。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_tetrahedra(
    world: *const WorldHandle,
    id: u32,
    out_tets: *mut u32,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_tetrahedra: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_tetrahedra: unknown id");
            return 0;
        };
        let n = body.tetrahedra.len();
        let cap = capacity as usize;
        if !out_tets.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_tets, cap) };
            for (i, t) in body.tetrahedra.iter().enumerate().take(cap / 4) {
                let base = i * 4;
                slice[base] = t[0];
                slice[base + 1] = t[1];
                slice[base + 2] = t[2];
                slice[base + 3] = t[3];
            }
        }
        clear_error();
        n as u32
    })
}

/// Phase 6 — cloth: 批量读回三角形面（shell 拓扑）。每个三角形是 3 个 `u32`
/// 粒子索引。 `out_tris` 容量需 ≥ `capacity` 个 `u32`（即 `capacity/3` 个三角形）。
/// 与 `soft_body_read_edges` 配合可让渲染层区分结构边与弯曲边。
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_triangles(
    world: *const WorldHandle,
    id: u32,
    out_tris: *mut u32,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_triangles: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_triangles: unknown id");
            return 0;
        };
        let n = body.triangles.len();
        let cap = capacity as usize;
        if !out_tris.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_tris, cap) };
            for (i, t) in body.triangles.iter().enumerate().take(cap / 3) {
                let base = i * 3;
                slice[base] = t[0];
                slice[base + 1] = t[1];
                slice[base + 2] = t[2];
            }
        }
        clear_error();
        n as u32
    })
}

// ── Phase 22: 逐边应力 / 张力读数（仅供渲染层 / 上层逻辑使用，不改求解器）──────
//
// 软体现在能回读位置 / 拓扑（read_particles / read_edges / read_tetrahedra /
// read_triangles），但**没有逐边张力**——调试可视化（颜色映射应力）和"撕裂风险"
// 逻辑只能靠自己算边长。补一个纯读数 FFI：对每个结构边（弹簧 + XPBD 距离约束）
// 返回归一化应变 strain = (len - rest) / rest（rest==0 → 0.0）。长度从现有质点
// 位置现算，不改 fork、不动求解器，沿用 read_edges 的 marshalling 形态。

/// Read per-edge normalized strain (stress proxy) for a soft body.
///
/// Edges are enumerated in the same order as [`soft_body_read_edges`]: every
/// `Spring` first, then every `DistanceConstraint`. For each edge the function
/// writes `strain = (current_len - rest) / rest` (0.0 when `rest == 0`) into
/// `out_strain[..]`. Returns the total edge count (so the caller can size its
/// buffer); when `out_strain` is null or `capacity` is 0 the count is returned
/// without writing.
///
/// This is a pure read-out for debug visualisation / "tear risk" UI; it does
/// not affect the solver. Symmetry / determinism are irrelevant because no state
/// is mutated.
///
/// # Safety
/// `world` must be a valid world pointer. `out_strain` must point to an array of
/// at least `capacity` `f64` elements when non-null.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_stress(
    world: *const WorldHandle,
    id: u32,
    out_strain: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_stress: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_stress: unknown id");
            return 0;
        };
        let total = body.springs.len() + body.distance_constraints.len();
        let cap = capacity as usize;
        if !out_strain.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_strain, cap) };
            let mut w = 0usize;
            for s in body.springs.iter() {
                if w >= cap {
                    break;
                }
                let len = (body.particles[s.a].pos - body.particles[s.b].pos).length();
                let rest = s.rest_length;
                slice[w] = if rest > 0.0 { (len - rest) / rest } else { 0.0 };
                w += 1;
            }
            for d in body.distance_constraints.iter() {
                if w >= cap {
                    break;
                }
                let len = (body.particles[d.a].pos - body.particles[d.b].pos).length();
                let rest = d.rest;
                slice[w] = if rest > 0.0 { (len - rest) / rest } else { 0.0 };
                w += 1;
            }
        }
        clear_error();
        total as u32
    })
}

// ── Phase 23a: 静止长度缩放（纯状态缩放，供呼吸 / 生长 / 挤压动画）──────────────
//
// 现有 FFI 能加弹簧 / 距离约束并设刚度，但**不能整体缩放静止长度**——做"呼吸"
// "肌肉收缩""被挤压拉长"这类动画时只能逐条改，或改不动（XPBD 距离约束 rest 是
// 加时捕获的）。补一个纯状态缩放：把所有弹簧 rest_length 与距离约束 rest 乘
// factor。不改 fork、不动求解器，和 Phase 22 同属"纯 mps-core 追加"。

/// Uniformly scale the rest length of every structural edge (springs + XPBD
/// distance constraints) in a soft body by `factor`.
///
/// This is a one-shot state mutation (not a per-step force): it multiplies each
/// `Spring::rest_length` and `DistanceConstraint::rest` by `factor`. It is the
/// cheap primitive behind "breathing" / "muscle contraction" / "squeeze-stretch"
/// effects — previously users had to retune every edge by hand. `factor` must be
/// strictly positive; a non-positive value returns `ERR_INVALID_ARGUMENT` and
/// touches nothing.
///
/// Returns the number of edges scaled (springs + distance constraints), or 0 on
/// null world / unknown id / invalid factor.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_scale_rest_length(
    world: *mut WorldHandle,
    id: u32,
    factor: f64,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_scale_rest_length: world is null",
            );
            return 0;
        };
        if !factor.is_finite() || factor <= 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_scale_rest_length: factor must be > 0",
            );
            return 0;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_scale_rest_length: unknown id");
            return 0;
        };
        let n = body.springs.len() + body.distance_constraints.len();
        for s in body.springs.iter_mut() {
            s.rest_length *= factor;
        }
        for d in body.distance_constraints.iter_mut() {
            d.rest *= factor;
        }
        clear_error();
        n as u32
    })
}

// ── Phase 23b: 逐三角形法线回读（纯只读，供渲染层打光 / 调试可视化）────────────
//
// 渲染层要正确地给软体 / 布料打光，需要逐三角形法线。现在能回读三角形拓扑
// （read_triangles），但法线得上层自己按粒子位置算。这里补一个纯只读回读：
// 对每个三角形现算法线（(p1-p0)×(p2-p0) 归一化）写出 3 个 f64。长度从现有粒子
// 位置现算，不改 fork、不动求解器，与 Phase 22 应力回读同源。

/// Read per-triangle unit normals for a soft body.
///
/// Triangles are enumerated in the order returned by [`soft_body_read_triangles`].
/// For each triangle `T = (i0, i1, i2)` the function writes the unit normal
/// `(p1 - p0) × (p2 - p0)` normalized into `out_normals[3*k .. 3*k+3]`. Returns
/// the triangle count (so the caller can size its buffer); when `out_normals` is
/// null or `capacity` is 0 the count is returned without writing. Degenerate
/// triangles yield a zero normal.
///
/// Pure read-out for rendering / debug visualisation; does not affect the solver.
///
/// # Safety
/// `world` must be a valid world pointer. `out_normals` must point to an array of
/// at least `capacity` `f64` elements when non-null.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_normals(
    world: *const WorldHandle,
    id: u32,
    out_normals: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_read_normals: world is null");
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_normals: unknown id");
            return 0;
        };
        let n = body.triangles.len();
        let cap = capacity as usize;
        if !out_normals.is_null() && cap > 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(out_normals, cap) };
            for (k, t) in body.triangles.iter().enumerate().take(cap / 3) {
                let base = k * 3;
                let p0 = body.particles[t[0] as usize].pos;
                let p1 = body.particles[t[1] as usize].pos;
                let p2 = body.particles[t[2] as usize].pos;
                let nrm = (p1 - p0).cross(p2 - p0);
                let len = nrm.length();
                let (nx, ny, nz) = if len > 1e-12 {
                    let inv = 1.0 / len;
                    (nrm.x * inv, nrm.y * inv, nrm.z * inv)
                } else {
                    (0.0, 0.0, 0.0)
                };
                slice[base] = nx;
                slice[base + 1] = ny;
                slice[base + 2] = nz;
            }
        }
        clear_error();
        n as u32
    })
}

// ── Phase 25 #1: 接触力回读（纯 mps-core，零 fork 改动）────────────────────────────
//
// soft_body_collision 让每个自由质点有一枚 proxy 动态刚体 + Ball collider；接触力由
// Rapier narrow-phase 在 proxy 上算。这里在每个 proxy collider 上累加
// ContactPair::total_impulse()，写回每个质点的净接触力向量。纯只读，不改求解器，
// 与 Phase 22 应力 / Phase 23 法线回读同源。collide==false 或没有 proxy 时全 0，
// 返回质点数（与 read_normals 同约定：capacity 不足只写满 cap 个）。

/// Read the per-particle net contact force for a collision-coupled soft body.
///
/// For each free particle that has a proxy collider, the function sums the
/// `ContactPair::total_impulse` over every active contact pair touching that
/// collider, writing the net force vector into `out_fx/out_fy/out_fz[k]`. This is
/// the contact reaction the soft body exerts/feels through its proxy colliders —
/// the primitive behind "step on a soft cushion and get pushed back up" logic.
///
/// Returns the particle count (so the caller can size its buffer); when `out_fx`
/// is null or `capacity` is 0 the count is returned without writing. Bodies with
/// `collide == false` (no proxies) yield zero force for every particle. Pure
/// read-out: does not affect the solver.
///
/// # Safety
/// `world` must be a valid world pointer. `out_fx/out_fy/out_fz` must each point
/// to an array of at least `capacity` `f64` elements when non-null.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_read_contact_force(
    world: *const WorldHandle,
    id: u32,
    out_fx: *mut f64,
    out_fy: *mut f64,
    out_fz: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_ref() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_read_contact_force: world is null",
            );
            return 0;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_read_contact_force: unknown id");
            return 0;
        };
        let n = body.particles.len();
        let cap = capacity as usize;

        // Accumulator per particle, reused whether or not we write it out.
        let mut fx = vec![0.0f64; n];
        let mut fy = vec![0.0f64; n];
        let mut fz = vec![0.0f64; n];

        // Only collision-coupled bodies have proxies; everything else is zero.
        if let Some(proxies) = world.inner.soft_body_proxies.get(&id) {
            // Map collider handle -> owning particle index (proxy colliders only).
            let mut col_to_particle: std::collections::HashMap<ColliderHandle, usize> =
                std::collections::HashMap::new();
            for (i, ph) in proxies.iter().enumerate() {
                let Some(rb_h) = ph else {
                    continue;
                };
                let Some(rb) = world.inner.bodies.get(*rb_h) else {
                    continue;
                };
                for c in rb.colliders() {
                    col_to_particle.insert(*c, i);
                }
            }
            for pair in world.inner.narrow_phase.contact_pairs() {
                let imp = pair.total_impulse();
                // Force on collider2 = +imp, on collider1 = -imp (Newton's 3rd law;
                // `total_impulse` is the force on collider2, pointing from c1 to c2).
                if let Some(&i1) = col_to_particle.get(&pair.collider1) {
                    fx[i1] -= imp.x;
                    fy[i1] -= imp.y;
                    fz[i1] -= imp.z;
                }
                if let Some(&i2) = col_to_particle.get(&pair.collider2) {
                    fx[i2] += imp.x;
                    fy[i2] += imp.y;
                    fz[i2] += imp.z;
                }
            }
        }

        if !out_fx.is_null() && !out_fy.is_null() && !out_fz.is_null() && cap > 0 {
            let m = cap.min(n);
            let sx = unsafe { std::slice::from_raw_parts_mut(out_fx, cap) };
            let sy = unsafe { std::slice::from_raw_parts_mut(out_fy, cap) };
            let sz = unsafe { std::slice::from_raw_parts_mut(out_fz, cap) };
            sx[..m].copy_from_slice(&fx[..m]);
            sy[..m].copy_from_slice(&fy[..m]);
            sz[..m].copy_from_slice(&fz[..m]);
        }
        clear_error();
        n as u32
    })
}

// ── Phase 24: XPBD/MassSpring substeps 暴露（动 fork：SoftBody.substeps 字段）──
//
// 求解器现在 step_xpbd / step_mass_spring 只有 1 substep（frame dt 一次性投影）。
// 暴露 substeps 后，每帧 dt 被切成 n 等份、每个子步独立投影，硬材质 / 高 compliance
// 收敛更快、更稳。改动在 fork 的 SoftBody::step 循环 + set_substeps，mps-core 只
// 镜像一个 FFI。n==0 拒绝（保持前值），与 fork 侧守卫一致。

/// Set the number of solver substeps per `soft_body_step` call for a soft body.
///
/// `n >= 1` splits the frame `dt` into `n` equal slices; the active solver
/// (XPBD or MassSpring) is run once per slice, projecting constraints at a finer
/// time resolution. Stiff materials and high-compliance edges converge faster
/// and stay stable with more substeps (at `n×` the per-step CPU cost). `n == 0`
/// is rejected and leaves the previous value unchanged.
///
/// Returns the new substep count, or 0 on null world / unknown id / invalid `n`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_set_substeps(world: *mut WorldHandle, id: u32, n: u32) -> u32 {
    ffi_guard(0, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_set_substeps: world is null");
            return 0;
        };
        if n == 0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_set_substeps: n must be >= 1",
            );
            return 0;
        }
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_set_substeps: unknown id");
            return 0;
        };
        body.set_substeps(n);
        clear_error();
        body.substeps
    })
}

// ── Phase 5d: 区块破坏 → 软体重建联动（Minecraft 闭环）──────────────────────

//
// 监听 voxel 地形 `set_cell` 破坏事件后，上层调用本 FFI：把被挖空的方块格
// (cx,cy,cz) 经 `voxel_soft_meta` 映射到对应质点下标，调 `SoftBody::remove_particle`
// 删该质点 + 其弹簧/约束，并就地重建映射（其余下标随 remove_particle 平移）。
// 软体在新拓扑下继续仿真 —— 这就是「破坏方块 → 软体塌缩」的闭环原子操作。

/// Dig out a single voxel cell of a soft body built via `soft_body_voxel_build`,
/// removing the particle that occupies it (plus its incident springs/constraints)
/// and rebuilding the voxel→particle map so further digs stay consistent.
///
/// Returns `Bool::TRUE` on success. `Bool::FALSE` if the body/id is unknown, the
/// cell is out of bounds, or the cell is already empty/dug.
///
/// # Safety
/// `world` must be a valid world pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_voxel_dig(
    world: *mut WorldHandle,
    id: u32,
    cell_x: u32,
    cell_y: u32,
    cell_z: u32,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "soft_body_voxel_dig: world is null");
            return Bool::FALSE;
        };
        soft_body_voxel_dig_inner(world, id, cell_x, cell_y, cell_z)
    })
}

/// Core of [`soft_body_voxel_dig`] operating on an already-unwrapped world
/// handle. Exposed as `pub(crate)` so `collider_voxel_edit` can propagate a
/// terrain dig to overlapping soft bodies (Phase 5g) without re-wrapping the
/// raw pointer.
pub(crate) fn soft_body_voxel_dig_inner(
    world: &mut WorldHandle,
    id: u32,
    cell_x: u32,
    cell_y: u32,
    cell_z: u32,
) -> Bool {
    let sid = SoftBodyId(id);
    let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
        set_error(ERR_NOT_FOUND, "soft_body_voxel_dig: unknown id");
        return Bool::FALSE;
    };
    let Some(meta) = world.inner.voxel_soft_meta.get_mut(&id) else {
        set_error(
            ERR_NOT_FOUND,
            "soft_body_voxel_dig: body has no voxel mapping",
        );
        return Bool::FALSE;
    };
    // cell_linear uses the same layout as VoxelGrid::index / soft_body_voxel_build.
    if cell_x as usize >= meta.sx || cell_y as usize >= meta.sy || cell_z as usize >= meta.sz {
        set_error(
            ERR_INVALID_ARGUMENT,
            "soft_body_voxel_dig: cell out of bounds",
        );
        return Bool::FALSE;
    }
    let cell_linear = cell_x as usize + meta.sx * (cell_z as usize + meta.sz * cell_y as usize);
    let p_idx = match meta.map.get(cell_linear).copied() {
        Some(v) if v >= 0 => v as usize,
        _ => {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_voxel_dig: cell is empty or already dug",
            );
            return Bool::FALSE;
        }
    };
    if !body.remove_particle(p_idx) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "soft_body_voxel_dig: remove_particle failed",
        );
        return Bool::FALSE;
    }
    // Rebuild the map: the dug cell becomes -1; every surviving index that was
    // > p_idx shifts down by one (mirrors SoftBody::remove_particle).
    for m in meta.map.iter_mut() {
        if *m == p_idx as i64 {
            *m = -1;
        } else if *m > p_idx as i64 {
            *m -= 1;
        }
    }
    clear_error();
    Bool::TRUE
}

/// Phase 5g: when a voxel collider cell is dug (set to empty), propagate the
/// dig to every soft body whose voxel grid overlaps that world-space cell. For
/// each soft body we convert the dug cell's world-center into the soft body's
/// own cell coordinates (stored in `VoxelSoftMeta.origin` / `voxel_size`) and
/// call `soft_body_voxel_dig_inner`. Digs are best-effort and idempotent: a
/// soft cell that is already empty/dug simply returns `Bool::FALSE` without
/// disturbing the body.
///
/// Must be called when no other borrow of `world.inner` (e.g. a `VoxelCache`)
/// is live, since it mutates `soft_bodies` + `voxel_soft_meta`.
pub(crate) fn propagate_dig_to_soft_bodies(world: &mut WorldHandle, world_center: Vec3) {
    // Snapshot the ids first to keep the borrow of `voxel_soft_meta` short.
    let ids: Vec<u32> = world.inner.voxel_soft_meta.keys().copied().collect();
    for id in ids {
        let (sx, sy, sz, origin, voxel_size) = {
            let meta = match world.inner.voxel_soft_meta.get(&id) {
                Some(m) => m,
                None => continue,
            };
            (meta.sx, meta.sy, meta.sz, meta.origin, meta.voxel_size)
        };
        if !voxel_size.is_finite() || voxel_size <= 0.0 {
            continue;
        }
        // World-center → soft-body cell coordinate (uniform grid).
        let scx = ((world_center.x - origin.x) / voxel_size - 0.5).round() as i64;
        let scy = ((world_center.y - origin.y) / voxel_size - 0.5).round() as i64;
        let scz = ((world_center.z - origin.z) / voxel_size - 0.5).round() as i64;
        if scx < 0
            || scy < 0
            || scz < 0
            || scx as usize >= sx
            || scy as usize >= sy
            || scz as usize >= sz
        {
            continue;
        }
        soft_body_voxel_dig_inner(world, id, scx as u32, scy as u32, scz as u32);
    }
}

// ── Phase 5f: 软体-刚体碰撞（proxy collider 桥接）──────────────────────────
//
// 给每个自由质点（inv_mass > 0）建一个动态 `RigidBody` + `Ball` collider（proxy，
// `gravity_scale = 0`，质量 = 1/inv_mass）。`world_step` 在刚性步进前把质点力/位姿
// 写入 proxy，narrow-phase/contact 自然作用于 proxy，步进后再把 proxy 受接触后的位姿
// 读回质点 —— 软体于是被地形/实体挡住、反弹、堆叠。完全复用现有接触，不改 rapier 求解器。
// `SoftBody.collide` 置 true 后 `SoftBody::step` 跳过自积分（位置由 proxy 驱动）。
// pinned 质点（inv_mass == 0）不建 proxy，位置由外部固定。

/// Enable or disable rigid-body collision coupling for a soft body.
///
/// When `enabled` is `Bool::TRUE`, one dynamic `Ball` collider (radius `particle_radius`)
/// is created per free particle and registered in the world's collision-proxy table; the
/// body's `collide` flag is set so the integration layer drives its particles from the
/// proxies. When `Bool::FALSE`, any existing proxies are removed and `collide` is cleared.
///
/// Returns `Bool::TRUE` on success.
///
/// # Safety
/// `world` must be a valid world pointer returned by `world_create`.
#[unsafe(no_mangle)]
pub extern "C" fn soft_body_enable_collision(
    world: *mut WorldHandle,
    id: u32,
    particle_radius: f64,
    enabled: Bool,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(
                ERR_NULL_POINTER,
                "soft_body_enable_collision: world is null",
            );
            return Bool::FALSE;
        };
        let sid = SoftBodyId(id);
        let Some(body) = world.inner.soft_bodies.get_mut(sid) else {
            set_error(ERR_NOT_FOUND, "soft_body_enable_collision: unknown id");
            return Bool::FALSE;
        };
        if particle_radius <= 0.0 || !particle_radius.is_finite() {
            set_error(
                ERR_INVALID_ARGUMENT,
                "soft_body_enable_collision: bad particle_radius",
            );
            return Bool::FALSE;
        }
        if enabled == Bool::FALSE {
            // Tear down existing proxies.
            if let Some(proxies) = world.inner.soft_body_proxies.remove(&id) {
                for ph in proxies.into_iter().flatten() {
                    world.inner.bodies.remove(
                        ph,
                        &mut world.inner.islands,
                        &mut world.inner.colliders,
                        &mut world.inner.impulse_joints,
                        &mut world.inner.multibody_joints,
                        false,
                    );
                }
            }
            body.collide = false;
            clear_error();
            return Bool::TRUE;
        }
        // Build proxies.
        body.collide = true;
        body.particle_radius = particle_radius;
        let mut proxies: Vec<Option<RigidBodyHandle>> = Vec::with_capacity(body.particles.len());
        for p in &body.particles {
            if p.inv_mass == 0.0 {
                proxies.push(None); // pinned: no proxy
                continue;
            }
            let mass = 1.0 / p.inv_mass;
            let rb = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .gravity_scale(0.0)
                .additional_mass(mass)
                .translation(p.pos)
                .linvel(p.vel)
                .build();
            let h = world.inner.bodies.insert(rb);
            // Phase 15: zero the collider's own density so the proxy's mass is *exactly*
            // the particle mass (`additional_mass`) and not inflated by the ball's volume.
            // A non-zero collider mass unbalances two-way momentum transfer (a light soft
            // particle would drive an over-heavy proxy that cannot cleanly push dynamic
            // rigid bodies). Density 0 keeps the reaction physically symmetric.
            let col = ColliderBuilder::ball(particle_radius).density(0.0).build();
            world
                .inner
                .colliders
                .insert_with_parent(col, h, &mut world.inner.bodies);
            proxies.push(Some(h));
        }
        world.inner.soft_body_proxies.insert(id, proxies);
        clear_error();
        Bool::TRUE
    })
}
