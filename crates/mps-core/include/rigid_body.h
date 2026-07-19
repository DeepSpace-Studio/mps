#ifndef RIGID_BODY_H
#define RIGID_BODY_H

#pragma once

/* Generated with cbindgen:0.29.4 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Gravitational constant (N·m²/kg²).
 */
#define G 6.67430e-11

/**
 * Magic number identifying a valid arena: "MPS_AREN"
 */
#define ARENA_MAGIC 5571044407640212814

/**
 * Current arena layout version — increment when layout changes
 */
#define ARENA_VERSION 1

/**
 * Strides (must match Java side exactly)
 */
#define BODY_SLOT_STRIDE 96

#define COLLIDER_SLOT_STRIDE 80

#define CMD_SLOT_STRIDE 32

#define EVENT_SLOT_STRIDE 64

/**
 * Header size in bytes
 */
#define HEADER_SIZE 128

typedef struct AnvilKitAppHandle AnvilKitAppHandle;

typedef struct CRbTreeHandle CRbTreeHandle;

typedef struct CharacterControllerHandle CharacterControllerHandle;

typedef struct ColliderBuilderHandle ColliderBuilderHandle;

typedef struct JointBuilderHandle JointBuilderHandle;

typedef struct RTreeHandle RTreeHandle;

typedef struct RigidBodyBuilderHandle RigidBodyBuilderHandle;

typedef struct WorldHandle WorldHandle;

/**
 * A mass concentration (mascon) on the Moon's surface.
 */
typedef struct LunarMascon {
  /**
   * Center position (Moon-fixed, meters)
   */
  Vec3 center;
  /**
   * Excess mass (kg) — positive = mass excess
   */
  double excess_mass;
  /**
   * Radius of the mascon (m) — used for softening
   */
  double radius;
} LunarMascon;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

Bool aero_apply_surfaces(struct WorldHandle *world,
                         RigidBodyHandleRaw body_handle,
                         Vec3 wind_velocity,
                         double air_density,
                         const AeroSurface *surfaces,
                         uint32_t surface_count,
                         Bool wake_up,
                         AeroForceReport *out_report);

Bool aero_apply_voxel_grid(struct WorldHandle *world,
                           RigidBodyHandleRaw body_handle,
                           Vec3 wind_velocity,
                           double air_density,
                           const uint8_t *voxels,
                           uint32_t size_x,
                           uint32_t size_y,
                           uint32_t size_z,
                           double voxel_size,
                           Vec3 local_origin,
                           double drag_coefficient,
                           double lift_coefficient,
                           Bool wake_up,
                           AeroForceReport *out_report);

uint8_t aero_apply_voxel_grid_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw body_handle,
                                   Vec3 wind_velocity,
                                   double air_density,
                                   const uint8_t *voxels,
                                   uint32_t size_x,
                                   uint32_t size_y,
                                   uint32_t size_z,
                                   double voxel_size,
                                   Vec3 local_origin,
                                   double drag_coefficient,
                                   double lift_coefficient,
                                   Bool wake_up,
                                   AeroForceReport *out_report);

uint8_t aero_apply_surfaces_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw body_handle,
                                 Vec3 wind_velocity,
                                 double air_density,
                                 const AeroSurface *surfaces,
                                 uint32_t surface_count,
                                 Bool wake_up,
                                 AeroForceReport *out_report);

Bool aero_estimate_surface_force(Vec3 body_linvel,
                                 Vec3 body_angvel,
                                 Vec3 body_center,
                                 Vec3 wind_velocity,
                                 double air_density,
                                 AeroSurface surface,
                                 AeroForceReport *out_report);

struct AnvilKitAppHandle *anvilkit_app_create(void);

void anvilkit_app_destroy(struct AnvilKitAppHandle *app);

void anvilkit_app_update(struct AnvilKitAppHandle *app);

uint64_t anvilkit_app_spawn_body(struct AnvilKitAppHandle *app,
                                 Vec3 translation,
                                 Quat rotation,
                                 uint32_t status);

uint64_t anvilkit_app_spawn_body_with_collider(struct AnvilKitAppHandle *app,
                                               Vec3 translation,
                                               Quat rotation,
                                               uint32_t status,
                                               ShapeDesc shape);

Bool anvilkit_app_set_transform(struct AnvilKitAppHandle *app,
                                uint64_t entity_bits,
                                Vec3 translation,
                                Quat rotation);

Bool anvilkit_app_set_material(struct AnvilKitAppHandle *app,
                               uint64_t entity_bits,
                               MaterialProperties material);

uint32_t anvilkit_app_sync_to_world(struct AnvilKitAppHandle *app, struct WorldHandle *world);

RigidBodyHandleRaw anvilkit_app_entity_to_body(const struct AnvilKitAppHandle *app,
                                               uint64_t entity_bits);

ColliderHandleRaw anvilkit_app_entity_to_collider(const struct AnvilKitAppHandle *app,
                                                  uint64_t entity_bits);

uint64_t anvilkit_app_create_constraint(struct AnvilKitAppHandle *app,
                                        struct WorldHandle *world,
                                        uint64_t entity1_bits,
                                        uint64_t entity2_bits,
                                        uint32_t joint_type,
                                        Vec3 axis_or_primary,
                                        double b,
                                        double c,
                                        Bool wake_up);

ImpulseJointHandleRaw anvilkit_app_constraint_to_joint(const struct AnvilKitAppHandle *app,
                                                       uint64_t constraint_id);

Bool anvilkit_app_remove_constraint(struct AnvilKitAppHandle *app,
                                    struct WorldHandle *world,
                                    uint64_t constraint_id,
                                    Bool wake_up);

Bool anvilkit_app_apply_aero_surfaces(struct AnvilKitAppHandle *app,
                                      struct WorldHandle *world,
                                      uint64_t entity_bits,
                                      Vec3 wind_velocity,
                                      double air_density,
                                      const AeroSurface *surfaces,
                                      uint32_t surface_count,
                                      Bool wake_up,
                                      AeroForceReport *out_report);

Bool anvilkit_app_apply_aero_voxel_grid(struct AnvilKitAppHandle *app,
                                        struct WorldHandle *world,
                                        uint64_t entity_bits,
                                        Vec3 wind_velocity,
                                        double air_density,
                                        const uint8_t *voxels,
                                        uint32_t size_x,
                                        uint32_t size_y,
                                        uint32_t size_z,
                                        double voxel_size,
                                        Vec3 local_origin,
                                        double drag_coefficient,
                                        double lift_coefficient,
                                        Bool wake_up,
                                        AeroForceReport *out_report);

Bool anvilkit_app_apply_fluid_aabb_forces(struct AnvilKitAppHandle *app,
                                          struct WorldHandle *world,
                                          uint64_t entity_bits,
                                          FluidVolume fluid_volume,
                                          Vec3 body_half_extents,
                                          double body_volume,
                                          Bool wake_up,
                                          FluidForceReport *out_report);

Bool anvilkit_app_apply_trajectory_forces(struct AnvilKitAppHandle *app,
                                          struct WorldHandle *world,
                                          uint64_t entity_bits,
                                          TrajectoryEnvironment environment,
                                          Bool wake_up,
                                          TrajectoryForceReport *out_report);

Bool material_stress_strain_linear(MaterialProperties material,
                                   double strain,
                                   double delta_temperature,
                                   StressStrainReport *out_report);

double material_elastic_collision_relative_speed(double relative_normal_speed, double restitution);

Bool material_hertz_contact_force(MaterialProperties material1,
                                  MaterialProperties material2,
                                  double radius1,
                                  double radius2,
                                  double penetration,
                                  double penetration_rate,
                                  double damping,
                                  HertzContactReport *out_report);

struct ColliderBuilderHandle *collider_builder_create_capsule(Capsule capsule);

struct ColliderBuilderHandle *collider_builder_create_ssv(Ssv ssv);

struct ColliderBuilderHandle *collider_builder_create_ellipsoid(Ellipsoid ellipsoid);

struct ColliderBuilderHandle *collider_builder_create_prism(Prism prism);

struct ColliderBuilderHandle *collider_builder_create_cylinder(Cylinder cylinder);

struct ColliderBuilderHandle *collider_builder_create_spherical_shell(SphericalShell shell);

uint32_t query_intersect_capsule_count(const struct WorldHandle *world,
                                       Capsule capsule,
                                       QueryFilterDesc filter);

uint32_t query_intersect_capsule_count_all(const struct WorldHandle *world, Capsule capsule);

