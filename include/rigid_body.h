#ifndef RIGID_BODY_H
#define RIGID_BODY_H

#pragma once

/* Generated with cbindgen:0.29.4 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define ABI_VERSION 1

typedef enum BodyStatus {
  Dynamic = 0,
  Fixed = 1,
  KinematicPositionBased = 2,
  KinematicVelocityBased = 3,
} BodyStatus;

typedef enum JointAxisDesc {
  LinX = 0,
  LinY = 1,
  LinZ = 2,
  AngX = 3,
  AngY = 4,
  AngZ = 5,
} JointAxisDesc;

typedef enum JointTypeDesc {
  Fixed = 0,
  Revolute = 1,
  Prismatic = 2,
  Rope = 3,
  Spring = 4,
  Spherical = 5,
} JointTypeDesc;

typedef enum KdopPreset {
  K6 = 6,
  K14 = 14,
  K18 = 18,
  K26 = 26,
} KdopPreset;

typedef enum NeuralActivation {
  Relu = 0,
  Tanh = 1,
  Sin = 2,
  Linear = 3,
} NeuralActivation;

typedef enum ShapeType {
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
} ShapeType;

typedef enum VoxelColliderMode {
  Auto = 0,
  Cuboids = 1,
  GreedyCuboids = 2,
  SurfaceMesh = 3,
} VoxelColliderMode;

typedef struct AnvilKitAppHandle AnvilKitAppHandle;

typedef struct CRbTreeHandle CRbTreeHandle;

typedef struct CharacterControllerHandle CharacterControllerHandle;

typedef struct ColliderBuilderHandle ColliderBuilderHandle;

typedef struct JointBuilderHandle JointBuilderHandle;

typedef struct RTreeHandle RTreeHandle;

typedef struct RigidBodyBuilderHandle RigidBodyBuilderHandle;

typedef struct WorldHandle WorldHandle;

typedef struct Bool {
  uint8_t _0;
} Bool;
#define Bool_FALSE (Bool){ ._0 = 0 }
#define Bool_TRUE (Bool){ ._0 = 1 }

typedef struct ModalAnalysisReport {
  uint32_t dof;
  uint32_t mode_count;
  uint32_t stable_mode_count;
  double max_frequency_hz;
} ModalAnalysisReport;

typedef struct StructuralModeReport {
  double angular_frequency;
  double frequency_hz;
  double damping_ratio;
  double damped_frequency_hz;
  double critical_damping;
} StructuralModeReport;

typedef struct AcousticWaveReport {
  uint32_t cell_count;
  double max_pressure;
  double acoustic_energy;
} AcousticWaveReport;

typedef struct AcousticResonanceReport {
  struct Bool resonant;
  uint32_t nearest_mode_index;
  double nearest_frequency_hz;
  double frequency_delta_hz;
  double amplification_estimate;
} AcousticResonanceReport;

typedef struct AcousticMaterial {
  double density;
  double hardness;
  double damping;
  double roughness;
  double restitution;
  double sound_speed;
} AcousticMaterial;

typedef struct AcousticContactDesc {
  double normal_force;
  double normal_velocity;
  double tangential_velocity;
  double contact_area;
  double dt;
} AcousticContactDesc;

typedef struct AcousticExcitationReport {
  double impulse;
  double normal_component;
  double scrape_component;
  double brightness;
  double damping;
  double amplitude;
} AcousticExcitationReport;

typedef struct ModalSynthesisReport {
  uint32_t mode_count;
  double sample;
  double peak_modal_displacement;
  double modal_energy;
} ModalSynthesisReport;

typedef struct Vec3 {
  double x;
  double y;
  double z;
} Vec3;

typedef struct SpatializedSample {
  double left;
  double right;
  double distance;
  double attenuation;
  double pan;
} SpatializedSample;

typedef uint64_t RigidBodyHandleRaw;

typedef struct AeroSurface {
  struct Vec3 point;
  struct Vec3 normal;
  double area;
  double drag_coefficient;
  double lift_coefficient;
} AeroSurface;

typedef struct AeroForceReport {
  struct Vec3 total_force;
  struct Vec3 total_torque;
  uint32_t surface_count;
  uint32_t active_surface_count;
} AeroForceReport;

typedef struct Quat {
  double i;
  double j;
  double k;
  double w;
} Quat;

typedef struct ShapeDesc {
  uint32_t shape_type;
  double a;
  double b;
  double c;
  double d;
} ShapeDesc;

typedef struct MaterialProperties {
  double density;
  double friction;
  double restitution;
  double youngs_modulus;
  double poisson_ratio;
  double thermal_expansion;
} MaterialProperties;

typedef uint64_t ColliderHandleRaw;

typedef uint64_t ImpulseJointHandleRaw;

typedef struct FluidVolume {
  struct Vec3 center;
  struct Vec3 half_extents;
  double density;
  double linear_drag;
  double quadratic_drag;
  double angular_drag;
  struct Vec3 flow_velocity;
  struct Vec3 gravity;
} FluidVolume;

typedef struct FluidForceReport {
  struct Vec3 buoyancy_force;
  struct Vec3 drag_force;
  struct Vec3 angular_damping_torque;
  struct Vec3 total_force;
  struct Vec3 total_torque;
  double submerged_fraction;
  double displaced_volume;
} FluidForceReport;

typedef struct TrajectoryEnvironment {
  struct Vec3 gravity;
  struct Vec3 flow_velocity;
  double mass;
  double reference_area;
  double density;
  double drag_coefficient;
  double lift_coefficient;
  struct Vec3 lift_direction;
} TrajectoryEnvironment;

typedef struct TrajectoryForceReport {
  struct Vec3 gravity_force;
  struct Vec3 drag_force;
  struct Vec3 lift_force;
  struct Vec3 total_force;
  struct Vec3 acceleration;
} TrajectoryForceReport;

typedef struct StressStrainReport {
  double strain;
  double stress;
  double elastic_energy_density;
  double thermal_strain;
} StressStrainReport;

typedef struct HertzContactReport {
  double effective_modulus;
  double effective_radius;
  double contact_radius;
  double contact_area;
  double normal_force;
  double stiffness;
  double damping_force;
  double total_force;
} HertzContactReport;

typedef struct NBodyParticle {
  struct Vec3 position;
  struct Vec3 velocity;
  double mass;
} NBodyParticle;

typedef struct NBodySolverParams {
  double gravitational_constant;
  double softening;
  double opening_angle;
  uint32_t multipole_order;
} NBodySolverParams;

typedef struct NBodyForceReport {
  uint32_t body_count;
  uint32_t approximate_node_count;
  uint32_t direct_pair_count;
  double max_acceleration;
  double potential_energy;
} NBodyForceReport;

typedef struct RelativisticOrbitReport {
  double schwarzschild_radius;
  double periapsis_precession_per_orbit;
  struct Vec3 correction_acceleration;
} RelativisticOrbitReport;

typedef struct RocheLimitReport {
  double fluid_roche_limit;
  double rigid_roche_limit;
  struct Bool inside_fluid_limit;
  struct Bool inside_rigid_limit;
} RocheLimitReport;

typedef struct OrbitalResonanceReport {
  uint32_t ratio_numerator;
  uint32_t ratio_denominator;
  double actual_ratio;
  double target_ratio;
  double relative_error;
  struct Bool resonant;
} OrbitalResonanceReport;

typedef struct HillMuscleDesc {
  double max_isometric_force;
  double optimal_fiber_length;
  double tendon_slack_length;
  double max_contraction_velocity;
  double parallel_stiffness;
  double series_stiffness;
  double damping;
  double pennation_angle;
} HillMuscleDesc;

typedef struct HillMuscleState {
  double activation;
  double fiber_length;
  double fiber_velocity;
  double tendon_length;
  double moment_arm;
} HillMuscleState;

typedef struct HillMuscleReport {
  double active_force;
  double parallel_elastic_force;
  double series_elastic_force;
  double damping_force;
  double total_fiber_force;
  double tendon_force;
  double joint_torque;
  double force_length_factor;
  double force_velocity_factor;
} HillMuscleReport;

typedef struct SkeletalJointLimit {
  double min_angle;
  double max_angle;
  double stiffness;
  double damping;
} SkeletalJointLimit;

typedef struct SkeletalConstraintReport {
  double clamped_angle;
  double angle_error;
  double corrective_torque;
  struct Bool limited;
} SkeletalConstraintReport;

typedef struct Capsule {
  struct Vec3 a;
  struct Vec3 b;
  double radius;
} Capsule;

typedef struct Ssv {
  struct Vec3 a;
  struct Vec3 b;
  double radius;
} Ssv;

typedef struct Ellipsoid {
  struct Vec3 center;
  struct Vec3 radii;
  struct Quat rotation;
  uint32_t segments;
} Ellipsoid;

typedef struct Prism {
  struct Vec3 center;
  double radius;
  double half_height;
  uint32_t sides;
  struct Quat rotation;
} Prism;

typedef struct Cylinder {
  struct Vec3 center;
  double radius;
  double half_height;
  struct Quat rotation;
} Cylinder;

typedef struct SphericalShell {
  struct Vec3 center;
  double inner_radius;
  double outer_radius;
} SphericalShell;

typedef struct InteractionGroupsDesc {
  uint32_t memberships;
  uint32_t filter;
} InteractionGroupsDesc;

typedef struct QueryFilterDesc {
  uint32_t flags;
  struct InteractionGroupsDesc groups;
  struct Bool use_groups;
  ColliderHandleRaw exclude_collider;
  struct Bool use_exclude_collider;
  RigidBodyHandleRaw exclude_rigid_body;
  struct Bool use_exclude_rigid_body;
} QueryFilterDesc;

typedef struct Obb {
  struct Vec3 center;
  struct Vec3 half_extents;
  struct Quat rotation;
} Obb;

typedef struct Sphere {
  struct Vec3 center;
  double radius;
} Sphere;

typedef struct AabbDesc {
  struct Vec3 mins;
  struct Vec3 maxs;
} AabbDesc;

typedef struct FemTetrahedron {
  struct Vec3 a;
  struct Vec3 b;
  struct Vec3 c;
  struct Vec3 d;
} FemTetrahedron;

typedef struct FemShapeFunctionReport {
  double weights[4];
  struct Vec3 gradients[4];
  double volume;
  struct Bool inside;
} FemShapeFunctionReport;

typedef struct FemConstitutiveReport {
  double lambda;
  double shear_modulus;
  double bulk_modulus;
  uint32_t matrix_size;
} FemConstitutiveReport;

typedef struct NewmarkBetaParameters {
  double beta;
  double gamma;
  double dt;
} NewmarkBetaParameters;

typedef struct NewmarkBetaReport {
  uint32_t dof;
  double beta;
  double gamma;
  double dt;
  double effective_stiffness_scale;
  double effective_damping_scale;
  double max_delta_displacement;
  double residual_norm;
} NewmarkBetaReport;

typedef struct PidGains {
  double kp;
  double ki;
  double kd;
  double output_min;
  double output_max;
  double integral_min;
  double integral_max;
} PidGains;

typedef struct PidState {
  double integral;
  double previous_error;
} PidState;

typedef struct PidReport {
  double error;
  double integral;
  double derivative;
  double unclamped_output;
  double output;
} PidReport;

typedef struct StateSpaceReport {
  uint32_t state_count;
  uint32_t input_count;
  uint32_t output_count;
  double max_state_delta;
  double output_norm;
} StateSpaceReport;

typedef struct MpcConfig {
  uint32_t state_count;
  uint32_t input_count;
  uint32_t horizon;
  double dt;
  double control_min;
  double control_max;
  uint32_t gradient_iterations;
  double step_size;
} MpcConfig;

typedef struct MpcReport {
  uint32_t horizon;
  uint32_t iterations;
  double initial_cost;
  double final_cost;
  double first_control_norm;
} MpcReport;

typedef struct EffectiveCharacterMovement {
  struct Vec3 translation;
  struct Bool grounded;
  struct Bool is_sliding_down_slope;
} EffectiveCharacterMovement;

typedef struct ElectromagneticField {
  struct Vec3 electric;
  struct Vec3 magnetic;
} ElectromagneticField;

typedef struct LorentzForceReport {
  struct Vec3 electric_force;
  struct Vec3 magnetic_force;
  struct Vec3 total_force;
  struct Vec3 acceleration;
} LorentzForceReport;

typedef struct MagneticFluxReport {
  double flux;
  double normal_component;
  double area;
} MagneticFluxReport;

typedef struct FaradayInductionReport {
  double flux_rate;
  double induced_emf;
  double induced_current;
} FaradayInductionReport;

typedef struct MaxwellPointReport {
  struct ElectromagneticField next_field;
  struct Vec3 electric_derivative;
  struct Vec3 magnetic_derivative;
  double gauss_electric_residual;
  double gauss_magnetic_residual;
} MaxwellPointReport;

typedef struct FdtdYeeReport {
  uint32_t cell_count;
  double max_electric_delta;
  double max_magnetic_delta;
  double total_energy_density;
  double courant_number;
} FdtdYeeReport;

typedef struct CoulombFrictionLaw {
  double static_coefficient;
  double dynamic_coefficient;
  double velocity_threshold;
  struct Bool enabled;
} CoulombFrictionLaw;

typedef struct AirDragLaw {
  struct Vec3 fluid_velocity;
  double density;
  double dynamic_viscosity;
  double characteristic_length;
  double reference_area;
  double drag_coefficient;
  double reynolds_stokes_limit;
  struct Bool enabled;
} AirDragLaw;

typedef struct ExternalForceLaw {
  struct Bool buoyancy_enabled;
  double fluid_density;
  double displaced_volume;
  struct Vec3 buoyancy_gravity;
  struct Bool electromagnetic_enabled;
  double charge;
  struct Vec3 electric_field;
  struct Vec3 magnetic_field;
  struct Bool elastic_enabled;
  struct Vec3 spring_anchor;
  double spring_stiffness;
  double spring_damping;
  struct Bool gravity_enabled;
  struct Vec3 gravity_source;
  double gravitational_parameter;
  struct Bool enabled;
} ExternalForceLaw;

typedef struct CustomPhysicsReport {
  uint32_t body_count;
  uint32_t drag_body_count;
  uint32_t external_force_body_count;
  struct Vec3 total_drag_force;
  struct Vec3 total_external_force;
  double max_reynolds_number;
} CustomPhysicsReport;

typedef struct CollisionEventRecord {
  struct Bool started;
  ColliderHandleRaw collider1;
  ColliderHandleRaw collider2;
  struct Bool sensor;
  struct Bool removed;
} CollisionEventRecord;

typedef struct ContactForceEventRecord {
  ColliderHandleRaw collider1;
  ColliderHandleRaw collider2;
  struct Vec3 total_force;
  double total_force_magnitude;
  struct Vec3 max_force_direction;
  double max_force_magnitude;
} ContactForceEventRecord;

typedef struct NavierStokesReport {
  struct Vec3 advection;
  struct Vec3 pressure_acceleration;
  struct Vec3 viscosity_acceleration;
  struct Vec3 external_acceleration;
  struct Vec3 total_acceleration;
  struct Vec3 next_velocity;
} NavierStokesReport;

typedef struct SphParticle {
  struct Vec3 position;
  struct Vec3 velocity;
  double mass;
  double density;
  double pressure;
} SphParticle;

typedef struct SphForceReport {
  double density;
  double pressure;
  struct Vec3 pressure_force;
  struct Vec3 viscosity_force;
  struct Vec3 surface_tension_force;
  struct Vec3 total_force;
} SphForceReport;

typedef struct BernoulliReport {
  double pressure;
  double velocity;
  double elevation;
  double total_head;
  double dynamic_pressure;
} BernoulliReport;

typedef struct StressIntensityReport {
  double stress_intensity;
  struct Bool critical;
  double safety_factor;
} StressIntensityReport;

typedef struct FractureMaterial {
  double youngs_modulus;
  double poisson_ratio;
  double fracture_toughness;
  double surface_energy;
  double density;
} FractureMaterial;

typedef struct GriffithReport {
  double critical_stress;
  double energy_release_rate;
  double critical_energy_release_rate;
  struct Bool will_fracture;
} GriffithReport;

typedef struct MinerDamageReport {
  double damage;
  double remaining_life_fraction;
  struct Bool failed;
} MinerDamageReport;

typedef struct SnCurveReport {
  double cycles_to_failure;
  struct Bool infinite_life;
} SnCurveReport;

typedef struct FractureEnergyReport {
  double available_energy;
  double surface_energy_required;
  double fragment_kinetic_energy;
  struct Bool will_fracture;
} FractureEnergyReport;

typedef struct FractureModeReport {
  uint32_t mode;
  double driving_stress;
  double mixed_mode_ratio;
} FractureModeReport;

typedef struct FractureFragmentDesc {
  struct Vec3 local_center;
  struct Vec3 half_extents;
  struct Vec3 initial_velocity;
  double density;
  double friction;
  double restitution;
} FractureFragmentDesc;

typedef struct FractureReplaceReport {
  uint32_t fragment_count;
  uint32_t joint_count;
  struct Bool removed_source;
} FractureReplaceReport;

typedef struct MolecularParticle {
  struct Vec3 position;
  struct Vec3 velocity;
  double mass;
  double charge;
  double epsilon;
  double sigma;
} MolecularParticle;

typedef struct MolecularForceLaw {
  double coulomb_constant;
  double relative_permittivity;
  double cutoff_radius;
  double softening;
  struct Bool lennard_jones_enabled;
  struct Bool coulomb_enabled;
} MolecularForceLaw;

typedef struct MolecularPairReport {
  struct Vec3 displacement;
  double distance;
  double lennard_jones_potential;
  double coulomb_potential;
  double total_potential;
  struct Vec3 lennard_jones_force;
  struct Vec3 coulomb_force;
  struct Vec3 total_force;
} MolecularPairReport;

typedef struct NeuralBoundsDesc {
  struct Vec3 center;
  struct Vec3 half_extents;
  struct Quat rotation;
  uint32_t sample_resolution;
  uint32_t hidden_width;
  uint32_t hidden_layers;
  uint32_t activation;
  double output_scale;
  double padding;
} NeuralBoundsDesc;

typedef struct CatalystEffect {
  double concentration;
  double strength;
  double saturation;
} CatalystEffect;

typedef struct CatalystReport {
  double rate_multiplier;
  double effective_rate;
} CatalystReport;

typedef struct GrayScottParams {
  double diffusion_u;
  double diffusion_v;
  double feed_rate;
  double kill_rate;
  double dx;
} GrayScottParams;

typedef struct GrayScottReactionReport {
  double reaction_rate;
  double diffusion_u_term;
  double diffusion_v_term;
  double du_dt;
  double dv_dt;
} GrayScottReactionReport;

typedef struct ReactionDiffusionReport {
  uint32_t cell_count;
  double max_delta_u;
  double max_delta_v;
  double total_u;
  double total_v;
  double max_reaction_rate;
} ReactionDiffusionReport;

typedef struct ConcentrationBuoyancyReport {
  double density;
  double density_delta;
  struct Vec3 buoyancy_acceleration;
  struct Vec3 buoyancy_force;
} ConcentrationBuoyancyReport;

typedef struct QuantumWaveFunction {
  double amplitude_real;
  double amplitude_imag;
} QuantumWaveFunction;

typedef struct QuantumBarrier {
  double particle_energy;
  double barrier_potential;
  double barrier_width;
  double particle_mass;
  double reduced_planck;
} QuantumBarrier;

typedef struct QuantumTunnelingReport {
  double wave_number;
  double decay_constant;
  double exponent;
  double transmission_coefficient;
  double reflection_coefficient;
} QuantumTunnelingReport;

typedef struct QuantumOscillatorReport {
  double angular_frequency;
  double zero_point_energy;
  double first_excited_energy;
  double level_spacing;
} QuantumOscillatorReport;

typedef struct RayHit {
  ColliderHandleRaw collider;
  double time_of_impact;
  struct Vec3 normal;
  uint32_t feature;
} RayHit;

typedef struct PointProjection {
  struct Vec3 point;
  struct Bool is_inside;
} PointProjection;

typedef struct ShapeCastHit {
  ColliderHandleRaw collider;
  double time_of_impact;
  struct Vec3 witness1;
  struct Vec3 witness2;
  struct Vec3 normal1;
  struct Vec3 normal2;
  uint32_t status;
} ShapeCastHit;

typedef struct ShapeCastOptionsDesc {
  double max_time_of_impact;
  double target_distance;
  struct Bool stop_at_penetration;
  struct Bool compute_impact_geometry_on_penetration;
} ShapeCastOptionsDesc;

typedef struct SoftBodyStepReport {
  uint32_t particle_count;
  uint32_t constraint_count;
  uint32_t active_particle_count;
  double max_correction;
  double total_error;
} SoftBodyStepReport;

typedef struct SoftSpring {
  uint32_t particle_a;
  uint32_t particle_b;
  double rest_length;
  double stiffness;
  double damping;
} SoftSpring;

typedef struct SoftDistanceConstraint {
  uint32_t particle_a;
  uint32_t particle_b;
  double rest_length;
  double stiffness;
  double compliance;
  double lambda;
} SoftDistanceConstraint;

typedef struct SoftBendingConstraint {
  uint32_t particle_a;
  uint32_t particle_b;
  double rest_distance;
  double stiffness;
  double compliance;
  double lambda;
} SoftBendingConstraint;

typedef struct SoftSphereCollision {
  struct Vec3 center;
  double radius;
} SoftSphereCollision;

typedef struct SoftVolumeConstraint {
  uint32_t particle_a;
  uint32_t particle_b;
  uint32_t particle_c;
  uint32_t particle_d;
  double rest_volume;
  double compliance;
  double lambda;
} SoftVolumeConstraint;

typedef struct OrbitalElements {
  double semi_major_axis;
  double eccentricity;
  double inclination;
  double raan;
  double argument_of_periapsis;
  double true_anomaly;
} OrbitalElements;

typedef struct StateVector {
  struct Vec3 position;
  struct Vec3 velocity;
} StateVector;

typedef struct QuaternionDerivative {
  double i_dot;
  double j_dot;
  double k_dot;
  double w_dot;
} QuaternionDerivative;

typedef struct RigidBodyEulerDerivative {
  struct Vec3 angular_acceleration;
} RigidBodyEulerDerivative;

typedef struct CmgExchange {
  struct Vec3 body_torque;
  struct Vec3 wheel_momentum_dot;
} CmgExchange;

typedef struct CwState {
  struct Vec3 position;
  struct Vec3 velocity;
} CwState;

typedef struct CwDerivative {
  struct Vec3 velocity;
  struct Vec3 acceleration;
} CwDerivative;

typedef struct DhTransform {
  double m00;
  double m01;
  double m02;
  double m03;
  double m10;
  double m11;
  double m12;
  double m13;
  double m20;
  double m21;
  double m22;
  double m23;
  double m30;
  double m31;
  double m32;
  double m33;
} DhTransform;

typedef struct ManipulatorDynamics {
  struct Vec3 torque;
} ManipulatorDynamics;

typedef struct SolarPanelPower {
  double incident_power;
  double electrical_power;
} SolarPanelPower;

typedef struct ThermalBalance {
  double net_power;
  double equilibrium_temperature;
} ThermalBalance;

typedef struct Co2MassBalance {
  double mass_rate;
  double next_mass;
  double concentration_rate;
} Co2MassBalance;

typedef struct FriisLink {
  double received_power;
  double path_loss;
} FriisLink;

typedef struct HohmannTransfer {
  double delta_v1;
  double delta_v2;
  double total_delta_v;
  double transfer_time;
} HohmannTransfer;

typedef struct ScalarKalman {
  double value;
  double covariance;
} ScalarKalman;

typedef struct LeastSquaresAttitude {
  struct Quat attitude;
  double rms_error;
} LeastSquaresAttitude;

typedef struct GnssObservation {
  double value;
  double geometric_range;
} GnssObservation;

typedef struct ContactForceModel {
  double normal_force;
  double damping_force;
  double total_force;
} ContactForceModel;

typedef struct BatteryEquivalentCircuit {
  double terminal_voltage;
  double rc_voltage_dot;
  double state_of_charge_dot;
} BatteryEquivalentCircuit;

typedef struct HallThrusterPerformance {
  double thrust;
  double specific_impulse;
  double efficiency;
} HallThrusterPerformance;

typedef struct CollisionProbability {
  double probability;
  double combined_sigma;
} CollisionProbability;

typedef struct AtomicOxygenErosion {
  double volume_loss;
  double mass_loss;
} AtomicOxygenErosion;

typedef struct FlexibleModeDerivative {
  double displacement_dot;
  double velocity_dot;
} FlexibleModeDerivative;

typedef struct SloshPendulumDerivative {
  double angle_dot;
  double angular_rate_dot;
} SloshPendulumDerivative;

typedef struct VariationalState {
  struct Vec3 position_dot;
  struct Vec3 velocity_dot;
} VariationalState;

typedef struct FluidLoopHeatTransfer {
  double heat_rate;
  double outlet_temperature;
} FluidLoopHeatTransfer;

typedef struct RadarMeasurement {
  double range;
  double range_rate;
} RadarMeasurement;

typedef struct MassProperties {
  struct Vec3 center_of_mass;
  struct Vec3 inertia_diag;
} MassProperties;

typedef struct BangOffBangProfile {
  double coast_time;
  double total_time;
  double switch_angle;
} BangOffBangProfile;

typedef struct CmgRobustInverse {
  struct Vec3 gimbal_rates;
  double damping;
} CmgRobustInverse;

typedef struct Sgp4SecularRates {
  double mean_motion_dot;
  double raan_dot;
  double argument_of_perigee_dot;
} Sgp4SecularRates;

typedef struct ChemicalReactionRate {
  double reactant_rate;
  double product_rate;
} ChemicalReactionRate;

typedef struct RadiatorPower {
  double emitted_power;
  double net_power;
} RadiatorPower;

typedef struct AirlockDepressurization {
  double pressure;
  double pressure_rate;
} AirlockDepressurization;

typedef struct HeatConductionReport {
  double temperature_delta;
  double temperature_gradient;
  double heat_flux;
  double heat_rate;
  double thermal_resistance;
} HeatConductionReport;

typedef struct PhaseChangeReport {
  double final_temperature;
  double sensible_heat;
  double latent_heat_used;
  double phase_fraction_delta;
  struct Bool phase_changed;
} PhaseChangeReport;

typedef struct ThermalRadiationReport {
  double emitted_power;
  double absorbed_power;
  double net_power;
  double radiative_coefficient;
} ThermalRadiationReport;

typedef struct FemHeatNode {
  double temperature;
  double heat_capacity;
  double heat_source;
} FemHeatNode;

typedef struct FemHeatEdge {
  uint32_t node_a;
  uint32_t node_b;
  double conductance;
} FemHeatEdge;

typedef struct FemHeatDiffusionReport {
  uint32_t node_count;
  uint32_t edge_count;
  double total_heat_rate;
  double max_temperature_delta;
} FemHeatDiffusionReport;

typedef struct ThermalStressReport {
  double free_thermal_strain;
  double mechanical_strain;
  double stress;
  double deformation;
  double elastic_energy_density;
} ThermalStressReport;

typedef struct ThermoelasticReport {
  double thermal_strain;
  double mechanical_strain_x;
  double mechanical_strain_y;
  double mechanical_strain_z;
  double stress_x;
  double stress_y;
  double stress_z;
  double bulk_modulus;
  double shear_modulus;
} ThermoelasticReport;

typedef struct TopologyOptimizationParams {
  double volume_fraction;
  double penalization;
  double min_density;
  double move_limit;
  double filter_radius;
  double stiffness_min;
  double stiffness_solid;
} TopologyOptimizationParams;

typedef struct SimpMaterialReport {
  double density;
  double stiffness;
  double stiffness_derivative;
} SimpMaterialReport;

typedef struct TopologyOptimizationReport {
  uint32_t cell_count;
  double average_density;
  double min_density;
  double max_density;
  double total_compliance;
  double max_density_change;
} TopologyOptimizationReport;

typedef struct DensityFieldStats {
  uint32_t cell_count;
  uint32_t solid_count;
  double average_density;
  double min_density;
  double max_density;
} DensityFieldStats;

typedef struct TrajectoryState {
  struct Vec3 position;
  struct Vec3 velocity;
} TrajectoryState;

typedef struct TrajectoryGlideState {
  double speed;
  double flight_path_angle;
  double altitude;
  double downrange;
} TrajectoryGlideState;

typedef struct TrajectoryGlideEnvironment {
  double gravity;
  double planet_radius;
  double ballistic_coefficient;
  double lift_to_drag;
  double bank_angle;
  double reference_density;
  double scale_height;
} TrajectoryGlideEnvironment;

typedef struct TrajectoryGlideReport {
  double density;
  double dynamic_pressure;
  double drag_acceleration;
  double lift_acceleration;
  double speed_dot;
  double flight_path_angle_dot;
  double altitude_dot;
  double downrange_dot;
} TrajectoryGlideReport;

typedef struct GearConstraintDesc {
  double ratio;
  double phase;
  double backlash;
  struct Bool opposite_direction;
} GearConstraintDesc;

typedef struct GearConstraintReport {
  double target_angle;
  double target_angular_velocity;
  double angle_error;
  double velocity_error;
  double effective_ratio;
} GearConstraintReport;

typedef struct ScrewConstraintDesc {
  double lead;
  double phase;
  struct Bool right_handed;
} ScrewConstraintDesc;

typedef struct ScrewConstraintReport {
  double target_translation;
  double target_linear_velocity;
  double translation_error;
  double velocity_error;
  double meters_per_radian;
} ScrewConstraintReport;

typedef struct CamConstraintDesc {
  double base_radius;
  double lift;
  double rise_angle;
  double return_angle;
  double phase;
} CamConstraintDesc;

typedef struct CamConstraintReport {
  double wrapped_angle;
  double radius;
  double follower_displacement;
  double displacement_derivative;
  double target_velocity;
  double displacement_error;
} CamConstraintReport;

typedef struct SpiralConstraintDesc {
  double initial_radius;
  double radial_pitch;
  double phase;
} SpiralConstraintDesc;

typedef struct SpiralConstraintReport {
  double radius;
  struct Vec3 position;
  struct Vec3 tangent;
  double radial_velocity;
  double constraint_error;
} SpiralConstraintReport;

typedef struct VoxelColliderOptions {
  uint32_t mode;
  struct Bool dynamic_body;
  uint32_t small_voxel_limit;
  uint32_t mesh_voxel_limit;
} VoxelColliderOptions;

typedef struct VoxelBuildStats {
  uint32_t cell_count;
  uint32_t solid_count;
  uint32_t selected_mode;
  uint32_t estimated_parts;
  uint32_t estimated_vertices;
  uint32_t estimated_triangles;
  uint32_t size_x;
  uint32_t size_y;
  uint32_t size_z;
} VoxelBuildStats;

typedef struct CharacterCollision {
  ColliderHandleRaw collider;
  struct Vec3 character_translation;
  struct Vec3 translation_applied;
  struct Vec3 translation_remaining;
  struct Vec3 world_witness1;
  struct Vec3 world_witness2;
  struct Vec3 normal1;
  struct Vec3 normal2;
  double time_of_impact;
} CharacterCollision;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

uint32_t abi_version(void);

struct Bool abi_supports_ffm(void);

struct Bool abi_supports_jni(void);

struct Bool acoustic_generalized_modal_analysis(const double *stiffness_matrix,
                                                const double *mass_matrix,
                                                uint32_t dof,
                                                uint32_t requested_modes,
                                                double *out_eigenvalues,
                                                double *out_frequencies_hz,
                                                double *out_mode_shapes,
                                                uint32_t eigen_capacity,
                                                uint32_t mode_shape_capacity,
                                                struct ModalAnalysisReport *out_report);

struct Bool acoustic_structural_mode_sdof(double stiffness,
                                          double mass,
                                          double damping,
                                          struct StructuralModeReport *out_report);

struct Bool acoustic_wave_equation_step(const double *previous_pressure,
                                        const double *current_pressure,
                                        const double *laplacian_pressure,
                                        uint32_t cell_count,
                                        double sound_speed,
                                        double damping,
                                        double dt,
                                        double *out_next_pressure,
                                        uint32_t capacity,
                                        struct AcousticWaveReport *out_report);

struct Bool acoustic_detect_resonance(double excitation_frequency_hz,
                                      const double *modal_frequencies_hz,
                                      const double *damping_ratios,
                                      uint32_t mode_count,
                                      double tolerance_hz,
                                      struct AcousticResonanceReport *out_report);

struct Bool acoustic_contact_material_excitation(struct AcousticMaterial material_a,
                                                 struct AcousticMaterial material_b,
                                                 struct AcousticContactDesc contact,
                                                 struct AcousticExcitationReport *out_report);

struct Bool acoustic_modal_synthesis_step(const double *modal_frequencies_hz,
                                          const double *damping_ratios,
                                          const double *modal_gains,
                                          double *mode_displacements,
                                          double *mode_velocities,
                                          uint32_t mode_count,
                                          double excitation,
                                          double dt,
                                          double output_gain,
                                          struct ModalSynthesisReport *out_report);

struct Bool acoustic_spatialize_mono_sample(double mono_sample,
                                            struct Vec3 source_position,
                                            struct Vec3 listener_position,
                                            struct Vec3 listener_right,
                                            double reference_distance,
                                            double rolloff,
                                            struct SpatializedSample *out_sample);

struct Bool aero_apply_surfaces(struct WorldHandle *world,
                                RigidBodyHandleRaw body_handle,
                                struct Vec3 wind_velocity,
                                double air_density,
                                const struct AeroSurface *surfaces,
                                uint32_t surface_count,
                                struct Bool wake_up,
                                struct AeroForceReport *out_report);

struct Bool aero_apply_voxel_grid(struct WorldHandle *world,
                                  RigidBodyHandleRaw body_handle,
                                  struct Vec3 wind_velocity,
                                  double air_density,
                                  const uint8_t *voxels,
                                  uint32_t size_x,
                                  uint32_t size_y,
                                  uint32_t size_z,
                                  double voxel_size,
                                  struct Vec3 local_origin,
                                  double drag_coefficient,
                                  double lift_coefficient,
                                  struct Bool wake_up,
                                  struct AeroForceReport *out_report);

uint8_t aero_apply_voxel_grid_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw body_handle,
                                   struct Vec3 wind_velocity,
                                   double air_density,
                                   const uint8_t *voxels,
                                   uint32_t size_x,
                                   uint32_t size_y,
                                   uint32_t size_z,
                                   double voxel_size,
                                   struct Vec3 local_origin,
                                   double drag_coefficient,
                                   double lift_coefficient,
                                   struct Bool wake_up,
                                   struct AeroForceReport *out_report);

uint8_t aero_apply_surfaces_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw body_handle,
                                 struct Vec3 wind_velocity,
                                 double air_density,
                                 const struct AeroSurface *surfaces,
                                 uint32_t surface_count,
                                 struct Bool wake_up,
                                 struct AeroForceReport *out_report);

struct Bool aero_estimate_surface_force(struct Vec3 body_linvel,
                                        struct Vec3 body_angvel,
                                        struct Vec3 body_center,
                                        struct Vec3 wind_velocity,
                                        double air_density,
                                        struct AeroSurface surface,
                                        struct AeroForceReport *out_report);

struct AnvilKitAppHandle *anvilkit_app_create(void);

void anvilkit_app_destroy(struct AnvilKitAppHandle *app);

void anvilkit_app_update(struct AnvilKitAppHandle *app);

uint64_t anvilkit_app_spawn_body(struct AnvilKitAppHandle *app,
                                 struct Vec3 translation,
                                 struct Quat rotation,
                                 uint32_t status);

uint64_t anvilkit_app_spawn_body_with_collider(struct AnvilKitAppHandle *app,
                                               struct Vec3 translation,
                                               struct Quat rotation,
                                               uint32_t status,
                                               struct ShapeDesc shape);

struct Bool anvilkit_app_set_transform(struct AnvilKitAppHandle *app,
                                       uint64_t entity_bits,
                                       struct Vec3 translation,
                                       struct Quat rotation);

struct Bool anvilkit_app_set_material(struct AnvilKitAppHandle *app,
                                      uint64_t entity_bits,
                                      struct MaterialProperties material);

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
                                        struct Vec3 axis_or_primary,
                                        double b,
                                        double c,
                                        struct Bool wake_up);

ImpulseJointHandleRaw anvilkit_app_constraint_to_joint(const struct AnvilKitAppHandle *app,
                                                       uint64_t constraint_id);

struct Bool anvilkit_app_remove_constraint(struct AnvilKitAppHandle *app,
                                           struct WorldHandle *world,
                                           uint64_t constraint_id,
                                           struct Bool wake_up);

struct Bool anvilkit_app_apply_aero_surfaces(struct AnvilKitAppHandle *app,
                                             struct WorldHandle *world,
                                             uint64_t entity_bits,
                                             struct Vec3 wind_velocity,
                                             double air_density,
                                             const struct AeroSurface *surfaces,
                                             uint32_t surface_count,
                                             struct Bool wake_up,
                                             struct AeroForceReport *out_report);

struct Bool anvilkit_app_apply_aero_voxel_grid(struct AnvilKitAppHandle *app,
                                               struct WorldHandle *world,
                                               uint64_t entity_bits,
                                               struct Vec3 wind_velocity,
                                               double air_density,
                                               const uint8_t *voxels,
                                               uint32_t size_x,
                                               uint32_t size_y,
                                               uint32_t size_z,
                                               double voxel_size,
                                               struct Vec3 local_origin,
                                               double drag_coefficient,
                                               double lift_coefficient,
                                               struct Bool wake_up,
                                               struct AeroForceReport *out_report);

struct Bool anvilkit_app_apply_fluid_aabb_forces(struct AnvilKitAppHandle *app,
                                                 struct WorldHandle *world,
                                                 uint64_t entity_bits,
                                                 struct FluidVolume fluid_volume,
                                                 struct Vec3 body_half_extents,
                                                 double body_volume,
                                                 struct Bool wake_up,
                                                 struct FluidForceReport *out_report);

struct Bool anvilkit_app_apply_trajectory_forces(struct AnvilKitAppHandle *app,
                                                 struct WorldHandle *world,
                                                 uint64_t entity_bits,
                                                 struct TrajectoryEnvironment environment,
                                                 struct Bool wake_up,
                                                 struct TrajectoryForceReport *out_report);

struct Bool material_stress_strain_linear(struct MaterialProperties material,
                                          double strain,
                                          double delta_temperature,
                                          struct StressStrainReport *out_report);

double material_elastic_collision_relative_speed(double relative_normal_speed, double restitution);

struct Bool material_hertz_contact_force(struct MaterialProperties material1,
                                         struct MaterialProperties material2,
                                         double radius1,
                                         double radius2,
                                         double penetration,
                                         double penetration_rate,
                                         double damping,
                                         struct HertzContactReport *out_report);

struct Bool astro_nbody_direct_accelerations(const struct NBodyParticle *particles,
                                             uint32_t particle_count,
                                             struct NBodySolverParams params,
                                             struct Vec3 *out_accelerations,
                                             uint32_t capacity,
                                             struct NBodyForceReport *out_report);

struct Bool astro_nbody_barnes_hut_accelerations(const struct NBodyParticle *particles,
                                                 uint32_t particle_count,
                                                 struct NBodySolverParams params,
                                                 struct Vec3 *out_accelerations,
                                                 uint32_t capacity,
                                                 struct NBodyForceReport *out_report);

struct Bool astro_fmm_monopole_acceleration(struct Vec3 position,
                                            struct Vec3 cluster_center,
                                            double cluster_mass,
                                            struct NBodySolverParams params,
                                            struct Vec3 *out_acceleration);

struct Bool astro_relativistic_orbit_correction(struct Vec3 position,
                                                struct Vec3 velocity,
                                                double central_mass,
                                                double gravitational_constant,
                                                struct RelativisticOrbitReport *out_report);

struct Bool astro_roche_limit(double primary_radius,
                              double primary_density,
                              double secondary_density,
                              double orbital_distance,
                              struct RocheLimitReport *out_report);

struct Bool astro_orbital_resonance_detect(double inner_period,
                                           double outer_period,
                                           uint32_t max_denominator,
                                           double tolerance,
                                           struct OrbitalResonanceReport *out_report);

struct Bool astro_barnes_hut_should_open(double node_width, double distance, double opening_angle);

double biomechanics_hill_force_length_factor(double fiber_length,
                                             double optimal_fiber_length,
                                             double width);

double biomechanics_hill_force_velocity_factor(double fiber_velocity,
                                               double max_contraction_velocity);

struct Bool biomechanics_hill_muscle_evaluate(struct HillMuscleDesc desc,
                                              struct HillMuscleState state,
                                              struct HillMuscleReport *out_report);

double biomechanics_hill_three_element_force(double activation,
                                             double fiber_length,
                                             double fiber_velocity,
                                             double tendon_length,
                                             struct HillMuscleDesc desc);

struct Bool biomechanics_skeletal_joint_limit(double angle,
                                              double angular_velocity,
                                              struct SkeletalJointLimit limit,
                                              struct SkeletalConstraintReport *out_report);

double biomechanics_muscle_joint_torque(double muscle_force, double moment_arm);

struct ColliderBuilderHandle *collider_builder_create_capsule(struct Capsule capsule);

struct ColliderBuilderHandle *collider_builder_create_ssv(struct Ssv ssv);

struct ColliderBuilderHandle *collider_builder_create_ellipsoid(struct Ellipsoid ellipsoid);

struct ColliderBuilderHandle *collider_builder_create_prism(struct Prism prism);

struct ColliderBuilderHandle *collider_builder_create_cylinder(struct Cylinder cylinder);

struct ColliderBuilderHandle *collider_builder_create_spherical_shell(struct SphericalShell shell);

uint32_t query_intersect_capsule_count(const struct WorldHandle *world,
                                       struct Capsule capsule,
                                       struct QueryFilterDesc filter);

uint32_t query_intersect_capsule_count_all(const struct WorldHandle *world, struct Capsule capsule);

uint32_t query_intersect_capsule(const struct WorldHandle *world,
                                 struct Capsule capsule,
                                 struct QueryFilterDesc filter,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_capsule_all(const struct WorldHandle *world,
                                     struct Capsule capsule,
                                     ColliderHandleRaw *out_handles,
                                     uint32_t capacity);

uint32_t query_intersect_ssv_count(const struct WorldHandle *world,
                                   struct Ssv ssv,
                                   struct QueryFilterDesc filter);

uint32_t query_intersect_ssv_count_all(const struct WorldHandle *world, struct Ssv ssv);

uint32_t query_intersect_ssv(const struct WorldHandle *world,
                             struct Ssv ssv,
                             struct QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

uint32_t query_intersect_ssv_all(const struct WorldHandle *world,
                                 struct Ssv ssv,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_ellipsoid_count(const struct WorldHandle *world,
                                         struct Ellipsoid ellipsoid,
                                         struct QueryFilterDesc filter);

uint32_t query_intersect_ellipsoid_count_all(const struct WorldHandle *world,
                                             struct Ellipsoid ellipsoid);

uint32_t query_intersect_ellipsoid(const struct WorldHandle *world,
                                   struct Ellipsoid ellipsoid,
                                   struct QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_ellipsoid_all(const struct WorldHandle *world,
                                       struct Ellipsoid ellipsoid,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

uint32_t query_intersect_prism_count(const struct WorldHandle *world,
                                     struct Prism prism,
                                     struct QueryFilterDesc filter);

uint32_t query_intersect_prism_count_all(const struct WorldHandle *world, struct Prism prism);

uint32_t query_intersect_prism(const struct WorldHandle *world,
                               struct Prism prism,
                               struct QueryFilterDesc filter,
                               ColliderHandleRaw *out_handles,
                               uint32_t capacity);

uint32_t query_intersect_prism_all(const struct WorldHandle *world,
                                   struct Prism prism,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_cylinder_count(const struct WorldHandle *world,
                                        struct Cylinder cylinder,
                                        struct QueryFilterDesc filter);

uint32_t query_intersect_cylinder_count_all(const struct WorldHandle *world,
                                            struct Cylinder cylinder);

uint32_t query_intersect_cylinder(const struct WorldHandle *world,
                                  struct Cylinder cylinder,
                                  struct QueryFilterDesc filter,
                                  ColliderHandleRaw *out_handles,
                                  uint32_t capacity);

uint32_t query_intersect_cylinder_all(const struct WorldHandle *world,
                                      struct Cylinder cylinder,
                                      ColliderHandleRaw *out_handles,
                                      uint32_t capacity);

uint32_t query_intersect_spherical_shell_count(const struct WorldHandle *world,
                                               struct SphericalShell shell,
                                               struct QueryFilterDesc filter);

uint32_t query_intersect_spherical_shell_count_all(const struct WorldHandle *world,
                                                   struct SphericalShell shell);

uint32_t query_intersect_spherical_shell(const struct WorldHandle *world,
                                         struct SphericalShell shell,
                                         struct QueryFilterDesc filter,
                                         ColliderHandleRaw *out_handles,
                                         uint32_t capacity);

uint32_t query_intersect_spherical_shell_all(const struct WorldHandle *world,
                                             struct SphericalShell shell,
                                             ColliderHandleRaw *out_handles,
                                             uint32_t capacity);

struct ColliderBuilderHandle *collider_builder_create(uint32_t shape_type, struct Vec3 shape_data);

struct ColliderBuilderHandle *collider_builder_create_ex(struct ShapeDesc shape_desc);

struct ColliderBuilderHandle *collider_builder_create_obb(struct Obb obb);

struct ColliderBuilderHandle *collider_builder_create_sphere(struct Sphere sphere);

struct ColliderBuilderHandle *collider_builder_create_heightmap(const double *data,
                                                                uint32_t data_x,
                                                                uint32_t data_y,
                                                                struct Vec3 scale);

struct ColliderBuilderHandle *collider_builder_create_convex_hull(const double *points_xyz,
                                                                  uint32_t point_count);

struct ColliderBuilderHandle *collider_builder_create_point_cloud_bounds(const double *points_xyz,
                                                                         uint32_t point_count);

struct ColliderBuilderHandle *collider_builder_create_double_bv(struct AabbDesc first,
                                                                struct AabbDesc second);

struct ColliderBuilderHandle *collider_builder_create_skewed_obb(struct Vec3 center,
                                                                 struct Vec3 axis_x,
                                                                 struct Vec3 axis_y,
                                                                 struct Vec3 axis_z);

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

void collider_builder_set_translation(struct ColliderBuilderHandle *builder,
                                      struct Vec3 translation);

void collider_builder_set_rotation(struct ColliderBuilderHandle *builder,
                                   struct Vec3 rotation_axis_angle);

void collider_builder_set_pose(struct ColliderBuilderHandle *builder,
                               struct Vec3 translation,
                               struct Quat rotation);

void collider_builder_set_sensor(struct ColliderBuilderHandle *builder, struct Bool sensor);

void collider_builder_set_friction(struct ColliderBuilderHandle *builder, double friction);

void collider_builder_set_restitution(struct ColliderBuilderHandle *builder, double restitution);

void collider_builder_set_density(struct ColliderBuilderHandle *builder, double density);

void collider_builder_set_collision_groups(struct ColliderBuilderHandle *builder,
                                           struct InteractionGroupsDesc groups);

void collider_builder_set_solver_groups(struct ColliderBuilderHandle *builder,
                                        struct InteractionGroupsDesc groups);

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

struct Bool world_remove_collider(struct WorldHandle *world,
                                  ColliderHandleRaw handle,
                                  struct Bool wake_up);

Collider *world_copy_collider(struct WorldHandle *world, ColliderHandleRaw handle);

uint8_t world_remove_collider_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   struct Bool wake_up);

struct Vec3 collider_get_translation(const struct WorldHandle *world, ColliderHandleRaw handle);

void collider_get_translation_out(const struct WorldHandle *world,
                                  ColliderHandleRaw handle,
                                  struct Vec3 *out_translation);

struct Quat collider_get_rotation(const struct WorldHandle *world, ColliderHandleRaw handle);

void collider_get_rotation_out(const struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               struct Quat *out_rotation);

struct Bool collider_set_pose(struct WorldHandle *world,
                              ColliderHandleRaw handle,
                              struct Vec3 translation,
                              struct Quat rotation);

uint8_t collider_set_pose_flag(struct WorldHandle *world,
                               ColliderHandleRaw handle,
                               struct Vec3 translation,
                               struct Quat rotation);

struct Bool collider_set_sensor(struct WorldHandle *world,
                                ColliderHandleRaw handle,
                                struct Bool sensor);

uint8_t collider_set_sensor_flag(struct WorldHandle *world,
                                 ColliderHandleRaw handle,
                                 struct Bool sensor);

struct Bool collider_set_friction(struct WorldHandle *world,
                                  ColliderHandleRaw handle,
                                  double friction);

uint8_t collider_set_friction_flag(struct WorldHandle *world,
                                   ColliderHandleRaw handle,
                                   double friction);

struct Bool collider_set_restitution(struct WorldHandle *world,
                                     ColliderHandleRaw handle,
                                     double restitution);

uint8_t collider_set_restitution_flag(struct WorldHandle *world,
                                      ColliderHandleRaw handle,
                                      double restitution);

struct Bool collider_set_collision_groups(struct WorldHandle *world,
                                          ColliderHandleRaw handle,
                                          struct InteractionGroupsDesc groups);

uint8_t collider_set_collision_groups_flag(struct WorldHandle *world,
                                           ColliderHandleRaw handle,
                                           struct InteractionGroupsDesc groups);

struct Bool collider_set_solver_groups(struct WorldHandle *world,
                                       ColliderHandleRaw handle,
                                       struct InteractionGroupsDesc groups);

uint8_t collider_set_solver_groups_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        struct InteractionGroupsDesc groups);

struct Bool collider_set_active_events(struct WorldHandle *world,
                                       ColliderHandleRaw handle,
                                       uint32_t active_events_bits);

uint8_t collider_set_active_events_flag(struct WorldHandle *world,
                                        ColliderHandleRaw handle,
                                        uint32_t active_events_bits);

struct Bool collider_set_active_hooks(struct WorldHandle *world,
                                      ColliderHandleRaw handle,
                                      uint32_t active_hooks_bits);

uint8_t collider_set_active_hooks_flag(struct WorldHandle *world,
                                       ColliderHandleRaw handle,
                                       uint32_t active_hooks_bits);

struct Bool collider_set_contact_force_event_threshold(struct WorldHandle *world,
                                                       ColliderHandleRaw handle,
                                                       double threshold);

uint8_t collider_set_contact_force_event_threshold_flag(struct WorldHandle *world,
                                                        ColliderHandleRaw handle,
                                                        double threshold);

double collider_get_density(const struct WorldHandle *world, ColliderHandleRaw handle);

RigidBodyHandleRaw world_insert_dynamic_cuboids(struct WorldHandle *world,
                                                struct Vec3 translation,
                                                struct Quat rotation,
                                                struct Vec3 linvel,
                                                const double *cuboids,
                                                uint32_t cuboid_count,
                                                double density,
                                                double friction,
                                                double restitution,
                                                struct InteractionGroupsDesc collision_groups,
                                                struct InteractionGroupsDesc solver_groups);

RigidBodyHandleRaw world_insert_static_trimesh(struct WorldHandle *world,
                                               const double *vertices_xyz,
                                               uint32_t vertex_xyz_len,
                                               const uint32_t *indices,
                                               uint32_t index_len,
                                               double friction,
                                               double restitution);

uint32_t query_intersect_aabb_rigid_body_count(const struct WorldHandle *world,
                                               struct AabbDesc aabb,
                                               struct QueryFilterDesc filter);

uint32_t query_intersect_aabb_rigid_bodies(const struct WorldHandle *world,
                                           struct AabbDesc aabb,
                                           struct QueryFilterDesc filter,
                                           RigidBodyHandleRaw *out_handles,
                                           uint32_t capacity);

double continuum_tetra_volume(struct FemTetrahedron tetra);

struct Bool continuum_tetra_shape_functions(struct FemTetrahedron tetra,
                                            struct Vec3 point,
                                            struct FemShapeFunctionReport *out_report);

struct Bool continuum_linear_elastic_constitutive_matrix(struct MaterialProperties material,
                                                         double *out_matrix,
                                                         uint32_t capacity,
                                                         struct FemConstitutiveReport *out_report);

struct Bool continuum_tetra_strain_displacement_matrix(struct FemTetrahedron tetra,
                                                       double *out_matrix,
                                                       uint32_t capacity,
                                                       double *out_volume);

struct Bool continuum_newmark_beta_solve(const double *mass_matrix,
                                         const double *damping_matrix,
                                         const double *stiffness_matrix,
                                         const double *displacement,
                                         const double *velocity,
                                         const double *acceleration,
                                         const double *external_force,
                                         uint32_t dof,
                                         struct NewmarkBetaParameters params,
                                         double *out_delta_displacement,
                                         double *out_next_displacement,
                                         double *out_next_velocity,
                                         double *out_next_acceleration,
                                         uint32_t capacity,
                                         struct NewmarkBetaReport *out_report);

struct Bool continuum_linear_tetra_element_stiffness(struct FemTetrahedron tetra,
                                                     struct MaterialProperties material,
                                                     double *out_stiffness,
                                                     uint32_t capacity,
                                                     double *out_volume);

struct Bool continuum_deformation_gradient(struct FemTetrahedron reference_tetra,
                                           struct FemTetrahedron deformed_tetra,
                                           double *out_matrix,
                                           uint32_t capacity);

struct Bool control_pid_step(double setpoint,
                             double measurement,
                             double dt,
                             struct PidGains gains,
                             struct PidState *state,
                             struct PidReport *out_report);

struct Bool control_state_space_step(const double *a_matrix,
                                     const double *b_matrix,
                                     const double *c_matrix,
                                     const double *d_matrix,
                                     const double *state,
                                     const double *input,
                                     uint32_t state_count,
                                     uint32_t input_count,
                                     uint32_t output_count,
                                     double *out_next_state,
                                     double *out_output,
                                     uint32_t state_capacity,
                                     uint32_t output_capacity,
                                     struct StateSpaceReport *out_report);

struct Bool control_mpc_solve_box_qp(const double *a_matrix,
                                     const double *b_matrix,
                                     const double *q_diag,
                                     const double *r_diag,
                                     const double *initial_state,
                                     const double *target_state,
                                     struct MpcConfig config,
                                     double *out_first_control,
                                     uint32_t control_capacity,
                                     struct MpcReport *out_report);

struct Bool control_lqr_like_stabilizing_input(const double *state,
                                               const double *gain_matrix,
                                               uint32_t state_count,
                                               uint32_t input_count,
                                               double control_min,
                                               double control_max,
                                               double *out_control,
                                               uint32_t capacity);

struct CharacterControllerHandle *character_controller_create(void);

void character_controller_destroy(struct CharacterControllerHandle *controller);

void character_controller_set_up(struct CharacterControllerHandle *controller, struct Vec3 up);

void character_controller_set_offset_absolute(struct CharacterControllerHandle *controller,
                                              double offset);

void character_controller_set_offset_relative(struct CharacterControllerHandle *controller,
                                              double offset);

void character_controller_set_slide(struct CharacterControllerHandle *controller,
                                    struct Bool slide);

void character_controller_set_autostep(struct CharacterControllerHandle *controller,
                                       struct Bool enabled,
                                       double max_height,
                                       double min_width,
                                       struct Bool include_dynamic_bodies);

void character_controller_set_snap_to_ground(struct CharacterControllerHandle *controller,
                                             struct Bool enabled,
                                             double distance);

void character_controller_set_slope_angles(struct CharacterControllerHandle *controller,
                                           double max_climb_angle,
                                           double min_slide_angle);

struct EffectiveCharacterMovement character_controller_move_shape(const struct WorldHandle *world,
                                                                  struct CharacterControllerHandle *controller,
                                                                  double dt,
                                                                  struct ShapeDesc shape_desc,
                                                                  struct Vec3 translation,
                                                                  struct Quat rotation,
                                                                  struct Vec3 desired_translation);

uint32_t character_controller_collision_count(const struct CharacterControllerHandle *controller);

FfiCharacterCollision character_controller_get_collision(const struct CharacterControllerHandle *controller,
                                                         uint32_t index);

struct Bool character_controller_solve_impulses(struct WorldHandle *world,
                                                struct CharacterControllerHandle *controller,
                                                double dt,
                                                struct ShapeDesc shape_desc,
                                                double character_mass);

struct CRbTreeHandle *crb_tree_create(void);

void crb_tree_destroy(struct CRbTreeHandle *tree);

void crb_tree_clear(struct CRbTreeHandle *tree);

uint32_t crb_tree_len(const struct CRbTreeHandle *tree);

struct Bool crb_tree_insert(struct CRbTreeHandle *tree, uint64_t id, struct AabbDesc aabb);

uint8_t crb_tree_insert_flag(struct CRbTreeHandle *tree, uint64_t id, struct AabbDesc aabb);

struct Bool crb_tree_update(struct CRbTreeHandle *tree, uint64_t id, struct AabbDesc aabb);

struct Bool crb_tree_remove(struct CRbTreeHandle *tree, uint64_t id);

uint32_t crb_tree_query_aabb_count(const struct CRbTreeHandle *tree, struct AabbDesc aabb);

uint32_t crb_tree_query_aabb(const struct CRbTreeHandle *tree,
                             struct AabbDesc aabb,
                             uint64_t *out_ids,
                             uint32_t capacity);

struct ColliderBuilderHandle *collider_builder_create_kdop(const double *points_xyz,
                                                           uint32_t point_count,
                                                           uint32_t preset);

struct ColliderBuilderHandle *collider_builder_create_fdh(const double *points_xyz,
                                                          uint32_t point_count,
                                                          const double *directions_xyz,
                                                          uint32_t direction_count);

struct Bool em_lorentz_force(double charge,
                             struct Vec3 velocity,
                             struct ElectromagneticField field,
                             double mass,
                             struct LorentzForceReport *out_report);

struct Bool em_magnetic_flux(struct Vec3 magnetic_field,
                             struct Vec3 area_normal,
                             double area,
                             struct MagneticFluxReport *out_report);

struct Bool em_faraday_induction(double previous_flux,
                                 double current_flux,
                                 double dt,
                                 double turns,
                                 double resistance,
                                 struct FaradayInductionReport *out_report);

struct Bool em_maxwell_point_update(struct ElectromagneticField field,
                                    struct Vec3 curl_electric,
                                    struct Vec3 curl_magnetic,
                                    struct Vec3 current_density,
                                    double charge_density,
                                    double divergence_electric,
                                    double divergence_magnetic,
                                    double permittivity,
                                    double permeability,
                                    double dt,
                                    struct MaxwellPointReport *out_report);

struct Bool em_fdtd_yee_update(const struct Vec3 *electric_fields,
                               const struct Vec3 *magnetic_fields,
                               const struct Vec3 *curl_electric,
                               const struct Vec3 *curl_magnetic,
                               uint32_t cell_count,
                               double permittivity,
                               double permeability,
                               double conductivity,
                               double dt,
                               struct Vec3 *out_electric_fields,
                               struct Vec3 *out_magnetic_fields,
                               uint32_t capacity,
                               struct FdtdYeeReport *out_report);

double em_vacuum_permittivity(void);

double em_vacuum_permeability(void);

uint32_t last_error_code(void);

const char *last_error_message(void);

void last_error_clear(void);

struct Bool world_set_coulomb_friction_law(struct WorldHandle *world,
                                           struct CoulombFrictionLaw law);

uint8_t world_set_coulomb_friction_law_flag(struct WorldHandle *world,
                                            struct CoulombFrictionLaw law);

void world_clear_coulomb_friction_law(struct WorldHandle *world);

struct Bool world_get_coulomb_friction_law(const struct WorldHandle *world,
                                           struct CoulombFrictionLaw *out_law);

struct Bool world_set_air_drag_law(struct WorldHandle *world, struct AirDragLaw law);

uint8_t world_set_air_drag_law_flag(struct WorldHandle *world, struct AirDragLaw law);

void world_clear_air_drag_law(struct WorldHandle *world);

struct Bool world_get_air_drag_law(const struct WorldHandle *world, struct AirDragLaw *out_law);

struct Bool world_set_external_force_law(struct WorldHandle *world, struct ExternalForceLaw law);

uint8_t world_set_external_force_law_flag(struct WorldHandle *world, struct ExternalForceLaw law);

void world_clear_external_force_law(struct WorldHandle *world);

struct Bool world_get_external_force_law(const struct WorldHandle *world,
                                         struct ExternalForceLaw *out_law);

struct Bool world_get_custom_physics_report(const struct WorldHandle *world,
                                            struct CustomPhysicsReport *out_report);

void world_clear_events(struct WorldHandle *world);

uint32_t world_collision_event_count(const struct WorldHandle *world);

struct CollisionEventRecord world_get_collision_event(const struct WorldHandle *world,
                                                      uint32_t index);

uint32_t world_get_collision_events(const struct WorldHandle *world,
                                    struct CollisionEventRecord *out_events,
                                    uint32_t capacity);

uint32_t world_contact_force_event_count(const struct WorldHandle *world);

struct ContactForceEventRecord world_get_contact_force_event(const struct WorldHandle *world,
                                                             uint32_t index);

uint32_t world_get_contact_force_events(const struct WorldHandle *world,
                                        struct ContactForceEventRecord *out_events,
                                        uint32_t capacity);

void world_set_contact_pair_filter_callback(struct WorldHandle *world,
                                            uintptr_t _callback,
                                            uintptr_t _user_data);

void world_set_intersection_pair_filter_callback(struct WorldHandle *world,
                                                 uintptr_t _callback,
                                                 uintptr_t _user_data);

void world_clear_contact_pair_filter_callback(struct WorldHandle *world);

void world_clear_intersection_pair_filter_callback(struct WorldHandle *world);

struct Bool fluid_estimate_aabb_forces(struct FluidVolume fluid,
                                       struct Vec3 body_center,
                                       struct Vec3 body_half_extents,
                                       double body_volume,
                                       struct Vec3 body_linvel,
                                       struct Vec3 body_angvel,
                                       struct FluidForceReport *out_report);

struct Bool fluid_apply_aabb_forces(struct WorldHandle *world,
                                    RigidBodyHandleRaw body_handle,
                                    struct FluidVolume fluid,
                                    struct Vec3 body_half_extents,
                                    double body_volume,
                                    struct Bool wake_up,
                                    struct FluidForceReport *out_report);

uint8_t fluid_apply_aabb_forces_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw body_handle,
                                     struct FluidVolume fluid,
                                     struct Vec3 body_half_extents,
                                     double body_volume,
                                     struct Bool wake_up,
                                     struct FluidForceReport *out_report);

struct Bool fluid_navier_stokes_simplified_step(struct Vec3 velocity,
                                                struct Vec3 advection,
                                                struct Vec3 pressure_gradient,
                                                struct Vec3 laplacian_velocity,
                                                struct Vec3 external_acceleration,
                                                double density,
                                                double kinematic_viscosity,
                                                double dt,
                                                struct NavierStokesReport *out_report);

double fluid_sph_poly6_kernel(double distance, double smoothing_radius);

struct Bool fluid_sph_spiky_gradient(struct Vec3 offset,
                                     double smoothing_radius,
                                     struct Vec3 *out_gradient);

double fluid_sph_viscosity_laplacian(double distance, double smoothing_radius);

struct Bool fluid_sph_estimate_density(struct Vec3 position,
                                       const struct SphParticle *particles,
                                       uint32_t particle_count,
                                       double smoothing_radius,
                                       double *out_density);

struct Bool fluid_sph_estimate_forces(struct SphParticle particle,
                                      const struct SphParticle *particles,
                                      uint32_t particle_count,
                                      double smoothing_radius,
                                      double gas_constant,
                                      double rest_density,
                                      double viscosity,
                                      double surface_tension,
                                      struct SphForceReport *out_report);

double fluid_bernoulli_pressure(double total_pressure,
                                double density,
                                double velocity,
                                double gravity,
                                double elevation);

struct Bool fluid_bernoulli_report(double pressure,
                                   double density,
                                   double velocity,
                                   double gravity,
                                   double elevation,
                                   struct BernoulliReport *out_report);

struct Bool fracture_stress_intensity_factor(double stress,
                                             double crack_length,
                                             double geometry_factor,
                                             double fracture_toughness,
                                             struct StressIntensityReport *out_report);

struct Bool fracture_griffith_criterion(double stress,
                                        double crack_length,
                                        struct FractureMaterial material,
                                        struct GriffithReport *out_report);

struct Bool fracture_miner_damage(const double *cycle_counts,
                                  const double *cycles_to_failure,
                                  uint32_t count,
                                  struct MinerDamageReport *out_report);

struct Bool fracture_sn_curve_life(double stress_amplitude,
                                   double coefficient,
                                   double exponent,
                                   double endurance_limit,
                                   struct SnCurveReport *out_report);

struct Bool fracture_energy_release(double strain_energy,
                                    double new_surface_area,
                                    double surface_energy,
                                    double kinetic_energy,
                                    struct FractureEnergyReport *out_report);

struct Bool fracture_mode_from_stress(double tensile_stress,
                                      double shear_stress,
                                      double compressive_stress,
                                      struct FractureModeReport *out_report);

struct Bool world_replace_body_with_fracture_fragments(struct WorldHandle *world,
                                                       RigidBodyHandleRaw source_body,
                                                       const struct FractureFragmentDesc *fragments,
                                                       uint32_t fragment_count,
                                                       struct Bool connect_fragments,
                                                       struct Bool remove_source,
                                                       RigidBodyHandleRaw *out_body_handles,
                                                       ImpulseJointHandleRaw *out_joint_handles,
                                                       uint32_t capacity,
                                                       struct FractureReplaceReport *out_report);

struct JointBuilderHandle *joint_builder_create(uint32_t joint_type,
                                                struct Vec3 axis_or_primary,
                                                double b,
                                                double c);

void joint_builder_destroy(struct JointBuilderHandle *builder);

void joint_builder_set_contacts_enabled(struct JointBuilderHandle *builder, struct Bool enabled);

void joint_builder_set_local_anchor1(struct JointBuilderHandle *builder, struct Vec3 anchor);

void joint_builder_set_local_anchor2(struct JointBuilderHandle *builder, struct Vec3 anchor);

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
                                                 struct Bool wake_up);

struct Bool world_remove_impulse_joint(struct WorldHandle *world,
                                       ImpulseJointHandleRaw handle,
                                       struct Bool wake_up);

double molecular_lennard_jones_potential(double distance, double epsilon, double sigma);

struct Bool molecular_lennard_jones_force(struct Vec3 displacement,
                                          double epsilon,
                                          double sigma,
                                          double softening,
                                          struct Vec3 *out_force);

double molecular_coulomb_potential(double distance,
                                   double charge_a,
                                   double charge_b,
                                   double coulomb_constant,
                                   double relative_permittivity);

struct Bool molecular_coulomb_force(struct Vec3 displacement,
                                    double charge_a,
                                    double charge_b,
                                    double coulomb_constant,
                                    double relative_permittivity,
                                    double softening,
                                    struct Vec3 *out_force);

struct Bool molecular_pair_interaction(struct MolecularParticle particle_a,
                                       struct MolecularParticle particle_b,
                                       struct MolecularForceLaw law,
                                       struct MolecularPairReport *out_report);

struct Bool molecular_apply_pair_forces(struct WorldHandle *world,
                                        RigidBodyHandleRaw body_a,
                                        RigidBodyHandleRaw body_b,
                                        struct MolecularParticle particle_a,
                                        struct MolecularParticle particle_b,
                                        struct MolecularForceLaw law,
                                        struct Bool wake_up,
                                        struct MolecularPairReport *out_report);

uint8_t molecular_apply_pair_forces_flag(struct WorldHandle *world,
                                         RigidBodyHandleRaw body_a,
                                         RigidBodyHandleRaw body_b,
                                         struct MolecularParticle particle_a,
                                         struct MolecularParticle particle_b,
                                         struct MolecularForceLaw law,
                                         struct Bool wake_up,
                                         struct MolecularPairReport *out_report);

double molecular_vacuum_coulomb_constant(void);

uint32_t neural_bounds_required_weight_count(uint32_t hidden_width, uint32_t hidden_layers);

struct ColliderBuilderHandle *collider_builder_create_neural_bounds(struct NeuralBoundsDesc desc,
                                                                    const double *weights,
                                                                    uint32_t weight_count);

uint32_t query_intersect_neural_bounds_count(const struct WorldHandle *world,
                                             struct NeuralBoundsDesc desc,
                                             const double *weights,
                                             uint32_t weight_count,
                                             struct QueryFilterDesc filter);

uint32_t query_intersect_neural_bounds_count_all(const struct WorldHandle *world,
                                                 struct NeuralBoundsDesc desc,
                                                 const double *weights,
                                                 uint32_t weight_count);

uint32_t query_intersect_neural_bounds(const struct WorldHandle *world,
                                       struct NeuralBoundsDesc desc,
                                       const double *weights,
                                       uint32_t weight_count,
                                       struct QueryFilterDesc filter,
                                       ColliderHandleRaw *out_handles,
                                       uint32_t capacity);

uint32_t query_intersect_neural_bounds_all(const struct WorldHandle *world,
                                           struct NeuralBoundsDesc desc,
                                           const double *weights,
                                           uint32_t weight_count,
                                           ColliderHandleRaw *out_handles,
                                           uint32_t capacity);

struct Bool physchem_catalyst_rate_multiplier(double base_rate,
                                              struct CatalystEffect catalyst,
                                              struct CatalystReport *out_report);

struct Bool physchem_gray_scott_reaction_terms(double u,
                                               double v,
                                               double laplacian_u,
                                               double laplacian_v,
                                               struct GrayScottParams params,
                                               struct CatalystEffect catalyst,
                                               struct GrayScottReactionReport *out_report);

struct Bool physchem_gray_scott_step_2d(const double *u_values,
                                        const double *v_values,
                                        uint32_t width,
                                        uint32_t height,
                                        struct GrayScottParams params,
                                        struct CatalystEffect catalyst,
                                        double dt,
                                        double *out_u_values,
                                        double *out_v_values,
                                        uint32_t capacity,
                                        struct ReactionDiffusionReport *out_report);

double physchem_reaction_diffusion_explicit(double concentration,
                                            double laplacian,
                                            double diffusion_coefficient,
                                            double reaction_rate,
                                            double source,
                                            double dt);

struct Bool physchem_concentration_buoyancy(double concentration,
                                            double reference_concentration,
                                            double reference_density,
                                            double expansion_coefficient,
                                            double volume,
                                            struct Vec3 gravity,
                                            struct ConcentrationBuoyancyReport *out_report);

double quantum_reduced_planck_constant(void);

double quantum_wave_probability_density(struct QuantumWaveFunction wave);

struct Bool quantum_wave_normalize(struct QuantumWaveFunction wave,
                                   struct QuantumWaveFunction *out_wave);

double quantum_wkb_transmission(double action_integral, double reduced_planck);

struct Bool quantum_rectangular_barrier_tunneling(struct QuantumBarrier barrier,
                                                  struct QuantumTunnelingReport *out_report);

double quantum_rectangular_barrier_probability(struct QuantumBarrier barrier);

double quantum_zero_point_energy(double angular_frequency, double reduced_planck);

struct Bool quantum_harmonic_oscillator_report(double angular_frequency,
                                               double reduced_planck,
                                               struct QuantumOscillatorReport *out_report);

struct RayHit query_cast_ray(const struct WorldHandle *world,
                             struct Vec3 origin,
                             struct Vec3 direction,
                             double max_toi,
                             struct Bool solid,
                             struct QueryFilterDesc filter);

ColliderHandleRaw query_cast_ray_out(const struct WorldHandle *world,
                                     struct Vec3 origin,
                                     struct Vec3 direction,
                                     double max_toi,
                                     struct Bool solid,
                                     struct QueryFilterDesc filter,
                                     struct RayHit *out_hit);

uint32_t query_cast_rays(const struct WorldHandle *world,
                         const double *rays,
                         uint32_t ray_count,
                         double max_toi,
                         struct Bool solid,
                         struct QueryFilterDesc filter,
                         struct RayHit *out_hits,
                         uint32_t capacity);

struct PointProjection query_project_point(const struct WorldHandle *world,
                                           struct Vec3 point,
                                           double max_dist,
                                           struct Bool solid,
                                           struct QueryFilterDesc filter,
                                           ColliderHandleRaw *out_collider);

ColliderHandleRaw query_project_point_out(const struct WorldHandle *world,
                                          struct Vec3 point,
                                          double max_dist,
                                          struct Bool solid,
                                          struct QueryFilterDesc filter,
                                          ColliderHandleRaw *out_collider,
                                          struct PointProjection *out_projection);

uint32_t query_intersect_point_count(const struct WorldHandle *world,
                                     struct Vec3 point,
                                     struct QueryFilterDesc filter);

uint32_t query_intersect_aabb_count(const struct WorldHandle *world,
                                    struct AabbDesc aabb,
                                    struct QueryFilterDesc filter);

uint32_t query_intersect_aabb(const struct WorldHandle *world,
                              struct AabbDesc aabb,
                              struct QueryFilterDesc filter,
                              ColliderHandleRaw *out_handles,
                              uint32_t capacity);

uint32_t query_intersect_aabb_count_all(const struct WorldHandle *world, struct AabbDesc aabb);

uint32_t query_intersect_aabb_counts(const struct WorldHandle *world,
                                     const struct AabbDesc *aabbs,
                                     uint32_t query_count,
                                     struct QueryFilterDesc filter,
                                     uint32_t *out_counts,
                                     uint32_t capacity);

uint32_t query_intersect_obb_count(const struct WorldHandle *world,
                                   struct Obb obb,
                                   struct QueryFilterDesc filter);

uint32_t query_intersect_obb_count_all(const struct WorldHandle *world, struct Obb obb);

uint32_t query_intersect_obb_counts(const struct WorldHandle *world,
                                    const struct Obb *obbs,
                                    uint32_t query_count,
                                    struct QueryFilterDesc filter,
                                    uint32_t *out_counts,
                                    uint32_t capacity);

uint32_t query_intersect_obb(const struct WorldHandle *world,
                             struct Obb obb,
                             struct QueryFilterDesc filter,
                             ColliderHandleRaw *out_handles,
                             uint32_t capacity);

uint32_t query_intersect_obb_all(const struct WorldHandle *world,
                                 struct Obb obb,
                                 ColliderHandleRaw *out_handles,
                                 uint32_t capacity);

uint32_t query_intersect_sphere_count(const struct WorldHandle *world,
                                      struct Sphere sphere,
                                      struct QueryFilterDesc filter);

uint32_t query_intersect_sphere_count_all(const struct WorldHandle *world, struct Sphere sphere);

uint32_t query_intersect_sphere_counts(const struct WorldHandle *world,
                                       const struct Sphere *spheres,
                                       uint32_t query_count,
                                       struct QueryFilterDesc filter,
                                       uint32_t *out_counts,
                                       uint32_t capacity);

uint32_t query_intersect_sphere(const struct WorldHandle *world,
                                struct Sphere sphere,
                                struct QueryFilterDesc filter,
                                ColliderHandleRaw *out_handles,
                                uint32_t capacity);

uint32_t query_intersect_sphere_all(const struct WorldHandle *world,
                                    struct Sphere sphere,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

uint32_t query_intersect_aabb_rigid_body_count_all(const struct WorldHandle *world,
                                                   struct AabbDesc aabb);

uint32_t query_intersect_aabb_rigid_bodies_all(const struct WorldHandle *world,
                                               struct AabbDesc aabb,
                                               RigidBodyHandleRaw *out_handles,
                                               uint32_t capacity);

struct ShapeCastHit query_cast_shape(const struct WorldHandle *world,
                                     struct ShapeDesc shape_desc,
                                     struct Vec3 translation,
                                     struct Quat rotation,
                                     struct Vec3 velocity,
                                     struct ShapeCastOptionsDesc options,
                                     struct QueryFilterDesc filter);

ColliderHandleRaw query_cast_shape_out(const struct WorldHandle *world,
                                       struct ShapeDesc shape_desc,
                                       struct Vec3 translation,
                                       struct Quat rotation,
                                       struct Vec3 velocity,
                                       struct ShapeCastOptionsDesc options,
                                       struct QueryFilterDesc filter,
                                       struct ShapeCastHit *out_hit);

struct RigidBodyBuilderHandle *rigid_body_builder_create(uint32_t status);

RigidBody *rigid_body_builder_build(struct RigidBodyBuilderHandle *builder);

void rigid_body_builder_destroy(struct RigidBodyBuilderHandle *builder);

void rigid_body_destroy_raw(RigidBody *rigid_body);

void rigid_body_builder_set_translation(struct RigidBodyBuilderHandle *builder,
                                        struct Vec3 translation);

void rigid_body_builder_set_rotation(struct RigidBodyBuilderHandle *builder,
                                     struct Vec3 rotation_axis_angle);

void rigid_body_builder_set_pose(struct RigidBodyBuilderHandle *builder,
                                 struct Vec3 translation,
                                 struct Quat rotation);

void rigid_body_builder_set_additional_mass_properties(struct RigidBodyBuilderHandle *builder,
                                                       struct Vec3 center,
                                                       double mass,
                                                       struct Vec3 inertia);

void rigid_body_builder_set_linvel(struct RigidBodyBuilderHandle *builder, struct Vec3 linvel);

void rigid_body_builder_set_angvel(struct RigidBodyBuilderHandle *builder, struct Vec3 angvel);

void rigid_body_builder_set_gravity_scale(struct RigidBodyBuilderHandle *builder,
                                          double gravity_scale);

void rigid_body_builder_set_linear_damping(struct RigidBodyBuilderHandle *builder,
                                           double linear_damping);

void rigid_body_builder_set_angular_damping(struct RigidBodyBuilderHandle *builder,
                                            double angular_damping);

void rigid_body_builder_set_can_sleep(struct RigidBodyBuilderHandle *builder,
                                      struct Bool can_sleep);

void rigid_body_builder_set_enabled_rotations(struct RigidBodyBuilderHandle *builder,
                                              struct Bool allow_x,
                                              struct Bool allow_y,
                                              struct Bool allow_z);

void rigid_body_builder_set_user_data(struct RigidBodyBuilderHandle *builder,
                                      uint64_t user_data_low,
                                      uint64_t user_data_high);

void rigid_body_builder_set_additional_mass(struct RigidBodyBuilderHandle *builder, double mass);

RigidBodyHandleRaw world_insert_rigid_body(struct WorldHandle *world, RigidBody *memory_handle);

struct Bool world_remove_rigid_body(struct WorldHandle *world,
                                    RigidBodyHandleRaw handle,
                                    struct Bool remove_attached_colliders);

RigidBody *world_copy_rigid_body(struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t world_remove_rigid_body_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     struct Bool remove_attached_colliders);

uint32_t rigid_body_get_status(const struct WorldHandle *world, RigidBodyHandleRaw handle);

struct Bool rigid_body_set_status(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  uint32_t status,
                                  struct Bool wake_up);

struct Vec3 rigid_body_get_translation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_translation_out(const struct WorldHandle *world,
                                    RigidBodyHandleRaw handle,
                                    struct Vec3 *out_translation);

struct Quat rigid_body_get_rotation(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_rotation_out(const struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 struct Quat *out_rotation);

struct Bool rigid_body_set_pose(struct WorldHandle *world,
                                RigidBodyHandleRaw handle,
                                struct Vec3 translation,
                                struct Quat rotation,
                                struct Bool wake_up);

struct Bool rigid_body_set_translation(struct WorldHandle *world,
                                       RigidBodyHandleRaw handle,
                                       struct Vec3 translation,
                                       struct Bool wake_up);

uint8_t rigid_body_set_translation_flag(struct WorldHandle *world,
                                        RigidBodyHandleRaw handle,
                                        struct Vec3 translation,
                                        struct Bool wake_up);

struct Bool rigid_body_set_rotation(struct WorldHandle *world,
                                    RigidBodyHandleRaw handle,
                                    struct Quat rotation,
                                    struct Bool wake_up);

uint8_t rigid_body_set_rotation_flag(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     struct Quat rotation,
                                     struct Bool wake_up);

uint8_t rigid_body_set_pose_flag(struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 struct Vec3 translation,
                                 struct Quat rotation,
                                 struct Bool wake_up);

struct Vec3 rigid_body_get_linvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_linvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               struct Vec3 *out_linvel);

struct Bool rigid_body_set_linvel(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  struct Vec3 linvel,
                                  struct Bool wake_up);

uint8_t rigid_body_set_linvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   struct Vec3 linvel,
                                   struct Bool wake_up);

struct Vec3 rigid_body_get_angvel(const struct WorldHandle *world, RigidBodyHandleRaw handle);

void rigid_body_get_angvel_out(const struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               struct Vec3 *out_angvel);

struct Bool rigid_body_set_angvel(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  struct Vec3 angvel,
                                  struct Bool wake_up);

uint8_t rigid_body_set_angvel_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   struct Vec3 angvel,
                                   struct Bool wake_up);

struct Bool rigid_body_add_force(struct WorldHandle *world,
                                 RigidBodyHandleRaw handle,
                                 struct Vec3 force,
                                 struct Bool wake_up);

uint8_t rigid_body_add_force_flag(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  struct Vec3 force,
                                  struct Bool wake_up);

struct Bool rigid_body_add_torque(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  struct Vec3 torque,
                                  struct Bool wake_up);

uint8_t rigid_body_add_torque_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   struct Vec3 torque,
                                   struct Bool wake_up);

struct Bool rigid_body_apply_impulse(struct WorldHandle *world,
                                     RigidBodyHandleRaw handle,
                                     struct Vec3 impulse,
                                     struct Bool wake_up);

uint8_t rigid_body_apply_impulse_flag(struct WorldHandle *world,
                                      RigidBodyHandleRaw handle,
                                      struct Vec3 impulse,
                                      struct Bool wake_up);

struct Bool rigid_body_apply_torque_impulse(struct WorldHandle *world,
                                            RigidBodyHandleRaw handle,
                                            struct Vec3 torque_impulse,
                                            struct Bool wake_up);

uint8_t rigid_body_apply_torque_impulse_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw handle,
                                             struct Vec3 torque_impulse,
                                             struct Bool wake_up);

struct Bool rigid_body_enable_ccd(struct WorldHandle *world,
                                  RigidBodyHandleRaw handle,
                                  struct Bool enabled);

uint8_t rigid_body_enable_ccd_flag(struct WorldHandle *world,
                                   RigidBodyHandleRaw handle,
                                   struct Bool enabled);

struct Bool rigid_body_sleep(struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t rigid_body_sleep_flag(struct WorldHandle *world, RigidBodyHandleRaw handle);

struct Bool rigid_body_wake_up(struct WorldHandle *world,
                               RigidBodyHandleRaw handle,
                               struct Bool strong);

uint8_t rigid_body_wake_up_flag(struct WorldHandle *world,
                                RigidBodyHandleRaw handle,
                                struct Bool strong);

struct Bool rigid_body_is_sleeping(const struct WorldHandle *world, RigidBodyHandleRaw handle);

uint8_t rigid_body_is_sleeping_flag(const struct WorldHandle *world, RigidBodyHandleRaw handle);

struct RTreeHandle *rtree_create(void);

void rtree_destroy(struct RTreeHandle *tree);

void rtree_clear(struct RTreeHandle *tree);

uint32_t rtree_len(const struct RTreeHandle *tree);

struct Bool rtree_insert(struct RTreeHandle *tree, uint64_t id, struct AabbDesc aabb);

struct Bool rtree_update(struct RTreeHandle *tree, uint64_t id, struct AabbDesc aabb);

struct Bool rtree_remove(struct RTreeHandle *tree, uint64_t id);

void rtree_rebuild(struct RTreeHandle *tree);

uint32_t rtree_query_aabb_count(struct RTreeHandle *tree, struct AabbDesc aabb);

uint32_t rtree_query_aabb(struct RTreeHandle *tree,
                          struct AabbDesc aabb,
                          uint64_t *out_ids,
                          uint32_t capacity);

struct Bool softbody_predict_positions(const struct Vec3 *positions,
                                       const struct Vec3 *velocities,
                                       const double *inverse_masses,
                                       uint32_t particle_count,
                                       struct Vec3 gravity,
                                       double damping,
                                       double dt,
                                       struct Vec3 *out_predicted_positions,
                                       uint32_t capacity,
                                       struct SoftBodyStepReport *out_report);

struct Bool softbody_mass_spring_forces(const struct Vec3 *positions,
                                        const struct Vec3 *velocities,
                                        uint32_t particle_count,
                                        const struct SoftSpring *springs,
                                        uint32_t spring_count,
                                        struct Vec3 *out_forces,
                                        uint32_t force_capacity,
                                        struct SoftBodyStepReport *out_report);

struct Bool softbody_solve_xpbd_distance_constraints(struct Vec3 *positions,
                                                     const double *inverse_masses,
                                                     uint32_t particle_count,
                                                     struct SoftDistanceConstraint *constraints,
                                                     uint32_t constraint_count,
                                                     double dt,
                                                     uint32_t iterations,
                                                     struct SoftBodyStepReport *out_report);

struct Bool softbody_solve_xpbd_bending_constraints(struct Vec3 *positions,
                                                    const double *inverse_masses,
                                                    uint32_t particle_count,
                                                    struct SoftBendingConstraint *constraints,
                                                    uint32_t constraint_count,
                                                    double dt,
                                                    uint32_t iterations,
                                                    struct SoftBodyStepReport *out_report);

struct Bool softbody_solve_sphere_collision_constraints(struct Vec3 *positions,
                                                        const double *inverse_masses,
                                                        uint32_t particle_count,
                                                        const struct SoftSphereCollision *spheres,
                                                        uint32_t sphere_count,
                                                        struct SoftBodyStepReport *out_report);

struct Bool softbody_solve_xpbd_volume_constraints(struct Vec3 *positions,
                                                   const double *inverse_masses,
                                                   uint32_t particle_count,
                                                   struct SoftVolumeConstraint *constraints,
                                                   uint32_t constraint_count,
                                                   double dt,
                                                   uint32_t iterations,
                                                   struct SoftBodyStepReport *out_report);

struct Bool softbody_update_velocities(const struct Vec3 *previous_positions,
                                       const struct Vec3 *current_positions,
                                       uint32_t particle_count,
                                       double dt,
                                       struct Vec3 *out_velocities,
                                       uint32_t capacity,
                                       struct SoftBodyStepReport *out_report);

double space_kepler_period(double mu, double semi_major_axis);

double space_kepler_semi_major_axis(double mu, double period);

struct Bool space_elements_to_state(struct OrbitalElements elements,
                                    double mu,
                                    struct StateVector *out_state);

struct Bool space_state_to_elements(struct StateVector state,
                                    double mu,
                                    struct OrbitalElements *out_elements);

struct Bool space_j2_acceleration(struct Vec3 position,
                                  double mu,
                                  double equatorial_radius,
                                  double j2,
                                  struct Vec3 *out_acceleration);

struct Bool space_apply_j2_force_to_body(struct WorldHandle *world,
                                         RigidBodyHandleRaw body_handle,
                                         double mu,
                                         double equatorial_radius,
                                         double j2,
                                         double mass,
                                         struct Bool wake_up,
                                         struct Vec3 *out_acceleration);

uint8_t space_apply_j2_force_to_body_flag(struct WorldHandle *world,
                                          RigidBodyHandleRaw body_handle,
                                          double mu,
                                          double equatorial_radius,
                                          double j2,
                                          double mass,
                                          struct Bool wake_up,
                                          struct Vec3 *out_acceleration);

struct Bool space_quaternion_derivative(struct Quat attitude,
                                        struct Vec3 angular_velocity,
                                        struct QuaternionDerivative *out_derivative);

struct Bool space_rigid_body_euler_derivative(struct Vec3 inertia_diag,
                                              struct Vec3 angular_velocity,
                                              struct Vec3 torque,
                                              struct RigidBodyEulerDerivative *out_derivative);

struct Bool space_cmg_exchange(struct Vec3 gimbal_axis,
                               struct Vec3 wheel_momentum,
                               double gimbal_rate,
                               struct CmgExchange *out_exchange);

struct Bool space_apply_cmg_torque_to_body(struct WorldHandle *world,
                                           RigidBodyHandleRaw body_handle,
                                           struct Vec3 gimbal_axis,
                                           struct Vec3 wheel_momentum,
                                           double gimbal_rate,
                                           struct Bool wake_up,
                                           struct CmgExchange *out_exchange);

uint8_t space_apply_cmg_torque_to_body_flag(struct WorldHandle *world,
                                            RigidBodyHandleRaw body_handle,
                                            struct Vec3 gimbal_axis,
                                            struct Vec3 wheel_momentum,
                                            double gimbal_rate,
                                            struct Bool wake_up,
                                            struct CmgExchange *out_exchange);

struct Bool space_cw_derivative(struct CwState state,
                                double mean_motion,
                                struct CwDerivative *out_derivative);

double space_lambert_time_elliptic(double mu,
                                   double semi_major_axis,
                                   double alpha,
                                   double beta,
                                   uint32_t revolutions);

struct Bool space_dh_transform(double theta,
                               double d,
                               double a,
                               double alpha,
                               struct DhTransform *out_transform);

double space_arm_first_joint_inverse(double wrist_x, double wrist_y);

double space_arm_third_joint_angle(double planar_radius,
                                   double vertical_offset,
                                   double link2,
                                   double link3,
                                   struct Bool elbow_up);

struct Bool space_manipulator_dynamics_diag(struct Vec3 mass_matrix_diag,
                                            struct Vec3 joint_acceleration,
                                            struct Vec3 coriolis,
                                            struct Vec3 gravity,
                                            struct ManipulatorDynamics *out_dynamics);

struct Bool space_solar_panel_power(double solar_flux,
                                    double area,
                                    double efficiency,
                                    double incidence_angle,
                                    double degradation,
                                    struct SolarPanelPower *out_power);

struct Bool space_thermal_balance(double absorbed_power,
                                  double internal_power,
                                  double emitted_area,
                                  double emissivity,
                                  struct ThermalBalance *out_balance);

struct Bool space_co2_mass_balance(double current_mass,
                                   double generation_rate,
                                   double removal_rate,
                                   double leakage_rate,
                                   double volume,
                                   double dt,
                                   struct Co2MassBalance *out_balance);

struct Bool space_friis_link(double transmit_power,
                             double transmit_gain,
                             double receive_gain,
                             double wavelength,
                             double range,
                             double system_loss,
                             struct FriisLink *out_link);

double space_friis_wavelength_from_frequency(double frequency);

double space_tsiolkovsky_delta_v(double specific_impulse,
                                 double standard_gravity,
                                 double initial_mass,
                                 double final_mass);

struct Bool space_hohmann_transfer(double mu,
                                   double radius1,
                                   double radius2,
                                   struct HohmannTransfer *out_transfer);

double space_atmospheric_density_scale_height(double reference_density,
                                              double altitude,
                                              double reference_altitude,
                                              double scale_height);

struct Bool space_atmospheric_drag_acceleration(struct Vec3 velocity,
                                                struct Vec3 atmosphere_velocity,
                                                double density,
                                                double drag_coefficient,
                                                double area,
                                                double mass,
                                                struct Vec3 *out_acceleration);

struct Bool space_apply_atmospheric_drag_to_body(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body_handle,
                                                 struct Vec3 atmosphere_velocity,
                                                 double density,
                                                 double drag_coefficient,
                                                 double area,
                                                 double mass,
                                                 struct Bool wake_up,
                                                 struct Vec3 *out_acceleration);

uint8_t space_apply_atmospheric_drag_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  struct Vec3 atmosphere_velocity,
                                                  double density,
                                                  double drag_coefficient,
                                                  double area,
                                                  double mass,
                                                  struct Bool wake_up,
                                                  struct Vec3 *out_acceleration);

struct Bool space_triad_attitude(struct Vec3 body_primary,
                                 struct Vec3 body_secondary,
                                 struct Vec3 reference_primary,
                                 struct Vec3 reference_secondary,
                                 struct Quat *out_attitude);

struct Bool space_ekf_predict_scalar(double state,
                                     double covariance,
                                     double nonlinear_delta,
                                     double jacobian,
                                     double process_noise,
                                     struct ScalarKalman *out_prediction);

double space_ekf_gain_scalar(double covariance,
                             double measurement_jacobian,
                             double measurement_noise);

struct Bool space_ekf_update_scalar(double predicted_state,
                                    double predicted_covariance,
                                    double measurement,
                                    double predicted_measurement,
                                    double kalman_gain,
                                    double measurement_jacobian,
                                    struct ScalarKalman *out_update);

struct Bool space_least_squares_attitude_two_vector(struct Vec3 body_primary,
                                                    struct Vec3 body_secondary,
                                                    struct Vec3 reference_primary,
                                                    struct Vec3 reference_secondary,
                                                    struct LeastSquaresAttitude *out_attitude);

struct Bool space_gnss_pseudorange(struct Vec3 receiver,
                                   struct Vec3 satellite,
                                   double receiver_clock_bias,
                                   double satellite_clock_bias,
                                   double ionosphere_delay,
                                   double troposphere_delay,
                                   struct GnssObservation *out_observation);

double space_gnss_double_difference_carrier_phase(double range_rover_sat_a,
                                                  double range_rover_sat_b,
                                                  double range_base_sat_a,
                                                  double range_base_sat_b,
                                                  double wavelength,
                                                  double ambiguity);

double space_structural_natural_frequency(double stiffness, double mass, double mode_factor);

struct Bool space_contact_force_hunt_crossley(double penetration,
                                              double penetration_rate,
                                              double stiffness,
                                              double damping,
                                              double exponent,
                                              struct ContactForceModel *out_force);

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

struct Bool space_battery_equivalent_circuit(double open_circuit_voltage,
                                             double current,
                                             double ohmic_resistance,
                                             double rc_voltage,
                                             double rc_resistance,
                                             double rc_capacitance,
                                             double capacity_coulombs,
                                             struct BatteryEquivalentCircuit *out_battery);

struct Bool space_hall_thruster_performance(double mass_flow_rate,
                                            double exhaust_velocity,
                                            double input_power,
                                            double standard_gravity,
                                            struct HallThrusterPerformance *out_performance);

struct Bool space_artificial_potential_guidance(struct Vec3 position,
                                                struct Vec3 target,
                                                struct Vec3 obstacle,
                                                double attractive_gain,
                                                double repulsive_gain,
                                                double influence_radius,
                                                struct Vec3 *out_command);

struct Bool space_debris_collision_probability(double miss_distance,
                                               double combined_radius,
                                               double sigma_radial,
                                               double sigma_intrack,
                                               struct CollisionProbability *out_probability);

struct Bool space_atomic_oxygen_erosion(double fluence,
                                        double erosion_yield,
                                        double area,
                                        double density,
                                        struct AtomicOxygenErosion *out_erosion);

struct Bool space_flexible_mode_derivative(double displacement,
                                           double velocity,
                                           double natural_frequency,
                                           double damping_ratio,
                                           double modal_force,
                                           double modal_mass,
                                           struct FlexibleModeDerivative *out_derivative);

struct Bool space_slosh_pendulum_derivative(double angle,
                                            double angular_rate,
                                            double length,
                                            double damping,
                                            double lateral_acceleration,
                                            double gravity,
                                            struct SloshPendulumDerivative *out_derivative);

struct Bool space_variational_two_body(struct Vec3 position,
                                       struct Vec3 velocity,
                                       double mu,
                                       struct VariationalState *out_derivative);

struct Bool space_single_phase_loop_heat_transfer(double mass_flow_rate,
                                                  double specific_heat,
                                                  double inlet_temperature,
                                                  double heat_input,
                                                  struct FluidLoopHeatTransfer *out_heat);

struct Bool space_radar_range_rate(struct Vec3 radar_position,
                                   struct Vec3 target_position,
                                   struct Vec3 radar_velocity,
                                   struct Vec3 target_velocity,
                                   struct RadarMeasurement *out_measurement);

struct Bool space_mass_properties_two_body(double mass1,
                                           struct Vec3 position1,
                                           struct Vec3 inertia1_diag,
                                           double mass2,
                                           struct Vec3 position2,
                                           struct Vec3 inertia2_diag,
                                           struct MassProperties *out_properties);

double space_docking_buffer_energy(double relative_speed,
                                   double reduced_mass,
                                   double stroke,
                                   double efficiency);

struct Bool space_bang_off_bang_profile(double angle,
                                        double max_acceleration,
                                        double max_rate,
                                        struct BangOffBangProfile *out_profile);

struct Bool space_solar_radiation_pressure_acceleration(struct Vec3 sun_direction,
                                                        double solar_flux,
                                                        double reflectivity,
                                                        double area,
                                                        double mass,
                                                        struct Vec3 *out_acceleration);

struct Bool space_apply_solar_radiation_pressure_to_body(struct WorldHandle *world,
                                                         RigidBodyHandleRaw body_handle,
                                                         struct Vec3 sun_direction,
                                                         double solar_flux,
                                                         double reflectivity,
                                                         double area,
                                                         double mass,
                                                         struct Bool wake_up,
                                                         struct Vec3 *out_acceleration);

uint8_t space_apply_solar_radiation_pressure_to_body_flag(struct WorldHandle *world,
                                                          RigidBodyHandleRaw body_handle,
                                                          struct Vec3 sun_direction,
                                                          double solar_flux,
                                                          double reflectivity,
                                                          double area,
                                                          double mass,
                                                          struct Bool wake_up,
                                                          struct Vec3 *out_acceleration);

struct Bool space_gravity_gradient_torque(struct Vec3 position,
                                          struct Vec3 inertia_diag,
                                          double mu,
                                          struct Vec3 *out_torque);

struct Bool space_apply_gravity_gradient_torque_to_body(struct WorldHandle *world,
                                                        RigidBodyHandleRaw body_handle,
                                                        struct Vec3 inertia_diag,
                                                        double mu,
                                                        struct Bool wake_up,
                                                        struct Vec3 *out_torque);

uint8_t space_apply_gravity_gradient_torque_to_body_flag(struct WorldHandle *world,
                                                         RigidBodyHandleRaw body_handle,
                                                         struct Vec3 inertia_diag,
                                                         double mu,
                                                         struct Bool wake_up,
                                                         struct Vec3 *out_torque);

struct Bool space_magnetic_torquer_dipole(struct Vec3 commanded_torque,
                                          struct Vec3 magnetic_field,
                                          double max_dipole,
                                          struct Vec3 *out_dipole);

struct Bool space_apply_magnetic_torquer_to_body(struct WorldHandle *world,
                                                 RigidBodyHandleRaw body_handle,
                                                 struct Vec3 commanded_torque,
                                                 struct Vec3 magnetic_field,
                                                 double max_dipole,
                                                 struct Bool wake_up,
                                                 struct Vec3 *out_dipole);

uint8_t space_apply_magnetic_torquer_to_body_flag(struct WorldHandle *world,
                                                  RigidBodyHandleRaw body_handle,
                                                  struct Vec3 commanded_torque,
                                                  struct Vec3 magnetic_field,
                                                  double max_dipole,
                                                  struct Bool wake_up,
                                                  struct Vec3 *out_dipole);

struct Bool space_cmg_robust_pseudoinverse_diag(struct Vec3 jacobian_diag,
                                                struct Vec3 desired_torque,
                                                double damping,
                                                struct CmgRobustInverse *out_inverse);

struct Bool space_sgp4_j2_secular_rates(double semi_major_axis,
                                        double eccentricity,
                                        double inclination,
                                        double mean_motion,
                                        double equatorial_radius,
                                        double j2,
                                        struct Sgp4SecularRates *out_rates);

double space_docking_glideslope_command(double range,
                                        double desired_slope,
                                        double closing_speed_limit);

double space_sagnac_phase_rate(double area, double angular_rate, double wavelength);

double space_solar_array_pd_torque(double angle_error, double rate_error, double kp, double kd);

struct Bool space_sabatier_methane_rate(double co2_molar_rate,
                                        double h2_molar_rate,
                                        double conversion,
                                        struct ChemicalReactionRate *out_rate);

struct Bool space_spe_oxygen_rate(double current,
                                  double cells,
                                  double faraday_efficiency,
                                  struct ChemicalReactionRate *out_rate);

struct Bool space_radiator_power(double area,
                                 double emissivity,
                                 double temperature,
                                 double sink_temperature,
                                 double absorbed_power,
                                 struct RadiatorPower *out_power);

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

struct Bool space_airlock_depressurization(double pressure,
                                           double ambient_pressure,
                                           double volume,
                                           double conductance,
                                           double dt,
                                           struct AirlockDepressurization *out_state);

struct Bool thermal_fourier_conduction(double hot_temperature,
                                       double cold_temperature,
                                       double conductivity,
                                       double area,
                                       double thickness,
                                       struct HeatConductionReport *out_report);

struct Bool thermal_phase_change(double temperature,
                                 double phase_temperature,
                                 double mass,
                                 double specific_heat,
                                 double latent_heat,
                                 double heat_input,
                                 struct PhaseChangeReport *out_report);

struct Bool thermal_phase_condition(double temperature,
                                    double solidus_temperature,
                                    double liquidus_temperature,
                                    struct PhaseChangeReport *out_report);

struct Bool thermal_stefan_boltzmann_radiation(double temperature,
                                               double ambient_temperature,
                                               double emissivity,
                                               double area,
                                               struct ThermalRadiationReport *out_report);

struct Bool thermal_fem_diffusion_step(const struct FemHeatNode *nodes,
                                       uint32_t node_count,
                                       const struct FemHeatEdge *edges,
                                       uint32_t edge_count,
                                       double dt,
                                       double *out_temperatures,
                                       uint32_t capacity,
                                       struct FemHeatDiffusionReport *out_report);

struct Bool thermal_stress_from_expansion(struct MaterialProperties material,
                                          double strain,
                                          double delta_temperature,
                                          struct ThermalStressReport *out_report);

struct Bool thermal_thermoelastic_stress_strain(struct MaterialProperties material,
                                                double strain_x,
                                                double strain_y,
                                                double strain_z,
                                                double delta_temperature,
                                                struct ThermoelasticReport *out_report);

struct Bool topology_simp_material(double density,
                                   struct TopologyOptimizationParams params,
                                   struct SimpMaterialReport *out_report);

double topology_simp_stiffness(double density,
                               double penalization,
                               double stiffness_min,
                               double stiffness_solid);

double topology_compliance_sensitivity(double density,
                                       double element_energy,
                                       struct TopologyOptimizationParams params);

struct Bool topology_oc_update(const double *densities,
                               const double *sensitivities,
                               uint32_t cell_count,
                               struct TopologyOptimizationParams params,
                               double *out_densities,
                               uint32_t capacity,
                               struct TopologyOptimizationReport *out_report);

struct Bool topology_density_filter_2d(const double *densities,
                                       uint32_t width,
                                       uint32_t height,
                                       double filter_radius,
                                       double *out_densities,
                                       uint32_t capacity);

struct Bool topology_density_to_voxels(const double *densities,
                                       uint32_t cell_count,
                                       double threshold,
                                       uint8_t *out_voxels,
                                       uint32_t capacity,
                                       struct DensityFieldStats *out_stats);

struct Bool topology_runtime_shape_density_step(const double *densities,
                                                const double *element_energies,
                                                uint32_t cell_count,
                                                struct TopologyOptimizationParams params,
                                                double *out_densities,
                                                uint32_t capacity,
                                                struct TopologyOptimizationReport *out_report);

struct Bool trajectory_estimate_forces(struct TrajectoryState state,
                                       struct TrajectoryEnvironment env,
                                       struct TrajectoryForceReport *out_report);

struct Bool trajectory_integrate_step(struct TrajectoryState state,
                                      struct TrajectoryEnvironment env,
                                      double dt,
                                      struct TrajectoryState *out_state,
                                      struct TrajectoryForceReport *out_report);

struct Bool trajectory_apply_forces_to_body(struct WorldHandle *world,
                                            RigidBodyHandleRaw body_handle,
                                            struct TrajectoryEnvironment env,
                                            struct Bool wake_up,
                                            struct TrajectoryForceReport *out_report);

uint8_t trajectory_apply_forces_to_body_flag(struct WorldHandle *world,
                                             RigidBodyHandleRaw body_handle,
                                             struct TrajectoryEnvironment env,
                                             struct Bool wake_up,
                                             struct TrajectoryForceReport *out_report);

struct Bool trajectory_glide_estimate(struct TrajectoryGlideState state,
                                      struct TrajectoryGlideEnvironment env,
                                      struct TrajectoryGlideReport *out_report);

struct Bool trajectory_glide_integrate_step(struct TrajectoryGlideState state,
                                            struct TrajectoryGlideEnvironment env,
                                            double dt,
                                            struct TrajectoryGlideState *out_state,
                                            struct TrajectoryGlideReport *out_report);

struct Bool transmission_gear_evaluate(double driver_angle,
                                       double driven_angle,
                                       double driver_angular_velocity,
                                       double driven_angular_velocity,
                                       struct GearConstraintDesc desc,
                                       struct GearConstraintReport *out_report);

double transmission_gear_target_angle(double driver_angle,
                                      double ratio,
                                      struct Bool opposite_direction,
                                      double phase);

struct Bool transmission_screw_evaluate(double screw_angle,
                                        double nut_translation,
                                        double screw_angular_velocity,
                                        double nut_linear_velocity,
                                        struct ScrewConstraintDesc desc,
                                        struct ScrewConstraintReport *out_report);

double transmission_screw_target_translation(double screw_angle,
                                             double lead,
                                             struct Bool right_handed,
                                             double phase);

struct Bool transmission_cycloidal_cam_evaluate(double cam_angle,
                                                double follower_displacement,
                                                double cam_angular_velocity,
                                                struct CamConstraintDesc desc,
                                                struct CamConstraintReport *out_report);

struct Bool transmission_archimedean_spiral_evaluate(double angle,
                                                     double radial_position,
                                                     double angular_velocity,
                                                     struct SpiralConstraintDesc desc,
                                                     struct SpiralConstraintReport *out_report);

double transmission_archimedean_spiral_radius(double angle,
                                              double initial_radius,
                                              double radial_pitch,
                                              double phase);

struct ColliderBuilderHandle *collider_builder_create_voxels(const uint8_t *voxels,
                                                             uint32_t size_x,
                                                             uint32_t size_y,
                                                             uint32_t size_z,
                                                             double voxel_size,
                                                             struct Vec3 origin,
                                                             struct VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxels_auto(const uint8_t *voxels,
                                                                  uint32_t size_x,
                                                                  uint32_t size_y,
                                                                  uint32_t size_z,
                                                                  double voxel_size,
                                                                  struct Vec3 origin,
                                                                  struct Bool dynamic_body);

struct VoxelBuildStats voxel_build_stats(const uint8_t *voxels,
                                         uint32_t size_x,
                                         uint32_t size_y,
                                         uint32_t size_z,
                                         double voxel_size,
                                         struct Vec3 origin,
                                         struct VoxelColliderOptions options);

struct VoxelBuildStats voxel_aabb_build_stats(struct AabbDesc aabb,
                                              double voxel_size,
                                              struct VoxelColliderOptions options);

struct VoxelBuildStats voxel_obb_build_stats(struct Obb obb,
                                             double voxel_size,
                                             struct VoxelColliderOptions options);

void voxel_aabb_build_stats_out(struct AabbDesc aabb,
                                double voxel_size,
                                struct VoxelColliderOptions options,
                                struct VoxelBuildStats *out_stats);

void voxel_obb_build_stats_out(struct Obb obb,
                               double voxel_size,
                               struct VoxelColliderOptions options,
                               struct VoxelBuildStats *out_stats);

struct ColliderBuilderHandle *collider_builder_create_voxel_aabb(struct AabbDesc aabb,
                                                                 double voxel_size,
                                                                 struct VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxel_aabb_auto(struct AabbDesc aabb,
                                                                      double voxel_size,
                                                                      struct Bool dynamic_body);

struct ColliderBuilderHandle *collider_builder_create_voxel_obb(struct Obb obb,
                                                                double voxel_size,
                                                                struct VoxelColliderOptions options);

struct ColliderBuilderHandle *collider_builder_create_voxel_obb_auto(struct Obb obb,
                                                                     double voxel_size,
                                                                     struct Bool dynamic_body);

uint32_t query_intersect_voxel_aabb(const struct WorldHandle *world,
                                    struct AabbDesc aabb,
                                    struct QueryFilterDesc filter,
                                    ColliderHandleRaw *out_handles,
                                    uint32_t capacity);

uint32_t query_intersect_voxel_aabb_count(const struct WorldHandle *world,
                                          struct AabbDesc aabb,
                                          struct QueryFilterDesc filter);

uint32_t query_intersect_voxel_obb(const struct WorldHandle *world,
                                   struct Obb obb,
                                   struct QueryFilterDesc filter,
                                   ColliderHandleRaw *out_handles,
                                   uint32_t capacity);

uint32_t query_intersect_voxel_obb_count(const struct WorldHandle *world,
                                         struct Obb obb,
                                         struct QueryFilterDesc filter);

RigidBodyHandleRaw world_insert_static_voxel_aabb(struct WorldHandle *world,
                                                  struct AabbDesc aabb,
                                                  double voxel_size,
                                                  struct VoxelColliderOptions options,
                                                  double friction,
                                                  double restitution);

RigidBodyHandleRaw world_insert_dynamic_voxel_obb(struct WorldHandle *world,
                                                  struct Obb obb,
                                                  double voxel_size,
                                                  struct VoxelColliderOptions options,
                                                  double density,
                                                  double friction,
                                                  double restitution);

struct WorldHandle *world_create(struct Vec3 gravity);

void world_destroy(struct WorldHandle *world);

void world_step(struct WorldHandle *world, double delta_seconds);

struct Bool world_set_integration_parameters(struct WorldHandle *world,
                                             double dt,
                                             uint32_t solver_iterations,
                                             uint32_t ccd_substeps);

uint32_t world_get_integration_parameters(const struct WorldHandle *world,
                                          double *out_values,
                                          uint32_t capacity);

void world_set_gravity(struct WorldHandle *world, struct Vec3 gravity);

struct Vec3 world_get_gravity(const struct WorldHandle *world);

int32_t world_get_rigid_body_set_size(const struct WorldHandle *world);

int32_t world_get_collider_set_size(const struct WorldHandle *world);

void world_get_gravity_out(const struct WorldHandle *world, struct Vec3 *out_gravity);

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
                                 struct Bool wake_up);

uint32_t world_update_body_velocities(struct WorldHandle *world,
                                      const RigidBodyHandleRaw *handles,
                                      const double *values,
                                      uint32_t count,
                                      struct Bool wake_up);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* RIGID_BODY_H */
