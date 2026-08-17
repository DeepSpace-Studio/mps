//! 旧域模块 —— 公式已迁移到 `crate::scientists::albert_einstein::formulas::*` 中。
//! 本文件保留为 re-export 兼容层,现有调用路径 `mps_formula::cosmology::<fn>`
//! 解析到对应科学家公式定义。注意函数体与 pierre_simon_laplace 相同,
//! 显式选 albert_einstein 以避免 glob 冲突 (E0252)。

pub use crate::scientists::albert_einstein::formulas::einstein_de_sitter_age;
pub use crate::scientists::albert_einstein::formulas::friedmann_hubble_distance;
pub use crate::scientists::albert_einstein::formulas::hubble_flow_velocity;
pub use crate::scientists::albert_einstein::formulas::luminosity_distance_hubble;
