//! Max Planck —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "max_planck",
    name: "Max Planck",
    birth_year: Some(1858),
    death_year: Some(1947),
    field_id: "quantum_mechanics",
    nationality: "German",
    contribution: "Quantum hypothesis; E=h·ν; blackbody",
    key_constants: "h",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    const COMPTON_C: f64 = 299_792_458.0;
    const EINSTEIN_EPS0: f64 = 8.854_187_812_8e-12;
    const PI: f64 = std::f64::consts::PI;
    pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;

    /// Landau energy level (non-relativistic, spinless): E_n = (n + ½)·(q·B/m)·ħ

    pub fn landau_level(
        quantum_number: i32,
        magnetic_field: f64,
        charge: f64,
        mass: f64,
    ) -> Option<f64> {
        if quantum_number < 0
            || !magnetic_field.is_finite()
            || !charge.is_finite()
            || !mass.is_finite()
            || mass <= 0.0
        {
            return None;
        }
        let n = quantum_number as f64;
        Some((n + 0.5) * (charge * magnetic_field / mass) * REDUCED_PLANCK)
    }

    /// Einstein A (spontaneous emission) coefficient for an electric-dipole
    /// transition: A = ω³·|d|² / (3·π·ε₀·ħ·c³), with ω = 2π·f.

    pub fn einstein_a_coefficient(transition_frequency: f64, dipole_moment: f64) -> Option<f64> {
        if !transition_frequency.is_finite()
            || transition_frequency < 0.0
            || !dipole_moment.is_finite()
            || dipole_moment < 0.0
        {
            return None;
        }
        let omega = 2.0 * PI * transition_frequency;
        Some(
            omega.powi(3) * dipole_moment * dipole_moment
                / (3.0 * PI * EINSTEIN_EPS0 * REDUCED_PLANCK * COMPTON_C.powi(3)),
        )
    }

    /// Fine structure constant: α ≈ 1/137.036

    pub fn fine_structure_constant() -> f64 {
        1.0 / 137.035_999_084
    }
}
