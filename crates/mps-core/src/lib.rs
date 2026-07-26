#![allow(clippy::missing_safety_doc)]
// C ABI entry points validate raw pointers at the boundary (length/null checks
// plus `ffi_guard`), so the safe-fn-raw-pointer lint is noise here.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub extern crate rapier3d;
pub mod rapier;

pub use rapier::ffi::*;

/// Re-export the JNI-facing types and utilities that `mps-jni` needs.
pub mod jni_api {
    #[cfg(feature = "anvilkit-bridge")]
    pub use crate::rapier::anvilkit;
    pub use crate::rapier::ffi::{self, *};
    pub use crate::rapier::{
        aerodynamics as aero, bounds, bridge, collider, compat, controller, crbtree, dop, error,
        events, fluid as fl, joints, molecular as mol, neural, query, rigid_body, rtree,
        spaceflight, trajectory as traj, voxel, world,
    };
}

#[cfg(feature = "anvilkit-bridge")]
pub use rapier::ffi::AnvilKitAppHandle;
