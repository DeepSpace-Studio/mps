#[cfg(test)]
mod tests {
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, last_error_code,
    };
    use mps_core::rapier::ffi::*;
    use mps_core::rapier::ffi::{Quat, Sphere, Vec3};
    use mps_core::rapier::query::*;

    #[test]
    fn obb_query_hits_inserted_obb_collider() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let obb = Obb {
            center: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            half_extents: Vec3 {
                x: 0.5,
                y: 1.0,
                z: 1.5,
            },
            rotation: Quat {
                i: 0.0,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
        };
        let builder = mps_core::rapier::collider::collider_builder_build(
            mps_core::rapier::collider::collider_builder_create_obb(obb),
        );
        assert!(!builder.is_null());

        let collider = mps_core::rapier::collider::world_insert_collider(world, builder);
        assert_ne!(collider, 0);
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        assert_eq!(query_intersect_obb_count_all(world, obb), 1);

        let mut handles = [0; 1];
        assert_eq!(
            query_intersect_obb_all(world, obb, handles.as_mut_ptr(), handles.len() as u32),
            1
        );
        assert_eq!(handles[0], collider);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn sphere_query_hits_inserted_sphere_collider() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let sphere = Sphere {
            center: Vec3 {
                x: 2.0,
                y: 3.0,
                z: 4.0,
            },
            radius: 1.25,
        };
        let builder = mps_core::rapier::collider::collider_builder_build(
            mps_core::rapier::collider::collider_builder_create_sphere(sphere),
        );
        assert!(!builder.is_null());

        let collider = mps_core::rapier::collider::world_insert_collider(world, builder);
        assert_ne!(collider, 0);
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        assert_eq!(query_intersect_sphere_count_all(world, sphere), 1);

        let mut handles = [0; 1];
        assert_eq!(
            query_intersect_sphere_all(world, sphere, handles.as_mut_ptr(), handles.len() as u32),
            1
        );
        assert_eq!(handles[0], collider);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn point_projection_and_batch_rays_hit_inserted_sphere() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let sphere = Sphere {
            center: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
        };
        let builder = mps_core::rapier::collider::collider_builder_build(
            mps_core::rapier::collider::collider_builder_create_sphere(sphere),
        );
        let collider = mps_core::rapier::collider::world_insert_collider(world, builder);
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let mut projected_collider = 0;
        let projection = query_project_point(
            world,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
            &mut projected_collider,
        );
        assert_eq!(projected_collider, collider);
        assert_eq!(projection.is_inside, Bool::TRUE);
        assert_eq!(
            query_intersect_point_count(
                world,
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                QueryFilterDesc::default()
            ),
            1
        );

        let rays = [0.0, 3.0, 0.0, 0.0, -1.0, 0.0, 3.0, 3.0, 0.0, 0.0, -1.0, 0.0];
        let mut hits = [RayHit::default(); 2];
        assert_eq!(
            query_cast_rays(
                world,
                rays.as_ptr(),
                2,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                hits.len() as u32,
            ),
            2
        );
        assert_eq!(hits[0].collider, collider);
        assert_eq!(hits[1].collider, 0);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn batch_intersection_counts_return_per_query_counts() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let sphere = Sphere {
            center: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
        };
        let builder = mps_core::rapier::collider::collider_builder_build(
            mps_core::rapier::collider::collider_builder_create_sphere(sphere),
        );
        let collider = mps_core::rapier::collider::world_insert_collider(world, builder);
        assert_ne!(collider, 0);
        mps_core::rapier::world::world_step(world, 1.0 / 60.0);

        let aabbs = [
            AabbDesc {
                mins: Vec3 {
                    x: -2.0,
                    y: -2.0,
                    z: -2.0,
                },
                maxs: Vec3 {
                    x: 2.0,
                    y: 2.0,
                    z: 2.0,
                },
            },
            AabbDesc {
                mins: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                maxs: Vec3 {
                    x: 11.0,
                    y: 11.0,
                    z: 11.0,
                },
            },
        ];
        let mut counts = [0; 2];
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                aabbs.as_ptr(),
                aabbs.len() as u32,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                counts.len() as u32,
            ),
            2
        );
        assert_eq!(counts, [1, 0]);

        let spheres = [
            sphere,
            Sphere {
                center: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                radius: 1.0,
            },
        ];
        counts = [0; 2];
        assert_eq!(
            query_intersect_sphere_counts(
                world,
                spheres.as_ptr(),
                spheres.len() as u32,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                counts.len() as u32,
            ),
            2
        );
        assert_eq!(counts, [1, 0]);

        let obbs = [
            Obb {
                center: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                half_extents: Vec3 {
                    x: 1.5,
                    y: 1.5,
                    z: 1.5,
                },
                rotation: Quat {
                    i: 0.0,
                    j: 0.0,
                    k: 0.0,
                    w: 1.0,
                },
            },
            Obb {
                center: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                half_extents: Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                rotation: Quat {
                    i: 0.0,
                    j: 0.0,
                    k: 0.0,
                    w: 1.0,
                },
            },
        ];
        counts = [0; 2];
        assert_eq!(
            query_intersect_obb_counts(
                world,
                obbs.as_ptr(),
                obbs.len() as u32,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                counts.len() as u32,
            ),
            2
        );
        assert_eq!(counts, [1, 0]);

        mps_core::rapier::world::world_destroy(world);
    }

    fn valid_obb() -> Obb {
        Obb {
            center: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            half_extents: Vec3 {
                x: 0.5,
                y: 1.0,
                z: 1.5,
            },
            rotation: Quat {
                i: 0.0,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
        }
    }

    fn unit_sphere() -> Sphere {
        Sphere {
            center: Vec3::default(),
            radius: 1.0,
        }
    }

    #[test]
    fn cast_ray_rejects_null_world_and_invalid_arguments() {
        let origin = Vec3 {
            x: 0.0,
            y: 3.0,
            z: 0.0,
        };
        let direction = Vec3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        };

        let hit = query_cast_ray(
            std::ptr::null(),
            origin,
            direction,
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = mps_core::rapier::world::world_create(Vec3::default());

        let nan_origin = Vec3 {
            x: f64::NAN,
            ..origin
        };
        let hit = query_cast_ray(
            world,
            nan_origin,
            direction,
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_direction = Vec3 {
            y: f64::NAN,
            ..direction
        };
        let hit = query_cast_ray(
            world,
            origin,
            nan_direction,
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Negative and NaN max_toi are rejected.
        let hit = query_cast_ray(
            world,
            origin,
            direction,
            -1.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let hit = query_cast_ray(
            world,
            origin,
            direction,
            f64::NAN,
            Bool::TRUE,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn cast_rays_batch_rejects_invalid_arguments() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let rays = [0.0, 3.0, 0.0, 0.0, -1.0, 0.0];
        let mut hits = [RayHit::default(); 1];

        // Null world.
        assert_eq!(
            query_cast_rays(
                std::ptr::null(),
                rays.as_ptr(),
                1,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Null ray input and null hit output.
        assert_eq!(
            query_cast_rays(
                world,
                std::ptr::null(),
                1,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            query_cast_rays(
                world,
                rays.as_ptr(),
                1,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Zero ray count, output capacity below ray count, and ray count above
        // MAX_OUTPUT_CAPACITY (1_000_000) all report ERR_CAPACITY.
        assert_eq!(
            query_cast_rays(
                world,
                rays.as_ptr(),
                0,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_cast_rays(
                world,
                rays.as_ptr(),
                1,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                0,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_cast_rays(
                world,
                rays.as_ptr(),
                1_000_001,
                10.0,
                Bool::TRUE,
                QueryFilterDesc::default(),
                hits.as_mut_ptr(),
                1_000_001,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn project_point_rejects_invalid_arguments() {
        let point = Vec3::default();

        let projection = query_project_point(
            std::ptr::null(),
            point,
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
        );
        assert_eq!(projection.is_inside, Bool::FALSE);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = mps_core::rapier::world::world_create(Vec3::default());

        let nan_point = Vec3 {
            x: f64::NAN,
            ..point
        };
        let projection = query_project_point(
            world,
            nan_point,
            10.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
        );
        assert_eq!(projection.is_inside, Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Negative and NaN max_dist are rejected.
        let projection = query_project_point(
            world,
            point,
            -1.0,
            Bool::TRUE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
        );
        assert_eq!(projection.is_inside, Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let projection = query_project_point(
            world,
            point,
            f64::NAN,
            Bool::TRUE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
        );
        assert_eq!(projection.is_inside, Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn intersect_point_and_aabb_count_reject_invalid_arguments() {
        let aabb = AabbDesc {
            mins: Vec3 {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            maxs: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        };

        assert_eq!(
            query_intersect_point_count(
                std::ptr::null(),
                Vec3::default(),
                QueryFilterDesc::default()
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            query_intersect_aabb_count(std::ptr::null(), aabb, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = mps_core::rapier::world::world_create(Vec3::default());

        let nan_point = Vec3 {
            x: f64::NAN,
            ..Vec3::default()
        };
        assert_eq!(
            query_intersect_point_count(world, nan_point, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // mins above maxs and NaN bounds are invalid AABBs.
        let inverted = AabbDesc {
            mins: aabb.maxs,
            maxs: aabb.mins,
        };
        assert_eq!(
            query_intersect_aabb_count(world, inverted, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_aabb = AabbDesc {
            mins: Vec3 {
                x: f64::NAN,
                ..aabb.mins
            },
            ..aabb
        };
        assert_eq!(
            query_intersect_aabb_count(world, nan_aabb, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn intersect_aabb_rejects_null_output_and_bad_capacity() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let aabb = AabbDesc {
            mins: Vec3 {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            maxs: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        };
        let mut handles = [0u64; 4];

        // Null output buffer.
        assert_eq!(
            query_intersect_aabb(
                world,
                aabb,
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                4,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Zero capacity and capacity above MAX_OUTPUT_CAPACITY (1_000_000).
        assert_eq!(
            query_intersect_aabb(
                world,
                aabb,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                0,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_intersect_aabb(
                world,
                aabb,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                1_000_001,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Invalid AABB with an otherwise valid call.
        let inverted = AabbDesc {
            mins: aabb.maxs,
            maxs: aabb.mins,
        };
        assert_eq!(
            query_intersect_aabb(
                world,
                inverted,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                4,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn intersect_aabb_counts_batch_rejects_invalid_arguments() {
        let world = mps_core::rapier::world::world_create(Vec3::default());
        let aabbs = [AabbDesc {
            mins: Vec3 {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            maxs: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }];
        let mut counts = [0u32; 1];

        // Null world.
        assert_eq!(
            query_intersect_aabb_counts(
                std::ptr::null(),
                aabbs.as_ptr(),
                1,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Null AABB input and null count output.
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                std::ptr::null(),
                1,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                aabbs.as_ptr(),
                1,
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Zero query count, capacity below query count, and query count above
        // MAX_OUTPUT_CAPACITY (1_000_000) all report ERR_CAPACITY.
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                aabbs.as_ptr(),
                0,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                aabbs.as_ptr(),
                1,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                0,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_intersect_aabb_counts(
                world,
                aabbs.as_ptr(),
                1_000_001,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1_000_001,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn obb_and_sphere_queries_reject_invalid_arguments() {
        let obb = valid_obb();

        // Null world.
        assert_eq!(
            query_intersect_obb_count(std::ptr::null(), obb, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = mps_core::rapier::world::world_create(Vec3::default());
        let mut handles = [0u64; 4];

        // Zero half extents and non-finite rotation are invalid OBBs.
        let flat_obb = Obb {
            half_extents: Vec3::default(),
            ..obb
        };
        assert_eq!(
            query_intersect_obb_count(world, flat_obb, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_rotation_obb = Obb {
            rotation: Quat {
                i: f64::NAN,
                j: 0.0,
                k: 0.0,
                w: 1.0,
            },
            ..obb
        };
        assert_eq!(
            query_intersect_obb(
                world,
                nan_rotation_obb,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                4
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Null output and zero capacity on the handle-returning variants.
        assert_eq!(
            query_intersect_obb(
                world,
                obb,
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                4,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            query_intersect_obb(
                world,
                obb,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                0,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        // Non-positive and NaN sphere radii are rejected.
        let zero_sphere = Sphere {
            radius: 0.0,
            ..unit_sphere()
        };
        assert_eq!(
            query_intersect_sphere_count(world, zero_sphere, QueryFilterDesc::default()),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        let nan_sphere = Sphere {
            radius: f64::NAN,
            ..unit_sphere()
        };
        assert_eq!(
            query_intersect_sphere(
                world,
                nan_sphere,
                QueryFilterDesc::default(),
                handles.as_mut_ptr(),
                4
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        assert_eq!(
            query_intersect_sphere(
                world,
                unit_sphere(),
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                4,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        // Batch count variants share the same validation.
        let obbs = [obb];
        let mut counts = [0u32; 1];
        assert_eq!(
            query_intersect_obb_counts(
                world,
                std::ptr::null(),
                1,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert_eq!(
            query_intersect_obb_counts(
                world,
                obbs.as_ptr(),
                1,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                0,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);

        let spheres = [unit_sphere()];
        assert_eq!(
            query_intersect_sphere_counts(
                world,
                spheres.as_ptr(),
                0,
                QueryFilterDesc::default(),
                counts.as_mut_ptr(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_CAPACITY);
        assert_eq!(
            query_intersect_sphere_counts(
                world,
                spheres.as_ptr(),
                1,
                QueryFilterDesc::default(),
                std::ptr::null_mut(),
                1,
            ),
            0
        );
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        mps_core::rapier::world::world_destroy(world);
    }

    #[test]
    fn cast_shape_rejects_invalid_arguments() {
        let ball = ShapeDesc {
            shape_type: 0,
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
        };
        let rotation = Quat {
            i: 0.0,
            j: 0.0,
            k: 0.0,
            w: 1.0,
        };
        let velocity = Vec3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        };
        let options = ShapeCastOptionsDesc {
            max_time_of_impact: 10.0,
            target_distance: 0.0,
            stop_at_penetration: Bool::FALSE,
            compute_impact_geometry_on_penetration: Bool::FALSE,
        };

        // Null world.
        let hit = query_cast_shape(
            std::ptr::null(),
            ball,
            Vec3::default(),
            rotation,
            velocity,
            options,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = mps_core::rapier::world::world_create(Vec3::default());

        // Invalid shape descriptor (ball with zero radius).
        let invalid_shape = ShapeDesc { a: 0.0, ..ball };
        let hit = query_cast_shape(
            world,
            invalid_shape,
            Vec3::default(),
            rotation,
            velocity,
            options,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // NaN velocity.
        let nan_velocity = Vec3 {
            y: f64::NAN,
            ..velocity
        };
        let hit = query_cast_shape(
            world,
            ball,
            Vec3::default(),
            rotation,
            nan_velocity,
            options,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Non-finite rotation.
        let nan_rotation = Quat {
            w: f64::NAN,
            ..rotation
        };
        let hit = query_cast_shape(
            world,
            ball,
            Vec3::default(),
            nan_rotation,
            velocity,
            options,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Negative max time of impact.
        let negative_toi = ShapeCastOptionsDesc {
            max_time_of_impact: -1.0,
            ..options
        };
        let hit = query_cast_shape(
            world,
            ball,
            Vec3::default(),
            rotation,
            velocity,
            negative_toi,
            QueryFilterDesc::default(),
        );
        assert_eq!(hit.collider, 0);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        mps_core::rapier::world::world_destroy(world);
    }
}