uint32_t query_intersect_capsule(const struct WorldHandle *world,
                                 Capsule capsule,
                                 QueryFilterDesc filter,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_capsule_all(const struct WorldHandle *world,
                                     Capsule capsule,
                                     ColliderHandleRaw *out_handles,
                                     uint32_t capacity);

uint32_t query_intersect_ssv_count(const struct WorldHandle *world,
                                   Ssv ssv,
                                   QueryFilterDesc filter);

uint32_t query_intersect_ssv_count_all(const struct WorldHandle *world, Ssv ssv);

uint32_t query_intersect_ssv(const struct WorldHandle *world,
                             Ssv ssv,
                             QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

uint32_t query_intersect_ssv_all(const struct WorldHandle *world,
                                 Ssv ssv,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_ellipsoid_count(const struct WorldHandle *world,
                                         Ellipsoid ellipsoid,
                                         QueryFilterDesc filter);

uint32_t query_intersect_ellipsoid_count_all(const struct WorldHandle *world, Ellipsoid ellipsoid);

uint32_t query_intersect_ellipsoid(const struct WorldHandle *world,
                                   Ellipsoid ellipsoid,
                                   QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_ellipsoid_all(const struct WorldHandle *world,
                                       Ellipsoid ellipsoid,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

uint32_t query_intersect_prism_count(const struct WorldHandle *world,
                                     Prism prism,
                                     QueryFilterDesc filter);

uint32_t query_intersect_prism_count_all(const struct WorldHandle *world, Prism prism);

uint32_t query_intersect_prism(const struct WorldHandle *world,
                               Prism prism,
                               QueryFilterDesc filter,
                               ColliderHandleRaw *out_handles,
                               uint32_t capacity);

uint32_t query_intersect_prism_all(const struct WorldHandle *world,
                                   Prism prism,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_cylinder_count(const struct WorldHandle *world,
                                        Cylinder cylinder,
                                        QueryFilterDesc filter);

uint32_t query_intersect_cylinder_count_all(const struct WorldHandle *world, Cylinder cylinder);

uint32_t query_intersect_cylinder(const struct WorldHandle *world,
                                  Cylinder cylinder,
                                  QueryFilterDesc filter,
                                  ColliderHandleRaw *out_handles,
                                  uint32_t capacity);

uint32_t query_intersect_cylinder_all(const struct WorldHandle *world,
                                      Cylinder cylinder,
                                      ColliderHandleRaw *out_handles,
                                      uint32_t capacity);

uint32_t query_intersect_spherical_shell_count(const struct WorldHandle *world,
                                               SphericalShell shell,
                                               QueryFilterDesc filter);

uint32_t query_intersect_spherical_shell_count_all(const struct WorldHandle *world,
                                                   SphericalShell shell);

uint32_t query_intersect_spherical_shell(const struct WorldHandle *world,
                                         SphericalShell shell,
                                         QueryFilterDesc filter,
                                         ColliderHandleRaw *out_handles,
                                         uint32_t capacity);

uint32_t query_intersect_spherical_shell_all(const struct WorldHandle *world,
                                             SphericalShell shell,
                                             ColliderHandleRaw *out_handles,
                                             uint32_t capacity);

struct ColliderBuilderHandle *collider_builder_create(uint32_t shape_type, Vec3 shape_data);

struct ColliderBuilderHandle *collider_builder_create_halfspace(Vec3 normal);

struct ColliderBuilderHandle *collider_builder_create_ex(ShapeDesc shape_desc);

struct ColliderBuilderHandle *collider_builder_create_obb(Obb obb);

struct ColliderBuilderHandle *collider_builder_create_sphere(Sphere sphere);

struct ColliderBuilderHandle *collider_builder_create_heightmap(const double *data,
                                                                uint32_t data_x,
                                                                uint32_t data_y,
                                                                Vec3 scale);

struct ColliderBuilderHandle *collider_builder_create_convex_hull(const double *points_xyz,
                                                                  uint32_t point_count);

struct ColliderBuilderHandle *collider_builder_create_point_cloud_bounds(const double *points_xyz,
                                                                         uint32_t point_count);

struct ColliderBuilderHandle *collider_builder_create_double_bv(AabbDesc first, AabbDesc second);

struct ColliderBuilderHandle *collider_builder_create_skewed_obb(Vec3 center,
                                                                 Vec3 axis_x,
                                                                 Vec3 axis_y,
                                                                 Vec3 axis_z);

struct ColliderBuilderHandle *collider_builder_create_discrete_obb(const double *points_xyz,
                                                                   uint32_t point_count,
                                                                   uint32_t axis);

struct ColliderBuilderHandle *collider_builder_create_fused_collapsing_bounds(const double *points_xyz,
                                                                              uint32_t point_count,
                                                                              double padding);

struct ColliderBuilderHandle *collider_builder_create_edge_bvh(const double *vertices_xyz,
                                                               uint32_t vertex_count,
                                                               const uint32_t *edges,
                                                               uint32_t edge_count,
                                                               double radius);

struct ColliderBuilderHandle *collider_builder_create_medial_spheres(const double *spheres_xyzw,
                                                                     uint32_t sphere_count);

Collider *collider_builder_build(struct ColliderBuilderHandle *builder);

void collider_builder_destroy(struct ColliderBuilderHandle *builder);

void collider_destroy_raw(Collider *collider);

void collider_builder_set_translation(struct ColliderBuilderHandle *builder, Vec3 translation);

void collider_builder_set_rotation(struct ColliderBuilderHandle *builder, Vec3 rotation_axis_angle);

void collider_builder_set_pose(struct ColliderBuilderHandle *builder,
                               Vec3 translation,
                               Quat rotation);

void collider_builder_set_sensor(struct ColliderBuilderHandle *builder, Bool sensor);

void collider_builder_set_friction(struct ColliderBuilderHandle *builder, double friction);

void collider_builder_set_restitution(struct ColliderBuilderHandle *builder, double restitution);

void collider_builder_set_density(struct ColliderBuilderHandle *builder, double density);

void collider_builder_set_collision_groups(struct ColliderBuilderHandle *builder,
                                           InteractionGroupsDesc groups);

void collider_builder_set_solver_groups(struct ColliderBuilderHandle *builder,
                                        InteractionGroupsDesc groups);

void collider_builder_set_active_events(struct ColliderBuilderHandle *builder,
                                        uint32_t active_events_bits);

void collider_builder_set_active_hooks(struct ColliderBuilderHandle *builder,
                                       uint32_t active_hooks_bits);

void collider_builder_set_contact_force_event_threshold(struct ColliderBuilderHandle *builder,
                                                        double threshold);

ColliderHandleRaw world_insert_collider(struct WorldHandle *world, Collider *memory_handle);

ColliderHandleRaw world_insert_collider_with_parent(struct WorldHandle *world,
                                                    Collider *memory_handle,
                                                    RigidBodyHandleRaw parent);

Bool world_remove_collider(struct WorldHandle *world, ColliderHandleRaw handle, Bool wake_up);

Collider *world_copy_collider(struct WorldHandle *world, ColliderHandleRaw handle);

uint8_t world_remove_collider_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   Bool wake_up);

Vec3 collider_get_translation(const struct WorldHandle *world, ColliderHandleRaw handle);

uintptr_t collider_get_shape_count(const struct WorldHandle *world, ColliderHandleRaw handle);

void collider_get_translation_out(const struct WorldHandle *world,
                                  ColliderHandleRaw handle,
                                  Vec3 *out_translation);

Quat collider_get_rotation(const struct WorldHandle *world, ColliderHandleRaw handle);

void collider_get_rotation_out(const struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               Quat *out_rotation);

Bool collider_set_pose(struct WorldHandle *world,
                       ColliderHandleRaw handle,
                       Vec3 translation,
                       Quat rotation);

Bool collider_set_translation(struct WorldHandle *world,
                              ColliderHandleRaw handle,
                              Vec3 translation);

Bool collider_set_rotation(struct WorldHandle *world, ColliderHandleRaw handle, Quat rotation);

uint8_t collider_set_pose_flag(struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               Vec3 translation,
                               Quat rotation);

Bool collider_set_sensor(struct WorldHandle *world, ColliderHandleRaw handle, Bool sensor);

uint8_t collider_set_sensor_flag(struct WorldHandle *world, ColliderHandleRaw handle, Bool sensor);

Bool collider_set_friction(struct WorldHandle *world, ColliderHandleRaw handle, double friction);

uint8_t collider_set_friction_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   double friction);

Bool collider_set_restitution(struct WorldHandle *world,
                              ColliderHandleRaw handle,
                              double restitution);

uint8_t collider_set_restitution_flag(struct WorldHandle *world,
                                      ColliderHandleRaw handle,
                                      double restitution);

Bool collider_set_collision_groups(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   InteractionGroupsDesc groups);

uint8_t collider_set_collision_groups_flag(struct WorldHandle *world,
                                           ColliderHandleRaw handle,
                                           InteractionGroupsDesc groups);

Bool collider_set_solver_groups(struct WorldHandle *world,
                                ColliderHandleRaw handle,
                                InteractionGroupsDesc groups);

uint8_t collider_set_solver_groups_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        InteractionGroupsDesc groups);

Bool collider_set_active_events(struct WorldHandle *world,
                                ColliderHandleRaw handle,
                                uint32_t active_events_bits);

uint8_t collider_set_active_events_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        uint32_t active_events_bits);

Bool collider_set_active_hooks(struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               uint32_t active_hooks_bits);

uint8_t collider_set_active_hooks_flag(struct WorldHandle *world,
                                       ColliderHandleRaw handle,
                                       uint32_t active_hooks_bits);

Bool collider_set_contact_force_event_threshold(struct WorldHandle *world,
                                                ColliderHandleRaw handle,
                                                double threshold);

uint8_t collider_set_contact_force_event_threshold_flag(struct WorldHandle *world,
                                                        ColliderHandleRaw handle,
                                                        double threshold);

double collider_get_density(const struct WorldHandle *world, ColliderHandleRaw handle);

RigidBodyHandleRaw world_insert_dynamic_cuboids(struct WorldHandle *world,
                                                Vec3 translation,
                                                Quat rotation,
                                                Vec3 linvel,
                                                const double *cuboids,
                                                uint32_t cuboid_count,
                                                double density,
                                                double friction,
                                                double restitution,
                                                InteractionGroupsDesc collision_groups,
                                                InteractionGroupsDesc solver_groups);

RigidBodyHandleRaw world_insert_static_trimesh(struct WorldHandle *world,
                                               const double *vertices_xyz,
                                               uint32_t vertex_xyz_len,
                                               const uint32_t *indices,
                                               uint32_t index_len,
                                               double friction,
                                               double restitution);

uint32_t query_intersect_aabb_rigid_body_count(const struct WorldHandle *world,
                                               AabbDesc aabb,
                                               QueryFilterDesc filter);

uint32_t query_intersect_aabb_rigid_bodies(const struct WorldHandle *world,
                                           AabbDesc aabb,
                                           QueryFilterDesc filter,
                                           RigidBodyHandleRaw *out_handles,
                                           uint32_t capacity);

struct CharacterControllerHandle *character_controller_create(void);

void character_controller_destroy(struct CharacterControllerHandle *controller);

void character_controller_set_up(struct CharacterControllerHandle *controller, Vec3 up);

void character_controller_set_offset_absolute(struct CharacterControllerHandle *controller,
                                              double offset);

void character_controller_set_offset_relative(struct CharacterControllerHandle *controller,
                                              double offset);

void character_controller_set_slide(struct CharacterControllerHandle *controller, Bool slide);

void character_controller_set_autostep(struct CharacterControllerHandle *controller,
                                       Bool enabled,
                                       double max_height,
                                       double min_width,
                                       Bool include_dynamic_bodies);

void character_controller_set_snap_to_ground(struct CharacterControllerHandle *controller,
                                             Bool enabled,
                                             double distance);

void character_controller_set_slope_angles(struct CharacterControllerHandle *controller,
                                           double max_climb_angle,
                                           double min_slide_angle);

EffectiveCharacterMovement character_controller_move_shape(const struct WorldHandle *world,
                                                           struct CharacterControllerHandle *controller,
                                                           double dt,
                                                           ShapeDesc shape_desc,
                                                           Vec3 translation,
                                                           Quat rotation,
                                                           Vec3 desired_translation);

uint32_t character_controller_collision_count(const struct CharacterControllerHandle *controller);

FfiCharacterCollision character_controller_get_collision(const struct CharacterControllerHandle *controller,
                                                         uint32_t index);

Bool character_controller_solve_impulses(struct WorldHandle *world,
                                         struct CharacterControllerHandle *controller,
                                         double dt,
                                         ShapeDesc shape_desc,
                                         double character_mass);

struct CRbTreeHandle *crb_tree_create(void);

void crb_tree_destroy(struct CRbTreeHandle *tree);

void crb_tree_clear(struct CRbTreeHandle *tree);

uint32_t crb_tree_len(const struct CRbTreeHandle *tree);

Bool crb_tree_insert(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

uint8_t crb_tree_insert_flag(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

Bool crb_tree_update(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

Bool crb_tree_remove(struct CRbTreeHandle *tree, uint64_t id);

uint32_t crb_tree_query_aabb_count(const struct CRbTreeHandle *tree, AabbDesc aabb);

uint32_t crb_tree_query_aabb(const struct CRbTreeHandle *tree,
                             AabbDesc aabb,
                             uint64_t *out_ids,
                             uint32_t capacity);

struct ColliderBuilderHandle *collider_builder_create_kdop(const double *points_xyz,
                                                           uint32_t point_count,
                                                           uint32_t preset);

struct ColliderBuilderHandle *collider_builder_create_fdh(const double *points_xyz,
                                                          uint32_t point_count,
                                                          const double *directions_xyz,
                                                          uint32_t direction_count);

uint32_t last_error_code(void);

const char *last_error_message(void);

void last_error_clear(void);

Bool world_set_coulomb_friction_law(struct WorldHandle *world, CoulombFrictionLaw law);

uint8_t world_set_coulomb_friction_law_flag(struct WorldHandle *world, CoulombFrictionLaw law);

void world_clear_coulomb_friction_law(struct WorldHandle *world);

Bool world_get_coulomb_friction_law(const struct WorldHandle *world, CoulombFrictionLaw *out_law);

Bool world_set_air_drag_law(struct WorldHandle *world, AirDragLaw law);

uint8_t world_set_air_drag_law_flag(struct WorldHandle *world, AirDragLaw law);

void world_clear_air_drag_law(struct WorldHandle *world);

Bool world_get_air_drag_law(const struct WorldHandle *world, AirDragLaw *out_law);

Bool world_set_external_force_law(struct WorldHandle *world, ExternalForceLaw law);

uint8_t world_set_external_force_law_flag(struct WorldHandle *world, ExternalForceLaw law);

void world_clear_external_force_law(struct WorldHandle *world);

Bool world_get_external_force_law(const struct WorldHandle *world, ExternalForceLaw *out_law);

Bool world_set_newton_gravity_law(struct WorldHandle *world, NewtonGravityLaw law);

uint8_t world_set_newton_gravity_law_flag(struct WorldHandle *world, NewtonGravityLaw law);

void world_clear_newton_gravity_law(struct WorldHandle *world);

Bool world_get_newton_gravity_law(const struct WorldHandle *world, NewtonGravityLaw *out_law);

Bool world_get_custom_physics_report(const struct WorldHandle *world,
                                     CustomPhysicsReport *out_report);

void world_clear_events(struct WorldHandle *world);

uint32_t world_collision_event_count(const struct WorldHandle *world);

CollisionEventRecord world_get_collision_event(const struct WorldHandle *world, uint32_t index);

uint32_t world_get_collision_events(const struct WorldHandle *world,
                                    CollisionEventRecord *out_events,
                                    uint32_t capacity);

uint32_t world_contact_force_event_count(const struct WorldHandle *world);

ContactForceEventRecord world_get_contact_force_event(const struct WorldHandle *world,
                                                      uint32_t index);

uint32_t world_get_contact_force_events(const struct WorldHandle *world,
                                        ContactForceEventRecord *out_events,
                                        uint32_t capacity);

void world_set_contact_pair_filter_callback(struct WorldHandle *world,
                                            uintptr_t _callback,
                                            uintptr_t _user_data);

void world_set_intersection_pair_filter_callback(struct WorldHandle *world,
                                                 uintptr_t _callback,
                                                 uintptr_t _user_data);

void world_clear_contact_pair_filter_callback(struct WorldHandle *world);

void world_clear_intersection_pair_filter_callback(struct WorldHandle *world);

/**
 * Allocate a collision-event ring buffer of `capacity` records.
 * Events will be written here during `world_step` instead of (or in addition to)
 * the legacy Vec queue.  Java drains the ring buffer at its own pace.
 */
Bool world_init_collision_event_ring(struct WorldHandle *world, uint32_t capacity);

/**
 * Allocate a contact-force-event ring buffer.
 */
Bool world_init_contact_force_event_ring(struct WorldHandle *world, uint32_t capacity);

/**
 * Drain the collision-event ring buffer into `out_events`.
 * Returns the number of events drained.  This is the **only** FFI call needed
 * per frame after init — no more count-then-allocate-then-read cycles.
 */
uint32_t world_drain_collision_event_ring(const struct WorldHandle *world,
                                          CollisionEventRecord *out_events,
                                          uint32_t capacity);

/**
 * Drain the contact-force-event ring buffer.
 */
uint32_t world_drain_contact_force_event_ring(const struct WorldHandle *world,
                                              ContactForceEventRecord *out_events,
                                              uint32_t capacity);

/**
 * Get the current number of events in the collision ring buffer (cheap, no lock).
 */
uint32_t world_collision_event_ring_len(const struct WorldHandle *world);

/**
 * Get the current number of events in the contact-force ring buffer.
 */
uint32_t world_contact_force_event_ring_len(const struct WorldHandle *world);

/**
 * Get ring buffer statistics (capacity, occupancy, drops, wraps).
 */
Bool world_collision_event_ring_stats(const struct WorldHandle *world,
                                      EventRingBufferStats *out_stats);

Bool world_contact_force_event_ring_stats(const struct WorldHandle *world,
                                          EventRingBufferStats *out_stats);

/**
 * Clear both ring buffers and reset drop counters.
 */
void world_clear_event_rings(struct WorldHandle *world);

/**
 * Register a collision-event callback.
 *
 * `callback` is a C function pointer (zero = unregister).
 * `user_data` is passed through unchanged to each invocation.
 * Returns an opaque handle for later unregistration.
 */
EventCallbackHandle world_register_collision_callback(struct WorldHandle *world,
                                                      uintptr_t callback,
                                                      uintptr_t user_data);

/**
 * Register a contact-force-event callback.
 */
EventCallbackHandle world_register_contact_force_callback(struct WorldHandle *world,
                                                          uintptr_t callback,
                                                          uintptr_t user_data);

/**
 * Unregister a previously registered callback by its handle.
 * Passing 0 or an invalid handle is a no-op.
 */
void world_unregister_callback(struct WorldHandle *world, EventCallbackHandle handle);

/**
 * Set the event dispatch mode.
 *
 * - `Poll` (0): legacy Vec queue only (default).
 * - `Callback` (1): registered callbacks only.
 * - `Both` (2): ring buffer + callbacks.
 */
Bool world_set_event_dispatch_mode(struct WorldHandle *world, uint32_t mode);

Bool fluid_estimate_aabb_forces(FluidVolume fluid,
                                Vec3 body_center,
                                Vec3 body_half_extents,
                                double body_volume,
                                Vec3 body_linvel,
                                Vec3 body_angvel,
                                FluidForceReport *out_report);

Bool fluid_apply_aabb_forces(struct WorldHandle *world,
                             RigidBodyHandleRaw body_handle,
                             FluidVolume fluid,
                             Vec3 body_half_extents,
                             double body_volume,
                             Bool wake_up,
                             FluidForceReport *out_report);

uint8_t fluid_apply_aabb_forces_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw body_handle,
                                     FluidVolume fluid,
                                     Vec3 body_half_extents,
                                     double body_volume,
                                     Bool wake_up,
                                     FluidForceReport *out_report);

Bool fluid_navier_stokes_simplified_step(Vec3 velocity,
                                         Vec3 advection,
                                         Vec3 pressure_gradient,
                                         Vec3 laplacian_velocity,
                                         Vec3 external_acceleration,
                                         double density,
                                         double kinematic_viscosity,
                                         double dt,
                                         NavierStokesReport *out_report);

double fluid_sph_poly6_kernel(double distance, double smoothing_radius);

Bool fluid_sph_spiky_gradient(Vec3 offset, double smoothing_radius, Vec3 *out_gradient);

double fluid_sph_viscosity_laplacian(double distance, double smoothing_radius);

Bool fluid_sph_estimate_density(Vec3 position,
                                const SphParticle *particles,
                                uint32_t particle_count,
                                double smoothing_radius,
                                double *out_density);

Bool fluid_sph_estimate_forces(SphParticle particle,
                               const SphParticle *particles,
                               uint32_t particle_count,
                               double smoothing_radius,
                               double gas_constant,
                               double rest_density,
                               double viscosity,
                               double surface_tension,
                               SphForceReport *out_report);

double fluid_bernoulli_pressure(double total_pressure,
                                double density,
                                double velocity,
                                double gravity,
                                double elevation);

Bool fluid_bernoulli_report(double pressure,
                            double density,
                            double velocity,
                            double gravity,
                            double elevation,
                            BernoulliReport *out_report);

Bool fracture_stress_intensity_factor(double stress,
                                      double crack_length,
                                      double geometry_factor,
                                      double fracture_toughness,
                                      StressIntensityReport *out_report);

Bool fracture_griffith_criterion(double stress,
                                 double crack_length,
                                 FractureMaterial material,
                                 GriffithReport *out_report);

Bool fracture_miner_damage(const double *cycle_counts,
                           const double *cycles_to_failure,
                           uint32_t count,
                           MinerDamageReport *out_report);

Bool fracture_sn_curve_life(double stress_amplitude,
                            double coefficient,
                            double exponent,
                            double endurance_limit,
                            SnCurveReport *out_report);

Bool fracture_energy_release(double strain_energy,
                             double new_surface_area,
                             double surface_energy,
                             double kinetic_energy,
                             FractureEnergyReport *out_report);

Bool fracture_mode_from_stress(double tensile_stress,
                               double shear_stress,
                               double compressive_stress,
                               FractureModeReport *out_report);

Bool world_replace_body_with_fracture_fragments(struct WorldHandle *world,
                                                RigidBodyHandleRaw source_body,
                                                const FractureFragmentDesc *fragments,
                                                uint32_t fragment_count,
                                                Bool connect_fragments,
                                                Bool remove_source,
                                                RigidBodyHandleRaw *out_body_handles,
                                                ImpulseJointHandleRaw *out_joint_handles,
                                                uint32_t capacity,
                                                FractureReplaceReport *out_report);

struct JointBuilderHandle *joint_builder_create(uint32_t joint_type,
                                                Vec3 axis_or_primary,
                                                double b,
                                                double c);

void joint_builder_destroy(struct JointBuilderHandle *builder);

void joint_builder_set_contacts_enabled(struct JointBuilderHandle *builder, Bool enabled);

void joint_builder_set_local_anchor1(struct JointBuilderHandle *builder, Vec3 anchor);

void joint_builder_set_local_anchor2(struct JointBuilderHandle *builder, Vec3 anchor);

void joint_builder_set_limits(struct JointBuilderHandle *builder,
                              uint32_t axis,
                              double min,
                              double max);

void joint_builder_set_motor_velocity(struct JointBuilderHandle *builder,
                                      uint32_t axis,
                                      double target_vel,
                                      double factor);

void joint_builder_set_motor_position(struct JointBuilderHandle *builder,
                                      uint32_t axis,
                                      double target_pos,
                                      double stiffness,
                                      double damping);

ImpulseJointHandleRaw world_insert_impulse_joint(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body1,
                                                 RigidBodyHandleRaw body2,
                                                 struct JointBuilderHandle *builder,
                                                 Bool wake_up);

Bool world_remove_impulse_joint(struct WorldHandle *world,
                                ImpulseJointHandleRaw handle,
                                Bool wake_up);

double molecular_lennard_jones_potential(double distance, double epsilon, double sigma);

Bool molecular_lennard_jones_force(Vec3 displacement,
                                   double epsilon,
                                   double sigma,
                                   double softening,
                                   Vec3 *out_force);

double molecular_coulomb_potential(double distance,
                                   double charge_a,
                                   double charge_b,
                                   double coulomb_constant,
                                   double relative_permittivity);

Bool molecular_coulomb_force(Vec3 displacement,
                             double charge_a,
                             double charge_b,
                             double coulomb_constant,
                             double relative_permittivity,
                             double softening,
                             Vec3 *out_force);

Bool molecular_pair_interaction(MolecularParticle particle_a,
                                MolecularParticle particle_b,
                                MolecularForceLaw law,
                                MolecularPairReport *out_report);

Bool molecular_apply_pair_forces(struct WorldHandle *world,
                                 RigidBodyHandleRaw body_a,
                                 RigidBodyHandleRaw body_b,
                                 MolecularParticle particle_a,
                                 MolecularParticle particle_b,
                                 MolecularForceLaw law,
                                 Bool wake_up,
                                 MolecularPairReport *out_report);

uint8_t molecular_apply_pair_forces_flag(struct WorldHandle *world,
                                         RigidBodyHandleRaw body_a,
                                         RigidBodyHandleRaw body_b,
                                         MolecularParticle particle_a,
                                         MolecularParticle particle_b,
                                         MolecularForceLaw law,
                                         Bool wake_up,
                                         MolecularPairReport *out_report);

double molecular_vacuum_coulomb_constant(void);

uint32_t neural_bounds_required_weight_count(uint32_t hidden_width, uint32_t hidden_layers);

struct ColliderBuilderHandle *collider_builder_create_neural_bounds(NeuralBoundsDesc desc,
                                                                    const double *weights,
                                                                    uint32_t weight_count);

uint32_t query_intersect_neural_bounds_count(const struct WorldHandle *world,
                                             NeuralBoundsDesc desc,
                                             const double *weights,
                                             uint32_t weight_count,
                                             QueryFilterDesc filter);

uint32_t query_intersect_neural_bounds_count_all(const struct WorldHandle *world,
                                                 NeuralBoundsDesc desc,
                                                 const double *weights,
                                                 uint32_t weight_count);

uint32_t query_intersect_neural_bounds(const struct WorldHandle *world,
                                       NeuralBoundsDesc desc,
                                       const double *weights,
                                       uint32_t weight_count,
                                       QueryFilterDesc filter,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

uint32_t query_intersect_neural_bounds_all(const struct WorldHandle *world,
                                           NeuralBoundsDesc desc,
                                           const double *weights,
                                           uint32_t weight_count,
                                           ColliderHandleRaw *out_handles,
                                           uint32_t capacity);

RayHit query_cast_ray(const struct WorldHandle *world,
                      Vec3 origin,
                      Vec3 direction,
                      double max_toi,
                      Bool solid,
                      QueryFilterDesc filter);

ColliderHandleRaw query_cast_ray_out(const struct WorldHandle *world,
                                     Vec3 origin,
                                     Vec3 direction,
                                     double max_toi,
                                     Bool solid,
                                     QueryFilterDesc filter,
                                     RayHit *out_hit);

uint32_t query_cast_rays(const struct WorldHandle *world,
                         const double *rays,
                         uint32_t ray_count,
                         double max_toi,
                         Bool solid,
                         QueryFilterDesc filter,
                         RayHit *out_hits,
                         uint32_t capacity);

PointProjection query_project_point(const struct WorldHandle *world,
                                    Vec3 point,
                                    double max_dist,
                                    Bool solid,
                                    QueryFilterDesc filter,
                                    ColliderHandleRaw *out_collider);

ColliderHandleRaw query_project_point_out(const struct WorldHandle *world,
                                          Vec3 point,
                                          double max_dist,
                                          Bool solid,
                                          QueryFilterDesc filter,
                                          ColliderHandleRaw *out_collider,
                                          PointProjection *out_projection);

uint32_t query_intersect_point_count(const struct WorldHandle *world,
                                     Vec3 point,
                                     QueryFilterDesc filter);

uint32_t query_intersect_aabb_count(const struct WorldHandle *world,
                                    AabbDesc aabb,
                                    QueryFilterDesc filter);

uint32_t query_intersect_aabb(const struct WorldHandle *world,
                              AabbDesc aabb,
                              QueryFilterDesc filter,
                              ColliderHandleRaw *out_handles,
                              uint32_t capacity);

uint32_t query_intersect_aabb_count_all(const struct WorldHandle *world, AabbDesc aabb);

uint32_t query_intersect_aabb_counts(const struct WorldHandle *world,
                                     const AabbDesc *aabbs,
                                     uint32_t query_count,
                                     QueryFilterDesc filter,
                                     uint32_t *out_counts,
                                     uint32_t capacity);

uint32_t query_intersect_obb_count(const struct WorldHandle *world,
                                   Obb obb,
                                   QueryFilterDesc filter);

uint32_t query_intersect_obb_count_all(const struct WorldHandle *world, Obb obb);

uint32_t query_intersect_obb_counts(const struct WorldHandle *world,
                                    const Obb *obbs,
                                    uint32_t query_count,
                                    QueryFilterDesc filter,
                                    uint32_t *out_counts,
                                    uint32_t capacity);

uint32_t query_intersect_obb(const struct WorldHandle *world,
                             Obb obb,
                             QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

uint32_t query_intersect_obb_all(const struct WorldHandle *world,
                                 Obb obb,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_sphere_count(const struct WorldHandle *world,
                                      Sphere sphere,
                                      QueryFilterDesc filter);

uint32_t query_intersect_sphere_count_all(const struct WorldHandle *world, Sphere sphere);

uint32_t query_intersect_sphere_counts(const struct WorldHandle *world,
                                       const Sphere *spheres,
                                       uint32_t query_count,
                                       QueryFilterDesc filter,
                                       uint32_t *out_counts,
                                       uint32_t capacity);

uint32_t query_intersect_sphere(const struct WorldHandle *world,
                                Sphere sphere,
                                QueryFilterDesc filter,
                                ColliderHandleRaw *out_handles,
                                uint32_t capacity);

uint32_t query_intersect_sphere_all(const struct WorldHandle *world,
                                    Sphere sphere,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

uint32_t query_intersect_aabb_rigid_body_count_all(const struct WorldHandle *world, AabbDesc aabb);

uint32_t query_intersect_aabb_rigid_bodies_all(const struct WorldHandle *world,
                                               AabbDesc aabb,
                                               RigidBodyHandleRaw *out_handles,
                                               uint32_t capacity);

ShapeCastHit query_cast_shape(const struct WorldHandle *world,
                              ShapeDesc shape_desc,
                              Vec3 translation,
                              Quat rotation,
                              Vec3 velocity,
                              ShapeCastOptionsDesc options,
                              QueryFilterDesc filter);

ColliderHandleRaw query_cast_shape_out(const struct WorldHandle *world,
                                       ShapeDesc shape_desc,
                                       Vec3 translation,
                                       Quat rotation,
                                       Vec3 velocity,
                                       ShapeCastOptionsDesc options,
                                       QueryFilterDesc filter,
                                       ShapeCastHit *out_hit);

struct RigidBodyBuilderHandle *rigid_body_builder_create(uint32_t status);

RigidBody *rigid_body_builder_build(struct RigidBodyBuilderHandle *builder);

void rigid_body_builder_destroy(struct RigidBodyBuilderHandle *builder);

void rigid_body_destroy_raw(RigidBody *rigid_body);

void rigid_body_builder_set_translation(struct RigidBodyBuilderHandle *builder, Vec3 translation);

void rigid_body_builder_set_rotation(struct RigidBodyBuilderHandle *builder,
                                     Vec3 rotation_axis_angle);

void rigid_body_builder_set_pose(struct RigidBodyBuilderHandle *builder,
                                 Vec3 translation,
                                 Quat rotation);

void rigid_body_builder_set_additional_mass_properties(struct RigidBodyBuilderHandle *builder,
                                                       Vec3 center,
                                                       double mass,
                                                       Vec3 inertia);

void rigid_body_builder_set_linvel(struct RigidBodyBuilderHandle *builder, Vec3 linvel);

void rigid_body_builder_set_angvel(struct RigidBodyBuilderHandle *builder, Vec3 angvel);

void rigid_body_builder_set_gravity_scale(struct RigidBodyBuilderHandle *builder,
                                          double gravity_scale);

void rigid_body_builder_set_linear_damping(struct RigidBodyBuilderHandle *builder,
                                           double linear_damping);

void rigid_body_builder_set_angular_damping(struct RigidBodyBuilderHandle *builder,
                                            double angular_damping);

void rigid_body_builder_set_can_sleep(struct RigidBodyBuilderHandle *builder, Bool can_sleep);

void rigid_body_builder_set_enabled_rotations(struct RigidBodyBuilderHandle *builder,
                                              Bool allow_x,
                                              Bool allow_y,
                                              Bool allow_z);

void rigid_body_builder_set_user_data(struct RigidBodyBuilderHandle *builder,
                                      uint64_t user_data_low,
                                      uint64_t user_data_high);

void rigid_body_builder_set_additional_mass(struct RigidBodyBuilderHandle *builder, double mass);

RigidBodyHandleRaw world_insert_rigid_body(struct WorldHandle *world, RigidBody *memory_handle);

Bool world_remove_rigid_body(struct WorldHandle *world,
                             RigidBodyHandleRaw handle,
                             Bool remove_attached_colliders);

RigidBody *world_copy_rigid_body(struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t world_remove_rigid_body_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Bool remove_attached_colliders);

uint32_t rigid_body_get_status(const struct WorldHandle *world, RigidBodyHandleRaw handle);

Bool rigid_body_set_status(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           uint32_t status,
                           Bool wake_up);

Vec3 rigid_body_get_translation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_translation_out(const struct WorldHandle *world,
                                    RigidBodyHandleRaw handle,
                                    Vec3 *out_translation);

Quat rigid_body_get_rotation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_rotation_out(const struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 Quat *out_rotation);

Bool rigid_body_set_pose(struct WorldHandle *world,
                         RigidBodyHandleRaw handle,
                         Vec3 translation,
                         Quat rotation,
                         Bool wake_up);

Bool rigid_body_set_translation(struct WorldHandle *world,
                                RigidBodyHandleRaw handle,
                                Vec3 translation,
                                Bool wake_up);

uint8_t rigid_body_set_translation_flag(struct WorldHandle *world,
                                        RigidBodyHandleRaw handle,
                                        Vec3 translation,
                                        Bool wake_up);

Bool rigid_body_set_rotation(struct WorldHandle *world,
                             RigidBodyHandleRaw handle,
                             Quat rotation,
                             Bool wake_up);

uint8_t rigid_body_set_rotation_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Quat rotation,
                                     Bool wake_up);

uint8_t rigid_body_set_pose_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 Vec3 translation,
                                 Quat rotation,
                                 Bool wake_up);

double rigid_body_get_mass(struct WorldHandle *world, RigidBodyHandleRaw handle);

Vec3 rigid_body_get_force(const struct WorldHandle *world, RigidBodyHandleRaw handle);

Vec3 rigid_body_get_linvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_linvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               Vec3 *out_linvel);

Bool rigid_body_set_linvel(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 linvel,
                           Bool wake_up);

uint8_t rigid_body_set_linvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 linvel,
                                   Bool wake_up);

Vec3 rigid_body_get_angvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_angvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               Vec3 *out_angvel);

Bool rigid_body_set_angvel(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 angvel,
                           Bool wake_up);

uint8_t rigid_body_set_angvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 angvel,
                                   Bool wake_up);

Bool rigid_body_add_force(struct WorldHandle *world,
                          RigidBodyHandleRaw handle,
                          Vec3 force,
                          Bool wake_up);

Bool rigid_body_add_force_at_point(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 force,
                                   Vec3 point,
                                   Bool wake_up);

Bool rigid_body_reset_force(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool wake_up);

uint8_t rigid_body_add_force_flag(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  Vec3 force,
                                  Bool wake_up);

Bool rigid_body_add_torque(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 torque,
                           Bool wake_up);

Bool rigid_body_reset_torque(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool wake_up);

uint8_t rigid_body_add_torque_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 torque,
                                   Bool wake_up);

Bool rigid_body_apply_impulse(struct WorldHandle *world,
                              RigidBodyHandleRaw handle,
                              Vec3 impulse,
                              Bool wake_up);

uint8_t rigid_body_apply_impulse_flag(struct WorldHandle *world,
                                      RigidBodyHandleRaw handle,
                                      Vec3 impulse,
                                      Bool wake_up);

Bool rigid_body_apply_torque_impulse(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Vec3 torque_impulse,
                                     Bool wake_up);

uint8_t rigid_body_apply_torque_impulse_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw handle,
                                             Vec3 torque_impulse,
                                             Bool wake_up);

