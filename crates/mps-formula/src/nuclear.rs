//! 旧域模块 —— 公式已迁移到 `crate::scientists::{enrico_fermi, ernest_rutherford,
//! marie_curie}::formulas::*` 中。本文件保留为 re-export 兼容层,
//! 现有调用路径 `mps_formula::nuclear::<fn>` 解析到对应科学家公式定义。
//! 注意歧义函数(specific_activity / gamma_attenuation / half_value_layer)
//! 同时存在于 enrico_fermi 与 marie_curie;本 shim 显式选 enrico_fermi
//! (函数体与 marie_curie 相同)以避免 glob 冲突 (E0252)。

pub use crate::scientists::enrico_fermi::formulas::activity;
pub use crate::scientists::enrico_fermi::formulas::bateman_abundance;
pub use crate::scientists::enrico_fermi::formulas::bethe_weizsaecker_binding_energy;
pub use crate::scientists::enrico_fermi::formulas::binding_energy_per_nucleon;
pub use crate::scientists::enrico_fermi::formulas::decay_constant;
pub use crate::scientists::enrico_fermi::formulas::dt_fusion_q_value;
pub use crate::scientists::enrico_fermi::formulas::four_factor_formula;
pub use crate::scientists::enrico_fermi::formulas::gamma_attenuation;
pub use crate::scientists::enrico_fermi::formulas::half_life;
pub use crate::scientists::enrico_fermi::formulas::half_value_layer;
pub use crate::scientists::enrico_fermi::formulas::macroscopic_cross_section;
pub use crate::scientists::enrico_fermi::formulas::mean_lifetime;
pub use crate::scientists::enrico_fermi::formulas::neutron_flux_sphere;
pub use crate::scientists::enrico_fermi::formulas::reaction_rate;
pub use crate::scientists::enrico_fermi::formulas::remaining_nuclei;
pub use crate::scientists::enrico_fermi::formulas::specific_activity;
pub use crate::scientists::ernest_rutherford::formulas::atomic_mass_approx;
pub use crate::scientists::ernest_rutherford::formulas::dd_fusion_branch1_energy;
pub use crate::scientists::ernest_rutherford::formulas::dd_fusion_branch2_energy;
pub use crate::scientists::ernest_rutherford::formulas::dt_fusion_energy;
pub use crate::scientists::ernest_rutherford::formulas::reaction_q_value;
pub use crate::scientists::ernest_rutherford::formulas::u235_fission_energy;
