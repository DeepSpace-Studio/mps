//! 旧域模块 —— 公式已迁移到 `crate::scientists::daniel_bernoulli::formulas::*` 中。
//! 本文件保留为 re-export 兼容层,现有调用路径 `mps_formula::aerodynamics::<fn>`
//! 解析到对应科学家公式定义。

pub use crate::scientists::daniel_bernoulli::formulas::compute_surface_force;
pub use crate::scientists::daniel_bernoulli::formulas::estimate_surface_force;