Bool rigid_body_enable_ccd(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool enabled);

uint8_t rigid_body_enable_ccd_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Bool enabled);

Bool rigid_body_sleep(struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t rigid_body_sleep_flag(struct WorldHandle *world, RigidBodyHandleRaw handle);

Bool rigid_body_wake_up(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool strong);

uint8_t rigid_body_wake_up_flag(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool strong);

Bool rigid_body_is_sleeping(const struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t rigid_body_is_sleeping_flag(const struct WorldHandle *world, RigidBodyHandleRaw handle);

struct RTreeHandle *rtree_create(void);

void rtree_destroy(struct RTreeHandle *tree);

void rtree_clear(struct RTreeHandle *tree);

uint32_t rtree_len(const struct RTreeHandle *tree);

Bool rtree_insert(struct RTreeHandle *tree, uint64_t id, AabbDesc aabb);

Bool rtree_update(struct RTreeHandle *tree, uint64_t id, AabbDesc aabb);

Bool rtree_remove(struct RTreeHandle *tree, uint64_t id);

void rtree_rebuild(struct RTreeHandle *tree);

uint32_t rtree_query_aabb_count(struct RTreeHandle *tree, AabbDesc aabb);

uint32_t rtree_query_aabb(struct RTreeHandle *tree,
                          AabbDesc aabb,
                          uint64_t *out_ids,
                          uint32_t capacity);

double space_kepler_period(double mu, double semi_major_axis);

double space_kepler_semi_major_axis(double mu, double period);

Bool space_elements_to_state(OrbitalElements elements, double mu, StateVector *out_state);

Bool space_state_to_elements(StateVector state, double mu, OrbitalElements *out_elements);

Bool space_j2_acceleration(Vec3 position,
                           double mu,
                           double equatorial_radius,
                           double j2,
                           Vec3 *out_acceleration);

Bool space_apply_j2_force_to_body(struct WorldHandle *world,
                                  RigidBodyHandleRaw body_handle,
                                  double mu,
                                  double equatorial_radius,
                                  double j2,
                                  double mass,
                                  Bool wake_up,
                                  Vec3 *out_acceleration);

uint8_t space_apply_j2_force_to_body_flag(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          double mu,
                                          double equatorial_radius,
                                          double j2,
                                          double mass,
                                          Bool wake_up,
                                          Vec3 *out_acceleration);

Bool space_quaternion_derivative(Quat attitude,
                                 Vec3 angular_velocity,
                                 QuaternionDerivative *out_derivative);

Bool space_rigid_body_euler_derivative(Vec3 inertia_diag,
                                       Vec3 angular_velocity,
                                       Vec3 torque,
                                       RigidBodyEulerDerivative *out_derivative);

Bool space_cmg_exchange(Vec3 gimbal_axis,
                        Vec3 wheel_momentum,
                        double gimbal_rate,
                        CmgExchange *out_exchange);

Bool space_apply_cmg_torque_to_body(struct WorldHandle *world,
                                    RigidBodyHandleRaw body_handle,
                                    Vec3 gimbal_axis,
                                    Vec3 wheel_momentum,
                                    double gimbal_rate,
                                    Bool wake_up,
                                    CmgExchange *out_exchange);

uint8_t space_apply_cmg_torque_to_body_flag(struct WorldHandle *world,
                                            RigidBodyHandleRaw body_handle,
                                            Vec3 gimbal_axis,
                                            Vec3 wheel_momentum,
                                            double gimbal_rate,
                                            Bool wake_up,
                                            CmgExchange *out_exchange);

Bool space_cw_derivative(CwState state, double mean_motion, CwDerivative *out_derivative);

double space_lambert_time_elliptic(double mu,
                                   double semi_major_axis,
                                   double alpha,
                                   double beta,
                                   uint32_t revolutions);

Bool space_dh_transform(double theta, double d, double a, double alpha, DhTransform *out_transform);

double space_arm_first_joint_inverse(double wrist_x, double wrist_y);

double space_arm_third_joint_angle(double planar_radius,
                                   double vertical_offset,
                                   double link2,
                                   double link3,
                                   Bool elbow_up);

Bool space_manipulator_dynamics_diag(Vec3 mass_matrix_diag,
                                     Vec3 joint_acceleration,
                                     Vec3 coriolis,
                                     Vec3 gravity,
                                     ManipulatorDynamics *out_dynamics);

Bool space_solar_panel_power(double solar_flux,
                             double area,
                             double efficiency,
                             double incidence_angle,
                             double degradation,
                             SolarPanelPower *out_power);

Bool space_thermal_balance(double absorbed_power,
                           double internal_power,
                           double emitted_area,
                           double emissivity,
                           ThermalBalance *out_balance);

Bool space_co2_mass_balance(double current_mass,
                            double generation_rate,
                            double removal_rate,
                            double leakage_rate,
                            double volume,
                            double dt,
                            Co2MassBalance *out_balance);

Bool space_friis_link(double transmit_power,
                      double transmit_gain,
                      double receive_gain,
                      double wavelength,
                      double range,
                      double system_loss,
                      FriisLink *out_link);

double space_friis_wavelength_from_frequency(double frequency);

double space_tsiolkovsky_delta_v(double specific_impulse,
                                 double standard_gravity,
                                 double initial_mass,
                                 double final_mass);

Bool space_hohmann_transfer(double mu,
                            double radius1,
                            double radius2,
                            HohmannTransfer *out_transfer);

double space_atmospheric_density_scale_height(double reference_density,
                                              double altitude,
                                              double reference_altitude,
                                              double scale_height);

Bool space_atmospheric_drag_acceleration(Vec3 velocity,
                                         Vec3 atmosphere_velocity,
                                         double density,
                                         double drag_coefficient,
                                         double area,
                                         double mass,
                                         Vec3 *out_acceleration);

Bool space_apply_atmospheric_drag_to_body(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          Vec3 atmosphere_velocity,
                                          double density,
                                          double drag_coefficient,
                                          double area,
                                          double mass,
                                          Bool wake_up,
                                          Vec3 *out_acceleration);

uint8_t space_apply_atmospheric_drag_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 atmosphere_velocity,
                                                  double density,
                                                  double drag_coefficient,
                                                  double area,
                                                  double mass,
                                                  Bool wake_up,
                                                  Vec3 *out_acceleration);

Bool space_triad_attitude(Vec3 body_primary,
                          Vec3 body_secondary,
                          Vec3 reference_primary,
                          Vec3 reference_secondary,
                          Quat *out_attitude);

Bool space_ekf_predict_scalar(double state,
                              double covariance,
                              double nonlinear_delta,
                              double jacobian,
                              double process_noise,
                              ScalarKalman *out_prediction);

double space_ekf_gain_scalar(double covariance,
                             double measurement_jacobian,
                             double measurement_noise);

Bool space_ekf_update_scalar(double predicted_state,
                             double predicted_covariance,
                             double measurement,
                             double predicted_measurement,
                             double kalman_gain,
                             double measurement_jacobian,
                             ScalarKalman *out_update);

Bool space_least_squares_attitude_two_vector(Vec3 body_primary,
                                             Vec3 body_secondary,
                                             Vec3 reference_primary,
                                             Vec3 reference_secondary,
                                             LeastSquaresAttitude *out_attitude);

Bool space_gnss_pseudorange(Vec3 receiver,
                            Vec3 satellite,
                            double receiver_clock_bias,
                            double satellite_clock_bias,
                            double ionosphere_delay,
                            double troposphere_delay,
                            GnssObservation *out_observation);

double space_gnss_double_difference_carrier_phase(double range_rover_sat_a,
                                                  double range_rover_sat_b,
                                                  double range_base_sat_a,
                                                  double range_base_sat_b,
                                                  double wavelength,
                                                  double ambiguity);

double space_structural_natural_frequency(double stiffness, double mass, double mode_factor);

Bool space_contact_force_hunt_crossley(double penetration,
                                       double penetration_rate,
                                       double stiffness,
                                       double damping,
                                       double exponent,
                                       ContactForceModel *out_force);

double space_radiation_absorbed_dose(double energy_joules, double mass_kg, double quality_factor);

double space_semi_major_axis_decay_rate(double semi_major_axis,
                                        double density,
                                        double drag_coefficient,
                                        double area,
                                        double mass,
                                        double mu);

double space_heat_pipe_thermal_resistance(double evaporator_resistance,
                                          double vapor_resistance,
                                          double condenser_resistance,
                                          double wick_resistance);

Bool space_battery_equivalent_circuit(double open_circuit_voltage,
                                      double current,
                                      double ohmic_resistance,
                                      double rc_voltage,
                                      double rc_resistance,
                                      double rc_capacitance,
                                      double capacity_coulombs,
                                      BatteryEquivalentCircuit *out_battery);

Bool space_hall_thruster_performance(double mass_flow_rate,
                                     double exhaust_velocity,
                                     double input_power,
                                     double standard_gravity,
                                     HallThrusterPerformance *out_performance);

Bool space_artificial_potential_guidance(Vec3 position,
                                         Vec3 target,
                                         Vec3 obstacle,
                                         double attractive_gain,
                                         double repulsive_gain,
                                         double influence_radius,
                                         Vec3 *out_command);

Bool space_debris_collision_probability(double miss_distance,
                                        double combined_radius,
                                        double sigma_radial,
                                        double sigma_intrack,
                                        CollisionProbability *out_probability);

Bool space_atomic_oxygen_erosion(double fluence,
                                 double erosion_yield,
                                 double area,
                                 double density,
                                 AtomicOxygenErosion *out_erosion);

Bool space_flexible_mode_derivative(double displacement,
                                    double velocity,
                                    double natural_frequency,
                                    double damping_ratio,
                                    double modal_force,
                                    double modal_mass,
                                    FlexibleModeDerivative *out_derivative);

Bool space_slosh_pendulum_derivative(double angle,
                                     double angular_rate,
                                     double length,
                                     double damping,
                                     double lateral_acceleration,
                                     double gravity,
                                     SloshPendulumDerivative *out_derivative);

Bool space_variational_two_body(Vec3 position,
                                Vec3 velocity,
                                double mu,
                                VariationalState *out_derivative);

Bool space_single_phase_loop_heat_transfer(double mass_flow_rate,
                                           double specific_heat,
                                           double inlet_temperature,
                                           double heat_input,
                                           FluidLoopHeatTransfer *out_heat);

Bool space_radar_range_rate(Vec3 radar_position,
                            Vec3 target_position,
                            Vec3 radar_velocity,
                            Vec3 target_velocity,
                            RadarMeasurement *out_measurement);

Bool space_mass_properties_two_body(double mass1,
                                    Vec3 position1,
                                    Vec3 inertia1_diag,
                                    double mass2,
                                    Vec3 position2,
                                    Vec3 inertia2_diag,
                                    MassProperties *out_properties);

double space_docking_buffer_energy(double relative_speed,
                                   double reduced_mass,
                                   double stroke,
                                   double efficiency);

Bool space_bang_off_bang_profile(double angle,
                                 double max_acceleration,
                                 double max_rate,
                                 BangOffBangProfile *out_profile);

Bool space_solar_radiation_pressure_acceleration(Vec3 sun_direction,
                                                 double solar_flux,
                                                 double reflectivity,
                                                 double area,
                                                 double mass,
                                                 Vec3 *out_acceleration);

Bool space_apply_solar_radiation_pressure_to_body(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 sun_direction,
                                                  double solar_flux,
                                                  double reflectivity,
                                                  double area,
                                                  double mass,
                                                  Bool wake_up,
                                                  Vec3 *out_acceleration);

uint8_t space_apply_solar_radiation_pressure_to_body_flag(struct WorldHandle *world,
                                                          RigidBodyHandleRaw body_handle,
                                                          Vec3 sun_direction,
                                                          double solar_flux,
                                                          double reflectivity,
                                                          double area,
                                                          double mass,
                                                          Bool wake_up,
                                                          Vec3 *out_acceleration);

Bool space_gravity_gradient_torque(Vec3 position, Vec3 inertia_diag, double mu, Vec3 *out_torque);

Bool space_apply_gravity_gradient_torque_to_body(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body_handle,
                                                 Vec3 inertia_diag,
                                                 double mu,
                                                 Bool wake_up,
                                                 Vec3 *out_torque);

uint8_t space_apply_gravity_gradient_torque_to_body_flag(struct WorldHandle *world,
                                                         RigidBodyHandleRaw body_handle,
                                                         Vec3 inertia_diag,
                                                         double mu,
                                                         Bool wake_up,
                                                         Vec3 *out_torque);

Bool space_magnetic_torquer_dipole(Vec3 commanded_torque,
                                   Vec3 magnetic_field,
                                   double max_dipole,
                                   Vec3 *out_dipole);

Bool space_apply_magnetic_torquer_to_body(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          Vec3 commanded_torque,
                                          Vec3 magnetic_field,
                                          double max_dipole,
                                          Bool wake_up,
                                          Vec3 *out_dipole);

uint8_t space_apply_magnetic_torquer_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 commanded_torque,
                                                  Vec3 magnetic_field,
                                                  double max_dipole,
                                                  Bool wake_up,
                                                  Vec3 *out_dipole);

