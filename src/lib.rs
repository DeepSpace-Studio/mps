#![allow(clippy::missing_safety_doc)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

mod abi;
mod helper;
mod rapier;

pub use rapier::ffi::{
    AabbDesc, AeroForceReport, AeroSurface, AnvilKitAppHandle, BodyStatus, Bool, CRbTreeHandle, Capsule, CharacterCollision,
    CharacterControllerHandle, ColliderBuilderHandle, ColliderHandleRaw, CollisionEventRecord,
    ContactForceEventRecord, Cylinder, EffectiveCharacterMovement, Ellipsoid, FluidForceReport,
    FluidVolume,
    ImpulseJointHandleRaw, InteractionGroupsDesc, JointAxisDesc, JointBuilderHandle, JointTypeDesc,
    KdopPreset, NeuralActivation, NeuralBoundsDesc, Obb, PointProjection, Prism, Quat,
    QueryFilterDesc, RTreeHandle, RayHit, RigidBodyBuilderHandle, RigidBodyHandleRaw, ShapeCastHit,
    ShapeCastOptionsDesc, ShapeDesc, ShapeType, Sphere, SphericalShell, Ssv, Vec3, VoxelBuildStats,
    VoxelColliderMode, VoxelColliderOptions, WorldHandle,
};
