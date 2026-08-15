#ifndef RIGID_BODY_H
#define RIGID_BODY_H

#pragma once

/* Generated with cbindgen:0.29.4 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Maximum number of requests a single batch can hold.  Prevents a runaway
 * caller from exhausting memory before [`ColliderBatch::execute`] runs.
 */
#define MAX_BATCH_REQUESTS 100000

/**
 * Maximum number of compound parts in a single merged collider.  Rapier's
 * compound shape stores parts in a `Vec` so the practical limit is available
 * memory; we cap to keep broadphase insertion tractable.
 */
#define MAX_COMPOUND_PARTS 50000

#define ERR_OK 0

#define ERR_NULL_POINTER 1

#define ERR_INVALID_ARGUMENT 2

#define ERR_NOT_FOUND 3

#define ERR_CAPACITY 4

#define ERR_UNSUPPORTED 5

#define ERR_INTERNAL 6

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
#define ARENA_VERSION 2

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

/**
 * Upper bounds for arena capacities — defense against absurd FFI requests.
 */
#define MAX_ARENA_BODIES 1000000

#define MAX_ARENA_COLLIDERS 1000000

#define MAX_ARENA_EVENTS 1000000

#define MAX_ARENA_COMMANDS 1000000

/**
 * Hard cap on the total arena allocation (256 MiB).
 */
#define MAX_ARENA_TOTAL_BYTES ((256 * 1024) * 1024)

/**
 * Integration params region: dt(8) + solver_iterations(4) + ccd_substeps(4) + gravity(24)
 */
#define INTEGRATION_PARAMS_SIZE 40

/**
 * Force summary region: max_reynolds(8) + external force(24) + drag force(24) + counts(8)
 */
#define FORCE_SUMMARY_SIZE 64

typedef struct AnvilKitAppHandle AnvilKitAppHandle;

typedef struct CRbTreeHandle CRbTreeHandle;

typedef struct CharacterControllerHandle CharacterControllerHandle;

typedef struct ColliderBuilderHandle ColliderBuilderHandle;

typedef struct JointBuilderHandle JointBuilderHandle;

typedef struct RTreeHandle RTreeHandle;

typedef struct RigidBodyBuilderHandle RigidBodyBuilderHandle;

typedef struct VoxelGrid VoxelGrid;

typedef struct WorldHandle WorldHandle;

/**
 * A single collider creation request, designed for batch submission via the
 * Box3D-style pipeline.
 *
 * Fields are flat `#[repr(C)]` so the FFI caller can build a contiguous array
 * and pass `(ptr, count)` to [`world_batch_add_colliders`].
 *
 * [`world_batch_add_colliders`]: crate::rapier::world::world_batch_add_colliders
 */
typedef struct ColliderRequest {
  /**
   * Shape descriptor (shape_type + 4 floats a/b/c/d).  See [`ShapeDesc`].
   */
  ShapeDesc shape;
  /**
   * Local translation relative to the merged collider origin (world pos if
   * `body_parent == 0` and no merge happens).
   */
  Vec3 translation;
  /**
   * Local rotation as a unit quaternion (xyzw, but stored as ijkw in [`Quat`]).
   */
  Quat rotation;
  /**
   * Coulomb friction coefficient (≥ 0).
   */
  double friction;
  /**
   * Coefficient of restitution (≥ 0, typically < 1).
   */
  double restitution;
  /**
   * Mass density (≥ 0).  Ignored for static (parentless) shapes.
   */
  double density;
  /**
   * Collision group memberships bitmask.
   */
  InteractionGroupsDesc collision_groups;
  /**
   * Solver group memberships bitmask.
   */
  InteractionGroupsDesc solver_groups;
  /**
   * If non-zero, this collider is attached to the given rigid body.
   */
  RigidBodyHandleRaw body_parent;
  /**
   * If non-zero, the collider is a sensor (no collision response).
   */
  Bool is_sensor;
  /**
   * Bitmask of [`ActiveEvents`] to enable.
   */
  uint32_t active_events;
  /**
   * Bitmask of [`ActiveHooks`] to enable.
   */
  uint32_t active_hooks;
  /**
   * Per-collider erosion margin (Rapier `contact_partitioning`).  Only
   * meaningful for round shapes; 0 = no erosion.
   */
  double erosion_margin;
} ColliderRequest;

/**
 * Parameter preset that approximates Box3D's sandbox physics feel.
 *
 * These values are applied to every collider in a batch unless the request
 * itself overrides the corresponding field (> 0 for floats, non-default for
 * groups).  The preset is passed to [`ColliderBatch::new`] and used during
 * [`ColliderBatch::execute`].
 */
typedef struct Box3DPreset {
  /**
   * Default friction when the request's `friction` is <= 0.
   * Box3D feel ≈ 0.6 (moderate grip, not icy).
   */
  double default_friction;
  /**
   * Default restitution when the request's `restitution` is < 0.
   * Box3D feel ≈ 0.2 (slight bounce, realistic).
   */
  double default_restitution;
  /**
   * Default density for dynamic shapes when the request's `density` is
   * <= 0.  Box3D feel ≈ 1.0 (water-equivalent for intuitive masses).
   */
  double default_density;
  /**
   * Erosion margin applied to round shapes for stable stacking.
   * Box3D feel ≈ 0.01 (small margin prevents jitter on stacked bodies).
   */
  double default_erosion_margin;
  /**
   * Linear damping applied to dynamic bodies created by merge_static_shapes.
   * Box3D feel ≈ 0.05 (slight slow-down, prevents perpetual motion).
   */
  double linear_damping;
  /**
   * Angular damping for dynamic bodies.
   * Box3D feel ≈ 0.05.
   */
  double angular_damping;
  /**
   * CCD sub-steps for fast-moving dynamic bodies.  0 = off.
   * Box3D feel ≈ 1 (enough to prevent tunneling at sandbox speeds).
   */
  uint32_t ccd_substeps;
  /**
   * Solver iterations.  Box3D feel ≈ 4 (GoodBalance between stability and CPU).
   */
  uint32_t solver_iterations;
} Box3DPreset;

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

/**
 * Output of `collider_voxel_ray_pick`: the voxel cell coordinate that a ray
 * hit on a voxel collider, plus the surface normal at the hit (so the caller
 * can derive the adjacent cell for "place on face").
 *
 * `found` is `FALSE` when the ray missed, hit a different collider, or the
 * resolved cell is out of the grid bounds.
 *
 * Layout (C ABI, read by Java via `Unsafe`):
 * `found` @0 (u8), `ix` @8, `iy` @16, `iz` @24 (i64),
 * `nx` @32, `ny` @40, `nz` @48 (f64). `SIZEOF` = 56.
 */
typedef struct VoxelCoord {
  Bool found;
  int64_t ix;
  int64_t iy;
  int64_t iz;
  double nx;
  double ny;
  double nz;
} VoxelCoord;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Apply aerodynamic forces from a set of surfaces to a rigid body.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `surfaces` must point to at
 * least `surface_count` readable `AeroSurface`s; `out_report`, when
 * non-null, must be valid for a single `AeroForceReport` write.
 */
Bool aero_apply_surfaces(struct WorldHandle *world,
                         RigidBodyHandleRaw body_handle,
                         Vec3 wind_velocity,
                         double air_density,
                         const AeroSurface *surfaces,
                         uint32_t surface_count,
                         Bool wake_up,
                         AeroForceReport *out_report);

/**
 * Apply aerodynamic forces derived from a voxel grid to a rigid body.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `voxels` must point to at
 * least size_x×size_y×size_z readable bytes; `out_report`, when non-null,
 * must be valid for a single `AeroForceReport` write.
 */
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

/**
 * Flag-returning variant of `aero_apply_voxel_grid`.
 *
 * # Safety
 *
 * Same pointer contract as `aero_apply_voxel_grid`.
 */
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

/**
 * Flag-returning variant of `aero_apply_surfaces`.
 *
 * # Safety
 *
 * Same pointer contract as `aero_apply_surfaces`.
 */
uint8_t aero_apply_surfaces_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw body_handle,
                                 Vec3 wind_velocity,
                                 double air_density,
                                 const AeroSurface *surfaces,
                                 uint32_t surface_count,
                                 Bool wake_up,
                                 AeroForceReport *out_report);

/**
 * Estimate the aerodynamic force of a single surface without a world.
 *
 * # Safety
 *
 * `out_report`, when non-null, must be valid for a single `AeroForceReport`
 * write.
 */
Bool aero_estimate_surface_force(Vec3 body_linvel,
                                 Vec3 body_angvel,
                                 Vec3 body_center,
                                 Vec3 wind_velocity,
                                 double air_density,
                                 AeroSurface surface,
                                 AeroForceReport *out_report);

/**
 * Creates a new AnvilKit app state and returns an opaque handle to it.
 *
 * # Safety
 *
 * Takes no pointers and cannot fail on input; the returned handle is owned by
 * the caller and must eventually be passed to `anvilkit_app_destroy` (or
 * leaked).
 */
struct AnvilKitAppHandle *anvilkit_app_create(void);

/**
 * # Safety
 *
 * `app` must be null or a handle returned by `anvilkit_app_create` that has
 * not been destroyed yet; ownership transfers back to Rust and the handle is
 * invalid after this call.
 */
void anvilkit_app_destroy(struct AnvilKitAppHandle *app);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
void anvilkit_app_update(struct AnvilKitAppHandle *app);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
uint64_t anvilkit_app_spawn_body(struct AnvilKitAppHandle *app,
                                 Vec3 translation,
                                 Quat rotation,
                                 uint32_t status);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
uint64_t anvilkit_app_spawn_body_with_collider(struct AnvilKitAppHandle *app,
                                               Vec3 translation,
                                               Quat rotation,
                                               uint32_t status,
                                               ShapeDesc shape);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
Bool anvilkit_app_set_transform(struct AnvilKitAppHandle *app,
                                uint64_t entity_bits,
                                Vec3 translation,
                                Quat rotation);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
Bool anvilkit_app_set_material(struct AnvilKitAppHandle *app,
                               uint64_t entity_bits,
                               MaterialProperties material);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles returned by
 * `anvilkit_app_create` / the world-creation ABI.
 */
uint32_t anvilkit_app_sync_to_world(struct AnvilKitAppHandle *app, struct WorldHandle *world);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
RigidBodyHandleRaw anvilkit_app_entity_to_body(const struct AnvilKitAppHandle *app,
                                               uint64_t entity_bits);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
ColliderHandleRaw anvilkit_app_entity_to_collider(const struct AnvilKitAppHandle *app,
                                                  uint64_t entity_bits);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles returned by
 * `anvilkit_app_create` / the world-creation ABI.
 */
uint64_t anvilkit_app_create_constraint(struct AnvilKitAppHandle *app,
                                        struct WorldHandle *world,
                                        uint64_t entity1_bits,
                                        uint64_t entity2_bits,
                                        uint32_t joint_type,
                                        Vec3 axis_or_primary,
                                        double b,
                                        double c,
                                        Bool wake_up);

/**
 * # Safety
 *
 * `app` must be null or a valid handle returned by `anvilkit_app_create`.
 */
ImpulseJointHandleRaw anvilkit_app_constraint_to_joint(const struct AnvilKitAppHandle *app,
                                                       uint64_t constraint_id);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles returned by
 * `anvilkit_app_create` / the world-creation ABI.
 */
Bool anvilkit_app_remove_constraint(struct AnvilKitAppHandle *app,
                                    struct WorldHandle *world,
                                    uint64_t constraint_id,
                                    Bool wake_up);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles. `surfaces` must point to
 * `surface_count` readable `AeroSurface` entries, and `out_report` must be
 * null or point to a valid, writable `AeroForceReport`.
 */
Bool anvilkit_app_apply_aero_surfaces(struct AnvilKitAppHandle *app,
                                      struct WorldHandle *world,
                                      uint64_t entity_bits,
                                      Vec3 wind_velocity,
                                      double air_density,
                                      const AeroSurface *surfaces,
                                      uint32_t surface_count,
                                      Bool wake_up,
                                      AeroForceReport *out_report);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles. `voxels` must point to at
 * least `size_x * size_y * size_z` readable bytes, and `out_report` must be
 * null or point to a valid, writable `AeroForceReport`.
 */
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

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles. `out_report` must be null
 * or point to a valid, writable `FluidForceReport`.
 */
Bool anvilkit_app_apply_fluid_aabb_forces(struct AnvilKitAppHandle *app,
                                          struct WorldHandle *world,
                                          uint64_t entity_bits,
                                          FluidVolume fluid_volume,
                                          Vec3 body_half_extents,
                                          double body_volume,
                                          Bool wake_up,
                                          FluidForceReport *out_report);

/**
 * # Safety
 *
 * `app` and `world` must be null or valid handles. `out_report` must be null
 * or point to a valid, writable `TrajectoryForceReport`.
 */
Bool anvilkit_app_apply_trajectory_forces(struct AnvilKitAppHandle *app,
                                          struct WorldHandle *world,
                                          uint64_t entity_bits,
                                          TrajectoryEnvironment environment,
                                          Bool wake_up,
                                          TrajectoryForceReport *out_report);

/**
 * # Safety
 *
 * `out_report` must be null or point to a valid, writable `StressStrainReport`.
 */
Bool material_stress_strain_linear(MaterialProperties material,
                                   double strain,
                                   double delta_temperature,
                                   StressStrainReport *out_report);

/**
 * Computes the post-collision relative normal speed from restitution.
 *
 * # Safety
 *
 * All parameters are passed by value; this function performs no memory
 * access and is always memory-safe. Non-finite inputs or a negative
 * `restitution` yield `NaN`.
 */
double material_elastic_collision_relative_speed(double relative_normal_speed, double restitution);

/**
 * # Safety
 *
 * `out_report` must be null or point to a valid, writable `HertzContactReport`.
 */
Bool material_hertz_contact_force(MaterialProperties material1,
                                  MaterialProperties material2,
                                  double radius1,
                                  double radius2,
                                  double penetration,
                                  double penetration_rate,
                                  double damping,
                                  HertzContactReport *out_report);

/**
 * Batch-add colliders from a flat array of [`ColliderRequest`]s.
 *
 * Creates a [`ColliderBatch`] internally, pushes all requests, executes the
 * merge + insert pipeline, and writes the resulting collider handles into
 * `out_handles`.  Returns the number of handles written.
 *
 * The Box3D feel preset is passed by value; use [`Box3DPreset::default`] for
 * zero-initialised fields, or [`Box3DPreset::box3d_default`] via the FFI
 * convenience function [`box3d_preset_default`].
 *
 * # Safety
 *
 * `world` must be a valid pointer from `world_create`.  `requests` must point
 * to at least `count` readable `ColliderRequest` values.  `out_handles` must
 * point to writable memory for at least `count * size_of(ColliderHandleRaw)`
 * bytes (each request could produce up to one handle, fewer if merged).
 */
uint32_t world_batch_add_colliders(struct WorldHandle *world,
                                   const struct ColliderRequest *requests,
                                   uint32_t count,
                                   struct Box3DPreset preset,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t out_capacity);

/**
 * Merge static shapes and insert with a single `ColliderSet::insert`.
 *
 * Like [`world_batch_add_colliders`] but requires all requests to be static
 * (parentless).  Returns the number of (compound) collider handles written.
 *
 * # Safety
 *
 * Same as [`world_batch_add_colliders`].
 */
uint32_t world_merge_static_shapes(struct WorldHandle *world,
                                   const struct ColliderRequest *requests,
                                   uint32_t count,
                                   struct Box3DPreset preset,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t out_capacity);

/**
 * Convenience: get the Box3D default-feel preset.
 */
struct Box3DPreset box3d_preset_default(void);

/**
 * Convenience: get the Box3D sticky-feel preset (high friction, no bounce).
 */
struct Box3DPreset box3d_preset_sticky(void);

/**
 * Convenience: get the Box3D bouncy-feel preset (low friction, high restitution).
 */
struct Box3DPreset box3d_preset_bouncy(void);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_capsule(Capsule capsule);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_ssv(Ssv ssv);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_ellipsoid(Ellipsoid ellipsoid);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_prism(Prism prism);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_cylinder(Cylinder cylinder);

/**
 * # Safety
 *
 * The returned builder is owned by the caller and must be consumed by
 * `collider_builder_build` or freed with `collider_builder_destroy`.
 */