Bool space_cmg_robust_pseudoinverse_diag(Vec3 jacobian_diag,
                                         Vec3 desired_torque,
                                         double damping,
                                         CmgRobustInverse *out_inverse);

Bool space_sgp4_j2_secular_rates(double semi_major_axis,
                                 double eccentricity,
                                 double inclination,
                                 double mean_motion,
                                 double equatorial_radius,
                                 double j2,
                                 Sgp4SecularRates *out_rates);

double space_docking_glideslope_command(double range,
                                        double desired_slope,
                                        double closing_speed_limit);

double space_sagnac_phase_rate(double area, double angular_rate, double wavelength);

double space_solar_array_pd_torque(double angle_error, double rate_error, double kp, double kd);

Bool space_sabatier_methane_rate(double co2_molar_rate,
                                 double h2_molar_rate,
                                 double conversion,
                                 ChemicalReactionRate *out_rate);

Bool space_spe_oxygen_rate(double current,
                           double cells,
                           double faraday_efficiency,
                           ChemicalReactionRate *out_rate);

Bool space_radiator_power(double area,
                          double emissivity,
                          double temperature,
                          double sink_temperature,
                          double absorbed_power,
                          RadiatorPower *out_power);

double space_whipple_critical_projectile_diameter(double bumper_thickness,
                                                  double bumper_density,
                                                  double projectile_density,
                                                  double impact_velocity,
                                                  double standoff);

