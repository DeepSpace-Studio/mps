//! Voronoi pre-fracture — seed-based convex-cell computation for fracture-mesh
//! splitting.
//!
//! Pure math only: no `WorldHandle`, no Rapier state. Given an AABB (in the
//! source body's local space) and a set of seed points, each seed's Voronoi
//! cell is the intersection of the six box half-spaces with the bisector
//! half-space against every other seed. Cells are computed by vertex
//! enumeration (intersect every plane triple, keep points satisfying all
//! half-spaces), which is robust and simple for the ~10–50 plane counts this
//! produces per cell.
//!
//! For large seed sets, exhaustive triple enumeration over every bisector is
//! O(N³) per cell. [`voronoi_cell`] therefore runs a two-pass scheme when the
//! neighbor count exceeds [`EXACT_NEIGHBOR_LIMIT`]: an inner cell is first
//! computed from only the [`PRUNE_KNN`] nearest neighbors, then any bisector
//! that does not cut that inner cell is dropped. Because the inner cell (a
//! subset of the half-spaces) is a superset of the true cell, a plane that
//! misses the inner cell provably cannot cut the true cell — the pruning is
//! exact, not an approximation. As a last-resort bound for pathological
//! distributions (dense clusters where pruning keeps hundreds of planes), the
//! surviving bisectors are capped at [`MAX_CELL_PLANES`], nearest first; only
//! that cap degrades exactness.
//!
//! `mps-core`'s `fracture_mesh_body_create_with_voronoi` wraps
//! [`voronoi_fragments_from_seeds`] into the fracture-mesh body creation flow;
//! this module stays formula-pure so it can be tested without a world.

use crate::ffi::{FractureFragmentDesc, Vec3};
use crate::math::{Vector3f64, finite_vec3};

/// Upper bound on unique seeds accepted by [`voronoi_fragments_from_seeds`]
/// (comfortably below the fracture-mesh layer's 1024-fragment cap).
pub const MAX_VORONOI_SEEDS: usize = 512;

/// Neighbor count above which the exact-pruning two-pass scheme kicks in
/// (below this, plain triple enumeration is fast enough).
pub const EXACT_NEIGHBOR_LIMIT: usize = 48;

/// Number of nearest neighbors used to build the conservative inner cell for
/// pruning (inner cell ⊇ true cell keeps the prune exact).
pub const PRUNE_KNN: usize = 24;

/// Hard cap on surviving bisector planes per cell after pruning; only
/// pathological seed distributions (dense clusters) ever hit this, and the
/// nearest planes — the ones that shape the cell — are kept.
pub const MAX_CELL_PLANES: usize = 96;

/// Relative epsilon for plane/vertex tolerances (scaled by the AABB diagonal).
const CELL_EPSILON: f64 = 1.0e-9;

/// A convex Voronoi cell: distinct vertices plus derived volume and centroid.
#[derive(Clone, Debug)]
pub struct VoronoiCell {
    /// Distinct cell vertices (at least 4 for a valid 3D cell).
    pub vertices: Vec<Vector3f64>,
    /// Cell volume (a degenerate cell is returned as `None`, never 0.0).
    pub volume: f64,
    /// Area-weighted centroid of the cell boundary.
    pub centroid: Vector3f64,
}

/// Half-space `dot(normal, p) <= offset` with a normalized normal.
struct HalfSpace {
    normal: Vector3f64,
    offset: f64,
}

impl HalfSpace {
    #[inline]
    fn contains(&self, p: Vector3f64, tol: f64) -> bool {
        self.normal.dot(p) <= self.offset + tol
    }

    #[inline]
    fn on_plane(&self, p: Vector3f64, tol: f64) -> bool {
        (self.normal.dot(p) - self.offset).abs() <= tol
    }
}

#[inline]
fn v3(value: Vec3) -> Vector3f64 {
    Vector3f64::new(value.x, value.y, value.z)
}

fn aabb_half_spaces(min: Vector3f64, max: Vector3f64) -> Vec<HalfSpace> {
    let axes = [
        (Vector3f64::new(1.0, 0.0, 0.0), max.x, -min.x),
        (Vector3f64::new(0.0, 1.0, 0.0), max.y, -min.y),
        (Vector3f64::new(0.0, 0.0, 1.0), max.z, -min.z),
    ];
    let mut planes = Vec::with_capacity(6);
    for (axis, hi, lo) in axes {
        planes.push(HalfSpace {
            normal: axis,
            offset: hi,
        });
        planes.push(HalfSpace {
            normal: -axis,
            offset: lo,
        });
    }
    planes
}

/// Bisector half-space of `center` against `other`: the set of points closer
/// to `center` than to `other`.
fn bisector(center: Vector3f64, other: Vector3f64) -> Option<HalfSpace> {
    let normal = (other - center).try_normalize()?;
    let mid = (center + other) * 0.5;
    Some(HalfSpace {
        normal,
        offset: normal.dot(mid),
    })
}

