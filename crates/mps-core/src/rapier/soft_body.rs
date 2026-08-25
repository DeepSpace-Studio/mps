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

// ── Phase 5a: general soft-body builder (unlock arbitrary topology) ──────────
//
// Phase 4 只暴露了「voxel 网格 → 软体」与「设置重力」两个高层入口，外部调用方
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
    })
}
