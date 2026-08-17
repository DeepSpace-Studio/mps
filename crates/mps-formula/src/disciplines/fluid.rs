//! 流体力学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "fluid"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::fluid::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "fluid",
    name_zh: "流体力学",
    name_en: "Fluid Mechanics",
    parent_id: "",
    summary: "气体与液体的运动、湍流与边界层",
    key_symbols: "Re, Mach, Bernoulli",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::archimedes::formulas`
///   - `crate::scientists::claude_louis_navier::formulas`
///   - `crate::scientists::daniel_bernoulli::formulas`
///   - `crate::scientists::ernst_mach::formulas`
///   - `crate::scientists::george_stokes::formulas`
///   - `crate::scientists::osborne_reynolds::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::archimedes::formulas::archimedes_screw_lift;
    pub use crate::scientists::archimedes::formulas::buoyancy_force;
    pub use crate::scientists::archimedes::formulas::displaced_volume;
    pub use crate::scientists::archimedes::formulas::lever_balance;
    pub use crate::scientists::archimedes::formulas::specific_gravity;
    pub use crate::scientists::claude_louis_navier::formulas::atwood_number;
    pub use crate::scientists::claude_louis_navier::formulas::kelvin_helmholtz_growth_rate;
    pub use crate::scientists::claude_louis_navier::formulas::minor_loss_pressure_drop;
    pub use crate::scientists::claude_louis_navier::formulas::rayleigh_taylor_growth_rate;
    pub use crate::scientists::claude_louis_navier::formulas::water_hammer_pressure_surge;
    pub use crate::scientists::daniel_bernoulli::formulas::bernoulli_pressure;
    pub use crate::scientists::daniel_bernoulli::formulas::bernoulli_report;
    pub use crate::scientists::daniel_bernoulli::formulas::compute_surface_force;
    pub use crate::scientists::daniel_bernoulli::formulas::darcy_friction_factor;
    pub use crate::scientists::daniel_bernoulli::formulas::estimate_surface_force;
    pub use crate::scientists::daniel_bernoulli::formulas::flow_regime;
    pub use crate::scientists::daniel_bernoulli::formulas::re_n;
    pub use crate::scientists::ernst_mach::formulas::mach_angle;
    pub use crate::scientists::ernst_mach::formulas::mach_area;
    pub use crate::scientists::ernst_mach::formulas::mach_line_angle;
    pub use crate::scientists::ernst_mach::formulas::mach_number_simple;
    pub use crate::scientists::ernst_mach::formulas::supersonic_shock_angle;
    pub use crate::scientists::george_stokes::formulas::area_mach_ratio;
    pub use crate::scientists::george_stokes::formulas::bingham_stress;
    pub use crate::scientists::george_stokes::formulas::doublet_stream_function_2d;
    pub use crate::scientists::george_stokes::formulas::epsilon_equation_source;
    pub use crate::scientists::george_stokes::formulas::isentropic_density_ratio;
    pub use crate::scientists::george_stokes::formulas::isentropic_pressure_ratio;
    pub use crate::scientists::george_stokes::formulas::isentropic_temperature_ratio;
    pub use crate::scientists::george_stokes::formulas::k_epsilon_constants;
    pub use crate::scientists::george_stokes::formulas::k_epsilon_eddy_viscosity;
    pub use crate::scientists::george_stokes::formulas::k_epsilon_production;
    pub use crate::scientists::george_stokes::formulas::k_equation_source;
    pub use crate::scientists::george_stokes::formulas::normal_shock_density_ratio;
    pub use crate::scientists::george_stokes::formulas::normal_shock_downstream_mach;
    pub use crate::scientists::george_stokes::formulas::normal_shock_pressure_ratio;
    pub use crate::scientists::george_stokes::formulas::power_law_viscosity;
    pub use crate::scientists::george_stokes::formulas::prandtl_meyer_angle;
    pub use crate::scientists::george_stokes::formulas::source_potential_2d;
    pub use crate::scientists::george_stokes::formulas::turbulent_length_scale;
    pub use crate::scientists::george_stokes::formulas::turbulent_reynolds;
    pub use crate::scientists::osborne_reynolds::formulas::critical_velocity;
    pub use crate::scientists::osborne_reynolds::formulas::darcy_weisbach_pressure_drop;
    pub use crate::scientists::osborne_reynolds::formulas::reynolds_number_pipe;
    pub use crate::scientists::osborne_reynolds::formulas::turbulent_shear_stress;
}
