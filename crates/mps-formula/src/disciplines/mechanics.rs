//! 经典力学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "mechanics"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::mechanics::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "mechanics",
    name_zh: "经典力学",
    name_en: "Classical Mechanics",
    parent_id: "",
    summary: "质点与刚体的运动规律、牛顿定律、拉格朗日/哈密顿形式",
    key_symbols: "F=ma, L, H",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::galileo_galilei::formulas`
///   - `crate::scientists::isaac_newton::formulas`
///   - `crate::scientists::joseph_louis_lagrange::formulas`
///   - `crate::scientists::leonhard_euler::formulas`
///   - `crate::scientists::william_rowan_hamilton::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::galileo_galilei::formulas::free_fall_distance;
    pub use crate::scientists::galileo_galilei::formulas::free_fall_velocity;
    pub use crate::scientists::galileo_galilei::formulas::inclined_plane_speed;
    pub use crate::scientists::galileo_galilei::formulas::pendulum_period_simple;
    pub use crate::scientists::galileo_galilei::formulas::projectile_range;
    pub use crate::scientists::isaac_newton::formulas::carlson_rd;
    pub use crate::scientists::isaac_newton::formulas::carlson_rf;
    pub use crate::scientists::isaac_newton::formulas::ellipsoid_gravity;
    pub use crate::scientists::isaac_newton::formulas::keplerian_elements;
    pub use crate::scientists::isaac_newton::formulas::normalized_legendre;
    pub use crate::scientists::isaac_newton::formulas::post_newtonian_1pn;
    pub use crate::scientists::isaac_newton::formulas::post_newtonian_2pn;
    pub use crate::scientists::isaac_newton::formulas::post_newtonian_full;
    pub use crate::scientists::isaac_newton::formulas::quadrupole_from_j2;
    pub use crate::scientists::isaac_newton::formulas::quadrupole_tensor_acceleration;
    pub use crate::scientists::isaac_newton::formulas::specific_angular_momentum;
    pub use crate::scientists::isaac_newton::formulas::specific_energy;
    pub use crate::scientists::isaac_newton::formulas::spherical_harmonics_acceleration;
    pub use crate::scientists::isaac_newton::formulas::zonal_harmonics_acceleration;
    pub use crate::scientists::joseph_louis_lagrange::formulas::adaptive_step_size;
    pub use crate::scientists::joseph_louis_lagrange::formulas::carlson_rd as carlson_rd_joseph_louis_lagrange;
    pub use crate::scientists::joseph_louis_lagrange::formulas::carlson_rf as carlson_rf_joseph_louis_lagrange;
    pub use crate::scientists::joseph_louis_lagrange::formulas::step_accepted;
    pub use crate::scientists::leonhard_euler::formulas::blasius_displacement_thickness;
    pub use crate::scientists::leonhard_euler::formulas::blasius_momentum_thickness;
    pub use crate::scientists::leonhard_euler::formulas::blasius_thickness;
    pub use crate::scientists::leonhard_euler::formulas::compute_fluid_forces;
    pub use crate::scientists::leonhard_euler::formulas::euler_buckling_load;
    pub use crate::scientists::leonhard_euler::formulas::laminar_skin_friction;
    pub use crate::scientists::leonhard_euler::formulas::navier_stokes_simplified_step;
    pub use crate::scientists::leonhard_euler::formulas::slenderness_ratio;
    pub use crate::scientists::leonhard_euler::formulas::sph_estimate_density;
    pub use crate::scientists::leonhard_euler::formulas::sph_estimate_forces;
    pub use crate::scientists::leonhard_euler::formulas::sph_poly6_kernel;
    pub use crate::scientists::leonhard_euler::formulas::sph_spiky_gradient;
    pub use crate::scientists::leonhard_euler::formulas::sph_viscosity_laplacian;
    pub use crate::scientists::leonhard_euler::formulas::turbulent_skin_friction;
    pub use crate::scientists::william_rowan_hamilton::formulas::forest_ruth8_step;
    pub use crate::scientists::william_rowan_hamilton::formulas::forest_ruth8_step_kahan;
    pub use crate::scientists::william_rowan_hamilton::formulas::leapfrog_step;
    pub use crate::scientists::william_rowan_hamilton::formulas::yoshida4_step;
}
