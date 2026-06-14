use std::slice;

use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::prelude::{ColliderBuilder, SharedShape};

use crate::rapier::ffi::{
    ColliderBuilderHandle, Vec3, VoxelColliderMode, VoxelColliderOptions,
    voxel_collider_mode_from_raw,
};

const MAX_VOXEL_CELLS: usize = 262_144;
const MAX_COMPOUND_PARTS: usize = 100_000;
const MAX_SURFACE_VERTICES: usize = 1_000_000;
const MAX_SURFACE_TRIANGLES: usize = 500_000;

struct VoxelGrid<'a> {
    voxels: &'a [u8],
    size_x: usize,
    size_y: usize,
    size_z: usize,
    voxel_size: f64,
    origin: Vec3,
}

impl VoxelGrid<'_> {
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        let plane = self.size_x.checked_mul(self.size_y)?;
        let base = z.checked_mul(plane)?;
        let row = y.checked_mul(self.size_x)?;
        base.checked_add(row)?.checked_add(x)
    }

    fn is_solid(&self, x: usize, y: usize, z: usize) -> bool {
        self.index(x, y, z)
            .and_then(|index| self.voxels.get(index))
            .is_some_and(|voxel| *voxel != 0)
    }

    fn is_solid_checked(&self, x: isize, y: isize, z: isize) -> bool {
        if x < 0
            || y < 0
            || z < 0
            || x as usize >= self.size_x
            || y as usize >= self.size_y
            || z as usize >= self.size_z
        {
            return false;
        }

        self.is_solid(x as usize, y as usize, z as usize)
    }

    fn cell_min(&self, x: usize, y: usize, z: usize) -> Vector {
        Vector::new(
            self.origin.x + x as f64 * self.voxel_size,
            self.origin.y + y as f64 * self.voxel_size,
            self.origin.z + z as f64 * self.voxel_size,
        )
    }
}

fn choose_mode(solid_count: usize, options: VoxelColliderOptions) -> VoxelColliderMode {
    let mode = voxel_collider_mode_from_raw(options.mode);
    if mode != VoxelColliderMode::Auto {
        return mode;
    }
    if solid_count <= options.small_voxel_limit as usize {
        return VoxelColliderMode::Cuboids;
    }
    if options.dynamic_body.0 != 0 {
        return VoxelColliderMode::GreedyCuboids;
    }
    if solid_count >= options.mesh_voxel_limit as usize {
        return VoxelColliderMode::SurfaceMesh;
    }
    VoxelColliderMode::GreedyCuboids
}

fn push_cuboid(
    grid: &VoxelGrid<'_>,
    parts: &mut Vec<(Pose, SharedShape)>,
    x: usize,
    y: usize,
    z: usize,
    max_x: usize,
    max_y: usize,
    max_z: usize,
) {
    let size_x = (max_x - x) as f64 * grid.voxel_size;
    let size_y = (max_y - y) as f64 * grid.voxel_size;
    let size_z = (max_z - z) as f64 * grid.voxel_size;
    if size_x <= 0.0 || size_y <= 0.0 || size_z <= 0.0 {
        return;
    }

    let center = Vector::new(
        grid.origin.x + (x as f64 + (max_x - x) as f64 * 0.5) * grid.voxel_size,
        grid.origin.y + (y as f64 + (max_y - y) as f64 * 0.5) * grid.voxel_size,
        grid.origin.z + (z as f64 + (max_z - z) as f64 * 0.5) * grid.voxel_size,
    );
    parts.push((
        Pose::from_parts(center, Rotation::IDENTITY),
        SharedShape::cuboid(size_x * 0.5, size_y * 0.5, size_z * 0.5),
    ));
}

fn build_cuboids(grid: &VoxelGrid<'_>, solid_count: usize) -> Option<ColliderBuilder> {
    if solid_count > MAX_COMPOUND_PARTS {
        return None;
    }

    let mut parts = Vec::with_capacity(solid_count);
    for z in 0..grid.size_z {
        for y in 0..grid.size_y {
            for x in 0..grid.size_x {
                if grid.is_solid(x, y, z) {
                    push_cuboid(grid, &mut parts, x, y, z, x + 1, y + 1, z + 1);
                    if parts.len() > MAX_COMPOUND_PARTS {
                        return None;
                    }
                }
            }
        }
    }
    (!parts.is_empty()).then(|| ColliderBuilder::compound(parts))
}

