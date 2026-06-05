#ifndef RIGID_BODY_H
#define RIGID_BODY_H

#pragma once

/* Generated with cbindgen:0.29.3 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define RC_ABI_VERSION 1

typedef enum RcShapeType {
  Ball = 0,
  Cuboid = 1,
  CapsuleY = 2,
  CapsuleX = 3,
  CapsuleZ = 4,
  Cylinder = 5,
  RoundCylinder = 6,
  Cone = 7,
  RoundCone = 8,
  RoundCuboid = 9,
} RcShapeType;

typedef enum RcJointType {
  Fixed = 0,
  Revolute = 1,
  Prismatic = 2,
  Rope = 3,
  Spring = 4,
  Spherical = 5,
} RcJointType;

typedef enum RcJointAxis {
  LinX = 0,
  LinY = 1,
  LinZ = 2,
  AngX = 3,
  AngY = 4,
  AngZ = 5,
} RcJointAxis;

typedef enum RcBodyStatus {
  Dynamic = 0,
  Fixed = 1,
  KinematicPositionBased = 2,
  KinematicVelocityBased = 3,
} RcBodyStatus;

typedef struct RcCharacterControllerHandle RcCharacterControllerHandle;

typedef struct RcColliderBuilderHandle RcColliderBuilderHandle;

typedef struct RcJointBuilderHandle RcJointBuilderHandle;

typedef struct RcRigidBodyBuilderHandle RcRigidBodyBuilderHandle;

typedef struct RcWorldHandle RcWorldHandle;

typedef uint64_t RcRigidBodyHandle;

typedef struct RcVec3 {
  double x;
  double y;
  double z;
} RcVec3;

typedef struct RcQuat {
  double i;
  double j;
  double k;
  double w;
} RcQuat;

typedef struct RcInteractionGroups {
  uint32_t memberships;
  uint32_t filter;
} RcInteractionGroups;

typedef struct RcAabb {
  struct RcVec3 mins;
  struct RcVec3 maxs;
} RcAabb;

typedef struct RcBool {
  uint8_t _0;
} RcBool;
#define RcBool_FALSE (RcBool){ ._0 = 0 }
#define RcBool_TRUE (RcBool){ ._0 = 1 }

typedef uint64_t RcColliderHandle;

typedef struct RcQueryFilterDesc {
  uint32_t flags;
  struct RcInteractionGroups groups;
  struct RcBool use_groups;
  RcColliderHandle exclude_collider;
  struct RcBool use_exclude_collider;
  RcRigidBodyHandle exclude_rigid_body;
  struct RcBool use_exclude_rigid_body;
} RcQueryFilterDesc;

typedef struct RcShapeDesc {
  enum RcShapeType shape_type;
  double a;
  double b;
  double c;
  double d;
} RcShapeDesc;

typedef struct RcEffectiveCharacterMovement {
  struct RcVec3 translation;
  struct RcBool grounded;
  struct RcBool is_sliding_down_slope;
} RcEffectiveCharacterMovement;

typedef struct RcCharacterCollision {
  RcColliderHandle collider;
  struct RcVec3 character_translation;
  struct RcVec3 translation_applied;
  struct RcVec3 translation_remaining;
  struct RcVec3 world_witness1;
  struct RcVec3 world_witness2;
  struct RcVec3 normal1;
  struct RcVec3 normal2;
  double time_of_impact;
} RcCharacterCollision;

typedef struct RcCollisionEventRecord {
  struct RcBool started;
  RcColliderHandle collider1;
  RcColliderHandle collider2;
  struct RcBool sensor;
  struct RcBool removed;
} RcCollisionEventRecord;

typedef struct RcContactForceEventRecord {
  RcColliderHandle collider1;
  RcColliderHandle collider2;
  struct RcVec3 total_force;
  double total_force_magnitude;
  struct RcVec3 max_force_direction;
  double max_force_magnitude;
} RcContactForceEventRecord;

typedef uint32_t (*RcContactPairFilterCallback)(uintptr_t,
                                                RcColliderHandle,
                                                RcColliderHandle,
                                                struct RcBool,
                                                RcRigidBodyHandle,
                                                struct RcBool,
                                                RcRigidBodyHandle);

typedef struct RcBool (*RcIntersectionPairFilterCallback)(uintptr_t,
                                                          RcColliderHandle,
                                                          RcColliderHandle,
                                                          struct RcBool,
                                                          RcRigidBodyHandle,
                                                          struct RcBool,
                                                          RcRigidBodyHandle);

typedef uint64_t RcImpulseJointHandle;

typedef struct RcRayHit {
  RcColliderHandle collider;
  double time_of_impact;
  struct RcVec3 normal;
  uint32_t feature;
} RcRayHit;

typedef struct RcPointProjection {
  struct RcVec3 point;
  struct RcBool is_inside;
} RcPointProjection;

typedef struct RcShapeCastHit {
  RcColliderHandle collider;
  double time_of_impact;
  struct RcVec3 witness1;
  struct RcVec3 witness2;
  struct RcVec3 normal1;
  struct RcVec3 normal2;
  uint32_t status;
} RcShapeCastHit;

typedef struct RcShapeCastOptions {
  double max_time_of_impact;
  double target_distance;
  struct RcBool stop_at_penetration;
  struct RcBool compute_impact_geometry_on_penetration;
} RcShapeCastOptions;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

RcRigidBodyHandle rc_world_insert_dynamic_cuboids(struct RcWorldHandle *world,
                                                  struct RcVec3 translation,
                                                  struct RcQuat rotation,
                                                  struct RcVec3 linvel,
                                                  const double *cuboids,
                                                  uint32_t cuboid_count,
                                                  double density,
                                                  double friction,
                                                  double restitution,
                                                  struct RcInteractionGroups collision_groups,
                                                  struct RcInteractionGroups solver_groups);

RcRigidBodyHandle rc_world_insert_static_trimesh(struct RcWorldHandle *world,
                                                 const double *vertices_xyz,
                                                 uint32_t vertex_xyz_len,
                                                 const uint32_t *indices,
                                                 uint32_t index_len,
                                                 double friction,
                                                 double restitution);

uint32_t rc_query_intersect_aabb_rigid_body_count(const struct RcWorldHandle *world,
                                                  struct RcAabb aabb,
                                                  struct RcQueryFilterDesc filter);

uint32_t rc_query_intersect_aabb_rigid_bodies(const struct RcWorldHandle *world,
                                              struct RcAabb aabb,
                                              struct RcQueryFilterDesc filter,
                                              RcRigidBodyHandle *out_handles,
                                              uint32_t capacity);

uint32_t rc_abi_version(void);

struct RcBool rc_abi_supports_ffm(void);

struct RcBool rc_abi_supports_jni(void);

struct RcColliderBuilderHandle *rc_collider_builder_create(enum RcShapeType shape_type,
                                                           struct RcVec3 shape_data);

struct RcColliderBuilderHandle *rc_collider_builder_create_ex(struct RcShapeDesc shape_desc);

void rc_collider_builder_destroy(struct RcColliderBuilderHandle *builder);

void rc_collider_builder_set_translation(struct RcColliderBuilderHandle *builder,
                                         struct RcVec3 translation);

void rc_collider_builder_set_rotation(struct RcColliderBuilderHandle *builder,
                                      struct RcVec3 rotation_axis_angle);

void rc_collider_builder_set_pose(struct RcColliderBuilderHandle *builder,
                                  struct RcVec3 translation,
                                  struct RcQuat rotation);

void rc_collider_builder_set_sensor(struct RcColliderBuilderHandle *builder, struct RcBool sensor);

void rc_collider_builder_set_friction(struct RcColliderBuilderHandle *builder, double friction);

void rc_collider_builder_set_restitution(struct RcColliderBuilderHandle *builder,
                                         double restitution);

void rc_collider_builder_set_density(struct RcColliderBuilderHandle *builder, double density);

void rc_collider_builder_set_collision_groups(struct RcColliderBuilderHandle *builder,
                                              struct RcInteractionGroups groups);

void rc_collider_builder_set_solver_groups(struct RcColliderBuilderHandle *builder,
                                           struct RcInteractionGroups groups);

void rc_collider_builder_set_active_events(struct RcColliderBuilderHandle *builder,
                                           uint32_t active_events_bits);

void rc_collider_builder_set_active_hooks(struct RcColliderBuilderHandle *builder,
                                          uint32_t active_hooks_bits);

void rc_collider_builder_set_contact_force_event_threshold(struct RcColliderBuilderHandle *builder,
                                                           double threshold);

RcColliderHandle rc_world_insert_collider(struct RcWorldHandle *world,
                                          struct RcColliderBuilderHandle *builder);

RcColliderHandle rc_world_insert_collider_with_parent(struct RcWorldHandle *world,
                                                      struct RcColliderBuilderHandle *builder,
                                                      RcRigidBodyHandle parent);

struct RcBool rc_world_remove_collider(struct RcWorldHandle *world,
                                       RcColliderHandle handle,
                                       struct RcBool wake_up);

uint8_t rc_world_remove_collider_flag(struct RcWorldHandle *world,
                                      RcColliderHandle handle,
                                      struct RcBool wake_up);

struct RcVec3 rc_collider_get_translation(const struct RcWorldHandle *world,
                                          RcColliderHandle handle);

struct RcQuat rc_collider_get_rotation(const struct RcWorldHandle *world, RcColliderHandle handle);

struct RcBool rc_collider_set_pose(struct RcWorldHandle *world,
                                   RcColliderHandle handle,
                                   struct RcVec3 translation,
                                   struct RcQuat rotation);

struct RcBool rc_collider_set_sensor(struct RcWorldHandle *world,
                                     RcColliderHandle handle,
                                     struct RcBool sensor);

struct RcBool rc_collider_set_friction(struct RcWorldHandle *world,
                                       RcColliderHandle handle,
                                       double friction);

struct RcBool rc_collider_set_restitution(struct RcWorldHandle *world,
                                          RcColliderHandle handle,
                                          double restitution);

struct RcBool rc_collider_set_collision_groups(struct RcWorldHandle *world,
                                               RcColliderHandle handle,
                                               struct RcInteractionGroups groups);

struct RcBool rc_collider_set_solver_groups(struct RcWorldHandle *world,
                                            RcColliderHandle handle,
                                            struct RcInteractionGroups groups);

struct RcBool rc_collider_set_active_events(struct RcWorldHandle *world,
                                            RcColliderHandle handle,
                                            uint32_t active_events_bits);

struct RcBool rc_collider_set_active_hooks(struct RcWorldHandle *world,
                                           RcColliderHandle handle,
                                           uint32_t active_hooks_bits);

struct RcBool rc_collider_set_contact_force_event_threshold(struct RcWorldHandle *world,
                                                            RcColliderHandle handle,
                                                            double threshold);

double rc_collider_get_density(const struct RcWorldHandle *world, RcColliderHandle handle);

struct RcCharacterControllerHandle *rc_character_controller_create(void);

void rc_character_controller_destroy(struct RcCharacterControllerHandle *controller);

void rc_character_controller_set_up(struct RcCharacterControllerHandle *controller,
                                    struct RcVec3 up);

void rc_character_controller_set_offset_absolute(struct RcCharacterControllerHandle *controller,
                                                 double offset);

void rc_character_controller_set_offset_relative(struct RcCharacterControllerHandle *controller,
                                                 double offset);

void rc_character_controller_set_slide(struct RcCharacterControllerHandle *controller,
                                       struct RcBool slide);

void rc_character_controller_set_autostep(struct RcCharacterControllerHandle *controller,
                                          struct RcBool enabled,
                                          double max_height,
                                          double min_width,
                                          struct RcBool include_dynamic_bodies);

void rc_character_controller_set_snap_to_ground(struct RcCharacterControllerHandle *controller,
                                                struct RcBool enabled,
                                                double distance);

void rc_character_controller_set_slope_angles(struct RcCharacterControllerHandle *controller,
                                              double max_climb_angle,
                                              double min_slide_angle);

struct RcEffectiveCharacterMovement rc_character_controller_move_shape(const struct RcWorldHandle *world,
                                                                       struct RcCharacterControllerHandle *controller,
                                                                       double dt,
                                                                       struct RcShapeDesc shape_desc,
                                                                       struct RcVec3 translation,
                                                                       struct RcQuat rotation,
                                                                       struct RcVec3 desired_translation);

uint32_t rc_character_controller_collision_count(const struct RcCharacterControllerHandle *controller);

struct RcCharacterCollision rc_character_controller_get_collision(const struct RcCharacterControllerHandle *controller,
                                                                  uint32_t index);

struct RcBool rc_character_controller_solve_impulses(struct RcWorldHandle *world,
                                                     struct RcCharacterControllerHandle *controller,
                                                     double dt,
                                                     struct RcShapeDesc shape_desc,
                                                     double character_mass);

void rc_world_clear_events(struct RcWorldHandle *world);

uint32_t rc_world_collision_event_count(const struct RcWorldHandle *world);

struct RcCollisionEventRecord rc_world_get_collision_event(const struct RcWorldHandle *world,
                                                           uint32_t index);

uint32_t rc_world_contact_force_event_count(const struct RcWorldHandle *world);

struct RcContactForceEventRecord rc_world_get_contact_force_event(const struct RcWorldHandle *world,
                                                                  uint32_t index);

void rc_world_set_contact_pair_filter_callback(struct RcWorldHandle *world,
                                               RcContactPairFilterCallback callback,
                                               uintptr_t user_data);

void rc_world_set_intersection_pair_filter_callback(struct RcWorldHandle *world,
                                                    RcIntersectionPairFilterCallback callback,
                                                    uintptr_t user_data);

void rc_world_clear_contact_pair_filter_callback(struct RcWorldHandle *world);

void rc_world_clear_intersection_pair_filter_callback(struct RcWorldHandle *world);

struct RcJointBuilderHandle *rc_joint_builder_create(enum RcJointType joint_type,
                                                     struct RcVec3 axis_or_primary,
                                                     double b,
                                                     double c);

void rc_joint_builder_destroy(struct RcJointBuilderHandle *builder);

void rc_joint_builder_set_contacts_enabled(struct RcJointBuilderHandle *builder,
                                           struct RcBool enabled);

void rc_joint_builder_set_local_anchor1(struct RcJointBuilderHandle *builder, struct RcVec3 anchor);

void rc_joint_builder_set_local_anchor2(struct RcJointBuilderHandle *builder, struct RcVec3 anchor);

void rc_joint_builder_set_limits(struct RcJointBuilderHandle *builder,
                                 enum RcJointAxis axis,
                                 double min,
                                 double max);

void rc_joint_builder_set_motor_velocity(struct RcJointBuilderHandle *builder,
                                         enum RcJointAxis axis,
                                         double target_vel,
                                         double factor);

void rc_joint_builder_set_motor_position(struct RcJointBuilderHandle *builder,
                                         enum RcJointAxis axis,
                                         double target_pos,
                                         double stiffness,
                                         double damping);

RcImpulseJointHandle rc_world_insert_impulse_joint(struct RcWorldHandle *world,
                                                   RcRigidBodyHandle body1,
                                                   RcRigidBodyHandle body2,
                                                   struct RcJointBuilderHandle *builder,
                                                   struct RcBool wake_up);

struct RcBool rc_world_remove_impulse_joint(struct RcWorldHandle *world,
                                            RcImpulseJointHandle handle,
                                            struct RcBool wake_up);

struct RcRayHit rc_query_cast_ray(const struct RcWorldHandle *world,
                                  struct RcVec3 origin,
                                  struct RcVec3 direction,
                                  double max_toi,
                                  struct RcBool solid,
                                  struct RcQueryFilterDesc filter);

struct RcPointProjection rc_query_project_point(const struct RcWorldHandle *world,
                                                struct RcVec3 point,
                                                double max_dist,
                                                struct RcBool solid,
                                                struct RcQueryFilterDesc filter,
                                                RcColliderHandle *out_collider);

uint32_t rc_query_intersect_point_count(const struct RcWorldHandle *world,
                                        struct RcVec3 point,
                                        struct RcQueryFilterDesc filter);

uint32_t rc_query_intersect_aabb_count(const struct RcWorldHandle *world,
                                       struct RcAabb aabb,
                                       struct RcQueryFilterDesc filter);

uint32_t rc_query_intersect_aabb_count_all(const struct RcWorldHandle *world, struct RcAabb aabb);

uint32_t rc_query_intersect_aabb_rigid_body_count_all(const struct RcWorldHandle *world,
                                                      struct RcAabb aabb);

uint32_t rc_query_intersect_aabb_rigid_bodies_all(const struct RcWorldHandle *world,
                                                  struct RcAabb aabb,
                                                  RcRigidBodyHandle *out_handles,
                                                  uint32_t capacity);

struct RcShapeCastHit rc_query_cast_shape(const struct RcWorldHandle *world,
                                          struct RcShapeDesc shape_desc,
                                          struct RcVec3 translation,
                                          struct RcQuat rotation,
                                          struct RcVec3 velocity,
                                          struct RcShapeCastOptions options,
                                          struct RcQueryFilterDesc filter);

struct RcRigidBodyBuilderHandle *rc_rigid_body_builder_create(enum RcBodyStatus status);

void rc_rigid_body_builder_destroy(struct RcRigidBodyBuilderHandle *builder);

void rc_rigid_body_builder_set_translation(struct RcRigidBodyBuilderHandle *builder,
                                           struct RcVec3 translation);

void rc_rigid_body_builder_set_rotation(struct RcRigidBodyBuilderHandle *builder,
                                        struct RcVec3 rotation_axis_angle);

void rc_rigid_body_builder_set_pose(struct RcRigidBodyBuilderHandle *builder,
                                    struct RcVec3 translation,
                                    struct RcQuat rotation);

void rc_rigid_body_builder_set_linvel(struct RcRigidBodyBuilderHandle *builder,
                                      struct RcVec3 linvel);

void rc_rigid_body_builder_set_angvel(struct RcRigidBodyBuilderHandle *builder,
                                      struct RcVec3 angvel);

void rc_rigid_body_builder_set_gravity_scale(struct RcRigidBodyBuilderHandle *builder,
                                             double gravity_scale);

void rc_rigid_body_builder_set_linear_damping(struct RcRigidBodyBuilderHandle *builder,
                                              double linear_damping);

void rc_rigid_body_builder_set_angular_damping(struct RcRigidBodyBuilderHandle *builder,
                                               double angular_damping);

void rc_rigid_body_builder_set_can_sleep(struct RcRigidBodyBuilderHandle *builder,
                                         struct RcBool can_sleep);

void rc_rigid_body_builder_set_enabled_rotations(struct RcRigidBodyBuilderHandle *builder,
                                                 struct RcBool allow_x,
                                                 struct RcBool allow_y,
                                                 struct RcBool allow_z);

void rc_rigid_body_builder_set_user_data(struct RcRigidBodyBuilderHandle *builder,
                                         uint64_t user_data_low,
                                         uint64_t user_data_high);

void rc_rigid_body_builder_set_additional_mass(struct RcRigidBodyBuilderHandle *builder,
                                               double mass);

RcRigidBodyHandle rc_world_insert_rigid_body(struct RcWorldHandle *world,
                                             struct RcRigidBodyBuilderHandle *builder);

struct RcBool rc_world_remove_rigid_body(struct RcWorldHandle *world,
                                         RcRigidBodyHandle handle,
                                         struct RcBool remove_attached_colliders);

uint8_t rc_world_remove_rigid_body_flag(struct RcWorldHandle *world,
                                        RcRigidBodyHandle handle,
                                        struct RcBool remove_attached_colliders);

enum RcBodyStatus rc_rigid_body_get_status(const struct RcWorldHandle *world,
                                           RcRigidBodyHandle handle);

struct RcBool rc_rigid_body_set_status(struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       enum RcBodyStatus status,
                                       struct RcBool wake_up);

struct RcVec3 rc_rigid_body_get_translation(const struct RcWorldHandle *world,
                                            RcRigidBodyHandle handle);

void rc_rigid_body_get_translation_out(const struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       struct RcVec3 *out_translation);

struct RcQuat rc_rigid_body_get_rotation(const struct RcWorldHandle *world,
                                         RcRigidBodyHandle handle);

void rc_rigid_body_get_rotation_out(const struct RcWorldHandle *world,
                                    RcRigidBodyHandle handle,
                                    struct RcQuat *out_rotation);

struct RcBool rc_rigid_body_set_pose(struct RcWorldHandle *world,
                                     RcRigidBodyHandle handle,
                                     struct RcVec3 translation,
                                     struct RcQuat rotation,
                                     struct RcBool wake_up);

uint8_t rc_rigid_body_set_pose_flag(struct RcWorldHandle *world,
                                    RcRigidBodyHandle handle,
                                    struct RcVec3 translation,
                                    struct RcQuat rotation,
                                    struct RcBool wake_up);

struct RcVec3 rc_rigid_body_get_linvel(const struct RcWorldHandle *world, RcRigidBodyHandle handle);

void rc_rigid_body_get_linvel_out(const struct RcWorldHandle *world,
                                  RcRigidBodyHandle handle,
                                  struct RcVec3 *out_linvel);

struct RcBool rc_rigid_body_set_linvel(struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       struct RcVec3 linvel,
                                       struct RcBool wake_up);

uint8_t rc_rigid_body_set_linvel_flag(struct RcWorldHandle *world,
                                      RcRigidBodyHandle handle,
                                      struct RcVec3 linvel,
                                      struct RcBool wake_up);

struct RcVec3 rc_rigid_body_get_angvel(const struct RcWorldHandle *world, RcRigidBodyHandle handle);

struct RcBool rc_rigid_body_set_angvel(struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       struct RcVec3 angvel,
                                       struct RcBool wake_up);

struct RcBool rc_rigid_body_add_force(struct RcWorldHandle *world,
                                      RcRigidBodyHandle handle,
                                      struct RcVec3 force,
                                      struct RcBool wake_up);

uint8_t rc_rigid_body_add_force_flag(struct RcWorldHandle *world,
                                     RcRigidBodyHandle handle,
                                     struct RcVec3 force,
                                     struct RcBool wake_up);

struct RcBool rc_rigid_body_add_torque(struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       struct RcVec3 torque,
                                       struct RcBool wake_up);

struct RcBool rc_rigid_body_apply_impulse(struct RcWorldHandle *world,
                                          RcRigidBodyHandle handle,
                                          struct RcVec3 impulse,
                                          struct RcBool wake_up);

struct RcBool rc_rigid_body_apply_torque_impulse(struct RcWorldHandle *world,
                                                 RcRigidBodyHandle handle,
                                                 struct RcVec3 torque_impulse,
                                                 struct RcBool wake_up);

struct RcBool rc_rigid_body_enable_ccd(struct RcWorldHandle *world,
                                       RcRigidBodyHandle handle,
                                       struct RcBool enabled);

struct RcBool rc_rigid_body_sleep(struct RcWorldHandle *world, RcRigidBodyHandle handle);

uint8_t rc_rigid_body_sleep_flag(struct RcWorldHandle *world, RcRigidBodyHandle handle);

struct RcBool rc_rigid_body_wake_up(struct RcWorldHandle *world,
                                    RcRigidBodyHandle handle,
                                    struct RcBool strong);

uint8_t rc_rigid_body_wake_up_flag(struct RcWorldHandle *world,
                                   RcRigidBodyHandle handle,
                                   struct RcBool strong);

struct RcBool rc_rigid_body_is_sleeping(const struct RcWorldHandle *world,
                                        RcRigidBodyHandle handle);

uint8_t rc_rigid_body_is_sleeping_flag(const struct RcWorldHandle *world, RcRigidBodyHandle handle);

struct RcWorldHandle *rc_world_create(struct RcVec3 gravity);

void rc_world_destroy(struct RcWorldHandle *world);

void rc_world_step(struct RcWorldHandle *world, double delta_seconds);

void rc_world_set_gravity(struct RcWorldHandle *world, struct RcVec3 gravity);

struct RcVec3 rc_world_get_gravity(const struct RcWorldHandle *world);

void rc_world_get_gravity_out(const struct RcWorldHandle *world, struct RcVec3 *out_gravity);

uint32_t rc_world_dynamic_body_snapshot_count(const struct RcWorldHandle *world);

uint32_t rc_world_dynamic_body_snapshot(const struct RcWorldHandle *world,
                                        RcRigidBodyHandle *out_handles,
                                        double *out_values,
                                        uint32_t capacity);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* RIGID_BODY_H */
