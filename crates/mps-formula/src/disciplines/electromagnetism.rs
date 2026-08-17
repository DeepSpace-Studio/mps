//! 电磁力学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "electromagnetism"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::electromagnetism::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "electromagnetism",
    name_zh: "电磁力学",
    name_en: "Electromagnetism",
    parent_id: "",
    summary: "电荷、电场、磁场及其统一描述 (Maxwell 方程组)",
    key_symbols: "ε0, μ0, Maxwell eq",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::andre_marie_ampere::formulas`
///   - `crate::scientists::charles_augustin_de_coulomb::formulas`
///   - `crate::scientists::georg_ohm::formulas`
///   - `crate::scientists::hans_christian_orsted::formulas`
///   - `crate::scientists::heinrich_hertz::formulas`
///   - `crate::scientists::james_clerk_maxwell::formulas`
///   - `crate::scientists::michael_faraday::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::andre_marie_ampere::formulas::ampere_circular_loop_field;
    pub use crate::scientists::andre_marie_ampere::formulas::ampere_force_between_wires;
    pub use crate::scientists::andre_marie_ampere::formulas::ampere_law_solenoid;
    pub use crate::scientists::andre_marie_ampere::formulas::magnetic_field_long_wire;
    pub use crate::scientists::charles_augustin_de_coulomb::formulas::coulomb_force_charges;
    pub use crate::scientists::charles_augustin_de_coulomb::formulas::coulomb_force_magnet_poles;
    pub use crate::scientists::charles_augustin_de_coulomb::formulas::coulomb_friction;
    pub use crate::scientists::charles_augustin_de_coulomb::formulas::rolling_resistance_torque;
    pub use crate::scientists::georg_ohm::formulas::electrical_conductance;
    pub use crate::scientists::georg_ohm::formulas::ohms_law_power;
    pub use crate::scientists::georg_ohm::formulas::ohms_law_voltage;
    pub use crate::scientists::georg_ohm::formulas::voltage_divider;
    pub use crate::scientists::hans_christian_orsted::formulas::magnetic_dipole_torque;
    pub use crate::scientists::hans_christian_orsted::formulas::magnetic_field_straight_current;
    pub use crate::scientists::hans_christian_orsted::formulas::magnetic_force_on_moving_charge;
    pub use crate::scientists::hans_christian_orsted::formulas::orbit_precession_magnetic;
    pub use crate::scientists::heinrich_hertz::formulas::acoustic_impedance;
    pub use crate::scientists::heinrich_hertz::formulas::active_sonar_echo_level;
    pub use crate::scientists::heinrich_hertz::formulas::cylindrical_spreading_loss;
    pub use crate::scientists::heinrich_hertz::formulas::doppler_shift;
    pub use crate::scientists::heinrich_hertz::formulas::eyring_rt60;
    pub use crate::scientists::heinrich_hertz::formulas::helmholtz_resonance_frequency;
    pub use crate::scientists::heinrich_hertz::formulas::maekawa_barrier_attenuation;
    pub use crate::scientists::heinrich_hertz::formulas::mass_law_tl;
    pub use crate::scientists::heinrich_hertz::formulas::passive_sonar_signal_excess;
    pub use crate::scientists::heinrich_hertz::formulas::sabine_rt60;
    pub use crate::scientists::heinrich_hertz::formulas::spherical_spreading_loss;
    pub use crate::scientists::heinrich_hertz::formulas::thorp_absorption;
    pub use crate::scientists::heinrich_hertz::formulas::transmission_coefficient;
    pub use crate::scientists::james_clerk_maxwell::formulas::biot_savart_element;
    pub use crate::scientists::james_clerk_maxwell::formulas::biot_savart_wire_segment;
    pub use crate::scientists::james_clerk_maxwell::formulas::coaxial_cutoff_frequency;
    pub use crate::scientists::james_clerk_maxwell::formulas::coaxial_impedance;
    pub use crate::scientists::james_clerk_maxwell::formulas::dipole_radiation_resistance;
    pub use crate::scientists::james_clerk_maxwell::formulas::effective_aperture;
    pub use crate::scientists::james_clerk_maxwell::formulas::far_field_distance;
    pub use crate::scientists::james_clerk_maxwell::formulas::faraday_rotation;
    pub use crate::scientists::james_clerk_maxwell::formulas::friis_power_received;
    pub use crate::scientists::james_clerk_maxwell::formulas::half_wave_dipole_directivity;
    pub use crate::scientists::james_clerk_maxwell::formulas::intrinsic_impedance;
    pub use crate::scientists::james_clerk_maxwell::formulas::phase_velocity;
    pub use crate::scientists::james_clerk_maxwell::formulas::poynting_magnitude_plane_wave;
    pub use crate::scientists::james_clerk_maxwell::formulas::poynting_vector;
    pub use crate::scientists::james_clerk_maxwell::formulas::quarter_wave_transformer;
    pub use crate::scientists::james_clerk_maxwell::formulas::rayleigh_scattering_cross_section;
    pub use crate::scientists::james_clerk_maxwell::formulas::reflection_coefficient;
    pub use crate::scientists::james_clerk_maxwell::formulas::return_loss;
    pub use crate::scientists::james_clerk_maxwell::formulas::skin_depth;
    pub use crate::scientists::james_clerk_maxwell::formulas::transmission_line_input_impedance;
    pub use crate::scientists::james_clerk_maxwell::formulas::vacuum_wavelength;
    pub use crate::scientists::james_clerk_maxwell::formulas::vswr;
    pub use crate::scientists::james_clerk_maxwell::formulas::wave_frequency;
    pub use crate::scientists::james_clerk_maxwell::formulas::wavelength_in_medium;
    pub use crate::scientists::michael_faraday::formulas::faraday_rotation as faraday_rotation_michael_faraday;
}