fn build_greedy_cuboids(grid: &VoxelGrid<'_>) -> Option<ColliderBuilder> {
    let mut visited = vec![false; grid.voxels.len()];
    let mut parts = Vec::with_capacity(grid.voxels.len().min(1024));

    for z in 0..grid.size_z {
        for y in 0..grid.size_y {
            for x in 0..grid.size_x {
                let Some(start) = grid.index(x, y, z) else {
                    return None;
                };
                if visited[start] || !grid.is_solid(x, y, z) {
                    continue;
                }

                let mut max_x = x + 1;
                while max_x < grid.size_x {
                    let Some(i) = grid.index(max_x, y, z) else {
                        return None;
                    };
                    if visited[i] || !grid.is_solid(max_x, y, z) {
                        break;
                    }
                    max_x += 1;
                }

                let mut max_y = y + 1;
                'expand_y: while max_y < grid.size_y {
                    for xx in x..max_x {
                        let Some(i) = grid.index(xx, max_y, z) else {
                            return None;
                        };
                        if visited[i] || !grid.is_solid(xx, max_y, z) {
                            break 'expand_y;
                        }
                    }
                    max_y += 1;
                }

                let mut max_z = z + 1;
                'expand_z: while max_z < grid.size_z {
                    for yy in y..max_y {
                        for xx in x..max_x {
                            let Some(i) = grid.index(xx, yy, max_z) else {
                                return None;
                            };
                            if visited[i] || !grid.is_solid(xx, yy, max_z) {
                                break 'expand_z;
                            }
                        }
                    }
                    max_z += 1;
                }

                for zz in z..max_z {
                    for yy in y..max_y {
                        for xx in x..max_x {
                            let Some(i) = grid.index(xx, yy, zz) else {
                                return None;
                            };
                            visited[i] = true;
                        }
                    }
                }

                push_cuboid(grid, &mut parts, x, y, z, max_x, max_y, max_z);
                if parts.len() > MAX_COMPOUND_PARTS {
                    return None;
                }
            }
        }
    }

    (!parts.is_empty()).then(|| ColliderBuilder::compound(parts))
}

fn push_face(
    vertices: &mut Vec<Vector>,
    indices: &mut Vec<[u32; 3]>,
    corners: [Vector; 4],
) -> Option<()> {
    if vertices.len() + 4 > MAX_SURFACE_VERTICES || indices.len() + 2 > MAX_SURFACE_TRIANGLES {
        return None;
    }

    let base = u32::try_from(vertices.len()).ok()?;
    vertices.extend(corners);
    indices.push([base, base + 1, base + 2]);
    indices.push([base, base + 2, base + 3]);
    Some(())
}

fn build_surface_mesh(grid: &VoxelGrid<'_>, solid_count: usize) -> Option<ColliderBuilder> {
    let face_capacity = solid_count.saturating_mul(6);
    let mut vertices = Vec::with_capacity(face_capacity.saturating_mul(4).min(65_536));
    let mut indices = Vec::with_capacity(face_capacity.saturating_mul(2).min(32_768));
    let s = grid.voxel_size;

    for z in 0..grid.size_z {
        for y in 0..grid.size_y {
            for x in 0..grid.size_x {
                if !grid.is_solid(x, y, z) {
                    continue;
                }

                let min = grid.cell_min(x, y, z);
                let max = min + Vector::new(s, s, s);
                let x = x as isize;
                let y = y as isize;
                let z = z as isize;

                if !grid.is_solid_checked(x - 1, y, z) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(min.x, min.y, min.z),
                            Vector::new(min.x, min.y, max.z),
                            Vector::new(min.x, max.y, max.z),
                            Vector::new(min.x, max.y, min.z),
                        ],
                    )?;
                }
                if !grid.is_solid_checked(x + 1, y, z) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(max.x, min.y, min.z),
                            Vector::new(max.x, max.y, min.z),
                            Vector::new(max.x, max.y, max.z),
                            Vector::new(max.x, min.y, max.z),
                        ],
                    )?;
                }
                if !grid.is_solid_checked(x, y - 1, z) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(min.x, min.y, min.z),
                            Vector::new(max.x, min.y, min.z),
                            Vector::new(max.x, min.y, max.z),
                            Vector::new(min.x, min.y, max.z),
                        ],
                    )?;
                }
                if !grid.is_solid_checked(x, y + 1, z) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(min.x, max.y, min.z),
                            Vector::new(min.x, max.y, max.z),
                            Vector::new(max.x, max.y, max.z),
                            Vector::new(max.x, max.y, min.z),
                        ],
                    )?;
                }
                if !grid.is_solid_checked(x, y, z - 1) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(min.x, min.y, min.z),
                            Vector::new(min.x, max.y, min.z),
                            Vector::new(max.x, max.y, min.z),
                            Vector::new(max.x, min.y, min.z),
                        ],
                    )?;
                }
                if !grid.is_solid_checked(x, y, z + 1) {
                    push_face(
                        &mut vertices,
                        &mut indices,
                        [
                            Vector::new(min.x, min.y, max.z),
                            Vector::new(max.x, min.y, max.z),
                            Vector::new(max.x, max.y, max.z),
                            Vector::new(min.x, max.y, max.z),
                        ],
                    )?;
                }
            }
        }
    }

    if vertices.is_empty() {
        return None;
    }

    ColliderBuilder::trimesh(vertices, indices).ok()
}

