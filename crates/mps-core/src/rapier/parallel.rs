//! Multi-threaded execution support for mps-core's per-frame work.
//!
//! ## Background — where the frame time goes
//!
//! rapier's `parallel` feature already runs the collision-detection and
//! solver stages of [`PhysicsPipeline::step`] on rayon's thread pool.
//! Everything mps-core does *around* that call was single-threaded: force-law
//! evaluation, terrain-gravity sampling (O(faces) or O(grid) **per body**),
//! O(n²) pairwise gravity, external-force application, and the snapshot
//! export that feeds the render loop. For scenes with many dynamic bodies or
//! a heavy terrain-gravity source those serial phases dominate the frame
//! budget.
//!
//! ## Design — two-phase per-body evaluation
//!
//! Every force law already follows the same shape:
//!
//! 1. a **fill** phase that reads body state (position / velocity / mass) and
//!    computes one force per body — read-only over `RigidBodySet`, and
//! 2. an **apply** phase that mutates bodies through `ForceFacade::add_force`
//!    — inherently serial, because the facade holds `&mut RigidBodySet`.
//!
//! The fill phase parallelizes cleanly: [`par_map_bodies`] maps a per-body
//! closure over the dynamic-body handles on the rayon pool while preserving
//! order, and the serial apply phase replays the results into the facade.
//! Order preservation makes the parallel output *bit-identical* to the serial
//! one for per-body laws (each body's force is produced by exactly one task
//! with an unchanged arithmetic sequence), so switching the threshold never
//! changes simulation results.
//!
//! The one law with cross-body coupling — Newtonian pairwise gravity — uses
//! [`pairwise_gravity_accumulate`]: bodies are split into fixed
//! [`GRAVITY_CHUNK`]-body chunks; every chunk pair computes its cross forces
//! into per-chunk-pair accumulators, and an ordered merge produces a
//! deterministic sum that is independent of thread count and scheduling
//! (it differs from the serial upper-triangle sum only by floating-point
//! reassociation, i.e. at the last ulp).
//!
//! ## Thresholds
//!
//! Parallel dispatch has fixed overhead (task scheduling, result buffers).
//! Below the per-call-site minimum body count — [`PAR_MIN_ITEMS`] by default —
//! everything runs on the calling thread, so small scenes keep the exact
//! serial behaviour and pay nothing.
//!
//! ## Thread pool
//!
//! All parallel sections run on rayon's global pool — the same pool rapier's
//! `parallel` feature uses. Force fills and `pipeline.step` never overlap in
//! time, so the shared pool cannot oversubscribe the machine. The pool size
//! defaults to all logical cores and can be configured (highest priority
//! first) via:
//!
//! 1. the [`parallel_set_thread_count`] FFI (only effective before the first
//!    parallel operation initialises the pool),
//! 2. `MPS_CORE_THREADS` (read once, same window),
//! 3. `RAYON_NUM_THREADS` (rayon's own env var, always honoured).
//!
//! [`PhysicsPipeline::step`]: rapier3d::pipeline::PhysicsPipeline::step

use rapier3d::math::Vector;
use rapier3d::prelude::{RigidBody, RigidBodyHandle, RigidBodySet};
use rayon::prelude::*;

use crate::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_UNSUPPORTED, clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{Bool, Vec3, quat_from_rapier, vec3_from_rapier};

/// Default minimum number of bodies before a per-body fill switches from the
/// calling thread to the rayon pool. Per-body law math is tens of nanoseconds;
/// below this many bodies the scheduling overhead outweighs the parallel gain.
pub(crate) const PAR_MIN_ITEMS: usize = 128;

/// Bodies per chunk in the parallel pairwise-gravity decomposition. Fixed (not
/// derived from the pool width) so the summation order — and therefore the
/// bit-level result — is identical on every machine.
pub(crate) const GRAVITY_CHUNK: usize = 128;

/// Minimum body count before the O(n²) pairwise-gravity law switches from the
/// serial upper-triangle loop to the chunked parallel decomposition.
pub(crate) const GRAVITY_PAR_MIN_BODIES: usize = 256;

/// Minimum body count for terrain-gravity sampling. Per-body cost is the
/// heaviest of any law (O(faces) polyhedron / O(grid) DEM sums), so the
/// crossover is much lower than [`PAR_MIN_ITEMS`].
pub(crate) const TERRAIN_GRAVITY_MIN_ITEMS: usize = 32;

// ---------------------------------------------------------------------------
// Thread pool configuration
// ---------------------------------------------------------------------------

/// Read `MPS_CORE_THREADS` and size the global pool accordingly, exactly once.
/// Later calls are no-ops; `RAYON_NUM_THREADS` wins when both are set because
/// rayon consults it during its own lazy initialisation.
fn ensure_pool_configured() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let Ok(raw) = std::env::var("MPS_CORE_THREADS") else {
            return;
        };
        if let Ok(n) = raw.trim().parse::<usize>()
            && n > 0
        {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global();
        }
    });
}

