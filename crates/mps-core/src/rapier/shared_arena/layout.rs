//! `shared_arena::layout` submodule — `CommandType` enum.
//!
//! Split out of the original 1028-line `shared_arena.rs` per OPTIMIZATION.md
//! §N5.  The `SharedPhysicsArena` struct itself lives in [`super`] so every
//! sibling impl-block file can construct / destructure it directly.

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandType {
    AddForce = 0,
    AddTorque = 1,
    SetPose = 2,
    SetVelocity = 3,
    ApplyImpulse = 4,
    ApplyTorqueImpulse = 5,
    WakeUp = 6,
    Sleep = 7,
    SetRotation = 8,
    SetGravityScale = 9,
    SetLinearDamping = 10,
    SetAngularDamping = 11,
    AddForceAtPoint = 12,
}