fn build_voxel_collider(
    grid: &VoxelGrid<'_>,
    options: VoxelColliderOptions,
) -> Option<ColliderBuilder> {
    let solid_count = grid.voxels.iter().filter(|voxel| **voxel != 0).count();
    if solid_count == 0 {
        return None;
    }

    match choose_mode(solid_count, options) {
        VoxelColliderMode::Auto => unreachable!(),
        VoxelColliderMode::Cuboids => build_cuboids(grid, solid_count),
        VoxelColliderMode::GreedyCuboids => build_greedy_cuboids(grid),
        VoxelColliderMode::SurfaceMesh => build_surface_mesh(grid, solid_count),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn collider_builder_create_voxels(
    voxels: *const u8,
    size_x: u32,
    size_y: u32,
    size_z: u32,
    voxel_size: f64,
    origin: Vec3,
    options: VoxelColliderOptions,
) -> *mut ColliderBuilderHandle {
    if voxels.is_null()
        || size_x == 0
        || size_y == 0
        || size_z == 0
        || !voxel_size.is_finite()
        || voxel_size <= 0.0
        || !origin.x.is_finite()
        || !origin.y.is_finite()
        || !origin.z.is_finite()
    {
        return std::ptr::null_mut();
    }

    let Some(xy) = (size_x as usize).checked_mul(size_y as usize) else {
        return std::ptr::null_mut();
    };
    let Some(len) = xy.checked_mul(size_z as usize) else {
        return std::ptr::null_mut();
    };
    if len > MAX_VOXEL_CELLS {
        return std::ptr::null_mut();
    }

    let voxels = unsafe { slice::from_raw_parts(voxels, len) };
    let grid = VoxelGrid {
        voxels,
        size_x: size_x as usize,
        size_y: size_y as usize,
        size_z: size_z as usize,
        voxel_size,
        origin,
    };

    let Some(builder) = build_voxel_collider(&grid, options) else {
        return std::ptr::null_mut();
    };

    Box::into_raw(Box::new(ColliderBuilderHandle { inner: builder }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rapier::ffi::Bool;

    fn options(mode: VoxelColliderMode) -> VoxelColliderOptions {
        VoxelColliderOptions {
            mode: mode as u32,
            dynamic_body: Bool::FALSE,
            small_voxel_limit: 128,
            mesh_voxel_limit: 20_000,
        }
    }

    #[test]
    fn empty_voxels_build_no_collider() {
        let grid = VoxelGrid {
            voxels: &[0; 8],
            size_x: 2,
            size_y: 2,
            size_z: 2,
            voxel_size: 1.0,
            origin: Vec3::default(),
        };

        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::Auto)).is_none());
    }

    #[test]
    fn solid_voxels_build_with_each_mode() {
        let voxels = [1; 8];
        let grid = VoxelGrid {
            voxels: &voxels,
            size_x: 2,
            size_y: 2,
            size_z: 2,
            voxel_size: 1.0,
            origin: Vec3::default(),
        };

        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::Cuboids)).is_some());
        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::GreedyCuboids)).is_some());
        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::SurfaceMesh)).is_some());
    }
}