struct ColliderBuilderHandle *collider_builder_create_spherical_shell(SphericalShell shell);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_capsule_count(const struct WorldHandle *world,
                                       Capsule capsule,
                                       QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_capsule_count_all(const struct WorldHandle *world, Capsule capsule);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_capsule(const struct WorldHandle *world,
                                 Capsule capsule,
                                 QueryFilterDesc filter,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_capsule_all(const struct WorldHandle *world,
                                     Capsule capsule,
                                     ColliderHandleRaw *out_handles,
                                     uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_ssv_count(const struct WorldHandle *world,
                                   Ssv ssv,
                                   QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_ssv_count_all(const struct WorldHandle *world, Ssv ssv);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_ssv(const struct WorldHandle *world,
                             Ssv ssv,
                             QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_ssv_all(const struct WorldHandle *world,
                                 Ssv ssv,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_ellipsoid_count(const struct WorldHandle *world,
                                         Ellipsoid ellipsoid,
                                         QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_ellipsoid_count_all(const struct WorldHandle *world, Ellipsoid ellipsoid);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_ellipsoid(const struct WorldHandle *world,
                                   Ellipsoid ellipsoid,
                                   QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_ellipsoid_all(const struct WorldHandle *world,
                                       Ellipsoid ellipsoid,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_prism_count(const struct WorldHandle *world,
                                     Prism prism,
                                     QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_prism_count_all(const struct WorldHandle *world, Prism prism);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_prism(const struct WorldHandle *world,
                               Prism prism,
                               QueryFilterDesc filter,
                               ColliderHandleRaw *out_handles,
                               uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_prism_all(const struct WorldHandle *world,
                                   Prism prism,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_cylinder_count(const struct WorldHandle *world,
                                        Cylinder cylinder,
                                        QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_cylinder_count_all(const struct WorldHandle *world, Cylinder cylinder);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_cylinder(const struct WorldHandle *world,
                                  Cylinder cylinder,
                                  QueryFilterDesc filter,
                                  ColliderHandleRaw *out_handles,
                                  uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_cylinder_all(const struct WorldHandle *world,
                                      Cylinder cylinder,
                                      ColliderHandleRaw *out_handles,
                                      uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_spherical_shell_count(const struct WorldHandle *world,
                                               SphericalShell shell,
                                               QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_spherical_shell_count_all(const struct WorldHandle *world,
                                                   SphericalShell shell);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_spherical_shell(const struct WorldHandle *world,
                                         SphericalShell shell,
                                         QueryFilterDesc filter,
                                         ColliderHandleRaw *out_handles,
                                         uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_spherical_shell_all(const struct WorldHandle *world,
                                             SphericalShell shell,
                                             ColliderHandleRaw *out_handles,
                                             uint32_t capacity);

/**
 * Creates a collider builder from a generic shape type and packed shape data.
 *
 * # Safety
 *
 * All parameters are passed by value; no raw pointers are dereferenced.
 * An invalid shape descriptor fails with `ERR_INVALID_ARGUMENT` and returns null.
 */
struct ColliderBuilderHandle *collider_builder_create(uint32_t shape_type, Vec3 shape_data);

/**
 * Creates a halfspace collider builder with the given plane normal.
 *
 * # Safety
 *
 * `normal` is passed by value; no raw pointers are dereferenced.
 * A non-finite normal fails with `ERR_INVALID_ARGUMENT` and returns null.
 */
struct ColliderBuilderHandle *collider_builder_create_halfspace(Vec3 normal);

/**
 * Creates a collider builder from an extended shape descriptor.
 *
 * # Safety
 *
 * `shape_desc` is passed by value; no raw pointers are dereferenced.
 * An invalid shape descriptor fails with `ERR_INVALID_ARGUMENT` and returns null.
 */
struct ColliderBuilderHandle *collider_builder_create_ex(ShapeDesc shape_desc);

/**
 * Creates an oriented box (cuboid) collider builder from an OBB descriptor.
 *
 * # Safety
 *
 * `obb` is passed by value; no raw pointers are dereferenced.
 * A non-finite center/rotation or non-positive half extents fail with
 * `ERR_INVALID_ARGUMENT` and return null.
 */
struct ColliderBuilderHandle *collider_builder_create_obb(Obb obb);

/**
 * Creates a ball collider builder from a sphere descriptor.
 *
 * # Safety
 *
 * `sphere` is passed by value; no raw pointers are dereferenced.
 * A non-finite center or a non-finite/non-positive radius fails with
 * `ERR_INVALID_ARGUMENT` and returns null.
 */
struct ColliderBuilderHandle *collider_builder_create_sphere(Sphere sphere);

/**
 * # Safety
 *
 * `data` must point to at least `data_x * data_y` readable `f64` height values.
 */
struct ColliderBuilderHandle *collider_builder_create_heightmap(const double *data,
                                                                uint32_t data_x,
                                                                uint32_t data_y,
                                                                Vec3 scale);

/**
 * # Safety
 *
 * `points_xyz` must point to at least `point_count * 3` readable `f64` values.
 */
struct ColliderBuilderHandle *collider_builder_create_convex_hull(const double *points_xyz,
                                                                  uint32_t point_count);

/**
 * # Safety
 *
 * `points_xyz` must point to at least `point_count * 3` readable `f64` values.
 */
struct ColliderBuilderHandle *collider_builder_create_point_cloud_bounds(const double *points_xyz,
                                                                         uint32_t point_count);

/**
 * Creates a collider builder covering the union of two AABBs.
 *
 * # Safety
 *
 * `first` and `second` are passed by value; no raw pointers are dereferenced.
 * An invalid AABB (non-finite or `mins > maxs`) fails with
 * `ERR_INVALID_ARGUMENT` and returns null.
 */
struct ColliderBuilderHandle *collider_builder_create_double_bv(AabbDesc first, AabbDesc second);

/**
 * Creates a convex-hull collider builder from a skewed box (center + 3 axis vectors).
 *
 * # Safety
 *
 * All parameters are passed by value; no raw pointers are dereferenced.
 * Non-finite vectors or near-zero-length axes fail with `ERR_INVALID_ARGUMENT`
 * and return null.
 */
struct ColliderBuilderHandle *collider_builder_create_skewed_obb(Vec3 center,
                                                                 Vec3 axis_x,
                                                                 Vec3 axis_y,
                                                                 Vec3 axis_z);

/**
 * # Safety
 *
 * `points_xyz` must point to at least `point_count * 3` readable `f64` values.
 */
struct ColliderBuilderHandle *collider_builder_create_discrete_obb(const double *points_xyz,
                                                                   uint32_t point_count,
                                                                   uint32_t axis);

/**
 * # Safety
 *
 * `points_xyz` must point to at least `point_count * 3` readable `f64` values.
 */
struct ColliderBuilderHandle *collider_builder_create_fused_collapsing_bounds(const double *points_xyz,
                                                                              uint32_t point_count,
                                                                              double padding);

/**
 * # Safety
 *
 * `vertices_xyz` must point to at least `vertex_count * 3` readable `f64`
 * values and `edges` to at least `edge_count * 2` readable `u32` indices.
 */
struct ColliderBuilderHandle *collider_builder_create_edge_bvh(const double *vertices_xyz,
                                                               uint32_t vertex_count,
                                                               const uint32_t *edges,
                                                               uint32_t edge_count,
                                                               double radius);

/**
 * # Safety
 *
 * `spheres_xyzw` must point to at least `sphere_count * 4` readable `f64`
 * values (center xyz + radius per sphere).
 */
struct ColliderBuilderHandle *collider_builder_create_medial_spheres(const double *spheres_xyzw,
                                                                     uint32_t sphere_count);

/**
 * # Safety
 *
 * `builder` must be a pointer returned by a `collider_builder_create_*`
 * function. It is consumed by this call and must not be used afterwards.
 */
Collider *collider_builder_build(struct ColliderBuilderHandle *builder);

/**
 * # Safety
 *
 * `builder` must be a pointer returned by a `collider_builder_create_*`
 * function that has not been consumed by `collider_builder_build`.
 */
void collider_builder_destroy(struct ColliderBuilderHandle *builder);

/**
 * # Safety
 *
 * `collider` must be a pointer returned by `collider_builder_build` or
 * `world_copy_collider` that has not already been destroyed.
 */
void collider_destroy_raw(Collider *collider);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_translation(struct ColliderBuilderHandle *builder, Vec3 translation);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_rotation(struct ColliderBuilderHandle *builder, Vec3 rotation_axis_angle);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_pose(struct ColliderBuilderHandle *builder,
                               Vec3 translation,
                               Quat rotation);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_sensor(struct ColliderBuilderHandle *builder, Bool sensor);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_friction(struct ColliderBuilderHandle *builder, double friction);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_restitution(struct ColliderBuilderHandle *builder, double restitution);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_density(struct ColliderBuilderHandle *builder, double density);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_collision_groups(struct ColliderBuilderHandle *builder,
                                           InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_solver_groups(struct ColliderBuilderHandle *builder,
                                        InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_active_events(struct ColliderBuilderHandle *builder,
                                        uint32_t active_events_bits);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_active_hooks(struct ColliderBuilderHandle *builder,
                                       uint32_t active_hooks_bits);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by a `collider_builder_create_*`
 * function and not yet consumed or destroyed.
 */
void collider_builder_set_contact_force_event_threshold(struct ColliderBuilderHandle *builder,
                                                        double threshold);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create`. `memory_handle`
 * must be a pointer returned by `collider_builder_build` or
 * `world_copy_collider`; it is consumed by this call.
 */
ColliderHandleRaw world_insert_collider(struct WorldHandle *world, Collider *memory_handle);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create`. `memory_handle`
 * must be a pointer returned by `collider_builder_build` or
 * `world_copy_collider`; it is consumed by this call.
 */
ColliderHandleRaw world_insert_collider_with_parent(struct WorldHandle *world,
                                                    Collider *memory_handle,
                                                    RigidBodyHandleRaw parent);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool world_remove_collider(struct WorldHandle *world, ColliderHandleRaw handle, Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Collider *world_copy_collider(struct WorldHandle *world, ColliderHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t world_remove_collider_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Vec3 collider_get_translation(const struct WorldHandle *world, ColliderHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uintptr_t collider_get_shape_count(const struct WorldHandle *world, ColliderHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create`; `out_translation`
 * must point to a writable `Vec3`.
 */
void collider_get_translation_out(const struct WorldHandle *world,
                                  ColliderHandleRaw handle,
                                  Vec3 *out_translation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Quat collider_get_rotation(const struct WorldHandle *world, ColliderHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create`; `out_rotation`
 * must point to a writable `Quat`.
 */
void collider_get_rotation_out(const struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               Quat *out_rotation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_pose(struct WorldHandle *world,
                       ColliderHandleRaw handle,
                       Vec3 translation,
                       Quat rotation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_translation(struct WorldHandle *world,
                              ColliderHandleRaw handle,
                              Vec3 translation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_rotation(struct WorldHandle *world, ColliderHandleRaw handle, Quat rotation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_pose_flag(struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               Vec3 translation,
                               Quat rotation);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_sensor(struct WorldHandle *world, ColliderHandleRaw handle, Bool sensor);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_sensor_flag(struct WorldHandle *world, ColliderHandleRaw handle, Bool sensor);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_friction(struct WorldHandle *world, ColliderHandleRaw handle, double friction);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_friction_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   double friction);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_restitution(struct WorldHandle *world,
                              ColliderHandleRaw handle,
                              double restitution);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_restitution_flag(struct WorldHandle *world,
                                      ColliderHandleRaw handle,
                                      double restitution);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_collision_groups(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_collision_groups_flag(struct WorldHandle *world,
                                           ColliderHandleRaw handle,
                                           InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_solver_groups(struct WorldHandle *world,
                                ColliderHandleRaw handle,
                                InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_solver_groups_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        InteractionGroupsDesc groups);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_active_events(struct WorldHandle *world,
                                ColliderHandleRaw handle,
                                uint32_t active_events_bits);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_active_events_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        uint32_t active_events_bits);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_active_hooks(struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               uint32_t active_hooks_bits);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_active_hooks_flag(struct WorldHandle *world,
                                       ColliderHandleRaw handle,
                                       uint32_t active_hooks_bits);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool collider_set_contact_force_event_threshold(struct WorldHandle *world,
                                                ColliderHandleRaw handle,
                                                double threshold);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
uint8_t collider_set_contact_force_event_threshold_flag(struct WorldHandle *world,
                                                        ColliderHandleRaw handle,
                                                        double threshold);

/**
 * # Safety
 *
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
double collider_get_density(const struct WorldHandle *world, ColliderHandleRaw handle);

/**
 * Insert a dynamic rigid body built from a list of cuboids.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `cuboids` must point to at
 * least 6×cuboid_count readable f64s (center xyz + half-extents xyz per
 * cuboid).
 */
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

/**
 * Insert a fixed rigid body with a trimesh collider.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `vertices_xyz` must point to
 * at least `vertex_xyz_len` readable f64s and `indices` to at least
 * `index_len` readable u32s.
 */
RigidBodyHandleRaw world_insert_static_trimesh(struct WorldHandle *world,
                                               const double *vertices_xyz,
                                               uint32_t vertex_xyz_len,
                                               const uint32_t *indices,
                                               uint32_t index_len,
                                               double friction,
                                               double restitution);

/**
 * Count the rigid bodies intersecting an AABB.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer.
 */
uint32_t query_intersect_aabb_rigid_body_count(const struct WorldHandle *world,
                                               AabbDesc aabb,
                                               QueryFilterDesc filter);

/**
 * Collect the rigid body handles intersecting an AABB.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `out_handles` must be valid
 * for `capacity` `RigidBodyHandleRaw` writes.
 */
uint32_t query_intersect_aabb_rigid_bodies(const struct WorldHandle *world,
                                           AabbDesc aabb,
                                           QueryFilterDesc filter,
                                           RigidBodyHandleRaw *out_handles,
                                           uint32_t capacity);

/**
 * Creates a new character controller and returns an opaque handle to it.
 *
 * # Safety
 *
 * The returned pointer is owned by Rust and must be passed to
 * `character_controller_destroy` exactly once. Returns null on internal
 * failure (see `last_error_code`).
 */
struct CharacterControllerHandle *character_controller_create(void);

/**
 * # Safety
 *
 * `controller` must be a pointer returned by `character_controller_create` (or null,
 * which is a no-op). Ownership is transferred to Rust and the pointer must not be
 * used after this call.
 */
void character_controller_destroy(struct CharacterControllerHandle *controller);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_up(struct CharacterControllerHandle *controller, Vec3 up);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_offset_absolute(struct CharacterControllerHandle *controller,
                                              double offset);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_offset_relative(struct CharacterControllerHandle *controller,
                                              double offset);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_slide(struct CharacterControllerHandle *controller, Bool slide);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_autostep(struct CharacterControllerHandle *controller,
                                       Bool enabled,
                                       double max_height,
                                       double min_width,
                                       Bool include_dynamic_bodies);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_snap_to_ground(struct CharacterControllerHandle *controller,
                                             Bool enabled,
                                             double distance);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
void character_controller_set_slope_angles(struct CharacterControllerHandle *controller,
                                           double max_climb_angle,
                                           double min_slide_angle);

/**
 * # Safety
 *
 * `world` must be a valid world pointer and `controller` a valid pointer returned by
 * `character_controller_create`; both must remain alive for the duration of the call.
 */
EffectiveCharacterMovement character_controller_move_shape(const struct WorldHandle *world,
                                                           struct CharacterControllerHandle *controller,
                                                           double dt,
                                                           ShapeDesc shape_desc,
                                                           Vec3 translation,
                                                           Quat rotation,
                                                           Vec3 desired_translation);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
uint32_t character_controller_collision_count(const struct CharacterControllerHandle *controller);

/**
 * # Safety
 *
 * `controller` must be a valid pointer returned by `character_controller_create`
 * and must remain alive for the duration of the call.
 */
FfiCharacterCollision character_controller_get_collision(const struct CharacterControllerHandle *controller,
                                                         uint32_t index);

/**
 * # Safety
 *
 * `world` must be a valid world pointer and `controller` a valid pointer returned by
 * `character_controller_create`; both must remain alive for the duration of the call.
 */
Bool character_controller_solve_impulses(struct WorldHandle *world,
                                         struct CharacterControllerHandle *controller,
                                         double dt,
                                         ShapeDesc shape_desc,
                                         double character_mass);

/**
 * # Safety
 *
 * `world` must be a valid world pointer and `controller` a valid pointer returned by
 * `character_controller_create`; both must remain alive for the duration of the call.
 *
 * Like [`character_controller_move_shape`] but additionally samples the world's
 * registered terrain gravity (polyhedron / DEM / lunar-mascon) at the character's
 * current `translation` and folds the resulting free-fall displacement
 * (`½·a·dt²`, directed along the local terrain-gravity acceleration `a`) into the
 * desired translation.  This lets a kinematic character fall toward and stand on an
 * irregular small-body surface instead of floating.  When no terrain-gravity law is
 * registered the call is identical to `character_controller_move_shape`.
 */
EffectiveCharacterMovement character_controller_move_shape_with_terrain(const struct WorldHandle *world,
                                                                        struct CharacterControllerHandle *controller,
                                                                        double dt,
                                                                        ShapeDesc shape_desc,
                                                                        Vec3 translation,
                                                                        Quat rotation,
                                                                        Vec3 desired_translation);

/**
 * Create an empty red-black-tree AABB index.
 *
 * # Safety
 *
 * The returned pointer is owned by the caller and must be freed exactly once
 * with `crb_tree_destroy`.
 */
struct CRbTreeHandle *crb_tree_create(void);

/**
 * Destroy an index created by `crb_tree_create`.
 *
 * # Safety
 *
 * `tree` must be null or a pointer returned by `crb_tree_create`; it must not
 * be used again after this call.
 */
void crb_tree_destroy(struct CRbTreeHandle *tree);

/**
 * Remove every entry from the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
void crb_tree_clear(struct CRbTreeHandle *tree);

/**
 * Return the number of entries stored in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
uint32_t crb_tree_len(const struct CRbTreeHandle *tree);

/**
 * Insert or overwrite the bounds of `id` in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
Bool crb_tree_insert(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

/**
 * Flag-returning variant of `crb_tree_insert`.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
uint8_t crb_tree_insert_flag(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

/**
 * Update the bounds of an existing `id` in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
Bool crb_tree_update(struct CRbTreeHandle *tree, uint64_t id, AabbDesc aabb);

/**
 * Remove `id` from the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
Bool crb_tree_remove(struct CRbTreeHandle *tree, uint64_t id);

/**
 * Count the entries whose bounds intersect `aabb`.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`.
 */
uint32_t crb_tree_query_aabb_count(const struct CRbTreeHandle *tree, AabbDesc aabb);

/**
 * Write the ids of entries whose bounds intersect `aabb` into `out_ids`.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `crb_tree_create`, and `out_ids`
 * must point to a writable buffer of at least `capacity` `u64` elements.
 */
uint32_t crb_tree_query_aabb(const struct CRbTreeHandle *tree,
                             AabbDesc aabb,
                             uint64_t *out_ids,
                             uint32_t capacity);

/**
 * Create a k-DOP collider builder from a point cloud.
 *
 * # Safety
 *
 * `points_xyz` must point to at least 3×point_count readable f64s. The
 * returned builder handle is owned by the caller and must be released
 * through the collider-builder destroy function.
 */
struct ColliderBuilderHandle *collider_builder_create_kdop(const double *points_xyz,
                                                           uint32_t point_count,
                                                           uint32_t preset);

/**
 * Create a fixed-directions-hull (FDH) collider builder from a point cloud.
 *
 * # Safety
 *
 * `points_xyz` must point to at least 3×point_count readable f64s and
 * `directions_xyz` to at least 3×direction_count readable f64s. The returned
 * builder handle is owned by the caller and must be released through the
 * collider-builder destroy function.
 */
struct ColliderBuilderHandle *collider_builder_create_fdh(const double *points_xyz,
                                                          uint32_t point_count,
                                                          const double *directions_xyz,
                                                          uint32_t direction_count);

/**
 * Current thread's last error code (`ERR_OK` when no error).
 *
 * # Safety
 *
 * No pointer parameters; safe to call from any thread. The error slot is
 * thread-local, so the result reflects only errors reported on the calling
 * thread.
 */
uint32_t last_error_code(void);

/**
 * Current thread's last error message ("ok" when no error).
 *
 * The returned pointer is borrowed from a thread-local slot owned by Rust;
 * it is invalidated by the next error-reporting call on the same thread and
 * must not be freed or stored.
 *
 * # Safety
 *
 * No pointer parameters; safe to call from any thread. The returned pointer
 * is borrowed from a thread-local slot owned by Rust (no ownership transfer):
 * it remains valid only until the next error-reporting call on the same
 * thread and must not be freed by the caller.
 */
const char *last_error_message(void);

/**
 * Reset the current thread's error slot to `ERR_OK` / "ok".
 *
 * # Safety
 *
 * No pointer parameters; safe to call from any thread. Only the calling
 * thread's error slot is affected.
 */
void last_error_clear(void);

/**
 * Static name of an error code ("ERR_OK", "ERR_NULL_POINTER", ...).
 *
 * Unknown codes yield "ERR_UNKNOWN". The returned pointer refers to a
 * string with `'static` lifetime owned by Rust; it must not be freed.
 *
 * # Safety
 *
 * No pointer parameters; safe to call from any thread with any `code` value
 * (unknown codes return "ERR_UNKNOWN"). The returned pointer refers to a
 * `'static` string owned by Rust (no ownership transfer) and must not be
 * freed by the caller.
 */
const char *error_code_name(uint32_t code);

/**
 * Set (or disable) the Coulomb friction law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_coulomb_friction_law(struct WorldHandle *world, CoulombFrictionLaw law);

/**
 * `u8`-returning variant of `world_set_coulomb_friction_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_coulomb_friction_law`.
 */
uint8_t world_set_coulomb_friction_law_flag(struct WorldHandle *world, CoulombFrictionLaw law);

/**
 * Clear the Coulomb friction law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_coulomb_friction_law(struct WorldHandle *world);

/**
 * Read the current Coulomb friction law into `out_law`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_law` must point to writable memory for one `CoulombFrictionLaw`.
 * Null pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_get_coulomb_friction_law(const struct WorldHandle *world, CoulombFrictionLaw *out_law);

/**
 * Set (or disable) the air drag law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_air_drag_law(struct WorldHandle *world, AirDragLaw law);

/**
 * `u8`-returning variant of `world_set_air_drag_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_air_drag_law`.
 */
uint8_t world_set_air_drag_law_flag(struct WorldHandle *world, AirDragLaw law);

/**
 * Clear the air drag law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_air_drag_law(struct WorldHandle *world);

/**
 * Read the current air drag law into `out_law`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_law` must point to writable memory for one `AirDragLaw`. Null
 * pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_get_air_drag_law(const struct WorldHandle *world, AirDragLaw *out_law);

/**
 * Set (or disable) the external force law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_external_force_law(struct WorldHandle *world, ExternalForceLaw law);

/**
 * `u8`-returning variant of `world_set_external_force_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_external_force_law`.
 */
uint8_t world_set_external_force_law_flag(struct WorldHandle *world, ExternalForceLaw law);

/**
 * Clear the external force law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_external_force_law(struct WorldHandle *world);

/**
 * Read the current external force law into `out_law`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_law` must point to writable memory for one `ExternalForceLaw`. Null
 * pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_get_external_force_law(const struct WorldHandle *world, ExternalForceLaw *out_law);

/**
 * Set (or disable) the Newton gravity law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_newton_gravity_law(struct WorldHandle *world, NewtonGravityLaw law);

/**
 * `u8`-returning variant of `world_set_newton_gravity_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_newton_gravity_law`.
 */
uint8_t world_set_newton_gravity_law_flag(struct WorldHandle *world, NewtonGravityLaw law);

/**
 * Register a polyhedron terrain-gravity law (Werner & Scheeres 1997) on the
 * world.  `vertices_xyz` is a flat `[x,y,z]` array (3·n_vertices f64),
 * `face_indices` a flat `[a,b,c]` array (3·n_faces u32), `density` the
 * constant density (kg/m³).  Replaces any prior terrain-gravity law.
 *
 * # Safety
 * `world` must be a valid world pointer; `vertices_xyz`/`face_indices` must
 * point to readable arrays of the declared sizes.
 */
Bool world_register_terrain_gravity_polyhedron(struct WorldHandle *world,
                                               const double *vertices_xyz,
                                               uint32_t n_vertices,
                                               const uint32_t *face_indices,
                                               uint32_t n_faces,
                                               double density);

/**
 * Register a DEM surface-mass-distribution terrain-gravity law (direct
 * summation) on the world.  `dem` is a flat `[nx·ny]` height map (m above the
 * reference ellipsoid); `resolution`/`reference_radius` define the grid (m);
 * `surface_density` is kg/m².  Replaces any prior terrain-gravity law.
 *
 * # Safety
 * `world` must be a valid world pointer; `dem` must point to `nx·ny` readable
 * f64s.
 */
Bool world_register_terrain_gravity_dem(struct WorldHandle *world,
                                        const double *dem,
                                        uint32_t nx,
                                        uint32_t ny,
                                        double resolution,
                                        double reference_radius,
                                        double surface_density);

/**
 * Register the built-in lunar-mascon terrain-gravity law (GRAIL-derived,
 * Plummer-softened point masses).  Replaces any prior terrain-gravity law.
 *
 * # Safety
 * `world` must be a valid world pointer.
 */
Bool world_register_terrain_gravity_mascon(struct WorldHandle *world);

/**
 * Unregister the terrain-gravity law from the world (disables terrain
 * gravity; uniform `world.gravity` still applies if it is non-zero).
 *
 * # Safety
 * `world` must be a valid world pointer.
 */
Bool world_unregister_terrain_gravity(struct WorldHandle *world);

/**
 * Clear the Newton gravity law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_newton_gravity_law(struct WorldHandle *world);

/**
 * Read the current Newton gravity law into `out_law`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_law` must point to writable memory for one `NewtonGravityLaw`. Null
 * pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_get_newton_gravity_law(const struct WorldHandle *world, NewtonGravityLaw *out_law);

/**
 * Read the last custom-physics report into `out_report`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_report` must point to writable memory for one `CustomPhysicsReport`.
 * Null pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_get_custom_physics_report(const struct WorldHandle *world,
                                     CustomPhysicsReport *out_report);

/**
 * Clear the legacy event queues of a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_events(struct WorldHandle *world);

/**
 * Number of queued collision events (legacy Vec queue).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer returns 0.
 */
uint32_t world_collision_event_count(const struct WorldHandle *world);

/**
 * Read one queued collision event by index (legacy Vec queue).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer or out-of-range index returns a zeroed record.
 */
CollisionEventRecord world_get_collision_event(const struct WorldHandle *world, uint32_t index);

/**
 * Copy up to `capacity` queued collision events into `out_events`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_events` must point to writable memory for `capacity`
 * `CollisionEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
 */
uint32_t world_get_collision_events(const struct WorldHandle *world,
                                    CollisionEventRecord *out_events,
                                    uint32_t capacity);

/**
 * Number of queued contact-force events (legacy Vec queue).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer returns 0.
 */
uint32_t world_contact_force_event_count(const struct WorldHandle *world);

/**
 * Read one queued contact-force event by index (legacy Vec queue).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer or out-of-range index returns a zeroed record.
 */
ContactForceEventRecord world_get_contact_force_event(const struct WorldHandle *world,
                                                      uint32_t index);

/**
 * Copy up to `capacity` queued contact-force events into `out_events`.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_events` must point to writable memory for `capacity`
 * `ContactForceEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
 */
uint32_t world_get_contact_force_events(const struct WorldHandle *world,
                                        ContactForceEventRecord *out_events,
                                        uint32_t capacity);

/**
 * Disabled external contact-pair filter callback (always reports
 * `ERR_UNSUPPORTED` and reinstalls the default hooks).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
void world_set_contact_pair_filter_callback(struct WorldHandle *world,
                                            uintptr_t _callback,
                                            uintptr_t _user_data);

/**
 * Disabled external intersection-pair filter callback (always reports
 * `ERR_UNSUPPORTED` and reinstalls the default hooks).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
void world_set_intersection_pair_filter_callback(struct WorldHandle *world,
                                                 uintptr_t _callback,
                                                 uintptr_t _user_data);

/**
 * Reinstall the default contact-pair filter hooks.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_contact_pair_filter_callback(struct WorldHandle *world);

/**
 * Reinstall the default intersection-pair filter hooks.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_intersection_pair_filter_callback(struct WorldHandle *world);

/**
 * Allocate a collision-event ring buffer of `capacity` records.
 * Events will be written here during `world_step` instead of (or in addition to)
 * the legacy Vec queue.  Java drains the ring buffer at its own pace.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`.
 * Init-time only: must be called before `world_step` runs on any thread and
 * with no concurrent event-ring FFI calls on the same world.  The producer
 * cache is an `UnsafeCell`; violations of this contract are caught at runtime
 * and fail with `ERR_UNSUPPORTED` (see the `events` module docs).
 */
Bool world_init_collision_event_ring(struct WorldHandle *world, uint32_t capacity);

/**
 * Allocate a contact-force-event ring buffer.
 *
 * # Safety
 *
 * Same init-time-only contract as `world_init_collision_event_ring`.
 */
Bool world_init_contact_force_event_ring(struct WorldHandle *world, uint32_t capacity);

/**
 * Drain the collision-event ring buffer into `out_events`.
 * Returns the number of events drained.  This is the **only** FFI call needed
 * per frame after init — no more count-then-allocate-then-read cycles.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_events` must point to writable memory for `capacity`
 * `CollisionEventRecord` elements (`0 < capacity <= MAX_OUTPUT_CAPACITY`).
 * May run concurrently with `world_step` (SPSC drain), but only from a
 * single consumer thread.
 */
uint32_t world_drain_collision_event_ring(const struct WorldHandle *world,
                                          CollisionEventRecord *out_events,
                                          uint32_t capacity);

/**
 * Drain the contact-force-event ring buffer.
 *
 * # Safety
 *
 * Same contract as `world_drain_collision_event_ring`, with
 * `ContactForceEventRecord` output elements.
 */
uint32_t world_drain_contact_force_event_ring(const struct WorldHandle *world,
                                              ContactForceEventRecord *out_events,
                                              uint32_t capacity);

/**
 * Get the current number of events in the collision ring buffer (cheap, no lock).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer returns 0.
 */
uint32_t world_collision_event_ring_len(const struct WorldHandle *world);

/**
 * Get the current number of events in the contact-force ring buffer.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer returns 0.
 */
uint32_t world_contact_force_event_ring_len(const struct WorldHandle *world);

/**
 * Get ring buffer statistics (capacity, occupancy, drops, wraps).
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`;
 * `out_stats` must point to writable memory for one `EventRingBufferStats`.
 * Null pointers fail with `ERR_NULL_POINTER`.
 */
Bool world_collision_event_ring_stats(const struct WorldHandle *world,
                                      EventRingBufferStats *out_stats);

/**
 * Get contact-force ring buffer statistics.
 *
 * # Safety
 *
 * Same contract as `world_collision_event_ring_stats`.
 */
Bool world_contact_force_event_ring_stats(const struct WorldHandle *world,
                                          EventRingBufferStats *out_stats);

/**
 * Clear both ring buffers and reset drop counters.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_event_rings(struct WorldHandle *world);

/**
 * Register a collision-event callback.
 *
 * `callback` is a C function pointer (zero = unregister).
 * `user_data` is passed through unchanged to each invocation.
 * Returns an opaque handle for later unregistration.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`.
 * `callback` must be `0` ("unset") or the address of a function with the
 * exact `CollisionEventFn` signature that stays valid while registered.
 * Init-time only: must be called before `world_step` runs on any thread and
 * with no concurrent event-ring/callback FFI calls on the same world.  The
 * producer cache is an `UnsafeCell`; violations of this contract are caught
 * at runtime and fail with `ERR_UNSUPPORTED` (see the `events` module docs).
 */
EventCallbackHandle world_register_collision_callback(struct WorldHandle *world,
                                                      uintptr_t callback,
                                                      uintptr_t user_data);

/**
 * Register a contact-force-event callback.
 *
 * # Safety
 *
 * Same init-time-only contract as `world_register_collision_callback`;
 * `callback` must be `0` ("unset") or the address of a function with the
 * exact `ContactForceEventFn` signature that stays valid while registered.
 */
EventCallbackHandle world_register_contact_force_callback(struct WorldHandle *world,
                                                          uintptr_t callback,
                                                          uintptr_t user_data);

/**
 * Unregister a previously registered callback by its handle.
 * Passing 0 or an invalid handle is a no-op.
 *
 * # Safety
 *
 * Same init-time-only contract as `world_register_collision_callback`.
 */
void world_unregister_callback(struct WorldHandle *world, EventCallbackHandle handle);

/**
 * Set the event dispatch mode.
 *
 * - `Poll` (0): legacy Vec queue only (default).
 * - `Callback` (1): registered callbacks only.
 * - `Both` (2): ring buffer + callbacks.
 *
 * # Safety
 *
 * Same init-time-only contract as `world_init_collision_event_ring`.
 */
Bool world_set_event_dispatch_mode(struct WorldHandle *world, uint32_t mode);

/**
 * Set (or disable) the solar-wind dynamic-pressure force law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_solar_wind_pressure_law(struct WorldHandle *world, SolarWindPressureLaw law);

/**
 * `u8`-returning variant of `world_set_solar_wind_pressure_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_solar_wind_pressure_law`.
 */
uint8_t world_set_solar_wind_pressure_law_flag(struct WorldHandle *world, SolarWindPressureLaw law);

/**
 * Clear the solar-wind pressure law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_solar_wind_pressure_law(struct WorldHandle *world);

/**
 * Set (or disable) the Chandrasekhar dynamical-friction force law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_dynamical_friction_law(struct WorldHandle *world, DynamicalFrictionLaw law);

/**
 * `u8`-returning variant of `world_set_dynamical_friction_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_dynamical_friction_law`.
 */
uint8_t world_set_dynamical_friction_law_flag(struct WorldHandle *world, DynamicalFrictionLaw law);

/**
 * Clear the dynamical-friction law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_dynamical_friction_law(struct WorldHandle *world);

/**
 * Set (or disable) the MOND-corrected gravity force law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_mond_gravity_law(struct WorldHandle *world, MonDGravityLaw law);

/**
 * `u8`-returning variant of `world_set_mond_gravity_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_mond_gravity_law`.
 */
uint8_t world_set_mond_gravity_law_flag(struct WorldHandle *world, MonDGravityLaw law);

/**
 * Clear the MOND gravity law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_mond_gravity_law(struct WorldHandle *world);

/**
 * Set (or disable) the Eddington-limited radiation-pressure force law on
 * a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_eddington_radiation_pressure_law(struct WorldHandle *world,
                                                EddingtonRadiationPressureLaw law);

/**
 * `u8`-returning variant of `world_set_eddington_radiation_pressure_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_eddington_radiation_pressure_law`.
 */
uint8_t world_set_eddington_radiation_pressure_law_flag(struct WorldHandle *world,
                                                        EddingtonRadiationPressureLaw law);

/**
 * Clear the Eddington radiation-pressure law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_eddington_radiation_pressure_law(struct WorldHandle *world);

/**
 * Set (or disable) the X-ray disc bolometric irradiation force law on a
 * world.  See `XrayIrradiationLaw` doc for parameter semantics.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_xray_irradiation_law(struct WorldHandle *world, XrayIrradiationLaw law);

/**
 * `u8`-returning variant of `world_set_xray_irradiation_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_xray_irradiation_law`.
 */
uint8_t world_set_xray_irradiation_law_flag(struct WorldHandle *world, XrayIrradiationLaw law);

/**
 * Clear the X-ray irradiation law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_xray_irradiation_law(struct WorldHandle *world);

/**
 * Set (or disable) the pulsar magnetic-dipole torque law on a world.
 * See `PulsarMagneticDipoleLaw` doc for parameter semantics.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_pulsar_magnetic_dipole_law(struct WorldHandle *world, PulsarMagneticDipoleLaw law);

/**
 * `u8`-returning variant of `world_set_pulsar_magnetic_dipole_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_pulsar_magnetic_dipole_law`.
 */
uint8_t world_set_pulsar_magnetic_dipole_law_flag(struct WorldHandle *world,
                                                  PulsarMagneticDipoleLaw law);

/**
 * Clear the pulsar magnetic-dipole torque law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_pulsar_magnetic_dipole_law(struct WorldHandle *world);

/**
 * Set (or disable) the Jeans-escape drag force law on a world.
 * See `JeansEscapeLaw` doc for parameter semantics.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer fails with `ERR_NULL_POINTER`.
 */
Bool world_set_jeans_escape_law(struct WorldHandle *world, JeansEscapeLaw law);

/**
 * `u8`-returning variant of `world_set_jeans_escape_law`.
 *
 * # Safety
 *
 * Same contract as `world_set_jeans_escape_law`.
 */
uint8_t world_set_jeans_escape_law_flag(struct WorldHandle *world, JeansEscapeLaw law);

/**
 * Clear the Jeans-escape drag law on a world.
 *
 * # Safety
 *
 * `world` must be a valid world pointer returned by `world_create`; a null
 * pointer is a no-op.
 */
void world_clear_jeans_escape_law(struct WorldHandle *world);

/**
 * # Safety
 *
 * `out_report` may be null or must point to writable space for one
 * `FluidForceReport`.
 */
Bool fluid_estimate_aabb_forces(FluidVolume fluid,
                                Vec3 body_center,
                                Vec3 body_half_extents,
                                double body_volume,
                                Vec3 body_linvel,
                                Vec3 body_angvel,
                                FluidForceReport *out_report);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_report` may be null or must
 * point to writable space for one `FluidForceReport`.
 */
Bool fluid_apply_aabb_forces(struct WorldHandle *world,
                             RigidBodyHandleRaw body_handle,
                             FluidVolume fluid,
                             Vec3 body_half_extents,
                             double body_volume,
                             Bool wake_up,
                             FluidForceReport *out_report);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_report` may be null or must
 * point to writable space for one `FluidForceReport`.
 */
uint8_t fluid_apply_aabb_forces_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw body_handle,
                                     FluidVolume fluid,
                                     Vec3 body_half_extents,
                                     double body_volume,
                                     Bool wake_up,
                                     FluidForceReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `NavierStokesReport`.
 */
Bool fluid_navier_stokes_simplified_step(Vec3 velocity,
                                         Vec3 advection,
                                         Vec3 pressure_gradient,
                                         Vec3 laplacian_velocity,
                                         Vec3 external_acceleration,
                                         double density,
                                         double kinematic_viscosity,
                                         double dt,
                                         NavierStokesReport *out_report);

/**
 * Evaluates the SPH poly6 kernel for a distance and smoothing radius.
 *
 * # Safety
 *
 * This function takes no pointers; all inputs are passed by value and there
 * are no safety requirements on the caller.
 */
double fluid_sph_poly6_kernel(double distance, double smoothing_radius);

/**
 * # Safety
 *
 * `out_gradient` must point to writable space for one `Vec3`.
 */
Bool fluid_sph_spiky_gradient(Vec3 offset, double smoothing_radius, Vec3 *out_gradient);

/**
 * Evaluates the Laplacian of the SPH viscosity kernel for a distance and
 * smoothing radius.
 *
 * # Safety
 *
 * This function takes no pointers; all inputs are passed by value and there
 * are no safety requirements on the caller.
 */
double fluid_sph_viscosity_laplacian(double distance, double smoothing_radius);

/**
 * # Safety
 *
 * `particles` must point to `particle_count` `SphParticle` values (or be
 * null when `particle_count` is 0); `out_density` must point to writable
 * space for one `f64`.
 */
Bool fluid_sph_estimate_density(Vec3 position,
                                const SphParticle *particles,
                                uint32_t particle_count,
                                double smoothing_radius,
                                double *out_density);

/**
 * # Safety
 *
 * `particles` must point to `particle_count` `SphParticle` values (or be
 * null when `particle_count` is 0); `out_report` must point to writable
 * space for one `SphForceReport`.
 */
Bool fluid_sph_estimate_forces(SphParticle particle,
                               const SphParticle *particles,
                               uint32_t particle_count,
                               double smoothing_radius,
                               double gas_constant,
                               double rest_density,
                               double viscosity,
                               double surface_tension,
                               SphForceReport *out_report);

/**
 * Computes the static pressure from a Bernoulli-equation total pressure.
 *
 * # Safety
 *
 * This function takes no pointers; all inputs are passed by value and there
 * are no safety requirements on the caller.
 */
double fluid_bernoulli_pressure(double total_pressure,
                                double density,
                                double velocity,
                                double gravity,
                                double elevation);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `BernoulliReport`.
 */
Bool fluid_bernoulli_report(double pressure,
                            double density,
                            double velocity,
                            double gravity,
                            double elevation,
                            BernoulliReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `StressIntensityReport`.
 */
Bool fracture_stress_intensity_factor(double stress,
                                      double crack_length,
                                      double geometry_factor,
                                      double fracture_toughness,
                                      StressIntensityReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `GriffithReport`.
 */
Bool fracture_griffith_criterion(double stress,
                                 double crack_length,
                                 FractureMaterial material,
                                 GriffithReport *out_report);

/**
 * # Safety
 *
 * `cycle_counts` and `cycles_to_failure` must each point to `count` `f64`
 * values; `out_report` must point to writable space for one
 * `MinerDamageReport`.
 */
Bool fracture_miner_damage(const double *cycle_counts,
                           const double *cycles_to_failure,
                           uint32_t count,
                           MinerDamageReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `SnCurveReport`.
 */
Bool fracture_sn_curve_life(double stress_amplitude,
                            double coefficient,
                            double exponent,
                            double endurance_limit,
                            SnCurveReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `FractureEnergyReport`.
 */
Bool fracture_energy_release(double strain_energy,
                             double new_surface_area,
                             double surface_energy,
                             double kinetic_energy,
                             FractureEnergyReport *out_report);

/**
 * # Safety
 *
 * `out_report` must point to writable space for one `FractureModeReport`.
 */
Bool fracture_mode_from_stress(double tensile_stress,
                               double shear_stress,
                               double compressive_stress,
                               FractureModeReport *out_report);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `fragments` must point to
 * `fragment_count` `FractureFragmentDesc` values; `out_body_handles` must
 * point to writable space for `capacity` body handles; `out_joint_handles`
 * must point to writable space for `capacity` joint handles when
 * `connect_fragments` is non-zero; `out_report` may be null or must point
 * to writable space for one `FractureReplaceReport`.
 */
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

/**
 * Creates a joint builder of the given type and returns an owned pointer to it.
 *
 * # Safety
 *
 * No pointers are dereferenced. The returned pointer is owned by the caller and
 * must be released with `joint_builder_destroy` (or consumed by
 * `world_insert_impulse_joint`). Invalid parameters fail with
 * `ERR_INVALID_ARGUMENT` and return null.
 */
struct JointBuilderHandle *joint_builder_create(uint32_t joint_type,
                                                Vec3 axis_or_primary,
                                                double b,
                                                double c);

/**
 * # Safety
 *
 * `builder` must be a pointer returned by `joint_builder_create` (or null, which is a
 * no-op). Ownership is transferred to Rust and the pointer must not be used after
 * this call.
 */
void joint_builder_destroy(struct JointBuilderHandle *builder);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_contacts_enabled(struct JointBuilderHandle *builder, Bool enabled);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_local_anchor1(struct JointBuilderHandle *builder, Vec3 anchor);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_local_anchor2(struct JointBuilderHandle *builder, Vec3 anchor);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_limits(struct JointBuilderHandle *builder,
                              uint32_t axis,
                              double min,
                              double max);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_motor_velocity(struct JointBuilderHandle *builder,
                                      uint32_t axis,
                                      double target_vel,
                                      double factor);

/**
 * # Safety
 *
 * `builder` must be a valid pointer returned by `joint_builder_create` and must
 * remain alive for the duration of the call.
 */
void joint_builder_set_motor_position(struct JointBuilderHandle *builder,
                                      uint32_t axis,
                                      double target_pos,
                                      double stiffness,
                                      double damping);

/**
 * # Safety
 *
 * `world` must be a valid world pointer. `builder` must be a pointer returned by
 * `joint_builder_create`; on success its ownership is consumed by this call and it
 * must not be used afterwards.
 */
ImpulseJointHandleRaw world_insert_impulse_joint(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body1,
                                                 RigidBodyHandleRaw body2,
                                                 struct JointBuilderHandle *builder,
                                                 Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a valid world pointer and must remain alive for the duration of
 * the call.
 */
Bool world_remove_impulse_joint(struct WorldHandle *world,
                                ImpulseJointHandleRaw handle,
                                Bool wake_up);

/**
 * Computes the Lennard-Jones potential at `distance` for well depth `epsilon`
 * and size parameter `sigma`; returns `NaN` with `ERR_INVALID_ARGUMENT` on
 * invalid parameters.
 *
 * # Safety
 *
 * This function takes no pointers and transfers no ownership; it is always
 * safe to call.
 */
double molecular_lennard_jones_potential(double distance, double epsilon, double sigma);

/**
 * # Safety
 *
 * `out_force` must be null or point to a valid, writable `Vec3`.
 */
Bool molecular_lennard_jones_force(Vec3 displacement,
                                   double epsilon,
                                   double sigma,
                                   double softening,
                                   Vec3 *out_force);

/**
 * Computes the Coulomb potential between `charge_a` and `charge_b` at
 * `distance`; returns `NaN` with `ERR_INVALID_ARGUMENT` on invalid parameters.
 *
 * # Safety
 *
 * This function takes no pointers and transfers no ownership; it is always
 * safe to call.
 */
double molecular_coulomb_potential(double distance,
                                   double charge_a,
                                   double charge_b,
                                   double coulomb_constant,
                                   double relative_permittivity);

/**
 * # Safety
 *
 * `out_force` must be null or point to a valid, writable `Vec3`.
 */
Bool molecular_coulomb_force(Vec3 displacement,
                             double charge_a,
                             double charge_b,
                             double coulomb_constant,
                             double relative_permittivity,
                             double softening,
                             Vec3 *out_force);

/**
 * # Safety
 *
 * `out_report` must be null or point to a valid, writable `MolecularPairReport`.
 */
Bool molecular_pair_interaction(MolecularParticle particle_a,
                                MolecularParticle particle_b,
                                MolecularForceLaw law,
                                MolecularPairReport *out_report);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle. `out_report` must be null or
 * point to a valid, writable `MolecularPairReport`.
 */
Bool molecular_apply_pair_forces(struct WorldHandle *world,
                                 RigidBodyHandleRaw body_a,
                                 RigidBodyHandleRaw body_b,
                                 MolecularParticle particle_a,
                                 MolecularParticle particle_b,
                                 MolecularForceLaw law,
                                 Bool wake_up,
                                 MolecularPairReport *out_report);

/**
 * # Safety
 *
 * Same pointer contract as `molecular_apply_pair_forces`.
 */
uint8_t molecular_apply_pair_forces_flag(struct WorldHandle *world,
                                         RigidBodyHandleRaw body_a,
                                         RigidBodyHandleRaw body_b,
                                         MolecularParticle particle_a,
                                         MolecularParticle particle_b,
                                         MolecularForceLaw law,
                                         Bool wake_up,
                                         MolecularPairReport *out_report);

/**
 * Returns the vacuum Coulomb constant (Coulomb's constant in vacuum).
 *
 * # Safety
 *
 * This function takes no pointers and transfers no ownership; it is always
 * safe to call.
 */
double molecular_vacuum_coulomb_constant(void);

/**
 * Return the number of weights the network layout requires.
 *
 * # Safety
 *
 * This function takes no pointers; any `u32` inputs are safe to pass.
 */
uint32_t neural_bounds_required_weight_count(uint32_t hidden_width, uint32_t hidden_layers);

/**
 * Create a collider builder whose shape is a neural-network-expanded bounds hull.
 *
 * # Safety
 *
 * `weights` must point to a readable buffer of `weight_count` `f64` values.
 * The returned pointer is owned by the caller and must be consumed or freed
 * through the collider-builder ABI.
 */
struct ColliderBuilderHandle *collider_builder_create_neural_bounds(NeuralBoundsDesc desc,
                                                                    const double *weights,
                                                                    uint32_t weight_count);

/**
 * Count the colliders intersecting a neural-bounds shape.
 *
 * # Safety
 *
 * `world` must be a valid world pointer, and `weights` must point to a
 * readable buffer of `weight_count` `f64` values.
 */
uint32_t query_intersect_neural_bounds_count(const struct WorldHandle *world,
                                             NeuralBoundsDesc desc,
                                             const double *weights,
                                             uint32_t weight_count,
                                             QueryFilterDesc filter);

/**
 * Count the colliders intersecting a neural-bounds shape with a default filter.
 *
 * # Safety
 *
 * `world` must be a valid world pointer, and `weights` must point to a
 * readable buffer of `weight_count` `f64` values.
 */
uint32_t query_intersect_neural_bounds_count_all(const struct WorldHandle *world,
                                                 NeuralBoundsDesc desc,
                                                 const double *weights,
                                                 uint32_t weight_count);

/**
 * Write the handles of colliders intersecting a neural-bounds shape.
 *
 * # Safety
 *
 * `world` must be a valid world pointer, `weights` must point to a readable
 * buffer of `weight_count` `f64` values, and `out_handles` must point to a
 * writable buffer of at least `capacity` handle elements.
 */
uint32_t query_intersect_neural_bounds(const struct WorldHandle *world,
                                       NeuralBoundsDesc desc,
                                       const double *weights,
                                       uint32_t weight_count,
                                       QueryFilterDesc filter,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

/**
 * Write the handles of colliders intersecting a neural-bounds shape with a
 * default filter.
 *
 * # Safety
 *
 * `world` must be a valid world pointer, `weights` must point to a readable
 * buffer of `weight_count` `f64` values, and `out_handles` must point to a
 * writable buffer of at least `capacity` handle elements.
 */
uint32_t query_intersect_neural_bounds_all(const struct WorldHandle *world,
                                           NeuralBoundsDesc desc,
                                           const double *weights,
                                           uint32_t weight_count,
                                           ColliderHandleRaw *out_handles,
                                           uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
RayHit query_cast_ray(const struct WorldHandle *world,
                      Vec3 origin,
                      Vec3 direction,
                      double max_toi,
                      Bool solid,
                      QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_hit` may be null or must point
 * to writable space for one `RayHit`.
 */
ColliderHandleRaw query_cast_ray_out(const struct WorldHandle *world,
                                     Vec3 origin,
                                     Vec3 direction,
                                     double max_toi,
                                     Bool solid,
                                     QueryFilterDesc filter,
                                     RayHit *out_hit);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `rays` must point to `ray_count * 6`
 * `f64` values and `out_hits` to writable space for `capacity` `RayHit`s.
 */
uint32_t query_cast_rays(const struct WorldHandle *world,
                         const double *rays,
                         uint32_t ray_count,
                         double max_toi,
                         Bool solid,
                         QueryFilterDesc filter,
                         RayHit *out_hits,
                         uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_collider` may be null or must
 * point to writable space for one collider handle.
 */
PointProjection query_project_point(const struct WorldHandle *world,
                                    Vec3 point,
                                    double max_dist,
                                    Bool solid,
                                    QueryFilterDesc filter,
                                    ColliderHandleRaw *out_collider);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_collider` and `out_projection`
 * may be null or must point to writable space for one value each.
 */
ColliderHandleRaw query_project_point_out(const struct WorldHandle *world,
                                          Vec3 point,
                                          double max_dist,
                                          Bool solid,
                                          QueryFilterDesc filter,
                                          ColliderHandleRaw *out_collider,
                                          PointProjection *out_projection);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_point_count(const struct WorldHandle *world,
                                     Vec3 point,
                                     QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_aabb_count(const struct WorldHandle *world,
                                    AabbDesc aabb,
                                    QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_aabb(const struct WorldHandle *world,
                              AabbDesc aabb,
                              QueryFilterDesc filter,
                              ColliderHandleRaw *out_handles,
                              uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_aabb_count_all(const struct WorldHandle *world, AabbDesc aabb);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `aabbs` must point to `query_count`
 * `AabbDesc` values and `out_counts` to writable space for `capacity` `u32`s.
 */
uint32_t query_intersect_aabb_counts(const struct WorldHandle *world,
                                     const AabbDesc *aabbs,
                                     uint32_t query_count,
                                     QueryFilterDesc filter,
                                     uint32_t *out_counts,
                                     uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_obb_count(const struct WorldHandle *world,
                                   Obb obb,
                                   QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_obb_count_all(const struct WorldHandle *world, Obb obb);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `obbs` must point to `query_count`
 * `Obb` values and `out_counts` to writable space for `capacity` `u32`s.
 */
uint32_t query_intersect_obb_counts(const struct WorldHandle *world,
                                    const Obb *obbs,
                                    uint32_t query_count,
                                    QueryFilterDesc filter,
                                    uint32_t *out_counts,
                                    uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_obb(const struct WorldHandle *world,
                             Obb obb,
                             QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_obb_all(const struct WorldHandle *world,
                                 Obb obb,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_sphere_count(const struct WorldHandle *world,
                                      Sphere sphere,
                                      QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_sphere_count_all(const struct WorldHandle *world, Sphere sphere);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `spheres` must point to `query_count`
 * `Sphere` values and `out_counts` to writable space for `capacity` `u32`s.
 */
uint32_t query_intersect_sphere_counts(const struct WorldHandle *world,
                                       const Sphere *spheres,
                                       uint32_t query_count,
                                       QueryFilterDesc filter,
                                       uint32_t *out_counts,
                                       uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_sphere(const struct WorldHandle *world,
                                Sphere sphere,
                                QueryFilterDesc filter,
                                ColliderHandleRaw *out_handles,
                                uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` collider handles.
 */
uint32_t query_intersect_sphere_all(const struct WorldHandle *world,
                                    Sphere sphere,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
uint32_t query_intersect_aabb_rigid_body_count_all(const struct WorldHandle *world, AabbDesc aabb);

/**
 * # Safety
 *
 * `world` must be a valid world handle and `out_handles` must point to
 * writable space for at least `capacity` rigid body handles.
 */
uint32_t query_intersect_aabb_rigid_bodies_all(const struct WorldHandle *world,
                                               AabbDesc aabb,
                                               RigidBodyHandleRaw *out_handles,
                                               uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be a valid world handle.
 */
ShapeCastHit query_cast_shape(const struct WorldHandle *world,
                              ShapeDesc shape_desc,
                              Vec3 translation,
                              Quat rotation,
                              Vec3 velocity,
                              ShapeCastOptionsDesc options,
                              QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be a valid world handle; `out_hit` may be null or must point
 * to writable space for one `ShapeCastHit`.
 */
ColliderHandleRaw query_cast_shape_out(const struct WorldHandle *world,
                                       ShapeDesc shape_desc,
                                       Vec3 translation,
                                       Quat rotation,
                                       Vec3 velocity,
                                       ShapeCastOptionsDesc options,
                                       QueryFilterDesc filter,
                                       ShapeCastHit *out_hit);

/**
 * Creates a rigid body builder for the given body status.
 *
 * # Safety
 *
 * Takes no pointers. The returned pointer is owned by the caller and must be released with
 * `rigid_body_builder_build` or `rigid_body_builder_destroy`.
 */
struct RigidBodyBuilderHandle *rigid_body_builder_create(uint32_t status);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create` (or null); ownership
 * is taken and the pointer must not be used afterwards.
 */
RigidBody *rigid_body_builder_build(struct RigidBodyBuilderHandle *builder);

/**
 * # Safety
 *
 * `builder` must be a pointer returned by `rigid_body_builder_create` (or null, which is a
 * no-op); ownership is taken and the pointer must not be used afterwards.
 */
void rigid_body_builder_destroy(struct RigidBodyBuilderHandle *builder);

/**
 * # Safety
 *
 * `rigid_body` must be a pointer returned by `rigid_body_builder_build` or
 * `world_copy_rigid_body` (or null, which is a no-op); ownership is taken and the pointer must
 * not be used afterwards.
 */
void rigid_body_destroy_raw(RigidBody *rigid_body);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_translation(struct RigidBodyBuilderHandle *builder, Vec3 translation);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_rotation(struct RigidBodyBuilderHandle *builder,
                                     Vec3 rotation_axis_angle);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_pose(struct RigidBodyBuilderHandle *builder,
                                 Vec3 translation,
                                 Quat rotation);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_additional_mass_properties(struct RigidBodyBuilderHandle *builder,
                                                       Vec3 center,
                                                       double mass,
                                                       Vec3 inertia);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_linvel(struct RigidBodyBuilderHandle *builder, Vec3 linvel);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_angvel(struct RigidBodyBuilderHandle *builder, Vec3 angvel);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_gravity_scale(struct RigidBodyBuilderHandle *builder,
                                          double gravity_scale);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_linear_damping(struct RigidBodyBuilderHandle *builder,
                                           double linear_damping);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_angular_damping(struct RigidBodyBuilderHandle *builder,
                                            double angular_damping);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_can_sleep(struct RigidBodyBuilderHandle *builder, Bool can_sleep);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_enabled_rotations(struct RigidBodyBuilderHandle *builder,
                                              Bool allow_x,
                                              Bool allow_y,
                                              Bool allow_z);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_user_data(struct RigidBodyBuilderHandle *builder,
                                      uint64_t user_data_low,
                                      uint64_t user_data_high);

/**
 * # Safety
 *
 * `builder` must be a live pointer returned by `rigid_body_builder_create`, or null.
 */
void rigid_body_builder_set_additional_mass(struct RigidBodyBuilderHandle *builder, double mass);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. `memory_handle` must be a
 * pointer returned by `rigid_body_builder_build`; ownership is taken and the pointer must not be
 * used afterwards.
 */
RigidBodyHandleRaw world_insert_rigid_body(struct WorldHandle *world, RigidBody *memory_handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool world_remove_rigid_body(struct WorldHandle *world,
                             RigidBodyHandleRaw handle,
                             Bool remove_attached_colliders);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. The returned pointer is
 * owned by the caller and must be released with `rigid_body_destroy_raw`.
 */
RigidBody *world_copy_rigid_body(struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t world_remove_rigid_body_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Bool remove_attached_colliders);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint32_t rigid_body_get_status(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_status(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           uint32_t status,
                           Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Vec3 rigid_body_get_translation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. `out_translation` must be
 * a valid writable pointer to a `Vec3`, or null.
 */
void rigid_body_get_translation_out(const struct WorldHandle *world,
                                    RigidBodyHandleRaw handle,
                                    Vec3 *out_translation);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Quat rigid_body_get_rotation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. `out_rotation` must be a
 * valid writable pointer to a `Quat`, or null.
 */
void rigid_body_get_rotation_out(const struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 Quat *out_rotation);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_pose(struct WorldHandle *world,
                         RigidBodyHandleRaw handle,
                         Vec3 translation,
                         Quat rotation,
                         Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_translation(struct WorldHandle *world,
                                RigidBodyHandleRaw handle,
                                Vec3 translation,
                                Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_set_translation_flag(struct WorldHandle *world,
                                        RigidBodyHandleRaw handle,
                                        Vec3 translation,
                                        Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_rotation(struct WorldHandle *world,
                             RigidBodyHandleRaw handle,
                             Quat rotation,
                             Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_set_rotation_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Quat rotation,
                                     Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_set_pose_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 Vec3 translation,
                                 Quat rotation,
                                 Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
double rigid_body_get_mass(struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Vec3 rigid_body_get_force(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Vec3 rigid_body_get_linvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. `out_linvel` must be a
 * valid writable pointer to a `Vec3`, or null.
 */
void rigid_body_get_linvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               Vec3 *out_linvel);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_linvel(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 linvel,
                           Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_set_linvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 linvel,
                                   Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Vec3 rigid_body_get_angvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null. `out_angvel` must be a
 * valid writable pointer to a `Vec3`, or null.
 */
void rigid_body_get_angvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               Vec3 *out_angvel);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_set_angvel(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 angvel,
                           Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_set_angvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 angvel,
                                   Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_add_force(struct WorldHandle *world,
                          RigidBodyHandleRaw handle,
                          Vec3 force,
                          Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_add_force_at_point(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 force,
                                   Vec3 point,
                                   Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_add_force_at_local_point(struct WorldHandle *world,
                                         RigidBodyHandleRaw handle,
                                         Vec3 force,
                                         Vec3 local_point,
                                         Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_add_torque_at_local_point(struct WorldHandle *world,
                                          RigidBodyHandleRaw handle,
                                          Vec3 torque,
                                          Vec3 _local_point,
                                          Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_add_force_at_local_point_flag(struct WorldHandle *world,
                                                 RigidBodyHandleRaw handle,
                                                 Vec3 force,
                                                 Vec3 local_point,
                                                 Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_add_torque_at_local_point_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw handle,
                                                  Vec3 torque,
                                                  Vec3 local_point,
                                                  Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_reset_force(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_add_force_flag(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  Vec3 force,
                                  Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_add_torque(struct WorldHandle *world,
                           RigidBodyHandleRaw handle,
                           Vec3 torque,
                           Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_reset_torque(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_add_torque_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Vec3 torque,
                                   Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_apply_impulse(struct WorldHandle *world,
                              RigidBodyHandleRaw handle,
                              Vec3 impulse,
                              Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_apply_impulse_flag(struct WorldHandle *world,
                                      RigidBodyHandleRaw handle,
                                      Vec3 impulse,
                                      Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_apply_torque_impulse(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     Vec3 torque_impulse,
                                     Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_apply_torque_impulse_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw handle,
                                             Vec3 torque_impulse,
                                             Bool wake_up);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_enable_ccd(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool enabled);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_enable_ccd_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   Bool enabled);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_sleep(struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_sleep_flag(struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_wake_up(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool strong);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_wake_up_flag(struct WorldHandle *world, RigidBodyHandleRaw handle, Bool strong);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
Bool rigid_body_is_sleeping(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * # Safety
 *
 * `world` must be a live pointer returned by `world_create`, or null.
 */
uint8_t rigid_body_is_sleeping_flag(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * Create an empty R-tree index.
 *
 * # Safety
 *
 * The returned pointer is owned by the caller and must be freed exactly once
 * with `rtree_destroy`.
 */
struct RTreeHandle *rtree_create(void);

/**
 * Destroy an R-tree index created by `rtree_create`.
 *
 * # Safety
 *
 * `tree` must be null or a pointer returned by `rtree_create`; it must not be
 * used again after this call.
 */
void rtree_destroy(struct RTreeHandle *tree);

/**
 * Remove every entry from the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
void rtree_clear(struct RTreeHandle *tree);

/**
 * Return the number of entries stored in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
uint32_t rtree_len(const struct RTreeHandle *tree);

/**
 * Insert or overwrite the bounds of `id` in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
Bool rtree_insert(struct RTreeHandle *tree, uint64_t id, AabbDesc aabb);

/**
 * Update the bounds of an existing `id` in the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
Bool rtree_update(struct RTreeHandle *tree, uint64_t id, AabbDesc aabb);

/**
 * Remove `id` from the tree.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
Bool rtree_remove(struct RTreeHandle *tree, uint64_t id);

/**
 * Force an immediate rebuild of the tree structure.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
void rtree_rebuild(struct RTreeHandle *tree);

/**
 * Count the entries whose bounds intersect `aabb`.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`.
 */
uint32_t rtree_query_aabb_count(struct RTreeHandle *tree, AabbDesc aabb);

/**
 * Write the ids of entries whose bounds intersect `aabb` into `out_ids`.
 *
 * # Safety
 *
 * `tree` must be a valid pointer returned by `rtree_create`, and `out_ids`
 * must point to a writable buffer of at least `capacity` `u64` elements.
 */
uint32_t rtree_query_aabb(struct RTreeHandle *tree,
                          AabbDesc aabb,
                          uint64_t *out_ids,
                          uint32_t capacity);

/**
 * # Safety
 * `out_probability` must be null or point to a valid, writable `CollisionProbability`.
 */
Bool space_debris_collision_probability(double miss_distance,
                                        double combined_radius,
                                        double sigma_radial,
                                        double sigma_intrack,
                                        CollisionProbability *out_probability);

/**
 * # Safety
 * `out_rates` must be null or point to a valid, writable `Sgp4SecularRates`.
 */
Bool space_sgp4_j2_secular_rates(double semi_major_axis,
                                 double eccentricity,
                                 double inclination,
                                 double mean_motion,
                                 double equatorial_radius,
                                 double j2,
                                 Sgp4SecularRates *out_rates);

/**
 * Computes the first (base) joint angle of a planar arm from the wrist position.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_arm_first_joint_inverse(double wrist_x, double wrist_y);

/**
 * Computes the third joint angle of a planar arm via the law of cosines.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_arm_third_joint_angle(double planar_radius,
                                   double vertical_offset,
                                   double link2,
                                   double link3,
                                   Bool elbow_up);

/**
 * # Safety
 * `out_command` must be null or point to a valid, writable `Vec3`.
 */
Bool space_artificial_potential_guidance(Vec3 position,
                                         Vec3 target,
                                         Vec3 obstacle,
                                         double attractive_gain,
                                         double repulsive_gain,
                                         double influence_radius,
                                         Vec3 *out_command);

/**
 * # Safety
 * `out_profile` must be null or point to a valid, writable `BangOffBangProfile`.
 */
Bool space_bang_off_bang_profile(double angle,
                                 double max_acceleration,
                                 double max_rate,
                                 BangOffBangProfile *out_profile);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `CwDerivative`.
 */
Bool space_cw_derivative(CwState state, double mean_motion, CwDerivative *out_derivative);

/**
 * # Safety
 * `out_transform` must be null or point to a valid, writable `DhTransform`.
 */
Bool space_dh_transform(double theta, double d, double a, double alpha, DhTransform *out_transform);

/**
 * Computes the kinetic energy a docking buffer must absorb, scaled by its efficiency.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_docking_buffer_energy(double relative_speed,
                                   double reduced_mass,
                                   double stroke,
                                   double efficiency);

/**
 * Computes a clamped closing-speed command for a docking glideslope.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_docking_glideslope_command(double range,
                                        double desired_slope,
                                        double closing_speed_limit);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `FlexibleModeDerivative`.
 */
Bool space_flexible_mode_derivative(double displacement,
                                    double velocity,
                                    double natural_frequency,
                                    double damping_ratio,
                                    double modal_force,
                                    double modal_mass,
                                    FlexibleModeDerivative *out_derivative);

/**
 * # Safety
 * `out_dynamics` must be null or point to a valid, writable `ManipulatorDynamics`.
 */
Bool space_manipulator_dynamics_diag(Vec3 mass_matrix_diag,
                                     Vec3 joint_acceleration,
                                     Vec3 coriolis,
                                     Vec3 gravity,
                                     ManipulatorDynamics *out_dynamics);

/**
 * # Safety
 * `out_properties` must be null or point to a valid, writable `MassProperties`.
 */
Bool space_mass_properties_two_body(double mass1,
                                    Vec3 position1,
                                    Vec3 inertia1_diag,
                                    double mass2,
                                    Vec3 position2,
                                    Vec3 inertia2_diag,
                                    MassProperties *out_properties);

/**
 * Computes the absorbed radiation dose including a quality factor.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_radiation_absorbed_dose(double energy_joules, double mass_kg, double quality_factor);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `SloshPendulumDerivative`.
 */
Bool space_slosh_pendulum_derivative(double angle,
                                     double angular_rate,
                                     double length,
                                     double damping,
                                     double lateral_acceleration,
                                     double gravity,
                                     SloshPendulumDerivative *out_derivative);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `VariationalState`.
 */
Bool space_variational_two_body(Vec3 position,
                                Vec3 velocity,
                                double mu,
                                VariationalState *out_derivative);

/**
 * # Safety
 * `out_link` must be null or point to a valid, writable `FriisLink`.
 */
Bool space_friis_link(double transmit_power,
                      double transmit_gain,
                      double receive_gain,
                      double wavelength,
                      double range,
                      double system_loss,
                      FriisLink *out_link);

/**
 * Converts a frequency to the corresponding free-space wavelength.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_friis_wavelength_from_frequency(double frequency);

/**
 * Computes the GNSS double-difference carrier phase observable in cycles.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_gnss_double_difference_carrier_phase(double range_rover_sat_a,
                                                  double range_rover_sat_b,
                                                  double range_base_sat_a,
                                                  double range_base_sat_b,
                                                  double wavelength,
                                                  double ambiguity);

/**
 * # Safety
 * `out_observation` must be null or point to a valid, writable `GnssObservation`.
 */
Bool space_gnss_pseudorange(Vec3 receiver,
                            Vec3 satellite,
                            double receiver_clock_bias,
                            double satellite_clock_bias,
                            double ionosphere_delay,
                            double troposphere_delay,
                            GnssObservation *out_observation);

/**
 * # Safety
 * `out_measurement` must be null or point to a valid, writable `RadarMeasurement`.
 */
Bool space_radar_range_rate(Vec3 radar_position,
                            Vec3 target_position,
                            Vec3 radar_velocity,
                            Vec3 target_velocity,
                            RadarMeasurement *out_measurement);

/**
 * # Safety
 * `out_state` must be null or point to a valid, writable `StateVector`.
 */
Bool space_elements_to_state(OrbitalElements elements, double mu, StateVector *out_state);

/**
 * # Safety
 * `out_transfer` must be null or point to a valid, writable `HohmannTransfer`.
 */
Bool space_hohmann_transfer(double mu,
                            double radius1,
                            double radius2,
                            HohmannTransfer *out_transfer);

/**
 * Computes the orbital period from the gravitational parameter and semi-major axis
 * (Kepler's third law).
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_kepler_period(double mu, double semi_major_axis);

/**
 * Computes the semi-major axis from the gravitational parameter and orbital period.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_kepler_semi_major_axis(double mu, double period);

/**
 * Computes the time of flight for an elliptic Lambert arc.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_lambert_time_elliptic(double mu,
                                   double semi_major_axis,
                                   double alpha,
                                   double beta,
                                   uint32_t revolutions);

/**
 * Computes the semi-major axis decay rate due to atmospheric drag.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_semi_major_axis_decay_rate(double semi_major_axis,
                                        double density,
                                        double drag_coefficient,
                                        double area,
                                        double mass,
                                        double mu);

/**
 * # Safety
 * `out_elements` must be null or point to a valid, writable `OrbitalElements`.
 */
Bool space_state_to_elements(StateVector state, double mu, OrbitalElements *out_elements);

/**
 * Computes the Tsiolkovsky rocket equation delta-v.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_tsiolkovsky_delta_v(double specific_impulse,
                                 double standard_gravity,
                                 double initial_mass,
                                 double final_mass);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_apply_atmospheric_drag_to_body(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          Vec3 atmosphere_velocity,
                                          double density,
                                          double drag_coefficient,
                                          double area,
                                          double mass,
                                          Bool wake_up,
                                          Vec3 *out_acceleration);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
uint8_t space_apply_atmospheric_drag_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 atmosphere_velocity,
                                                  double density,
                                                  double drag_coefficient,
                                                  double area,
                                                  double mass,
                                                  Bool wake_up,
                                                  Vec3 *out_acceleration);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_torque` must be null or point to a valid, writable `Vec3`.
 */
Bool space_apply_gravity_gradient_torque_to_body(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body_handle,
                                                 Vec3 inertia_diag,
                                                 double mu,
                                                 Bool wake_up,
                                                 Vec3 *out_torque);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_torque` must be null or point to a valid, writable `Vec3`.
 */
uint8_t space_apply_gravity_gradient_torque_to_body_flag(struct WorldHandle *world,
                                                         RigidBodyHandleRaw body_handle,
                                                         Vec3 inertia_diag,
                                                         double mu,
                                                         Bool wake_up,
                                                         Vec3 *out_torque);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_apply_j2_force_to_body(struct WorldHandle *world,
                                  RigidBodyHandleRaw body_handle,
                                  double mu,
                                  double equatorial_radius,
                                  double j2,
                                  double mass,
                                  Bool wake_up,
                                  Vec3 *out_acceleration);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
uint8_t space_apply_j2_force_to_body_flag(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          double mu,
                                          double equatorial_radius,
                                          double j2,
                                          double mass,
                                          Bool wake_up,
                                          Vec3 *out_acceleration);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_apply_solar_radiation_pressure_to_body(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 sun_direction,
                                                  double solar_flux,
                                                  double reflectivity,
                                                  double area,
                                                  double mass,
                                                  Bool wake_up,
                                                  Vec3 *out_acceleration);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
uint8_t space_apply_solar_radiation_pressure_to_body_flag(struct WorldHandle *world,
                                                          RigidBodyHandleRaw body_handle,
                                                          Vec3 sun_direction,
                                                          double solar_flux,
                                                          double reflectivity,
                                                          double area,
                                                          double mass,
                                                          Bool wake_up,
                                                          Vec3 *out_acceleration);

/**
 * Computes atmospheric density using the exponential scale-height model.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_atmospheric_density_scale_height(double reference_density,
                                              double altitude,
                                              double reference_altitude,
                                              double scale_height);

/**
 * # Safety
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_atmospheric_drag_acceleration(Vec3 velocity,
                                         Vec3 atmosphere_velocity,
                                         double density,
                                         double drag_coefficient,
                                         double area,
                                         double mass,
                                         Vec3 *out_acceleration);

/**
 * # Safety
 * `out_erosion` must be null or point to a valid, writable `AtomicOxygenErosion`.
 */
Bool space_atomic_oxygen_erosion(double fluence,
                                 double erosion_yield,
                                 double area,
                                 double density,
                                 AtomicOxygenErosion *out_erosion);

/**
 * # Safety
 * `out_torque` must be null or point to a valid, writable `Vec3`.
 */
Bool space_gravity_gradient_torque(Vec3 position, Vec3 inertia_diag, double mu, Vec3 *out_torque);

/**
 * # Safety
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_j2_acceleration(Vec3 position,
                           double mu,
                           double equatorial_radius,
                           double j2,
                           Vec3 *out_acceleration);

/**
 * Computes the Sagnac phase rate of a ring interferometer.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_sagnac_phase_rate(double area, double angular_rate, double wavelength);

/**
 * # Safety
 * `out_acceleration` must be null or point to a valid, writable `Vec3`.
 */
Bool space_solar_radiation_pressure_acceleration(Vec3 sun_direction,
                                                 double solar_flux,
                                                 double reflectivity,
                                                 double area,
                                                 double mass,
                                                 Vec3 *out_acceleration);

/**
 * # Safety
 * `out_battery` must be null or point to a valid, writable `BatteryEquivalentCircuit`.
 */
Bool space_battery_equivalent_circuit(double open_circuit_voltage,
                                      double current,
                                      double ohmic_resistance,
                                      double rc_voltage,
                                      double rc_resistance,
                                      double rc_capacitance,
                                      double capacity_coulombs,
                                      BatteryEquivalentCircuit *out_battery);

/**
 * # Safety
 * `out_balance` must be null or point to a valid, writable `Co2MassBalance`.
 */
Bool space_co2_mass_balance(double current_mass,
                            double generation_rate,
                            double removal_rate,
                            double leakage_rate,
                            double volume,
                            double dt,
                            Co2MassBalance *out_balance);

/**
 * # Safety
 * `out_force` must be null or point to a valid, writable `ContactForceModel`.
 */
Bool space_contact_force_hunt_crossley(double penetration,
                                       double penetration_rate,
                                       double stiffness,
                                       double damping,
                                       double exponent,
                                       ContactForceModel *out_force);

/**
 * # Safety
 * `out_performance` must be null or point to a valid, writable `HallThrusterPerformance`.
 */
Bool space_hall_thruster_performance(double mass_flow_rate,
                                     double exhaust_velocity,
                                     double input_power,
                                     double standard_gravity,
                                     HallThrusterPerformance *out_performance);

/**
 * # Safety
 * `out_rate` must be null or point to a valid, writable `ChemicalReactionRate`.
 */
Bool space_sabatier_methane_rate(double co2_molar_rate,
                                 double h2_molar_rate,
                                 double conversion,
                                 ChemicalReactionRate *out_rate);

/**
 * # Safety
 * `out_power` must be null or point to a valid, writable `SolarPanelPower`.
 */
Bool space_solar_panel_power(double solar_flux,
                             double area,
                             double efficiency,
                             double incidence_angle,
                             double degradation,
                             SolarPanelPower *out_power);

/**
 * Computes a structural natural frequency from stiffness, mass, and a mode factor.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_structural_natural_frequency(double stiffness, double mass, double mode_factor);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_exchange` must be null or point to a valid, writable `CmgExchange`.
 */
Bool space_apply_cmg_torque_to_body(struct WorldHandle *world,
                                    RigidBodyHandleRaw body_handle,
                                    Vec3 gimbal_axis,
                                    Vec3 wheel_momentum,
                                    double gimbal_rate,
                                    Bool wake_up,
                                    CmgExchange *out_exchange);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_exchange` must be null or point to a valid, writable `CmgExchange`.
 */
uint8_t space_apply_cmg_torque_to_body_flag(struct WorldHandle *world,
                                            RigidBodyHandleRaw body_handle,
                                            Vec3 gimbal_axis,
                                            Vec3 wheel_momentum,
                                            double gimbal_rate,
                                            Bool wake_up,
                                            CmgExchange *out_exchange);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_dipole` must be null or point to a valid, writable `Vec3`.
 */
Bool space_apply_magnetic_torquer_to_body(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          Vec3 commanded_torque,
                                          Vec3 magnetic_field,
                                          double max_dipole,
                                          Bool wake_up,
                                          Vec3 *out_dipole);

/**
 * # Safety
 * `world` must be a valid pointer to a `WorldHandle` created by this library.
 * `out_dipole` must be null or point to a valid, writable `Vec3`.
 */
uint8_t space_apply_magnetic_torquer_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  Vec3 commanded_torque,
                                                  Vec3 magnetic_field,
                                                  double max_dipole,
                                                  Bool wake_up,
                                                  Vec3 *out_dipole);

/**
 * # Safety
 * `out_exchange` must be null or point to a valid, writable `CmgExchange`.
 */
Bool space_cmg_exchange(Vec3 gimbal_axis,
                        Vec3 wheel_momentum,
                        double gimbal_rate,
                        CmgExchange *out_exchange);

/**
 * # Safety
 * `out_inverse` must be null or point to a valid, writable `CmgRobustInverse`.
 */
Bool space_cmg_robust_pseudoinverse_diag(Vec3 jacobian_diag,
                                         Vec3 desired_torque,
                                         double damping,
                                         CmgRobustInverse *out_inverse);

/**
 * Computes the scalar Kalman gain.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_ekf_gain_scalar(double covariance,
                             double measurement_jacobian,
                             double measurement_noise);

/**
 * # Safety
 * `out_prediction` must be null or point to a valid, writable `ScalarKalman`.
 */
Bool space_ekf_predict_scalar(double state,
                              double covariance,
                              double nonlinear_delta,
                              double jacobian,
                              double process_noise,
                              ScalarKalman *out_prediction);

/**
 * # Safety
 * `out_update` must be null or point to a valid, writable `ScalarKalman`.
 */
Bool space_ekf_update_scalar(double predicted_state,
                             double predicted_covariance,
                             double measurement,
                             double predicted_measurement,
                             double kalman_gain,
                             double measurement_jacobian,
                             ScalarKalman *out_update);

/**
 * # Safety
 * `out_attitude` must be null or point to a valid, writable `LeastSquaresAttitude`.
 */
Bool space_least_squares_attitude_two_vector(Vec3 body_primary,
                                             Vec3 body_secondary,
                                             Vec3 reference_primary,
                                             Vec3 reference_secondary,
                                             LeastSquaresAttitude *out_attitude);

/**
 * # Safety
 * `out_dipole` must be null or point to a valid, writable `Vec3`.
 */
Bool space_magnetic_torquer_dipole(Vec3 commanded_torque,
                                   Vec3 magnetic_field,
                                   double max_dipole,
                                   Vec3 *out_dipole);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `QuaternionDerivative`.
 */
Bool space_quaternion_derivative(Quat attitude,
                                 Vec3 angular_velocity,
                                 QuaternionDerivative *out_derivative);

/**
 * # Safety
 * `out_derivative` must be null or point to a valid, writable `RigidBodyEulerDerivative`.
 */
Bool space_rigid_body_euler_derivative(Vec3 inertia_diag,
                                       Vec3 angular_velocity,
                                       Vec3 torque,
                                       RigidBodyEulerDerivative *out_derivative);

/**
 * Computes the PD control torque for a solar array drive.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_solar_array_pd_torque(double angle_error, double rate_error, double kp, double kd);

/**
 * Computes the net spacecraft surface charging current balance.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_surface_charging_current_balance(double photo_current,
                                              double secondary_current,
                                              double backscatter_current,
                                              double electron_current,
                                              double ion_current);

/**
 * # Safety
 * `out_attitude` must be null or point to a valid, writable `Quat`.
 */
Bool space_triad_attitude(Vec3 body_primary,
                          Vec3 body_secondary,
                          Vec3 reference_primary,
                          Vec3 reference_secondary,
                          Quat *out_attitude);

/**
 * # Safety
 * `out_state` must be null or point to a valid, writable `AirlockDepressurization`.
 */
Bool space_airlock_depressurization(double pressure,
                                    double ambient_pressure,
                                    double volume,
                                    double conductance,
                                    double dt,
                                    AirlockDepressurization *out_state);

/**
 * Sums the evaporator, vapor, condenser, and wick thermal resistances of a heat pipe.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_heat_pipe_thermal_resistance(double evaporator_resistance,
                                          double vapor_resistance,
                                          double condenser_resistance,
                                          double wick_resistance);

/**
 * # Safety
 * `out_power` must be null or point to a valid, writable `RadiatorPower`.
 */
Bool space_radiator_power(double area,
                          double emissivity,
                          double temperature,
                          double sink_temperature,
                          double absorbed_power,
                          RadiatorPower *out_power);

/**
 * # Safety
 * `out_heat` must be null or point to a valid, writable `FluidLoopHeatTransfer`.
 */
Bool space_single_phase_loop_heat_transfer(double mass_flow_rate,
                                           double specific_heat,
                                           double inlet_temperature,
                                           double heat_input,
                                           FluidLoopHeatTransfer *out_heat);

/**
 * # Safety
 * `out_rate` must be null or point to a valid, writable `ChemicalReactionRate`.
 */
Bool space_spe_oxygen_rate(double current,
                           double cells,
                           double faraday_efficiency,
                           ChemicalReactionRate *out_rate);

/**
 * # Safety
 * `out_balance` must be null or point to a valid, writable `ThermalBalance`.
 */
Bool space_thermal_balance(double absorbed_power,
                           double internal_power,
                           double emitted_area,
                           double emissivity,
                           ThermalBalance *out_balance);

/**
 * Computes the critical projectile diameter a Whipple shield can defeat.
 *
 * # Safety
 * This function takes no pointers and transfers no ownership; it is safe to call with
 * any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
 */
double space_whipple_critical_projectile_diameter(double bumper_thickness,
                                                  double bumper_density,
                                                  double projectile_density,
                                                  double impact_velocity,
                                                  double standoff);

/**
 * Compute polyhedron gravity.
 *
 * `vertices_xyz` — flat array of vertex positions (3×n_verts f64s)
 * `face_indices` — flat array of triangle indices (3×n_faces u32s)
 * `density` — constant density (kg/m³)
 *
 * # Safety
 *
 * `vertices_xyz` must point to at least 3×n_vertices readable f64s and
 * `face_indices` to at least 3×n_faces readable u32s; `out_acceleration`
 * must be valid for a single `Vec3` write.
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
 *
 * # Safety
 *
 * `dem` must point to at least nx×ny readable f64s; `out_acceleration` must
 * be valid for a single `Vec3` write.
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
 *
 * # Safety
 *
 * `dem` must point to at least nx×ny readable f64s; `out_acceleration` must
 * be valid for a single `Vec3` write.
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
 *
 * # Safety
 *
 * `out_acceleration` must be valid for a single `Vec3` write.
 */
Bool terrain_lunar_mascon_gravity(Vec3 position, Vec3 *out_acceleration);

/**
 * Get the number of built-in lunar mascons.
 *
 * # Safety
 *
 * This function takes no pointers and performs no memory access; it is safe
 * to call from any context.
 */
uint32_t terrain_lunar_mascon_count(void);

/**
 * Get a specific lunar mascon by index.
 *
 * # Safety
 *
 * `out_mascon` must be valid for a single `LunarMascon` write.
 */
Bool terrain_lunar_mascon_get(uint32_t index, struct LunarMascon *out_mascon);

Bool acoustics_spherical_spreading_loss(double range, double *out);

Bool acoustics_cylindrical_spreading_loss(double range, double *out);

Bool acoustics_thorp_absorption(double frequency_khz, double *out);

Bool acoustics_sabine_rt60(double volume, double surface_area, double mean_absorption, double *out);

Bool acoustics_eyring_rt60(double volume, double surface_area, double mean_absorption, double *out);

Bool acoustics_acoustic_impedance(double density, double sound_speed, double *out);

Bool acoustics_transmission_coefficient(double z1, double z2, double *out);

Bool acoustics_mass_law_tl(double frequency, double surface_density, double *out);

Bool acoustics_helmholtz_resonance_frequency(double sound_speed,
                                             double neck_area,
                                             double cavity_volume,
                                             double neck_length,
                                             double *out);

Bool acoustics_doppler_shift(double source_frequency,
                             double sound_speed,
                             double receiver_velocity,
                             double source_velocity,
                             Bool approach,
                             double *out);

Bool acoustics_maekawa_barrier_attenuation(double fresnel_number, double *out);

Bool acoustics_active_sonar_echo_level(double source_level,
                                       double transmission_loss,
                                       double target_strength,
                                       double noise_level,
                                       double directivity_index,
                                       double detection_threshold,
                                       double *out);

Bool astrophysics_hill_sphere_radius(double primary_mass,
                                     double secondary_mass,
                                     double semi_major_axis,
                                     double eccentricity,
                                     double *out);

Bool astrophysics_lane_emden_first_zero(double polytropic_index, double *out);

Bool astrophysics_mass_luminosity_relation(double mass_solar, double exponent, double *out);

Bool astrophysics_eddington_luminosity(double mass, double opacity, double *out);

Bool astrophysics_eddington_luminosity_solar(double mass_solar, double opacity, double *out);

Bool astrophysics_hubble_velocity(double hubble_constant, double distance, double *out);

Bool astrophysics_hubble_distance(double velocity, double hubble_constant, double *out);

Bool astrophysics_nfw_density(double radius,
                              double scale_radius,
                              double characteristic_density,
                              double *out);

Bool astrophysics_nfw_enclosed_mass(double radius,
                                    double scale_radius,
                                    double characteristic_density,
                                    double *out);

Bool astrophysics_blackbody_spectral_radiance(double wavelength, double temperature, double *out);

Bool astrophysics_wien_displacement(double temperature, double *out);

Bool astrophysics_jeans_mass(double temperature,
                             double density,
                             double mean_molecular_weight,
                             double *out);

Bool astrophysics_jeans_length(double temperature,
                               double density,
                               double mean_molecular_weight,
                               double *out);

Bool astrophysics_main_sequence_lifetime(double mass_solar, double *out);

Bool astrophysics_mass_radius_relation(double mass_solar, double *out);

Bool astrophysics_chandrasekhar_mass_limit(double *out);

Bool astrophysics_chandrasekhar_mass_kg(double *out);

Bool astrophysics_mass_function(double period_seconds, double semi_amplitude, double *out);

Bool astrophysics_binary_semi_major_axis(double total_mass, double period, double *out);

Bool astrophysics_ss73_disk_temperature(double mass_kg,
                                        double accretion_rate,
                                        double radius,
                                        double inner_radius,
                                        double *out);

Bool astrophysics_nickel56_decay_luminosity(double nickel_mass_kg, double time_days, double *out);

Bool astrophysics_transit_depth(double planet_radius, double star_radius, double *out);

Bool astrophysics_radial_velocity_semi_amplitude(double planet_mass_kg,
                                                 double star_mass_kg,
                                                 double period,
                                                 double inclination,
                                                 double *out);

Bool astrophysics_nfw_circular_velocity(double r, double v_max, double r_scale, double *out);

/**
 * Roche fluid/rigid limits. Writes (fluid, rigid) into `out_fluid` /
 * `out_rigid`. Returns `Bool::FALSE` on invalid input or a null output.
 */
Bool astrophysics_roche_limit(double primary_radius,
                              double primary_density,
                              double secondary_density,
                              double *out_fluid,
                              double *out_rigid);

/**
 * Habitable-zone inner/outer radii. Writes (inner, outer) into `out_inner` /
 * `out_outer`. Returns `Bool::FALSE` on invalid input or a null output.
 */
Bool astrophysics_habitable_zone_boundaries(double star_luminosity_solar,
                                            double *out_inner,
                                            double *out_outer);

Bool electromagnetism_poynting_magnitude_plane_wave(double e_field_magnitude, double *out);

Bool electromagnetism_phase_velocity(double refractive_index, double *out);

Bool electromagnetism_wavelength_in_medium(double frequency, double refractive_index, double *out);

Bool electromagnetism_intrinsic_impedance(double permeability, double permittivity, double *out);

Bool electromagnetism_skin_depth(double frequency,
                                 double permeability,
                                 double conductivity,
                                 double *out);

Bool electromagnetism_vacuum_wavelength(double frequency, double *out);

Bool electromagnetism_wave_frequency(double wavelength, double *out);

Bool electromagnetism_dipole_radiation_resistance(double dipole_length,
                                                  double wavelength,
                                                  double *out);

Bool electromagnetism_half_wave_dipole_directivity(double *out);

Bool electromagnetism_effective_aperture(double gain_linear, double wavelength, double *out);

Bool electromagnetism_far_field_distance(double antenna_size, double wavelength, double *out);

Bool electromagnetism_friis_power_received(double transmit_power,
                                           double tx_gain,
                                           double rx_gain,
                                           double wavelength,
                                           double range,
                                           double *out);

Bool electromagnetism_reflection_coefficient(double load_impedance,
                                             double characteristic_impedance,
                                             double *out);

Bool electromagnetism_vswr(double reflection_coeff, double *out);

Bool electromagnetism_return_loss(double reflection_coeff, double *out);

Bool electromagnetism_quarter_wave_transformer(double z0, double z_load, double *out);

Bool electromagnetism_coaxial_impedance(double inner_diameter,
                                        double outer_diameter,
                                        double relative_permittivity,
                                        double *out);

Bool electromagnetism_coaxial_cutoff_frequency(double inner_diameter,
                                               double outer_diameter,
                                               double relative_permittivity,
                                               double *out);

Bool electromagnetism_rayleigh_scattering_cross_section(double refractive_index,
                                                        double diameter,
                                                        double wavelength,
                                                        double *out);

Bool electromagnetism_faraday_rotation(double verdet_constant,
                                       double magnetic_field,
                                       double path_length,
                                       double *out);

/**
 * Transmission-line input impedance (lossless). Writes (real, imag) into
 * `out_real` / `out_imag`. Returns `Bool::FALSE` on invalid input or a null output.
 */
Bool electromagnetism_transmission_line_input_impedance(double z0,
                                                        double z_load_real,
                                                        double z_load_imag,
                                                        double phase_constant,
                                                        double length,
                                                        double *out_real,
                                                        double *out_imag);

Bool material_mechanics_hookes_law_uniaxial(double stress, double youngs_modulus, double *out);

Bool material_mechanics_stress_from_strain(double youngs_modulus, double strain, double *out);

Bool material_mechanics_shear_modulus(double youngs_modulus, double poisson_ratio, double *out);

Bool material_mechanics_bulk_modulus(double youngs_modulus, double poisson_ratio, double *out);

Bool material_mechanics_lame_lambda(double youngs_modulus, double poisson_ratio, double *out);

Bool material_mechanics_von_mises_stress(double sx,
                                         double sy,
                                         double sz,
                                         double txy,
                                         double tyz,
                                         double tzx,
                                         double *out);

Bool material_mechanics_von_mises_yield_check(double von_mises_stress,
                                              double yield_stress,
                                              double *out);

Bool material_mechanics_tresca_shear_stress(double sigma_1, double sigma_3, double *out);

Bool material_mechanics_tresca_yield_check(double sigma_1,
                                           double sigma_3,
                                           double yield_stress,
                                           double *out);

Bool material_mechanics_ki_center_crack(double stress, double crack_half_length, double *out);

Bool material_mechanics_ki_edge_crack(double stress, double crack_length, double *out);

Bool material_mechanics_fracture_check(double stress_intensity,
                                       double fracture_toughness,
                                       double *out);

Bool material_mechanics_critical_crack_length(double stress,
                                              double fracture_toughness,
                                              double *out);

Bool material_mechanics_basquin_stress_amplitude(double cycles_to_failure,
                                                 double fatigue_strength_coefficient,
                                                 double fatigue_exponent,
                                                 double *out);

Bool material_mechanics_basquin_cycles_to_failure(double stress_amplitude,
                                                  double fatigue_strength_coefficient,
                                                  double fatigue_exponent,
                                                  double *out);

Bool material_mechanics_coffin_manson_strain_amplitude(double cycles_to_failure,
                                                       double ductility_coefficient,
                                                       double ductility_exponent,
                                                       double *out);

Bool material_mechanics_goodman_correction(double stress_amplitude,
                                           double mean_stress,
                                           double ultimate_tensile,
                                           double *out);

Bool material_mechanics_norton_creep_rate(double stress,
                                          double temperature,
                                          double a,
                                          double n,
                                          double activation_energy,
                                          double gas_constant,
                                          double *out);

Bool material_mechanics_beam_bending_stress(double bending_moment,
                                            double distance_from_neutral_axis,
                                            double area_moment_of_inertia,
                                            double *out);

Bool material_mechanics_beam_deflection_center_point_load(double load,
                                                          double span,
                                                          double youngs_modulus,
                                                          double moment_of_inertia,
                                                          double *out);

Bool material_mechanics_euler_buckling_load(double youngs_modulus,
                                            double moment_of_inertia,
                                            double effective_length_factor,
                                            double column_length,
                                            double *out);

Bool material_mechanics_slenderness_ratio(double effective_length_factor,
                                          double column_length,
                                          double radius_of_gyration,
                                          double *out);

/**
 * Principal stresses from a 3D stress tensor. Writes (σ₁, σ₂, σ₃) sorted
 * descending into `out` (capacity must be ≥ 3). Returns `Bool::FALSE` on
 * invalid input or null/short `out`.
 */
Bool material_mechanics_principal_stresses(double sx,
                                           double sy,
                                           double sz,
                                           double txy,
                                           double tyz,
                                           double tzx,
                                           double *out);

/**
 * Miner's linear damage rule: D = Σ (nᵢ / N_fᵢ). `ratios` points to
 * `count` `f64` elements (each nᵢ/N_fᵢ). Writes the summed damage into `out`.
 * Returns `Bool::FALSE` on null pointers, empty/short input, or invalid data.
 */
Bool material_mechanics_miners_damage(const double *ratios, uint32_t count, double *out);

Bool nuclear_decay_constant(double half_life, double *out);

Bool nuclear_remaining_nuclei(double initial, double decay_constant, double time, double *out);

Bool nuclear_activity(double decay_constant, double nuclei, double *out);

Bool nuclear_half_life(double decay_constant, double *out);

Bool nuclear_mean_lifetime(double decay_constant, double *out);

Bool nuclear_bethe_weizsaecker_binding_energy(double mass_number,
                                              double atomic_number,
                                              double *out);

Bool nuclear_binding_energy_per_nucleon(double mass_number, double atomic_number, double *out);

Bool nuclear_reaction_q_value(double initial_mass_u, double final_mass_u, double *out);

Bool nuclear_dt_fusion_energy(double *out);

Bool nuclear_dd_fusion_branch1_energy(double *out);

Bool nuclear_dd_fusion_branch2_energy(double *out);

Bool nuclear_u235_fission_energy(double *out);

Bool nuclear_four_factor_formula(double eta, double epsilon, double p, double f, double *out);

Bool nuclear_reaction_rate(double macroscopic_cross_section, double neutron_flux, double *out);

Bool nuclear_atomic_mass_approx(double mass_number, double binding_energy_mev, double *out);

Bool nuclear_specific_activity(double decay_constant, double mass_number, double *out);

Bool nuclear_half_value_layer(double linear_attenuation, double *out);

Bool nuclear_dt_fusion_q_value(double *out);

Bool plasma_beta(double density, double temperature, double magnetic_field, double *out);

Bool plasma_gyrofrequency(double charge, double magnetic_field, double mass, double *out);

Bool plasma_larmor_radius(double mass,
                          double perpendicular_velocity,
                          double charge,
                          double magnetic_field,
                          double *out);

Bool plasma_mirror_ratio(double max_field, double min_field, double *out);

Bool plasma_mirror_loss_cone_angle(double max_field, double min_field, double *out);

Bool quantum_free_particle_energy(double wave_number, double mass, double *out);

Bool quantum_de_broglie_wavelength(double mass, double velocity, double *out);

Bool quantum_infinite_well_energy(uint32_t quantum_number,
                                  double mass,
                                  double well_width,
                                  double *out);

Bool quantum_infinite_well_wave_function(uint32_t quantum_number,
                                         double well_width,
                                         double x,
                                         double *out);

Bool quantum_bohr_radius(double *out);

Bool quantum_hydrogen_energy_level(uint32_t quantum_number, double *out);

Bool quantum_hydrogen_orbital_radius(uint32_t quantum_number, double *out);

Bool quantum_hydrogen_transition_wavelength(uint32_t n1, uint32_t n2, double *out);

Bool quantum_minimum_uncertainty_product(double *out);

Bool quantum_fermi_golden_rule_linear(double matrix_element2,
                                      double density_of_states,
                                      double *out);

Bool quantum_spin_orbit_energy(double n, double l, double j, double atomic_number, double *out);

Bool quantum_fine_structure_constant(double *out);

Bool quantum_variational_hydrogen_energy(double alpha, double *out);

Bool quantum_variational_hydrogen_optimal_alpha(double *out);

Bool quantum_coherent_state_photon_probability(double alpha_squared, uint32_t n, double *out);

Bool quantum_spherical_harmonic_real(int32_t l, int32_t m, double theta, double phi, double *out);

Bool quantum_angular_momentum_squared(double j, double *out);

/**
 * Degenerate 2×2 perturbation eigenvalues. Writes (λ₁, λ₂) into `out_e1` /
 * `out_e2`. Returns `Bool::FALSE` on invalid input or a null output.
 */
Bool quantum_degenerate_perturbation_2x2(double h11,
                                         double h12,
                                         double h22,
                                         double *out_e1,
                                         double *out_e2);

/**
 * Time-evolution phase factor e^{-iEt/ℏ}. Writes (real, imag) into
 * `out_real` / `out_imag`. Returns `Bool::FALSE` on a null output.
 */
Bool quantum_time_evolution_phase(double energy, double time, double *out_real, double *out_imag);

Bool relativity_kerr_horizon_radii(double mass,
                                   double spin_parameter,
                                   double g,
                                   double *out_event,
                                   double *out_cauchy);

Bool relativity_kerr_ergosphere_radius(double mass,
                                       double spin_parameter,
                                       double polar_angle,
                                       double g,
                                       double *out);

Bool relativity_kerr_frame_dragging_frequency(double mass,
                                              double spin_parameter,
                                              double r,
                                              double theta,
                                              double g,
                                              double *out);

Bool relativity_schwarzschild_isco(double mass, double g, double *out);

Bool relativity_kerr_isco(double mass, double spin_parameter, double g, Bool prograde, double *out);

Bool relativity_gravitational_redshift(double mass, double radius, double g, double *out);

Bool relativity_reissner_nordstrom_horizons(double mass,
                                            double charge,
                                            double g,
                                            double *out_outer,
                                            double *out_inner);

Bool relativity_gw_strain_amplitude(double distance,
                                    double chirp_mass_kg,
                                    double orbital_frequency,
                                    double *out);

Bool relativity_chirp_mass(double mass1, double mass2, double *out);

Bool relativity_gw_frequency_derivative(double frequency, double chirp_mass_kg, double *out);

Bool relativity_relativistic_doppler_longitudinal(double source_frequency,
                                                  double relative_velocity,
                                                  Bool approaching,
                                                  double *out);

Bool relativity_relativistic_doppler_transverse(double source_frequency,
                                                double relative_velocity,
                                                double *out);

Bool relativity_einstein_radius(double mass_kg,
                                double dist_lens,
                                double dist_source,
                                double dist_ls,
                                double *out);

Bool relativity_cosmological_redshift(double scale_factor, double *out);

Bool relativity_redshift_from_wavelengths(double observed, double emitted, double *out);

Bool relativity_lense_thirring_angular_frequency(double mass_kg,
                                                 double spin_parameter,
                                                 double orbital_radius,
                                                 double *out);

Bool relativity_schwarzschild_effective_potential(double r,
                                                  double rs,
                                                  double angular_momentum,
                                                  double *out);

Bool relativity_gw_inspiral_snr(double strain_rss,
                                double f_min,
                                double f_max,
                                double noise_psd,
                                double *out);

Bool relativity_gw_inspiral_time_to_coalescence(double chirp_mass_kg, double f_gw_hz, double *out);

Bool thermodynamics_ideal_gas_pressure(double volume,
                                       double moles,
                                       double temperature,
                                       double *out);

Bool thermodynamics_ideal_gas_volume(double pressure,
                                     double moles,
                                     double temperature,
                                     double *out);

Bool thermodynamics_ideal_gas_temperature(double pressure,
                                          double volume,
                                          double moles,
                                          double *out);

Bool thermodynamics_polytropic_pressure(double p1, double v1, double v2, double gamma, double *out);

Bool thermodynamics_polytropic_work(double p1,
                                    double v1,
                                    double p2,
                                    double v2,
                                    double gamma,
                                    double *out);

/**
 * Estimate the aerodynamic/gravity forces acting on a trajectory state.
 *
 * # Safety
 *
 * `out_report`, when non-null, must be valid for a single
 * `TrajectoryForceReport` write.
 */
Bool trajectory_estimate_forces(TrajectoryState state,
                                TrajectoryEnvironment env,
                                TrajectoryForceReport *out_report);

/**
 * Advance a trajectory state by one integration step.
 *
 * # Safety
 *
 * `out_state` and `out_report`, when non-null, must each be valid for a
 * single write of `TrajectoryState` / `TrajectoryForceReport`.
 */
Bool trajectory_integrate_step(TrajectoryState state,
                               TrajectoryEnvironment env,
                               double dt,
                               TrajectoryState *out_state,
                               TrajectoryForceReport *out_report);

/**
 * Apply trajectory forces to a rigid body in the world.
 *
 * # Safety
 *
 * `world` must be a valid, live world pointer; `out_report`, when non-null,
 * must be valid for a single `TrajectoryForceReport` write.
 */
Bool trajectory_apply_forces_to_body(struct WorldHandle *world,
                                     RigidBodyHandleRaw body_handle,
                                     TrajectoryEnvironment env,
                                     Bool wake_up,
                                     TrajectoryForceReport *out_report);

/**
 * Flag-returning variant of `trajectory_apply_forces_to_body`.
 *
 * # Safety
 *
 * Same pointer contract as `trajectory_apply_forces_to_body`.
 */
uint8_t trajectory_apply_forces_to_body_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw body_handle,
                                             TrajectoryEnvironment env,
                                             Bool wake_up,
                                             TrajectoryForceReport *out_report);

/**
 * Estimate the glide forces acting on a gliding trajectory state.
 *
 * # Safety
 *
 * `out_report`, when non-null, must be valid for a single
 * `TrajectoryGlideReport` write.
 */
Bool trajectory_glide_estimate(TrajectoryGlideState state,
                               TrajectoryGlideEnvironment env,
                               TrajectoryGlideReport *out_report);

/**
 * Advance a gliding trajectory state by one integration step.
 *
 * # Safety
 *
 * `out_state` and `out_report`, when non-null, must each be valid for a
 * single write of `TrajectoryGlideState` / `TrajectoryGlideReport`.
 */
Bool trajectory_glide_integrate_step(TrajectoryGlideState state,
                                     TrajectoryGlideEnvironment env,
                                     double dt,
                                     TrajectoryGlideState *out_state,
                                     TrajectoryGlideReport *out_report);

/**
 * # Safety
 *
 * `voxels` must point to at least `size_x * size_y * size_z` readable bytes
 * for the duration of the call. The returned builder handle is owned by the
 * caller and must be released through the collider-builder ABI.
 */
struct ColliderBuilderHandle *collider_builder_create_voxels(const uint8_t *voxels,
                                                             uint32_t size_x,
                                                             uint32_t size_y,
                                                             uint32_t size_z,
                                                             double voxel_size_x,
                                                             double voxel_size_y,
                                                             double voxel_size_z,
                                                             Vec3 origin,
                                                             VoxelColliderOptions options);

/**
 * # Safety
 *
 * Same pointer contract as `collider_builder_create_voxels`.
 */
struct ColliderBuilderHandle *collider_builder_create_voxels_auto(const uint8_t *voxels,
                                                                  uint32_t size_x,
                                                                  uint32_t size_y,
                                                                  uint32_t size_z,
                                                                  double voxel_size_x,
                                                                  double voxel_size_y,
                                                                  double voxel_size_z,
                                                                  Vec3 origin,
                                                                  Bool dynamic_body);

/**
 * # Safety
 *
 * `voxels` must point to at least `size_x * size_y * size_z` readable bytes
 * for the duration of the call.
 */
VoxelBuildStats voxel_build_stats(const uint8_t *voxels,
                                  uint32_t size_x,
                                  uint32_t size_y,
                                  uint32_t size_z,
                                  double voxel_size_x,
                                  double voxel_size_y,
                                  double voxel_size_z,
                                  Vec3 origin,
                                  VoxelColliderOptions options);

/**
 * Computes build statistics for a voxelized AABB without building a collider.
 *
 * # Safety
 *
 * All arguments are passed by value; no pointers are dereferenced. `aabb`
 * must have finite mins/maxs with `mins < maxs` on every axis, and each
 * voxel size must be finite and positive; violations fail with
 * `ERR_INVALID_ARGUMENT` (or `ERR_CAPACITY` when the grid exceeds the cell
 * limit) and return a zeroed `VoxelBuildStats`.
 */
VoxelBuildStats voxel_aabb_build_stats(AabbDesc aabb,
                                       double voxel_size_x,
                                       double voxel_size_y,
                                       double voxel_size_z,
                                       VoxelColliderOptions options);

/**
 * Computes build statistics for a voxelized OBB without building a collider.
 *
 * # Safety
 *
 * All arguments are passed by value; no pointers are dereferenced. `obb`
 * must have a finite center and rotation and finite, positive half extents,
 * and each voxel size must be finite and positive; violations fail with
 * `ERR_INVALID_ARGUMENT` (or `ERR_CAPACITY` when the grid exceeds the cell
 * limit) and return a zeroed `VoxelBuildStats`.
 */
VoxelBuildStats voxel_obb_build_stats(Obb obb,
                                      double voxel_size_x,
                                      double voxel_size_y,
                                      double voxel_size_z,
                                      VoxelColliderOptions options);

/**
 * # Safety
 *
 * `out_stats` must be null or point to a valid, writable `VoxelBuildStats`.
 */
void voxel_aabb_build_stats_out(AabbDesc aabb,
                                double voxel_size_x,
                                double voxel_size_y,
                                double voxel_size_z,
                                VoxelColliderOptions options,
                                VoxelBuildStats *out_stats);

/**
 * # Safety
 *
 * `out_stats` must be null or point to a valid, writable `VoxelBuildStats`.
 */
void voxel_obb_build_stats_out(Obb obb,
                               double voxel_size_x,
                               double voxel_size_y,
                               double voxel_size_z,
                               VoxelColliderOptions options,
                               VoxelBuildStats *out_stats);

/**
 * Builds a collider builder from an AABB voxelized at the given voxel size.
 *
 * # Safety
 *
 * All arguments are passed by value; no pointers are dereferenced. `aabb`
 * must have finite mins/maxs with `mins < maxs` on every axis, and each
 * voxel size must be finite and positive; violations fail with
 * `ERR_INVALID_ARGUMENT` (or `ERR_CAPACITY` when the grid exceeds the cell
 * limit) and return null. The returned builder handle is owned by the
 * caller and must be released through the collider-builder ABI.
 */
struct ColliderBuilderHandle *collider_builder_create_voxel_aabb(AabbDesc aabb,
                                                                 double voxel_size_x,
                                                                 double voxel_size_y,
                                                                 double voxel_size_z,
                                                                 VoxelColliderOptions options);

/**
 * Builds a collider builder from a voxelized AABB with default options.
 *
 * # Safety
 *
 * Same argument contract as `collider_builder_create_voxel_aabb`.
 */
struct ColliderBuilderHandle *collider_builder_create_voxel_aabb_auto(AabbDesc aabb,
                                                                      double voxel_size_x,
                                                                      double voxel_size_y,
                                                                      double voxel_size_z,
                                                                      Bool dynamic_body);

/**
 * Builds a collider builder from an OBB voxelized at the given voxel size.
 *
 * # Safety
 *
 * All arguments are passed by value; no pointers are dereferenced. `obb`
 * must have a finite center and rotation and finite, positive half extents,
 * and each voxel size must be finite and positive; violations fail with
 * `ERR_INVALID_ARGUMENT` (or `ERR_CAPACITY` when the grid exceeds the cell
 * limit) and return null. The returned builder handle is owned by the
 * caller and must be released through the collider-builder ABI.
 */
struct ColliderBuilderHandle *collider_builder_create_voxel_obb(Obb obb,
                                                                double voxel_size_x,
                                                                double voxel_size_y,
                                                                double voxel_size_z,
                                                                VoxelColliderOptions options);

/**
 * Builds a collider builder from a voxelized OBB with default options.
 *
 * # Safety
 *
 * Same argument contract as `collider_builder_create_voxel_obb`.
 */
struct ColliderBuilderHandle *collider_builder_create_voxel_obb_auto(Obb obb,
                                                                     double voxel_size_x,
                                                                     double voxel_size_y,
                                                                     double voxel_size_z,
                                                                     Bool dynamic_body);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle. `out_handles` must be null
 * or point to `capacity` writable `ColliderHandleRaw` entries.
 */
uint32_t query_intersect_voxel_aabb(const struct WorldHandle *world,
                                    AabbDesc aabb,
                                    QueryFilterDesc filter,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle.
 */
uint32_t query_intersect_voxel_aabb_count(const struct WorldHandle *world,
                                          AabbDesc aabb,
                                          QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle. `out_handles` must be null
 * or point to `capacity` writable `ColliderHandleRaw` entries.
 */
uint32_t query_intersect_voxel_obb(const struct WorldHandle *world,
                                   Obb obb,
                                   QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle.
 */
uint32_t query_intersect_voxel_obb_count(const struct WorldHandle *world,
                                         Obb obb,
                                         QueryFilterDesc filter);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle. On failure any partially
 * inserted body is removed again before returning 0.
 */
RigidBodyHandleRaw world_insert_static_voxel_aabb(struct WorldHandle *world,
                                                  AabbDesc aabb,
                                                  double voxel_size_x,
                                                  double voxel_size_y,
                                                  double voxel_size_z,
                                                  VoxelColliderOptions options,
                                                  double friction,
                                                  double restitution);

/**
 * # Safety
 *
 * `world` must be null or a valid world handle. On failure any partially
 * inserted body is removed again before returning 0.
 */
RigidBodyHandleRaw world_insert_dynamic_voxel_obb(struct WorldHandle *world,
                                                  Obb obb,
                                                  double voxel_size_x,
                                                  double voxel_size_y,
                                                  double voxel_size_z,
                                                  VoxelColliderOptions options,
                                                  double density,
                                                  double friction,
                                                  double restitution);

/**
 * Flip a single voxel cell of an already-inserted voxel collider **in place**,
 * rebuilding its shape and keeping the same `ColliderHandleRaw`.
 *
 * `solid` is treated as boolean (non-zero = solid). The world must hold the
 * voxel source grid for `handle` (i.e. the collider was built from
 * `collider_builder_create_voxel*`). Out-of-range coordinates are a no-op that
 * still returns `Bool::TRUE` (nothing to update). If the cell did not change,
 * the collider is left untouched (no rebuild). When the last solid cell is
 * removed and the grid becomes empty, the collider is removed from the world
 * and its handle becomes invalid — callers should drop their reference.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create`.
 */
Bool collider_voxel_cell_at_point(const struct WorldHandle *world,
                                  ColliderHandleRaw collider,
                                  Vec3 point,
                                  struct VoxelCoord *out_block);

/**
 * Read whether a single voxel cell of a voxel collider is solid (non-zero)
 * or empty (zero) without modifying the grid.
 *
 * The read counterpart of `collider_voxel_edit`: `edit` writes a cell, this
 * one reads it back. It completes the in-place voxel editing toolkit so the
 * mod no longer has to keep its own mirror copy of the grid just to answer
 * "is this block solid?" — needed for block-break drops / place checks /
 * standing-on-block queries (pair it with `collider_voxel_cell_at_point` to
 * turn a world point into a (ix,iy,iz) and then ask this fn for its state).
 *
 * # Output
 * On success `out_solid` is written with the cell's solidity (non-zero if the
 * byte at `(x,y,z)` is non-zero) and the function returns `TRUE`. On a null
 * `world`, a non-voxel collider, or out-of-range coordinates it returns
 * `FALSE` and writes `0` to `out_solid`.
 *
 * # Errors
 * Returns `Bool::FALSE` and sets an error code for a null `world`, or a
 * `collider` that is not backed by a voxel grid (out-of-range coordinates use
 * `ERR_INVALID_ARGUMENT`).
 */
Bool collider_voxel_read_cell(const struct WorldHandle *world,
                              ColliderHandleRaw collider,
                              int64_t x,
                              int64_t y,
                              int64_t z,
                              uint8_t *out_solid);

Bool collider_voxel_edit(struct WorldHandle *world,
                         ColliderHandleRaw handle,
                         int64_t x,
                         int64_t y,
                         int64_t z,
                         int32_t solid);

/**
 * Overwrite the entire voxel grid of an already-inserted voxel collider **in
 * place**, rebuilding its shape and keeping the same `ColliderHandleRaw`.
 *
 * This is the bulk counterpart of `collider_voxel_edit` for chunk reloads /
 * regeneration: pass the full grid plus the same voxel sizing, origin, and
 * build options used at creation time. When the new grid is empty the
 * collider is removed (its handle becomes invalid).
 *
 * # Safety
 * `voxels` must point to at least `size_x * size_y * size_z` readable bytes
 * for the duration of the call. `world` must be a valid `world_create` handle.
 */
Bool collider_set_voxels(struct WorldHandle *world,
                         ColliderHandleRaw handle,
                         const uint8_t *voxels,
                         uint32_t size_x,
                         uint32_t size_y,
                         uint32_t size_z,
                         double voxel_size_x,
                         double voxel_size_y,
                         double voxel_size_z,
                         Vec3 origin,
                         uint32_t mode,
                         int32_t dynamic_body,
                         uint32_t small_voxel_limit,
                         uint32_t mesh_voxel_limit);

/**
 * Cast a ray restricted to a single voxel collider and resolve the hit back
 * to the voxel cell coordinate in that collider's local grid.
 *
 * Pairs with `collider_voxel_edit`: pick the cell a player's ray points at,
 * then flip it. `origin` / `direction` / `max_toi` / `solid` mirror
 * `query_cast_ray`. Returns `TRUE` and fills `out_block` only when the ray
 * actually hit `collider` (a voxel collider with a retained source grid).
 *
 * # Safety
 * `world` must be a valid `world_create` handle; `out_block` may be null or
 * must point to writable space for one `VoxelCoord`.
 */
Bool collider_voxel_ray_pick(const struct WorldHandle *world,
                             ColliderHandleRaw collider,
                             Vec3 origin,
                             Vec3 direction,
                             double max_toi,
                             Bool solid,
                             struct VoxelCoord *out_block);

/**
 * Create a new physics world.  Non-finite gravity components fall back to zero.
 *
 * The returned pointer is owned by Rust; release it with `world_destroy`.
 *
 * # Safety
 * No pointer arguments are dereferenced.  The returned pointer is owned by
 * Rust and must be released exactly once with `world_destroy`.
 */
struct WorldHandle *world_create(Vec3 gravity);

/**
 * Destroy a physics world created by `world_create`.  Null is a no-op.
 *
 * # Safety
 * `world` must be a pointer returned by `world_create` (or null) and must not
 * be used again after this call.
 */
void world_destroy(struct WorldHandle *world);

/**
 * Advance the simulation by `delta_seconds` (clamped to (0, 1]).
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
void world_step(struct WorldHandle *world, double delta_seconds);

/**
 * Set integration parameters (dt, solver iterations, CCD substeps).
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
Bool world_set_integration_parameters(struct WorldHandle *world,
                                      double dt,
                                      uint32_t solver_iterations,
                                      uint32_t ccd_substeps);

/**
 * Read integration parameters into `out_values` (dt, iterations, CCD substeps).
 *
 * # Safety
 * `world` must be a valid world pointer (or null); `out_values` must point to
 * writable memory for at least `capacity` f64 values.
 */
uint32_t world_get_integration_parameters(const struct WorldHandle *world,
                                          double *out_values,
                                          uint32_t capacity);

/**
 * Set the world gravity vector.  Non-finite input is ignored.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` and not yet destroyed.
 */
void world_set_gravity(struct WorldHandle *world, Vec3 gravity);

/**
 * Get the world gravity vector.
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
Vec3 world_get_gravity(const struct WorldHandle *world);

/**
 * Number of rigid bodies in the world (-1 on null world).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
int32_t world_get_rigid_body_set_size(const struct WorldHandle *world);

/**
 * Number of colliders in the world (-1 on null world).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
int32_t world_get_collider_set_size(const struct WorldHandle *world);

/**
 * Write the world gravity into `out_gravity`.
 *
 * # Safety
 * `out_gravity` must point to a writable `Vec3` (or be null); `world` must be
 * a valid world pointer (or null).
 */
void world_get_gravity_out(const struct WorldHandle *world, Vec3 *out_gravity);

/**
 * Count of dynamic bodies (for sizing a `world_dynamic_body_snapshot` call).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint32_t world_dynamic_body_snapshot_count(const struct WorldHandle *world);

/**
 * Snapshot dynamic body handles + poses (7 f64 per body: pos3 + quat4).
 *
 * # Safety
 * `world` must be a valid world pointer (or null); `out_handles` must point to
 * writable memory for `capacity` handles and `out_values` for `capacity * 7`
 * f64 values.
 */
uint32_t world_dynamic_body_snapshot(const struct WorldHandle *world,
                                     RigidBodyHandleRaw *out_handles,
                                     double *out_values,
                                     uint32_t capacity);

/**
 * Count of all bodies (for sizing a `world_body_snapshot` call).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint32_t world_body_snapshot_count(const struct WorldHandle *world);

/**
 * Snapshot all body handles + poses + velocities (13 f64 per body:
 * pos3 + quat4 + linvel3 + angvel3).
 *
 * # Safety
 * `world` must be a valid world pointer (or null); `out_handles` must point to
 * writable memory for `capacity` handles and `out_values` for `capacity * 13`
 * f64 values.
 */
uint32_t world_body_snapshot(const struct WorldHandle *world,
                             RigidBodyHandleRaw *out_handles,
                             double *out_values,
                             uint32_t capacity);

/**
 * Batch-update body poses (7 f64 per body: pos3 + quat4).  Returns the number
 * of bodies actually updated.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create`; `handles` and
 * `values` must point to readable arrays of `count` handles and `count * 7`
 * f64 values respectively.
 */
uint32_t world_update_body_poses(struct WorldHandle *world,
                                 const RigidBodyHandleRaw *handles,
                                 const double *values,
                                 uint32_t count,
                                 Bool wake_up);

/**
 * Batch-update body velocities (6 f64 per body: linvel3 + angvel3).  Returns
 * the number of bodies actually updated.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create`; `handles` and
 * `values` must point to readable arrays of `count` handles and `count * 6`
 * f64 values respectively.
 */
uint32_t world_update_body_velocities(struct WorldHandle *world,
                                      const RigidBodyHandleRaw *handles,
                                      const double *values,
                                      uint32_t count,
                                      Bool wake_up);

/**
 * Number of force laws registered in the world's ForceRegistry.
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint32_t world_get_force_registry_count(const struct WorldHandle *world);

/**
 * Get count of registered force laws of a specific type.
 * `law_type` is the numeric discriminant of `ForceLawType`.
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint32_t world_get_force_registry_typed_count(const struct WorldHandle *world, uint32_t law_type);

/**
 * Create a shared-memory physics arena.
 *
 * Returns the arena pointer as a u64 (suitable for `MemorySegment.ofAddress` in Java).
 * The arena persists for the lifetime of the world.
 *
 * At most one arena may exist per world. Calling this again while an arena
 * is still live fails with `ERR_INVALID_ARGUMENT` and leaves the existing
 * arena untouched — call `world_destroy_shared_arena` first to recreate one.
 *
 * WARNING (Java side): before calling `world_destroy_shared_arena`, the
 * `MemorySegment` mapping the arena must be released/unmapped; destroying
 * the arena frees the underlying memory, and any still-mapped Java segment
 * would become a use-after-free.
 *
 * `max_bodies` — max concurrent bodies to mirror
 * `max_events` — max pending collision/contact events
 * `max_commands` — max pending commands (force/set pose etc.)
 * `out_address` — receives the arena base address
 * `out_size` — receives the total arena size in bytes (for Java MemorySegment mapping)
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create`; `out_address`
 * and `out_size` may be null, otherwise each must point to a writable u64.
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
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).  Any
 * Java `MemorySegment` mapping the arena must be released before this call.
 */
void world_destroy_shared_arena(struct WorldHandle *world);

/**
 * Get the arena address (returns 0 if no arena).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint64_t world_get_shared_arena_address(const struct WorldHandle *world);

/**
 * Get the arena size (returns 0 if no arena).
 *
 * # Safety
 * `world` must be a valid world pointer (or null).
 */
uint64_t world_get_shared_arena_size(const struct WorldHandle *world);

/**
 * Reset the event ring (Java calls this after draining events).
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null) and not
 * yet destroyed.
 */
void world_reset_shared_arena_events(struct WorldHandle *world);

/**
 * Enable or disable relative force for a rigid body.
 * When enabled, forces applied via `rigid_body_add_force_at_local_point`
 * will be applied at the local attachment point instead of world coordinates.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).
 */
Bool world_set_relative_force_enabled(struct WorldHandle *world,
                                      RigidBodyHandleRaw handle,
                                      Bool enabled,
                                      Vec3 local_point);

/**
 * Check if relative force is enabled for a rigid body.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).
 */
Bool world_get_relative_force_enabled(const struct WorldHandle *world, RigidBodyHandleRaw handle);

/**
 * Get the local attachment point for relative force.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).
 */
Vec3 world_get_relative_force_local_point(const struct WorldHandle *world,
                                          RigidBodyHandleRaw handle);

/**
 * Set the local attachment point for relative force.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).
 */
Bool world_set_relative_force_local_point(struct WorldHandle *world,
                                          RigidBodyHandleRaw handle,
                                          Vec3 local_point);

/**
 * Remove relative force configuration for a rigid body.
 *
 * # Safety
 * `world` must be a valid pointer returned by `world_create` (or null).
 */
Bool world_remove_relative_force(struct WorldHandle *world, RigidBodyHandleRaw handle);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* RIGID_BODY_H */