/// Number of worker threads in the global rayon pool (lazily created).
pub fn thread_count() -> u32 {
    ensure_pool_configured();
    rayon::current_num_threads() as u32
}

/// Try to (re)size the global rayon pool. Only succeeds before the pool has
/// been initialised (first parallel operation, `pipeline.step` with bodies, or
/// an earlier call); afterwards use `RAYON_NUM_THREADS` at process start.
pub fn set_thread_count(n: u32) -> bool {
    if n == 0 {
        return false;
    }
    rayon::ThreadPoolBuilder::new()
        .num_threads(n as usize)
        .build_global()
        .is_ok()
}

// ---------------------------------------------------------------------------
// Per-body parallel map — the two-phase fill helper
// ---------------------------------------------------------------------------

/// Map a per-body computation over `handles` on the rayon pool.
///
/// `compute` must be a pure read of body state: it receives the body handle
/// and a shared reference to the body, and its results are collected in
/// `handles` order. When the handle count is below `min_items` — or the pool
/// has a single worker — the map runs on the calling thread with the identical
/// closure, so results are bit-identical across the threshold.
///
/// Callers replay the returned values through `ForceFacade::add_force` (or
/// push them into a scratch buffer) on the calling thread.
pub(crate) fn par_map_bodies<R, F>(
    handles: &[RigidBodyHandle],
    bodies: &RigidBodySet,
    min_items: usize,
    compute: F,
) -> Vec<R>
where
    R: Send,
    F: Fn(RigidBodyHandle, &RigidBody) -> R + Sync,
{
    ensure_pool_configured();
    if handles.len() < min_items.max(2) || rayon::current_num_threads() <= 1 {
        return handles.iter().map(|&h| compute(h, &bodies[h])).collect();
    }
    handles
        .par_iter()
        .map(|&h| compute(h, &bodies[h]))
        .collect()
}

// ---------------------------------------------------------------------------
// O(n²) pairwise gravity — chunked parallel accumulation
// ---------------------------------------------------------------------------

/// Per-chunk-pair accumulator: forces acting on chunk `a`'s bodies (and, for
/// cross-chunk pairs, the equal-and-opposite forces on chunk `b`'s bodies).
struct ChunkPairAccum {
    a: usize,
    b: usize,
    /// Intra-chunk sums when `a == b`, otherwise forces on `a` bodies from `b`.
    a_side: Vec<Vector>,
    /// Forces on `b` bodies from `a` (empty for intra-chunk pairs).
    b_side: Vec<Vector>,
}

#[inline]
fn gravity_pair(
    mi: f64,
    pi: Vector,
    mj: f64,
    pj: Vector,
    g: f64,
    min_dist: f64,
    max_dist_sq: f64,
) -> Option<Vector> {
    let offset = pj - pi;
    let dist_sq = offset.length_squared();
    if dist_sq > max_dist_sq {
        return None;
    }
    let dist = dist_sq.sqrt().max(min_dist);
    // F = G · mᵢ · mⱼ / r² · r̂ = G · mᵢ · mⱼ / r³ · r
    Some(offset * (g * mi * mj / (dist_sq * dist)))
}

/// Accumulate Newtonian pairwise gravity for `body_data`
/// (`(handle, mass, position)`, all with `mass > 0`) into one net force per
/// body, parallelised with Newton's third law (each pair evaluated once).
///
/// Bodies are split into fixed [`GRAVITY_CHUNK`]-body chunks. Every chunk pair
/// `(a, b)` with `a ≤ b` is processed by one rayon task that accumulates into
/// its own disjoint buffers; the merge afterwards walks the chunk pairs in a
/// fixed lexicographic order, so the summation sequence per body — and hence
/// the result — is fully deterministic and independent of the pool width.
///
/// Compared to the serial upper-triangle loop the per-body sums are
/// reassociated (different order of the same addends), so results agree only
/// to floating-point rounding; use the serial path (below
/// [`GRAVITY_PAR_MIN_BODIES`]) when bit-identity with the legacy loop matters.
pub(crate) fn pairwise_gravity_accumulate(
    body_data: &[(RigidBodyHandle, f64, Vector)],
    g: f64,
    min_dist: f64,
    max_dist_sq: f64,
) -> Vec<Vector> {
    let n = body_data.len();
    let n_chunks = n.div_ceil(GRAVITY_CHUNK);
    // Chunk pairs (a, b), a ≤ b, in lexicographic order — the merge relies on
    // this order for a deterministic summation sequence.
    let mut pairs = Vec::with_capacity(n_chunks * (n_chunks + 1) / 2);
    for a in 0..n_chunks {
        for b in a..n_chunks {
            pairs.push((a, b));
        }
    }

    let accums: Vec<ChunkPairAccum> = pairs
        .into_par_iter()
        .map(|(a, b)| {
            let a0 = a * GRAVITY_CHUNK;
            let a1 = ((a + 1) * GRAVITY_CHUNK).min(n);
            if a == b {
                // Intra-chunk: symmetric accumulation, both directions at once.
                let mut local = vec![Vector::ZERO; a1 - a0];
                for i in a0..a1 {
                    let (_, mi, pi) = body_data[i];
                    for j in (i + 1)..a1 {
                        let (_, mj, pj) = body_data[j];
                        if let Some(f_ij) = gravity_pair(mi, pi, mj, pj, g, min_dist, max_dist_sq) {
                            local[i - a0] += f_ij;
                            local[j - a0] -= f_ij;
                        }
                    }
                }
                ChunkPairAccum {
                    a,
                    b,
                    a_side: local,
                    b_side: Vec::new(),
                }
            } else {
                let b0 = b * GRAVITY_CHUNK;
                let b1 = ((b + 1) * GRAVITY_CHUNK).min(n);
                let mut on_a = vec![Vector::ZERO; a1 - a0];
                let mut on_b = vec![Vector::ZERO; b1 - b0];
                for i in a0..a1 {
                    let (_, mi, pi) = body_data[i];
                    for j in b0..b1 {
                        let (_, mj, pj) = body_data[j];
                        if let Some(f_ij) = gravity_pair(mi, pi, mj, pj, g, min_dist, max_dist_sq) {
                            on_a[i - a0] += f_ij;
                            on_b[j - b0] -= f_ij;
                        }
                    }
                }
                ChunkPairAccum {
                    a,
                    b,
                    a_side: on_a,
                    b_side: on_b,
                }
            }
        })
        .collect();

    // Deterministic ordered merge: each body's net force sums its chunk-pair
    // contributions in the fixed (a, b) order produced above.
    let mut net = vec![Vector::ZERO; n];
    for acc in &accums {
        let a0 = acc.a * GRAVITY_CHUNK;
        for (k, f) in acc.a_side.iter().enumerate() {
            net[a0 + k] += *f;
        }
        if acc.a != acc.b {
            let b0 = acc.b * GRAVITY_CHUNK;
            for (k, f) in acc.b_side.iter().enumerate() {
                net[b0 + k] += *f;
            }
        }
    }
    net
}