double space_surface_charging_current_balance(double photo_current,
                                              double secondary_current,
                                              double backscatter_current,
                                              double electron_current,
                                              double ion_current);

Bool space_airlock_depressurization(double pressure,
                                    double ambient_pressure,
                                    double volume,
                                    double conductance,
                                    double dt,
                                    AirlockDepressurization *out_state);

/**
 * Compute polyhedron gravity.
 *
 * `vertices_xyz` — flat array of vertex positions (3×n_verts f64s)
 * `face_indices` — flat array of triangle indices (3×n_faces u32s)
 * `density` — constant density (kg/m³)
 */
Bool terrain_polyhedron_gravity(Vec3 position,
                                const double *vertices_xyz,
                                uint32_t n_vertices,
                                const uint32_t *face_indices,
                                uint32_t n_faces,
                                double density,
                                Vec3 *out_acceleration);

/**
 * Compute terrain gravity from DEM (direct summation method).
 */
Bool terrain_gravity_dem(Vec3 position,
                         const double *dem,
                         uint32_t nx,
                         uint32_t ny,
                         double resolution,
                         double reference_radius,
                         double surface_density,
                         Vec3 *out_acceleration);

/**
 * Compute terrain gravity from DEM (FFT/quadrupole approximation).
 */
Bool terrain_gravity_dem_fft(Vec3 position,
                             const double *dem,
                             uint32_t nx,
                             uint32_t ny,
                             double resolution,
                             double reference_radius,
                             double surface_density,
                             Vec3 *out_acceleration);

