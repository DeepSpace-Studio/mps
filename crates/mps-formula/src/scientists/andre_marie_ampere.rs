//! André-Marie Ampère —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "andre_marie_ampere",
    name: "André-Marie Ampère",
    birth_year: Some(1775),
    death_year: Some(1836),
    field_id: "electromagnetism",
    nationality: "French",
    contribution: "Ampère's force law; electrodynamics",
    key_constants: "A",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    const MU0: f64 = 1.256_637_061_4e-6;

    /// Force per unit length between two parallel currents separated by `d`:
    /// `F/L = (μ0 / 2π) · I1 · I2 / d`. Returns N/m.
    #[allow(dead_code)]
    pub fn ampere_force_between_wires(
        current1: f64,
        current2: f64,
        separation: f64,
    ) -> Option<f64> {
        if !current1.is_finite() || !current2.is_finite() || !finite_positive(separation) {
            return None;
        }
        let pi = std::f64::consts::PI;
        Some(MU0 / (2.0 * pi) * current1 * current2 / separation)
    }

    /// Magnetic field at distance `r` from a long straight current `I`
    /// (Ampère law, free-space form): `B = μ0 · I / (2π · r)`.
    #[allow(dead_code)]
    pub fn magnetic_field_long_wire(current: f64, distance: f64) -> Option<f64> {
        if !current.is_finite() || !finite_positive(distance) {
            return None;
        }
        let pi = std::f64::consts::PI;
        Some(MU0 * current / (2.0 * pi * distance))
    }

    /// Magnetic field at the center of a circular loop of radius `R` carrying
    /// current `I`: `B = μ0 · I / (2 · R)`.
    #[allow(dead_code)]
    pub fn ampere_circular_loop_field(current: f64, radius: f64) -> Option<f64> {
        if !current.is_finite() || !finite_positive(radius) {
            return None;
        }
        Some(MU0 * current / (2.0 * radius))
    }

    /// Magnetic field inside an ideal solenoid (long coil): `B = μ0 · n · I`,
    /// where `n` is turns-per-length (turns/meter).
    #[allow(dead_code)]
    pub fn ampere_law_solenoid(turns_per_length: f64, current: f64) -> Option<f64> {
        if !finite_non_negative(turns_per_length) || !current.is_finite() {
            return None;
        }
        Some(MU0 * turns_per_length * current)
    }
}