// ---------------------------------------------------------------------------
// Body pose snapshots (render-loop export)
// ---------------------------------------------------------------------------

/// 13-f64 snapshot of one body: pos3 + quat4 (+ linvel3 + angvel3 when
/// `with_velocity`); velocity lanes are zero otherwise.
fn body_snapshot13(body: &RigidBody, with_velocity: bool) -> [f64; 13] {
    let translation = vec3_from_rapier(body.translation());
    let rotation = quat_from_rapier(*body.rotation());
    let (linvel, angvel) = if with_velocity {
        (
            vec3_from_rapier(body.linvel()),
            vec3_from_rapier(body.angvel()),
        )
    } else {
        (Vec3::default(), Vec3::default())
    };
    [
        translation.x,
        translation.y,
        translation.z,
        rotation.i,
        rotation.j,
        rotation.k,
        rotation.w,
        linvel.x,
        linvel.y,
        linvel.z,
        angvel.x,
        angvel.y,
        angvel.z,
    ]
}

/// Compute pose (+ optional velocity) snapshots for `handles` in order,
/// parallelised above [`PAR_MIN_ITEMS`] handles. Read-only over the body set;
/// used by the `world_body_snapshot` / `world_dynamic_body_snapshot` FFI
/// export that runs every frame on the render thread.
pub(crate) fn body_pose_snapshots(
    handles: &[RigidBodyHandle],
    bodies: &RigidBodySet,
    with_velocity: bool,
) -> Vec<[f64; 13]> {
    par_map_bodies(handles, bodies, PAR_MIN_ITEMS, |_, body| {
        body_snapshot13(body, with_velocity)
    })
}

// ---------------------------------------------------------------------------
// FFI — thread pool introspection / configuration
// ---------------------------------------------------------------------------

/// Number of worker threads in the shared rayon pool used by mps-core's
/// parallel force fills, pairwise gravity, snapshot export, and rapier's own
/// parallel solver stages.
///
/// Defaults to the machine's logical core count; see the `parallel` module
/// docs for the configuration knobs.
#[unsafe(no_mangle)]
pub extern "C" fn parallel_thread_count() -> u32 {
    ffi_guard(0, thread_count)
}

/// Resize the shared rayon pool. Returns `true` on success; `false` when
/// `threads == 0` (`ERR_INVALID_ARGUMENT`) or the pool is already running
/// (`ERR_UNSUPPORTED` — set the count before the first parallel operation, or
/// via `RAYON_NUM_THREADS` at process start).
///
/// # Safety
///
/// No pointer parameters; safe to call from any thread.
#[unsafe(no_mangle)]
pub extern "C" fn parallel_set_thread_count(threads: u32) -> Bool {
    ffi_guard(Bool::FALSE, || {
        ensure_pool_configured();
        if threads == 0 {
            set_error(ERR_INVALID_ARGUMENT, "thread count must be non-zero");
            return Bool::FALSE;
        }
        if set_thread_count(threads) {
            clear_error();
            Bool::TRUE
        } else {
            set_error(
                ERR_UNSUPPORTED,
                "global thread pool already initialised; configure before first use",
            );
            Bool::FALSE
        }
    })
}