/**
 * Compute lunar mascon gravitational acceleration.
 */
Bool terrain_lunar_mascon_gravity(Vec3 position, Vec3 *out_acceleration);

/**
 * Get the number of built-in lunar mascons.
 */
uint32_t terrain_lunar_mascon_count(void);

/**
 * Get a specific lunar mascon by index.
 */
Bool terrain_lunar_mascon_get(uint32_t index, struct LunarMascon *out_mascon);

Bool trajectory_estimate_forces(TrajectoryState state,
                                TrajectoryEnvironment env,
                                TrajectoryForceReport *out_report);

Bool trajectory_integrate_step(TrajectoryState state,
                               TrajectoryEnvironment env,
                               double dt,
                               TrajectoryState *out_state,
                               TrajectoryForceReport *out_report);

Bool trajectory_apply_forces_to_body(struct WorldHandle *world,
                                     RigidBodyHandleRaw body_handle,
                                     TrajectoryEnvironment env,
                                     Bool wake_up,
                                     TrajectoryForceReport *out_report);

uint8_t trajectory_apply_forces_to_body_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw body_handle,
                                             TrajectoryEnvironment env,
                                             Bool wake_up,
                                             TrajectoryForceReport *out_report);

Bool trajectory_glide_estimate(TrajectoryGlideState state,
                               TrajectoryGlideEnvironment env,
                               TrajectoryGlideReport *out_report);

