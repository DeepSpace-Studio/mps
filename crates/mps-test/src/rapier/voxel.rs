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

    #[test]
    fn cell_at_point_resolves_voxel_cell() {
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
        // pointer, so it must not be passed to `collider_builder_destroy`.
        assert!(!collider.is_null());
        let world = world_create(Vec3::default());
        assert!(!world.is_null());
        let handle = world_insert_collider(world, collider);
        assert_ne!(handle, 0);

        // A point just inside the centre of the top-front-right cell lives in
        // the grid; the voxel collider occupies [0,2]^3.
        let mut out = VoxelCoord::default();
        let ok = collider_voxel_cell_at_point(
            world,
            handle,
            Vec3 {
                x: 1.5,
                y: 0.5,
                z: 1.5,
            },
            &mut out,
        );
        assert!(ok.0 != 0, "point inside the grid should resolve a cell");
        assert!(out.found.0 != 0, "cell should be reported as found");
        assert_eq!(
            (out.ix, out.iy, out.iz),
            (1, 0, 1),
            "point (1.5, 0.5, 1.5) sits in cell (1, 0, 1)"
        );

        // A point outside the grid bounds resolves to nothing.
        let mut miss = VoxelCoord::default();
        let ok_miss = collider_voxel_cell_at_point(
            world,
            handle,
            Vec3 {
                x: 5.0,
                y: 5.0,
                z: 5.0,
            },
            &mut miss,
        );
        assert!(ok_miss.0 == 0, "point outside the grid should not resolve");
        assert!(miss.found.0 == 0, "missed cell should report not found");

        world_destroy(world);
    }

    #[test]
    fn get_reads_voxel_cell_solidity() {
        // 2x2x2 grid, unit voxel size, origin at 0. Fill only cells touching
        // the y==0 plane (indices 0..3), leave the top plane (4..7) empty so we
        // can assert BOTH solid and empty reads from the same collider.
        let mut data = [0u8; 8];
        data[..4].fill(1);
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
        assert!(!collider.is_null());
        let world = world_create(Vec3::default());
        assert!(!world.is_null());
        let handle = world_insert_collider(world, collider);
        assert_ne!(handle, 0);
        // Ray/shape queries read the QueryPipeline rebuilt by step.
        world_step(world, 1.0 / 60.0);

        // Solid cell (bottom-front-left, index 0).
        let mut solid_out: u8 = 0;
        let ok = collider_voxel_read_cell(world, handle, 0, 0, 0, &mut solid_out);
        assert!(ok.0 != 0, "get on a valid voxel collider should succeed");
        assert!(solid_out != 0, "cell (0,0,0) was filled -> solid");

        // Empty cell (top-front-left, index 4).
        let mut empty_out: u8 = 1;
        let ok_empty = collider_voxel_read_cell(world, handle, 0, 1, 0, &mut empty_out);
        assert!(
            ok_empty.0 != 0,
            "get on a valid voxel collider should succeed"
        );
        assert!(empty_out == 0, "cell (0,1,0) was left empty -> not solid");

        // Out-of-range coordinate: returns FALSE and writes 0 to the out ptr.
        let mut oob: u8 = 1;
        let ok_oob = collider_voxel_read_cell(world, handle, 5, 0, 0, &mut oob);
        assert!(ok_oob.0 == 0, "out-of-range cell should fail");
        assert!(oob == 0, "out ptr must be zeroed on out-of-range");

        // Non-voxel collider: returns FALSE.
        let mut nv: u8 = 1;
        let ok_nv = collider_voxel_read_cell(world, 0xDEAD_BEEF, 0, 0, 0, &mut nv);
        assert!(ok_nv.0 == 0, "non-voxel collider should fail");
        assert!(nv == 0, "out ptr must be zeroed on non-voxel collider");

        world_destroy(world);
    }
}
