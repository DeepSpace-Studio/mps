#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::{collider_builder_build, world_insert_collider};
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::ffi::{Bool, Quat};
    use mps_core::rapier::voxel::*;
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

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
            voxel_size_x: 1.0,
            voxel_size_y: 1.0,
            voxel_size_z: 1.0,
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
            voxel_size_x: 1.0,
            voxel_size_y: 1.0,
            voxel_size_z: 1.0,
            origin: Vec3::default(),
        };

        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::Cuboids)).is_some());
        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::GreedyCuboids)).is_some());
        assert!(build_voxel_collider(&grid, options(VoxelColliderMode::SurfaceMesh)).is_some());
    }

    #[test]
    fn voxel_aabb_and_obb_build() {
        let aabb = AabbDesc {
            mins: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            maxs: Vec3 {
                x: 2.0,
                y: 1.0,
                z: 1.0,
            },
        };
        let aabb_builder = collider_builder_create_voxel_aabb(
            aabb,
            0.5,
            0.5,
            0.5,
            options(VoxelColliderMode::Auto),
        );
        assert!(!aabb_builder.is_null());
        mps_core::rapier::collider::collider_builder_destroy(aabb_builder);

        let obb = Obb {
            center: Vec3::default(),
            half_extents: Vec3 {
                x: 1.0,
                y: 0.5,
                z: 0.5,
            },
            rotation: Quat {
                i: 0.0,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
        };
        let obb_builder =
            collider_builder_create_voxel_obb(obb, 0.5, 0.5, 0.5, options(VoxelColliderMode::Auto));
        assert!(!obb_builder.is_null());
        mps_core::rapier::collider::collider_builder_destroy(obb_builder);
    }

    #[test]
    fn ray_pick_resolves_voxel_cell() {
        // 2x2x2 solid voxel grid at origin, unit voxel size.
        let data = [1u8; 8];
        let builder = collider_builder_create_voxels(
            data.as_ptr(),
            2,
            2,
            2,
            1.0,
            1.0,
            1.0,
            Vec3::default(),
            options(VoxelColliderMode::Cuboids),
        );
        assert!(!builder.is_null());
        let collider = collider_builder_build(builder);
        // NOTE: `collider_builder_build` consumes (Box::from_raw) the builder
        // pointer, so it must NOT be passed to `collider_builder_destroy` after.
        assert!(!collider.is_null());

        let world = world_create(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let handle = world_insert_collider(world, collider);
        assert_ne!(handle, 0);
        // Advance one step so the QueryPipeline (broad_phase) is rebuilt and
        // ray/shape queries can see the freshly inserted collider.
        world_step(world, 1.0 / 60.0);

        let mut out = VoxelCoord::default();
        // Ray from (0.5, 5, 0.5) straight down hits the top face (y=2);
        // the epsilon nudge resolves it to cell (0,1,0) with normal +Y.
        let ok = collider_voxel_ray_pick(
            world,
            handle,
            Vec3 {
                x: 0.5,
                y: 5.0,
                z: 0.5,
            },
            Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            10.0,
            Bool::TRUE,
            &mut out,
        );
        assert!(ok.0 != 0, "ray pick should hit the voxel collider");
        assert!(out.found.0 != 0, "hit cell should be reported as found");
        assert_eq!(
            (out.ix, out.iy, out.iz),
            (0, 1, 0),
            "ray should resolve to the top voxel cell"
        );
        assert!(
            (out.nx, out.ny, out.nz) == (0.0, 1.0, 0.0),
            "surface normal should point +Y"
        );

        world_destroy(world);
    }
}
