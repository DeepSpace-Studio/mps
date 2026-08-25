//! 数学物理 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "mathphys"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::mathphys::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "mathphys",
    name_zh: "数学物理",
    name_en: "Mathematical Physics",
    parent_id: "",
    summary: "为物理提供数学工具的学科：傅里叶、几何、方程",
    key_symbols: "Fourier, Riemannian",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::bernhard_riemann::formulas`
///   - `crate::scientists::carl_friedrich_gauss::formulas`
///   - `crate::scientists::henri_poincare::formulas`
///   - `crate::scientists::joseph_fourier::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::bernhard_riemann::formulas::christoffel_gamma;
    pub use crate::scientists::bernhard_riemann::formulas::riemann_metric_distance;
    pub use crate::scientists::bernhard_riemann::formulas::riemann_zeta_two;
    pub use crate::scientists::carl_friedrich_gauss::formulas::normalized_legendre;
    pub use crate::scientists::henri_poincare::formulas::cosmological_redshift;
    pub use crate::scientists::henri_poincare::formulas::gravitational_redshift;
    pub use crate::scientists::henri_poincare::formulas::redshift_from_wavelengths;
    pub use crate::scientists::henri_poincare::formulas::relativistic_aberration;
    pub use crate::scientists::henri_poincare::formulas::relativistic_doppler_beaming_factor;
    pub use crate::scientists::henri_poincare::formulas::relativistic_doppler_longitudinal;
    pub use crate::scientists::henri_poincare::formulas::relativistic_doppler_transverse;
    pub use crate::scientists::henri_poincare::formulas::relativistic_energy_from_momentum;
    pub use crate::scientists::henri_poincare::formulas::relativistic_momentum;
    pub use crate::scientists::henri_poincare::formulas::relativistic_total_energy;
    pub use crate::scientists::joseph_fourier::formulas::convective_heat_flux;
    pub use crate::scientists::joseph_fourier::formulas::dittus_boelter_nusselt;
    pub use crate::scientists::joseph_fourier::formulas::heat_capacity_rate;
    pub use crate::scientists::joseph_fourier::formulas::homogeneous_void_fraction;
    pub use crate::scientists::joseph_fourier::formulas::htc_from_nusselt;
    pub use crate::scientists::joseph_fourier::formulas::lmtd_counter_flow;
    pub use crate::scientists::joseph_fourier::formulas::lmtd_parallel_flow;
    pub use crate::scientists::joseph_fourier::formulas::ntu;
    pub use crate::scientists::joseph_fourier::formulas::ntu_epsilon_counter_flow;
    pub use crate::scientists::joseph_fourier::formulas::prandtl_number;
    pub use crate::scientists::joseph_fourier::formulas::quality;
    pub use crate::scientists::joseph_fourier::formulas::view_factor_coaxial_disks;
    pub use crate::scientists::joseph_fourier::formulas::view_factor_parallel_rectangles;
    pub use crate::scientists::joseph_fourier::formulas::virial_second_coefficient;
}