/// Computes the Voronoi cell of `center` inside the AABB, bounded by the
/// bisector planes against every neighbor seed. Returns `None` when the input
/// is invalid or the cell degenerates (fewer than 4 vertices, zero volume).
pub fn voronoi_cell(
    center: Vec3,
    neighbors: &[Vec3],
    aabb_min: Vec3,
    aabb_max: Vec3,
) -> Option<VoronoiCell> {
    if !finite_vec3(center) || !finite_vec3(aabb_min) || !finite_vec3(aabb_max) {
        return None;
    }
    let c = v3(center);
    let lo = v3(aabb_min);
    let hi = v3(aabb_max);
    if lo.x >= hi.x || lo.y >= hi.y || lo.z >= hi.z {
        return None;
    }
    let tol = CELL_EPSILON * (hi - lo).length().max(1.0);

    let mut planes = aabb_half_spaces(lo, hi);
    let mut neighbor_planes: Vec<HalfSpace> = Vec::with_capacity(neighbors.len());
    for neighbor in neighbors {
        if let Some(plane) = bisector(c, v3(*neighbor)) {
            neighbor_planes.push(plane);
        }
    }

    // A bisector's offset equals half the seed distance (the normal points
    // from `center` toward the other seed through the midpoint), so sorting by
    // offset orders neighbors nearest first.
    if neighbor_planes.len() > EXACT_NEIGHBOR_LIMIT {
        let mut by_distance = neighbor_planes.iter().collect::<Vec<_>>();
        by_distance.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut inner: Vec<HalfSpace> = aabb_half_spaces(lo, hi);
        inner.extend(by_distance[..PRUNE_KNN].iter().map(|p| HalfSpace {
            normal: p.normal,
            offset: p.offset,
        }));
        if let Some(inner_cell) = cell_from_planes(&inner, tol) {
            // Exact prune: the inner cell (fewer half-spaces) is a superset of
            // the true cell, so any plane that does not cut it cannot cut the
            // true cell either.
            neighbor_planes.retain(|plane| {
                inner_cell
                    .vertices
                    .iter()
                    .map(|v| plane.normal.dot(*v))
                    .fold(f64::NEG_INFINITY, f64::max)
                    > plane.offset + tol
            });
            // Last-resort bound for pathological clusters: keep the nearest
            // planes, which dominate the cell shape.
            if neighbor_planes.len() > MAX_CELL_PLANES {
                neighbor_planes.sort_by(|a, b| {
                    a.offset
                        .partial_cmp(&b.offset)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                neighbor_planes.truncate(MAX_CELL_PLANES);
            }
        }
        // If the inner cell degenerated, fall through with every bisector —
        // slower but exact.
    }
    planes.extend(neighbor_planes);

    cell_from_planes(&planes, tol)
}

/// Enumerates the cell as the intersection of the given half-spaces: every
/// plane-triple intersection that satisfies all constraints is a candidate
/// vertex; faces, volume, and centroid are derived per plane.
fn cell_from_planes(planes: &[HalfSpace], tol: f64) -> Option<VoronoiCell> {
    // Candidate vertices: every plane-triple intersection that satisfies all
    // half-spaces. p = (o_i (n_j×n_k) + o_j (n_k×n_i) + o_k (n_i×n_j)) / det.
    let mut vertices: Vec<Vector3f64> = Vec::new();
    for i in 0..planes.len() {
        for j in i + 1..planes.len() {
            for k in j + 1..planes.len() {
                let (ni, nj, nk) = (planes[i].normal, planes[j].normal, planes[k].normal);
                let det = ni.dot(nj.cross(nk));
                if det.abs() <= CELL_EPSILON {
                    continue; // (near-)parallel planes — no unique point
                }
                let p = planes[i].offset * nj.cross(nk)
                    + planes[j].offset * nk.cross(ni)
                    + planes[k].offset * ni.cross(nj);
                let p = p / det;
                if planes.iter().all(|plane| plane.contains(p, tol))
                    && !vertices.iter().any(|v| (*v - p).length() <= tol)
                {
                    vertices.push(p);
                }
            }
        }
    }
    if vertices.len() < 4 {
        return None;
    }

    // Faces (one per plane) → volume via the divergence theorem over
    // fan-triangulated faces, and an area-weighted boundary centroid.
    let face_tol = tol * 10.0;
    let mut oriented_volume = 0.0;
    let mut weighted_centroid = Vector3f64::zeros();
    let mut total_area = 0.0;
    for plane in planes {
        let face: Vec<Vector3f64> = vertices
            .iter()
            .copied()
            .filter(|v| plane.on_plane(*v, face_tol))
            .collect();
        if face.len() < 3 {
            continue;
        }
        let face_center =
            face.iter().fold(Vector3f64::zeros(), |acc, v| acc + *v) / face.len() as f64;
        // In-plane basis for angular ordering; the chosen reference axis keeps
        // the cross products well away from zero (|n·axis| < 0.9).
        let ref_axis = if plane.normal.x.abs() < 0.9 {
            Vector3f64::new(1.0, 0.0, 0.0)
        } else {
            Vector3f64::new(0.0, 1.0, 0.0)
        };
        let u = plane.normal.cross(ref_axis).try_normalize()?;
        let w = plane.normal.cross(u);
        let mut ordered = face;
        ordered.sort_by(|p, q| {
            let pr = *p - face_center;
            let qr = *q - face_center;
            let ap = pr.dot(w).atan2(pr.dot(u));
            let aq = qr.dot(w).atan2(qr.dot(u));
            ap.partial_cmp(&aq).unwrap_or(std::cmp::Ordering::Equal)
        });
        let v0 = ordered[0];
        for pair in ordered.windows(2) {
            let (vi, vj) = (pair[0], pair[1]);
            oriented_volume += v0.cross(vi).dot(vj);
            let tri_area = (vi - v0).cross(vj - v0).length() * 0.5;
            weighted_centroid += (v0 + vi + vj) * (tri_area / 3.0);
            total_area += tri_area;
        }
    }
    let volume = oriented_volume.abs() / 6.0;
    if volume <= tol || total_area <= tol {
        return None;
    }
    Some(VoronoiCell {
        vertices,
        volume,
        centroid: weighted_centroid / total_area,
    })
}

/// Box-fit fragment descriptors, one per seed's Voronoi cell.
///
/// Each cell is replaced by its local AABB (the fracture layer's
/// `FractureFragmentDesc` is a box), optionally shrunk by `edge_shrink` — a
/// fraction in `[0.0, 0.5)` removed from each side of every half-extent, so
/// adjacent fragments start with a small gap instead of interpenetrating.
/// `template` supplies `initial_velocity`, `density`, `friction`, and
/// `restitution` for every fragment; `local_center`/`half_extents` are
/// computed per cell. Seeds are de-duplicated within a `CELL_EPSILON`
/// (box-diagonal-scaled) tolerance and degenerate cells are skipped; returns
/// `None` on invalid input or when no valid cell remains.
///
/// Note the approximation: the AABB of a non-box cell slightly overlaps its
/// neighbors (unless shrunk); the cell volume itself is available via
/// [`voronoi_cell`] for callers that need exact per-cell mass.
pub fn voronoi_fragments_from_seeds(
    aabb_min: Vec3,
    aabb_max: Vec3,
    seeds: &[Vec3],
    template: FractureFragmentDesc,
    edge_shrink: f64,
) -> Option<Vec<FractureFragmentDesc>> {
    if seeds.is_empty() || seeds.len() > MAX_VORONOI_SEEDS {
        return None;
    }
    if !edge_shrink.is_finite() || !(0.0..0.5).contains(&edge_shrink) {
        return None;
    }
    let lo = v3(aabb_min);
    let hi = v3(aabb_max);
    if lo.x >= hi.x || lo.y >= hi.y || lo.z >= hi.z {
        return None;
    }
    let tol = CELL_EPSILON * (hi - lo).length().max(1.0);

    // De-duplicate seeds so bisectors against (near-)identical points are
    // never generated — a duplicated seed would otherwise yield two
    // coincident cells.
    let mut unique: Vec<Vec3> = Vec::with_capacity(seeds.len());
    for seed in seeds {
        if !finite_vec3(*seed) {
            return None;
        }
        let sv = v3(*seed);
        if !unique.iter().any(|u| (v3(*u) - sv).length() <= tol) {
            unique.push(*seed);
        }
    }

    let scale = 1.0 - 2.0 * edge_shrink;
    let mut fragments = Vec::with_capacity(unique.len());
    for (index, seed) in unique.iter().copied().enumerate() {
        let neighbors: Vec<Vec3> = unique
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(j, s)| if j != index { Some(s) } else { None })
            .collect();
        let Some(cell) = voronoi_cell(seed, &neighbors, aabb_min, aabb_max) else {
            continue;
        };
        // Local AABB of the cell, clamped to the requested box to absorb the
        // vertex tolerance.
        let mut min = cell.vertices[0];
        let mut max = cell.vertices[0];
        for vtx in &cell.vertices[1..] {
            min = Vector3f64::new(min.x.min(vtx.x), min.y.min(vtx.y), min.z.min(vtx.z));
            max = Vector3f64::new(max.x.max(vtx.x), max.y.max(vtx.y), max.z.max(vtx.z));
        }
        let min = Vector3f64::new(min.x.max(lo.x), min.y.max(lo.y), min.z.max(lo.z));
        let max = Vector3f64::new(max.x.min(hi.x), max.y.min(hi.y), max.z.min(hi.z));
        let center = (min + max) * 0.5;
        let half = (max - min) * (0.5 * scale);
        if half.x <= 0.0 || half.y <= 0.0 || half.z <= 0.0 {
            continue;
        }
        fragments.push(FractureFragmentDesc {
            local_center: Vec3 {
                x: center.x,
                y: center.y,
                z: center.z,
            },
            half_extents: Vec3 {
                x: half.x,
                y: half.y,
                z: half.z,
            },
            initial_velocity: template.initial_velocity,
            density: template.density,
            friction: template.friction,
            restitution: template.restitution,
        });
    }
    if fragments.is_empty() {
        None
    } else {
        Some(fragments)
    }
}
