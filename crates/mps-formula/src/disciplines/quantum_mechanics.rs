//! 量子力学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "quantum_mechanics"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::quantum_mechanics::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "quantum_mechanics",
    name_zh: "量子力学",
    name_en: "Quantum Mechanics",
    parent_id: "",
    summary: "微观粒子运动规律，波函数与算符描述",
    key_symbols: "ħ, Schrödinger eq, [x,p]=iħ",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::enrico_fermi::formulas`
///   - `crate::scientists::erwin_schrodinger::formulas`
///   - `crate::scientists::max_planck::formulas`
///   - `crate::scientists::niels_bohr::formulas`
///   - `crate::scientists::paul_dirac::formulas`
///   - `crate::scientists::richard_feynman::formulas`
///   - `crate::scientists::satyendra_nath_bose::formulas`
///   - `crate::scientists::werner_heisenberg::formulas`
///   - `crate::scientists::wolfgang_pauli::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::chen_ning_yang::formulas::superconducting_flux_quantum;
    pub use crate::scientists::chen_ning_yang::formulas::weak_parity_asymmetry;
    pub use crate::scientists::chen_ning_yang::formulas::yang_baxter_weight;
    pub use crate::scientists::chen_ning_yang::formulas::yang_mills_coupling;
    pub use crate::scientists::enrico_fermi::formulas::*;
    pub use crate::scientists::erwin_schrodinger::formulas::angular_momentum_squared;
    pub use crate::scientists::erwin_schrodinger::formulas::coherent_state_alpha;
    pub use crate::scientists::erwin_schrodinger::formulas::coherent_state_photon_probability;
    pub use crate::scientists::erwin_schrodinger::formulas::free_particle_energy;
    pub use crate::scientists::erwin_schrodinger::formulas::free_particle_wave_function;
    pub use crate::scientists::erwin_schrodinger::formulas::harmonic_oscillator_energy;
    pub use crate::scientists::erwin_schrodinger::formulas::infinite_well_energy;
    pub use crate::scientists::erwin_schrodinger::formulas::infinite_well_probability_density;
    pub use crate::scientists::erwin_schrodinger::formulas::infinite_well_wave_function;
    pub use crate::scientists::erwin_schrodinger::formulas::probability_current;
    pub use crate::scientists::erwin_schrodinger::formulas::spherical_harmonic_real;
    pub use crate::scientists::erwin_schrodinger::formulas::time_evolution_phase;
    pub use crate::scientists::max_planck::formulas::einstein_a_coefficient;
    pub use crate::scientists::max_planck::formulas::fine_structure_constant;
    pub use crate::scientists::max_planck::formulas::landau_level;
    pub use crate::scientists::max_planck::formulas::planck_energy;
    pub use crate::scientists::max_planck::formulas::planck_length;
    pub use crate::scientists::max_planck::formulas::planck_mass;
    pub use crate::scientists::max_planck::formulas::planck_radiation_spectral_density;
    pub use crate::scientists::max_planck::formulas::planck_time;
    pub use crate::scientists::niels_bohr::formulas::angular_momentum_quantum;
    pub use crate::scientists::niels_bohr::formulas::bohr_magneton;
    pub use crate::scientists::niels_bohr::formulas::bohr_radius;
    pub use crate::scientists::niels_bohr::formulas::hydrogen_energy_level;
    pub use crate::scientists::niels_bohr::formulas::hydrogen_orbital_radius;
    pub use crate::scientists::niels_bohr::formulas::hydrogen_transition_wavelength;
    pub use crate::scientists::niels_bohr::formulas::rydberg_energy;
    pub use crate::scientists::other::formulas::de_broglie_wavelength;
    pub use crate::scientists::paul_dirac::formulas::clebsch_gordan_allowed;
    pub use crate::scientists::paul_dirac::formulas::compton_scattered_wavelength;
    pub use crate::scientists::paul_dirac::formulas::compton_wavelength_shift;
    pub use crate::scientists::paul_dirac::formulas::dirac_equation_energy;
    pub use crate::scientists::paul_dirac::formulas::rabi_oscillation_probability;
    pub use crate::scientists::richard_feynman::formulas::born_yukawa_cross_section;
    pub use crate::scientists::richard_feynman::formulas::degenerate_perturbation_2x2;
    pub use crate::scientists::richard_feynman::formulas::spin_orbit_energy;
    pub use crate::scientists::richard_feynman::formulas::variational_hydrogen_energy;
    pub use crate::scientists::richard_feynman::formulas::variational_hydrogen_optimal_alpha;
    pub use crate::scientists::satyendra_nath_bose::formulas::bose_einstein_critical_temperature;
    pub use crate::scientists::satyendra_nath_bose::formulas::bose_einstein_distribution;
    pub use crate::scientists::satyendra_nath_bose::formulas::bose_number_density;
    pub use crate::scientists::satyendra_nath_bose::formulas::phonon_thermal_wavelength;
    pub use crate::scientists::werner_heisenberg::formulas::heisenberg_uncertainty_satisfied;
    pub use crate::scientists::werner_heisenberg::formulas::minimum_uncertainty_product;
    pub use crate::scientists::werner_heisenberg::formulas::uncertainty_energy_time;
    pub use crate::scientists::werner_heisenberg::formulas::uncertainty_momentum;
    pub use crate::scientists::wolfgang_pauli::formulas::pauli_sigma_x;
    pub use crate::scientists::wolfgang_pauli::formulas::pauli_sigma_y;
    pub use crate::scientists::wolfgang_pauli::formulas::pauli_sigma_z;
    pub use crate::scientists::wolfgang_pauli::formulas::spin_expectation;
}