Bool trajectory_glide_integrate_step(TrajectoryGlideState state,
                                     TrajectoryGlideEnvironment env,
                                     double dt,
                                     TrajectoryGlideState *out_state,
                                     TrajectoryGlideReport *out_report);

struct ColliderBuilderHandle *collider_builder_create_voxels(const uint8_t *voxels,
                                                             uint32_t size_x,
                                                             uint32_t size_y,
                                                             uint32_t size_z,
                                                             double voxel_size_x,
                                                             double voxel_size_y,
                                                             double voxel_size_z,
                                                             Vec3 origin,
                                                             VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxels_auto(const uint8_t *voxels,
                                                                  uint32_t size_x,
                                                                  uint32_t size_y,
                                                                  uint32_t size_z,
                                                                  double voxel_size_x,
                                                                  double voxel_size_y,
                                                                  double voxel_size_z,
                                                                  Vec3 origin,
                                                                  Bool dynamic_body);

VoxelBuildStats voxel_build_stats(const uint8_t *voxels,
                                  uint32_t size_x,
                                  uint32_t size_y,
                                  uint32_t size_z,
                                  double voxel_size_x,
                                  double voxel_size_y,
                                  double voxel_size_z,
                                  Vec3 origin,
                                  VoxelColliderOptions options);

VoxelBuildStats voxel_aabb_build_stats(AabbDesc aabb,
                                       double voxel_size_x,
                                       double voxel_size_y,
                                       double voxel_size_z,
                                       VoxelColliderOptions options);

VoxelBuildStats voxel_obb_build_stats(Obb obb,
                                      double voxel_size_x,
                                      double voxel_size_y,
                                      double voxel_size_z,
                                      VoxelColliderOptions options);

void voxel_aabb_build_stats_out(AabbDesc aabb,
                                double voxel_size_x,
                                double voxel_size_y,
                                double voxel_size_z,
                                VoxelColliderOptions options,
                                VoxelBuildStats *out_stats);

void voxel_obb_build_stats_out(Obb obb,
                               double voxel_size_x,
                               double voxel_size_y,
                               double voxel_size_z,
                               VoxelColliderOptions options,
                               VoxelBuildStats *out_stats);

struct ColliderBuilderHandle *collider_builder_create_voxel_aabb(AabbDesc aabb,
                                                                 double voxel_size_x,
                                                                 double voxel_size_y,
                                                                 double voxel_size_z,
                                                                 VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxel_aabb_auto(AabbDesc aabb,
                                                                      double voxel_size_x,
                                                                      double voxel_size_y,
                                                                      double voxel_size_z,
                                                                      Bool dynamic_body);

struct ColliderBuilderHandle *collider_builder_create_voxel_obb(Obb obb,
                                                                double voxel_size_x,
                                                                double voxel_size_y,
                                                                double voxel_size_z,
                                                                VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxel_obb_auto(Obb obb,
                                                                     double voxel_size_x,
                                                                     double voxel_size_y,
                                                                     double voxel_size_z,
                                                                     Bool dynamic_body);

uint32_t query_intersect_voxel_aabb(const struct WorldHandle *world,
                                    AabbDesc aabb,
                                    QueryFilterDesc filter,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

uint32_t query_intersect_voxel_aabb_count(const struct WorldHandle *world,
                                          AabbDesc aabb,
                                          QueryFilterDesc filter);

uint32_t query_intersect_voxel_obb(const struct WorldHandle *world,
                                   Obb obb,
                                   QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_voxel_obb_count(const struct WorldHandle *world,
                                         Obb obb,
                                         QueryFilterDesc filter);

RigidBodyHandleRaw world_insert_static_voxel_aabb(struct WorldHandle *world,
                                                  AabbDesc aabb,
                                                  double voxel_size_x,
                                                  double voxel_size_y,
                                                  double voxel_size_z,
                                                  VoxelColliderOptions options,
                                                  double friction,
                                                  double restitution);

RigidBodyHandleRaw world_insert_dynamic_voxel_obb(struct WorldHandle *world,
                                                  Obb obb,
                                                  double voxel_size_x,
                                                  double voxel_size_y,
                                                  double voxel_size_z,
                                                  VoxelColliderOptions options,
                                                  double density,
                                                  double friction,
                                                  double restitution);

struct WorldHandle *world_create(Vec3 gravity);

void world_destroy(struct WorldHandle *world);

void world_step(struct WorldHandle *world, double delta_seconds);

Bool world_set_integration_parameters(struct WorldHandle *world,
                                      double dt,
                                      uint32_t solver_iterations,
                                      uint32_t ccd_substeps);

uint32_t world_get_integration_parameters(const struct WorldHandle *world,
                                          double *out_values,
                                          uint32_t capacity);

void world_set_gravity(struct WorldHandle *world, Vec3 gravity);

Vec3 world_get_gravity(const struct WorldHandle *world);

int32_t world_get_rigid_body_set_size(const struct WorldHandle *world);

int32_t world_get_collider_set_size(const struct WorldHandle *world);

void world_get_gravity_out(const struct WorldHandle *world, Vec3 *out_gravity);

uint32_t world_dynamic_body_snapshot_count(const struct WorldHandle *world);

uint32_t world_dynamic_body_snapshot(const struct WorldHandle *world,
                                     RigidBodyHandleRaw *out_handles,
                                     double *out_values,
                                     uint32_t capacity);

uint32_t world_body_snapshot_count(const struct WorldHandle *world);

uint32_t world_body_snapshot(const struct WorldHandle *world,
                             RigidBodyHandleRaw *out_handles,
                             double *out_values,
                             uint32_t capacity);

uint32_t world_update_body_poses(struct WorldHandle *world,
                                 const RigidBodyHandleRaw *handles,
                                 const double *values,
                                 uint32_t count,
                                 Bool wake_up);

uint32_t world_update_body_velocities(struct WorldHandle *world,
                                      const RigidBodyHandleRaw *handles,
                                      const double *values,
                                      uint32_t count,
                                      Bool wake_up);

/**
 * Register celestial body gravity as a ForceLaw in the world's registry.
 *
 * `body_id` maps to `CelestialBodyId` (0=Sun, 3=Earth, 4=Moon, 5=Mars, etc.).
 *
 * Returns handle (non-zero) on success, 0 on invalid body_id.
 */
uint64_t world_register_celestial_gravity(struct WorldHandle *world,
                                          uint32_t body_id,
                                          uint32_t max_degree);

uint32_t world_get_force_registry_count(const struct WorldHandle *world);

/**
 * Get count of registered force laws of a specific type.
 * `law_type` is the numeric discriminant of `ForceLawType`.
 */
uint32_t world_get_force_registry_typed_count(const struct WorldHandle *world, uint32_t law_type);

/**
 * Create a shared-memory physics arena.
 *
 * Returns the arena pointer as a u64 (suitable for `MemorySegment.ofAddress` in Java).
 * The arena persists for the lifetime of the world.
 *
 * `max_bodies` — max concurrent bodies to mirror
 * `max_events` — max pending collision/contact events
 * `max_commands` — max pending commands (force/set pose etc.)
 * `out_address` — receives the arena base address
 * `out_size` — receives the total arena size in bytes (for Java MemorySegment mapping)
 */
Bool world_create_shared_arena(struct WorldHandle *world,
                               uint32_t max_bodies,
                               uint32_t max_colliders,
                               uint32_t max_events,
                               uint32_t max_commands,
                               uint64_t *out_address,
                               uint64_t *out_size);

/**
 * Destroy the shared arena (if any).
 */
void world_destroy_shared_arena(struct WorldHandle *world);

/**
 * Get the arena address (returns 0 if no arena).
 */
uint64_t world_get_shared_arena_address(const struct WorldHandle *world);

/**
 * Get the arena size (returns 0 if no arena).
 */
uint64_t world_get_shared_arena_size(const struct WorldHandle *world);

/**
 * Reset the event ring (Java calls this after draining events).
 */
void world_reset_shared_arena_events(struct WorldHandle *world);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* RIGID_BODY_H */
