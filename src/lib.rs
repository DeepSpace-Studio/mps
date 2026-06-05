#![allow(clippy::missing_safety_doc)]

mod compat;
mod abi;
mod collider;
mod controller;
mod events;
mod ffi;
mod joints;
mod query;
mod rigid_body;
mod world;

pub use ffi::{
    RcAabb, RcBodyStatus, RcBool, RcCharacterCollision, RcCharacterControllerHandle,
    RcColliderBuilderHandle, RcColliderHandle, RcCollisionEventRecord,
    RcContactForceEventRecord, RcEffectiveCharacterMovement, RcImpulseJointHandle,
    RcInteractionGroups, RcJointAxis, RcJointBuilderHandle, RcJointType,
    RcPointProjection, RcQuat, RcQueryFilterDesc, RcRayHit, RcRigidBodyBuilderHandle,
    RcRigidBodyHandle, RcShapeCastHit, RcShapeCastOptions, RcShapeDesc, RcShapeType, RcVec3,
    RcWorldHandle,
};
