//! mps-cosmos — 太空刚体演算。
//!
//! 基于 `rapier3d-f64` 维护一套太空场景物理世界，使用 `mps-formula`
//! 提供的天体数据、引力模型与积分器施加天体重力、n-body 互引力及
//! 环境扰动力。
//!
//! 与 `mps-core` 不同，本 crate 是一个独立的太空演练器，自行持有
//! `RigidBodySet`/`PhysicsPipeline` 等后端，仅复用 `mps-formula` 的纯
//! 计算函数，不介入 `mps-core` 的 C ABI / 共享 arena / 力律登记表。
//!
//! mps-cosmos 的 C ABI 由 `ffi` 模块导出（`cosmos_*` 符号），由 cbindgen
//! 生成 `include/cosmos.h`，被 `mps-jni`（JNI）与 `test25/RigidBodyFfm`（FFM）
//! 共同消费。

// C ABI entry points validate raw pointers at the boundary (null checks plus
// `ffi_guard`), so the safe-fn-raw-pointer lint is noise here.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub extern crate rapier3d;

pub mod arena;
pub mod bodies;
pub mod ffi;
pub mod gravity;
pub mod integrator;
pub mod orbit;
pub mod perturbation;
pub mod world;

pub use world::{CosmosWorld, CosmosWorldConfig};
