//! 热力学与统计物理 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "statistical"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::statistical::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "statistical",
    name_zh: "热力学与统计物理",
    name_en: "Thermodynamics & Statistical Physics",
    parent_id: "",
    summary: "热、功、熵与大量粒子的统计行为",
    key_symbols: "k_B, S, Carnot",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::james_watt::formulas`
///   - `crate::scientists::lord_kelvin::formulas`
///   - `crate::scientists::ludwig_boltzmann::formulas`
///   - `crate::scientists::rudolf_clausius::formulas`
///   - `crate::scientists::sadi_carnot::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::albert_einstein::formulas::einstein_heat_capacity;
    pub use crate::scientists::hermann_von_helmholtz::formulas::helmholtz_free_energy;
    pub use crate::scientists::james_thomson::formulas::joule_thomson_coefficient;
    pub use crate::scientists::james_thomson::formulas::joule_thomson_inversion_temperature;
    pub use crate::scientists::james_watt::formulas::governor_speed;
    pub use crate::scientists::james_watt::formulas::horsepower_metric;
    pub use crate::scientists::james_watt::formulas::mechanical_efficiency;
    pub use crate::scientists::james_watt::formulas::rotational_work;
    pub use crate::scientists::lord_kelvin::formulas::prandtl_number;
    pub use crate::scientists::ludwig_boltzmann::formulas::brayton_efficiency;
    pub use crate::scientists::ludwig_boltzmann::formulas::carnot_efficiency;
    pub use crate::scientists::ludwig_boltzmann::formulas::carnot_refrigeration_cop;
    pub use crate::scientists::ludwig_boltzmann::formulas::clausius_clapeyron_pressure;
    pub use crate::scientists::ludwig_boltzmann::formulas::diesel_efficiency;
    pub use crate::scientists::ludwig_boltzmann::formulas::entropy_change_constant_pressure;
    pub use crate::scientists::ludwig_boltzmann::formulas::entropy_change_constant_volume;
    pub use crate::scientists::ludwig_boltzmann::formulas::heat_pump_cop;
    pub use crate::scientists::ludwig_boltzmann::formulas::ideal_gas_pressure;
    pub use crate::scientists::ludwig_boltzmann::formulas::ideal_gas_temperature;
    pub use crate::scientists::ludwig_boltzmann::formulas::ideal_gas_volume;
    pub use crate::scientists::ludwig_boltzmann::formulas::maxwell_relation_1;
    pub use crate::scientists::ludwig_boltzmann::formulas::otto_efficiency;
    pub use crate::scientists::ludwig_boltzmann::formulas::polytropic_pressure;
    pub use crate::scientists::ludwig_boltzmann::formulas::polytropic_work;
    pub use crate::scientists::ludwig_boltzmann::formulas::reynolds_number;
    pub use crate::scientists::ludwig_boltzmann::formulas::van_der_waals_critical_point;
    pub use crate::scientists::ludwig_boltzmann::formulas::van_der_waals_pressure;
    pub use crate::scientists::peter_debye::formulas::debye_heat_capacity_low_t;
    pub use crate::scientists::rudolf_clausius::formulas::clausius_clapeyron_pressure as clausius_clapeyron_pressure_rudolf_clausius;
    pub use crate::scientists::sadi_carnot::formulas::brayton_efficiency as brayton_efficiency_sadi_carnot;
    pub use crate::scientists::sadi_carnot::formulas::carnot_efficiency as carnot_efficiency_sadi_carnot;
    pub use crate::scientists::sadi_carnot::formulas::carnot_refrigeration_cop as carnot_refrigeration_cop_sadi_carnot;
    pub use crate::scientists::sadi_carnot::formulas::diesel_efficiency as diesel_efficiency_sadi_carnot;
    pub use crate::scientists::sadi_carnot::formulas::heat_pump_cop as heat_pump_cop_sadi_carnot;
    pub use crate::scientists::sadi_carnot::formulas::otto_efficiency as otto_efficiency_sadi_carnot;
    pub use crate::scientists::willard_gibbs::formulas::enthalpy;
    pub use crate::scientists::willard_gibbs::formulas::gibbs_free_energy;
}
