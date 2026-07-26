#[cfg(test)]
mod tests {
    use mps_core::rapier::collider::*;
    use mps_core::rapier::error::{
        ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK, last_error_code,
    };
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy};

    fn aabb(min: f64, max: f64) -> AabbDesc {
        AabbDesc {
            mins: Vec3 {
                x: min,
                y: min,
                z: min,
            },
            maxs: Vec3 {
                x: max,
                y: max,
                z: max,
            },
        }
    }

    fn assert_builder(builder: *mut ColliderBuilderHandle) {
        assert!(!builder.is_null());
        collider_builder_destroy(builder);
    }

    #[test]
    fn convex_hull_builder_accepts_cube_points() {
        let points = [
            -1.0, -1.0, -1.0, //
            -1.0, -1.0, 1.0, //
            -1.0, 1.0, -1.0, //
            -1.0, 1.0, 1.0, //
            1.0, -1.0, -1.0, //
            1.0, -1.0, 1.0, //
            1.0, 1.0, -1.0, //
            1.0, 1.0, 1.0,
        ];

        assert_builder(collider_builder_create_convex_hull(points.as_ptr(), 8));
    }

    #[test]
    fn point_cloud_bounds_builder_accepts_points() {
        let points = [
            -2.0, 1.0, 0.5, //
            3.0, -4.0, 2.0, //
            1.0, 2.0, -6.0,
        ];

        assert_builder(collider_builder_create_point_cloud_bounds(
            points.as_ptr(),
            3,
        ));
    }

    #[test]
    fn broad_volume_builders_accept_valid_inputs() {
        let points = [
            -2.0, 1.0, 0.5, //
            3.0, -4.0, 2.0, //
            1.0, 2.0, -6.0, //
            0.0, 0.0, 0.0,
        ];
        let vertices = [
            0.0, 0.0, 0.0, //
            1.0, 0.0, 0.0, //
            1.0, 1.0, 0.0,
        ];
        let edges = [0u32, 1, 1, 2];
        let spheres = [
            0.0, 0.0, 0.0, 0.5, //
            1.0, 0.0, 0.0, 0.25,
        ];

        assert_builder(collider_builder_create_double_bv(
            aabb(0.0, 1.0),
            aabb(2.0, 3.0),
        ));
        assert_builder(collider_builder_create_skewed_obb(
            Vec3::default(),
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.25,
                y: 1.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ));
        assert_builder(collider_builder_create_discrete_obb(points.as_ptr(), 4, 1));
        assert_builder(collider_builder_create_fused_collapsing_bounds(
            points.as_ptr(),
            4,
            0.1,
        ));
        assert_builder(collider_builder_create_edge_bvh(
            vertices.as_ptr(),
            3,
            edges.as_ptr(),
            2,
            0.05,
        ));
        assert_builder(collider_builder_create_medial_spheres(spheres.as_ptr(), 2));
    }

    // ---- shared helpers ----

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn identity_quat() -> Quat {
        Quat {
            i: 0.0,
            j: 0.0,
            k: 0.0,
            w: 1.0,
        }
    }

    fn make_world() -> *mut WorldHandle {
        world_create(v3(0.0, -9.81, 0.0))
    }

    fn ball_desc() -> ShapeDesc {
        ShapeDesc {
            shape_type: 0,
            a: 0.5,
            b: 0.0,
            c: 0.0,
            d: 0.0,
        }
    }

    fn make_ball_builder() -> *mut ColliderBuilderHandle {
        let builder = collider_builder_create_ex(ball_desc());
        assert!(!builder.is_null());
        builder
    }

    fn insert_ball(world: *mut WorldHandle) -> ColliderHandleRaw {
        let collider = collider_builder_build(make_ball_builder());
        assert!(!collider.is_null());
        let handle = world_insert_collider(world, collider);
        assert_ne!(handle, 0);
        handle
    }

    // ---- shape builders: happy paths ----

    #[test]
    fn builder_create_accepts_all_shape_types() {
        let cases = [
            (0, v3(0.5, 0.0, 0.0)),   // ball: radius
            (1, v3(0.5, 0.5, 0.5)),   // cuboid: half extents
            (2, v3(0.5, 0.25, 0.0)),  // capsule_y
            (3, v3(0.5, 0.25, 0.0)),  // capsule_x
            (4, v3(0.5, 0.25, 0.0)),  // capsule_z
            (5, v3(0.5, 0.25, 0.0)),  // cylinder
            (6, v3(0.5, 0.25, 0.05)), // round cylinder
            (7, v3(0.5, 0.25, 0.0)),  // cone
            (8, v3(0.5, 0.25, 0.05)), // round cone
            (9, v3(0.5, 0.5, 0.5)),   // round cuboid (border radius 0)
        ];
        for (shape_type, data) in cases {
            let builder = collider_builder_create(shape_type, data);
            assert!(!builder.is_null(), "shape_type {shape_type}");
            collider_builder_destroy(builder);
        }
    }

    #[test]
    fn builder_create_rejects_invalid_dimensions() {
        assert!(collider_builder_create(0, v3(-1.0, 0.0, 0.0)).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert!(collider_builder_create(1, v3(f64::NAN, 1.0, 1.0)).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn builder_create_ex_accepts_round_cuboid_with_border() {
        let desc = ShapeDesc {
            shape_type: 9,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            d: 0.1,
        };
        assert_builder(collider_builder_create_ex(desc));
    }

    #[test]
    fn builder_create_ex_rejects_negative_border_radius() {
        let desc = ShapeDesc {
            shape_type: 9,
            a: 1.0,
            b: 1.0,
            c: 1.0,
            d: -0.1,
        };
        assert!(collider_builder_create_ex(desc).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn halfspace_builder_accepts_finite_normal() {
        assert_builder(collider_builder_create_halfspace(v3(0.0, 1.0, 0.0)));
    }

    #[test]
    fn halfspace_builder_rejects_non_finite_normal() {
        assert!(collider_builder_create_halfspace(v3(f64::NAN, 0.0, 0.0)).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn obb_builder_accepts_valid_obb() {
        let obb = Obb {
            center: v3(1.0, 2.0, 3.0),
            half_extents: v3(0.5, 0.5, 0.5),
            rotation: identity_quat(),
        };
        assert_builder(collider_builder_create_obb(obb));
    }

    #[test]
    fn obb_builder_rejects_invalid_obb() {
        let degenerate = Obb {
            center: Vec3::default(),
            half_extents: v3(0.0, 1.0, 1.0),
            rotation: identity_quat(),
        };
        assert!(collider_builder_create_obb(degenerate).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_rotation = Obb {
            center: Vec3::default(),
            half_extents: v3(1.0, 1.0, 1.0),
            rotation: Quat {
                i: f64::NAN,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
        };
        assert!(collider_builder_create_obb(nan_rotation).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn sphere_builder_accepts_valid_sphere() {
        let sphere = Sphere {
            center: v3(1.0, 2.0, 3.0),
            radius: 0.75,
        };
        assert_builder(collider_builder_create_sphere(sphere));
    }

    #[test]
    fn sphere_builder_rejects_invalid_sphere() {
        let zero_radius = Sphere {
            center: Vec3::default(),
            radius: 0.0,
        };
        assert!(collider_builder_create_sphere(zero_radius).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_center = Sphere {
            center: v3(f64::NAN, 0.0, 0.0),
            radius: 1.0,
        };
        assert!(collider_builder_create_sphere(nan_center).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn heightmap_builder_accepts_valid_grid() {
        let data = [0.0f64, 0.1, 0.2, 0.3];
        assert_builder(collider_builder_create_heightmap(
            data.as_ptr(),
            2,
            2,
            v3(1.0, 1.0, 1.0),
        ));
    }

    #[test]
    fn heightmap_builder_rejects_invalid_input() {
        let data = [0.0f64; 4];
        assert!(
            collider_builder_create_heightmap(std::ptr::null(), 2, 2, v3(1.0, 1.0, 1.0)).is_null()
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(
            collider_builder_create_heightmap(data.as_ptr(), 0, 2, v3(1.0, 1.0, 1.0)).is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_data = [0.0f64, f64::NAN, 0.0, 0.0];
        assert!(
            collider_builder_create_heightmap(nan_data.as_ptr(), 2, 2, v3(1.0, 1.0, 1.0)).is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    // ---- point-buffer builders: negative cases ----

    #[test]
    fn convex_hull_builder_rejects_invalid_input() {
        let cube = [
            -1.0, -1.0, -1.0, //
            -1.0, -1.0, 1.0, //
            -1.0, 1.0, -1.0, //
            -1.0, 1.0, 1.0, //
            1.0, -1.0, -1.0, //
            1.0, -1.0, 1.0, //
            1.0, 1.0, -1.0, //
            1.0, 1.0, 1.0,
        ];

        assert!(collider_builder_create_convex_hull(std::ptr::null(), 8).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(collider_builder_create_convex_hull(cube.as_ptr(), 0).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // A hull needs at least 4 points.
        assert!(collider_builder_create_convex_hull(cube.as_ptr(), 3).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_points = [
            0.0,
            0.0,
            0.0, //
            f64::NAN,
            1.0,
            0.0, //
            1.0,
            0.0,
            1.0, //
            0.0,
            1.0,
            1.0,
        ];
        assert!(collider_builder_create_convex_hull(nan_points.as_ptr(), 4).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn point_buffer_builders_reject_null_points() {
        assert!(collider_builder_create_point_cloud_bounds(std::ptr::null(), 1).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(collider_builder_create_discrete_obb(std::ptr::null(), 1, 0).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(
            collider_builder_create_fused_collapsing_bounds(std::ptr::null(), 1, 0.0).is_null()
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn fused_collapsing_bounds_rejects_negative_padding() {
        let points = [0.0f64, 0.0, 0.0];
        assert!(
            collider_builder_create_fused_collapsing_bounds(points.as_ptr(), 1, -0.5).is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn double_bv_rejects_inverted_aabb() {
        assert!(collider_builder_create_double_bv(aabb(1.0, 0.0), aabb(0.0, 1.0)).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn skewed_obb_rejects_zero_axis() {
        let builder = collider_builder_create_skewed_obb(
            Vec3::default(),
            v3(0.0, 0.0, 0.0),
            v3(0.0, 1.0, 0.0),
            v3(0.0, 0.0, 1.0),
        );
        assert!(builder.is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn edge_bvh_rejects_invalid_input() {
        let vertices = [
            0.0, 0.0, 0.0, //
            1.0, 0.0, 0.0, //
            1.0, 1.0, 0.0,
        ];
        let edges = [0u32, 1];

        assert!(
            collider_builder_create_edge_bvh(vertices.as_ptr(), 3, std::ptr::null(), 1, 0.05)
                .is_null()
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(
            collider_builder_create_edge_bvh(vertices.as_ptr(), 3, edges.as_ptr(), 0, 0.05)
                .is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert!(
            collider_builder_create_edge_bvh(vertices.as_ptr(), 3, edges.as_ptr(), 1, 0.0)
                .is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let out_of_range = [0u32, 7];
        assert!(
            collider_builder_create_edge_bvh(vertices.as_ptr(), 3, out_of_range.as_ptr(), 1, 0.05)
                .is_null()
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    #[test]
    fn medial_spheres_rejects_invalid_input() {
        let spheres = [0.0f64, 0.0, 0.0, 0.5];

        assert!(collider_builder_create_medial_spheres(std::ptr::null(), 1).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(collider_builder_create_medial_spheres(spheres.as_ptr(), 0).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let negative_radius = [0.0f64, 0.0, 0.0, -1.0];
        assert!(collider_builder_create_medial_spheres(negative_radius.as_ptr(), 1).is_null());
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
    }

    // ---- build / destroy ----

    #[test]
    fn builder_build_produces_destroyable_collider() {
        let collider = collider_builder_build(make_ball_builder());
        assert!(!collider.is_null());
        collider_destroy_raw(collider);
    }

    #[test]
    fn null_raw_pointers_report_null_pointer() {
        assert!(collider_builder_build(std::ptr::null_mut()).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        collider_builder_destroy(std::ptr::null_mut());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        collider_destroy_raw(std::ptr::null_mut());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    // ---- builder setters ----

    #[test]
    fn builder_setters_accept_valid_values() {
        let builder = make_ball_builder();
        let groups = InteractionGroupsDesc {
            memberships: 0b1,
            filter: 0b10,
        };
        collider_builder_set_translation(builder, v3(1.0, 2.0, 3.0));
        collider_builder_set_rotation(builder, v3(0.0, 0.5, 0.0));
        collider_builder_set_pose(builder, v3(1.0, 0.0, 0.0), identity_quat());
        collider_builder_set_sensor(builder, Bool::TRUE);
        collider_builder_set_friction(builder, 0.8);
        collider_builder_set_restitution(builder, 0.2);
        collider_builder_set_density(builder, 2.5);
        collider_builder_set_collision_groups(builder, groups);
        collider_builder_set_solver_groups(builder, groups);
        collider_builder_set_active_events(builder, 1);
        collider_builder_set_active_hooks(builder, 1);
        collider_builder_set_contact_force_event_threshold(builder, 10.0);
        assert_eq!(last_error_code(), ERR_OK);

        let collider = collider_builder_build(builder);
        assert!(!collider.is_null());
        collider_destroy_raw(collider);
    }

    #[test]
    fn builder_setters_reject_null_builder() {
        let groups = InteractionGroupsDesc::default();
        let null = std::ptr::null_mut();

        collider_builder_set_translation(null, Vec3::default());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_rotation(null, Vec3::default());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_pose(null, Vec3::default(), identity_quat());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_sensor(null, Bool::TRUE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_friction(null, 0.5);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_restitution(null, 0.5);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_density(null, 1.0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_collision_groups(null, groups);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_solver_groups(null, groups);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_active_events(null, 1);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_active_hooks(null, 1);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_builder_set_contact_force_event_threshold(null, 1.0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn builder_setters_reject_invalid_values() {
        let builder = make_ball_builder();
        collider_builder_set_friction(builder, -1.0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_friction(builder, f64::NAN);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_restitution(builder, -0.5);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_density(builder, -1.0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_contact_force_event_threshold(builder, -1.0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_translation(builder, v3(f64::NAN, 0.0, 0.0));
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_rotation(builder, v3(f64::NAN, 0.0, 0.0));
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_set_pose(
            builder,
            Vec3::default(),
            Quat {
                i: f64::NAN,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        collider_builder_destroy(builder);
    }

    // ---- world insert / remove / copy ----

    #[test]
    fn insert_and_remove_collider_round_trip() {
        let world = make_world();
        let handle = insert_ball(world);

        assert_eq!(world_remove_collider(world, handle, Bool::TRUE), Bool::TRUE);
        assert_eq!(last_error_code(), ERR_OK);

        // Removing the same handle again reports not found.
        assert_eq!(
            world_remove_collider(world, handle, Bool::FALSE),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn insert_collider_rejects_null_arguments() {
        let world = make_world();
        let collider = collider_builder_build(make_ball_builder());

        assert_eq!(world_insert_collider(std::ptr::null_mut(), collider), 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        // The failed insert did not consume the collider.
        collider_destroy_raw(collider);

        assert_eq!(world_insert_collider(world, std::ptr::null_mut()), 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        world_destroy(world);
    }

    #[test]
    fn insert_collider_with_parent_attaches_to_body() {
        let world = make_world();
        let body_builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
        assert!(!body_builder.is_null());
        let body = rigid_body_builder_build(body_builder);
        assert!(!body.is_null());
        let body_handle = world_insert_rigid_body(world, body);
        assert_ne!(body_handle, 0);

        let collider = collider_builder_build(make_ball_builder());
        let handle = world_insert_collider_with_parent(world, collider, body_handle);
        assert_ne!(handle, 0);
        world_destroy(world);
    }

    #[test]
    fn insert_collider_with_parent_rejects_null_arguments() {
        let world = make_world();
        let collider = collider_builder_build(make_ball_builder());

        assert_eq!(
            world_insert_collider_with_parent(std::ptr::null_mut(), collider, 1),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_destroy_raw(collider);

        assert_eq!(
            world_insert_collider_with_parent(world, std::ptr::null_mut(), 1),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        world_destroy(world);
    }

    #[test]
    fn copy_collider_round_trip() {
        let world = make_world();
        let handle = insert_ball(world);

        let copy = world_copy_collider(world, handle);
        assert!(!copy.is_null());
        collider_destroy_raw(copy);
        world_destroy(world);
    }

    #[test]
    fn copy_collider_rejects_invalid_arguments() {
        let world = make_world();

        assert!(world_copy_collider(std::ptr::null_mut(), 1).is_null());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        assert!(world_copy_collider(world, 12345).is_null());
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn remove_collider_flag_matches_bool_variant() {
        let world = make_world();
        let handle = insert_ball(world);

        assert_eq!(world_remove_collider_flag(world, handle, Bool::TRUE), 1);
        assert_eq!(world_remove_collider_flag(world, handle, Bool::FALSE), 0);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn remove_collider_rejects_null_world() {
        assert_eq!(
            world_remove_collider(std::ptr::null_mut(), 1, Bool::FALSE),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    // ---- getters ----

    #[test]
    fn translation_and_rotation_round_trip() {
        let world = make_world();
        let builder = make_ball_builder();
        collider_builder_set_translation(builder, v3(1.0, 2.0, 3.0));
        let collider = collider_builder_build(builder);
        let handle = world_insert_collider(world, collider);
        assert_ne!(handle, 0);

        let t = collider_get_translation(world, handle);
        assert_eq!((t.x, t.y, t.z), (1.0, 2.0, 3.0));

        let mut out = Vec3::default();
        collider_get_translation_out(world, handle, &mut out);
        assert_eq!((out.x, out.y, out.z), (1.0, 2.0, 3.0));

        let r = collider_get_rotation(world, handle);
        assert!((r.w - 1.0).abs() < 1.0e-9);

        let mut out_quat = Quat::default();
        collider_get_rotation_out(world, handle, &mut out_quat);
        assert!((out_quat.w - 1.0).abs() < 1.0e-9);
        world_destroy(world);
    }

    #[test]
    fn getters_reject_null_world_and_unknown_handle() {
        let world = make_world();
        let handle = insert_ball(world);

        collider_get_translation(std::ptr::null(), handle);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_get_translation(world, 12345);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);

        collider_get_rotation(std::ptr::null(), handle);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_get_rotation(world, 12345);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);

        assert_eq!(collider_get_shape_count(std::ptr::null(), handle), 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_get_shape_count(world, 12345), 0);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);

        assert_eq!(collider_get_density(std::ptr::null(), handle), 0.0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_get_density(world, 12345), 0.0);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn out_getters_reject_null_output() {
        let world = make_world();
        let handle = insert_ball(world);

        collider_get_translation_out(world, handle, std::ptr::null_mut());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        collider_get_rotation_out(world, handle, std::ptr::null_mut());
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        world_destroy(world);
    }

    #[test]
    fn shape_count_reflects_compound_parts() {
        let world = make_world();
        let ball = insert_ball(world);
        assert_eq!(collider_get_shape_count(world, ball), 1);

        let spheres = [
            0.0, 0.0, 0.0, 0.5, //
            1.0, 0.0, 0.0, 0.25,
        ];
        let builder = collider_builder_create_medial_spheres(spheres.as_ptr(), 2);
        assert!(!builder.is_null());
        let collider = collider_builder_build(builder);
        let compound = world_insert_collider(world, collider);
        assert_eq!(collider_get_shape_count(world, compound), 2);
        world_destroy(world);
    }

    #[test]
    fn density_reflects_builder_value() {
        let world = make_world();
        let builder = make_ball_builder();
        collider_builder_set_density(builder, 2.5);
        let collider = collider_builder_build(builder);
        let handle = world_insert_collider(world, collider);

        assert_eq!(collider_get_density(world, handle), 2.5);
        world_destroy(world);
    }

    // ---- world-side setters ----

    #[test]
    fn pose_setters_move_collider() {
        let world = make_world();
        let handle = insert_ball(world);

        assert_eq!(
            collider_set_translation(world, handle, v3(4.0, 5.0, 6.0)),
            Bool::TRUE
        );
        let t = collider_get_translation(world, handle);
        assert_eq!((t.x, t.y, t.z), (4.0, 5.0, 6.0));

        assert_eq!(
            collider_set_rotation(world, handle, identity_quat()),
            Bool::TRUE
        );

        assert_eq!(
            collider_set_pose(world, handle, v3(7.0, 8.0, 9.0), identity_quat()),
            Bool::TRUE
        );
        let t = collider_get_translation(world, handle);
        assert_eq!((t.x, t.y, t.z), (7.0, 8.0, 9.0));

        assert_eq!(
            collider_set_pose_flag(world, handle, Vec3::default(), identity_quat()),
            1
        );
        world_destroy(world);
    }

    #[test]
    fn material_and_group_setters_return_true() {
        let world = make_world();
        let handle = insert_ball(world);
        let groups = InteractionGroupsDesc {
            memberships: 0b1,
            filter: 0b10,
        };

        assert_eq!(collider_set_sensor(world, handle, Bool::TRUE), Bool::TRUE);
        assert_eq!(collider_set_sensor_flag(world, handle, Bool::FALSE), 1);
        assert_eq!(collider_set_friction(world, handle, 0.9), Bool::TRUE);
        assert_eq!(collider_set_friction_flag(world, handle, 0.5), 1);
        assert_eq!(collider_set_restitution(world, handle, 0.3), Bool::TRUE);
        assert_eq!(collider_set_restitution_flag(world, handle, 0.1), 1);
        assert_eq!(
            collider_set_collision_groups(world, handle, groups),
            Bool::TRUE
        );
        assert_eq!(collider_set_collision_groups_flag(world, handle, groups), 1);
        assert_eq!(
            collider_set_solver_groups(world, handle, groups),
            Bool::TRUE
        );
        assert_eq!(collider_set_solver_groups_flag(world, handle, groups), 1);
        assert_eq!(collider_set_active_events(world, handle, 1), Bool::TRUE);
        assert_eq!(collider_set_active_events_flag(world, handle, 1), 1);
        assert_eq!(collider_set_active_hooks(world, handle, 1), Bool::TRUE);
        assert_eq!(collider_set_active_hooks_flag(world, handle, 1), 1);
        assert_eq!(
            collider_set_contact_force_event_threshold(world, handle, 5.0),
            Bool::TRUE
        );
        assert_eq!(
            collider_set_contact_force_event_threshold_flag(world, handle, 5.0),
            1
        );
        assert_eq!(last_error_code(), ERR_OK);
        world_destroy(world);
    }

    #[test]
    fn world_setters_reject_null_world() {
        let groups = InteractionGroupsDesc::default();
        let null = std::ptr::null_mut();

        assert_eq!(
            collider_set_translation(null, 1, Vec3::default()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_rotation(null, 1, identity_quat()), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            collider_set_pose(null, 1, Vec3::default(), identity_quat()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_sensor(null, 1, Bool::TRUE), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_friction(null, 1, 0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_restitution(null, 1, 0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_collision_groups(null, 1, groups), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_solver_groups(null, 1, groups), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_active_events(null, 1, 1), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(collider_set_active_hooks(null, 1, 1), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            collider_set_contact_force_event_threshold(null, 1, 1.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
    }

    #[test]
    fn world_setters_reject_unknown_handle() {
        let world = make_world();
        let groups = InteractionGroupsDesc::default();

        assert_eq!(
            collider_set_translation(world, 12345, Vec3::default()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(
            collider_set_rotation(world, 12345, identity_quat()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(
            collider_set_pose(world, 12345, Vec3::default(), identity_quat()),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(collider_set_sensor(world, 12345, Bool::TRUE), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(collider_set_friction(world, 12345, 0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(collider_set_restitution(world, 12345, 0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(
            collider_set_collision_groups(world, 12345, groups),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(
            collider_set_solver_groups(world, 12345, groups),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(collider_set_active_events(world, 12345, 1), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(collider_set_active_hooks(world, 12345, 1), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        assert_eq!(
            collider_set_contact_force_event_threshold(world, 12345, 1.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_NOT_FOUND);
        world_destroy(world);
    }

    #[test]
    fn world_setters_reject_invalid_values() {
        let world = make_world();
        let handle = insert_ball(world);

        assert_eq!(collider_set_friction(world, handle, -1.0), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(collider_set_friction(world, handle, f64::NAN), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(collider_set_restitution(world, handle, -0.5), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(
            collider_set_contact_force_event_threshold(world, handle, -1.0),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(
            collider_set_translation(world, handle, v3(f64::NAN, 0.0, 0.0)),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_quat = Quat {
            i: f64::NAN,
            j: 0.0,
            k: 0.0,
            w: 1.0,
        };
        assert_eq!(collider_set_rotation(world, handle, nan_quat), Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        assert_eq!(
            collider_set_pose(world, handle, Vec3::default(), nan_quat),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }
}
