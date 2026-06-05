#![allow(clippy::missing_safety_doc)]

mod abi;
mod bounds;
mod collider;
mod compat;
mod controller;
mod crbtree;
mod dop;
mod events;
mod ffi;
mod joints;
mod neural;
mod query;
mod rigid_body;
mod rtree;
mod voxel;
mod world;

pub use ffi::{
    AabbDesc, BodyStatus, Bool, CRbTreeHandle, Capsule, CharacterCollision,
    CharacterControllerHandle, ColliderBuilderHandle, ColliderHandleRaw, CollisionEventRecord,
    ContactForceEventRecord, Cylinder, EffectiveCharacterMovement, Ellipsoid,
    ImpulseJointHandleRaw, InteractionGroupsDesc, JointAxisDesc, JointBuilderHandle, JointTypeDesc,
    KdopPreset, NeuralActivation, NeuralBoundsDesc, Obb, PointProjection, Prism, Quat,
    QueryFilterDesc, RTreeHandle, RayHit, RigidBodyBuilderHandle, RigidBodyHandleRaw, ShapeCastHit,
    ShapeCastOptionsDesc, ShapeDesc, ShapeType, Sphere, SphericalShell, Ssv, Vec3,
    VoxelColliderMode, VoxelColliderOptions, WorldHandle,
};
