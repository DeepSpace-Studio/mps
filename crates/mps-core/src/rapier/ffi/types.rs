pub use mps_formula::ffi::types::*;

pub struct WorldHandle {
    pub inner: crate::rapier::world::PhysicsWorld,
}

#[cfg(feature = "anvilkit-bridge")]
pub struct AnvilKitAppHandle {
    pub(crate) inner: crate::rapier::anvilkit::AnvilKitAppState,
}

pub struct RigidBodyBuilderHandle {
    pub(crate) inner: rapier3d::prelude::RigidBodyBuilder,
}

pub struct ColliderBuilderHandle {
    pub(crate) inner: rapier3d::prelude::ColliderBuilder,
}

pub struct JointBuilderHandle {
    pub(crate) inner: crate::rapier::joints::JointBuilderKind,
}

pub struct CharacterControllerHandle {
    pub(crate) inner: crate::rapier::controller::CharacterControllerState,
}

pub struct RTreeHandle {
    pub(crate) inner: crate::rapier::rtree::RTreeIndex,
}

pub struct CRbTreeHandle {
    pub(crate) inner: crate::rapier::crbtree::CRbTreeIndex,
}
