//! Charles-Augustin de Coulomb —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "charles_augustin_de_coulomb",
    name: "Charles-Augustin de Coulomb",
    birth_year: Some(1736),
    death_year: Some(1806),
    field_id: "electromagnetism",
    nationality: "French",
    contribution: "Coulomb's law; electrostatics; friction",
    key_constants: "k_e",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    const COULOMB_K: f64 = 8.987_551_787_368_176e9;

    /// Coulomb force between two point charges q1 and q2 separated by `r`:
    /// `F = k_e · q1 · q2 / r²`. Sign encodes attraction/repulsion: same sign
    /// gives positive (repulsive) F.
    #[allow(dead_code)]
    pub fn coulomb_force_charges(q1: f64, q2: f64, separation: f64) -> Option<f64> {
        if !q1.is_finite() || !q2.is_finite() || !finite_positive(separation) {
            return None;
        }
        Some(COULOMB_K * q1 * q2 / (separation * separation))
    }

    /// Coulomb force between two magnetic poles of strengths p1 and p2 (magnetic
    /// Coulomb's law in SI): `F = (μ0 / 4π) · p1 · p2 / r²`.
    #[allow(dead_code)]
    pub fn coulomb_force_magnet_poles(pole1: f64, pole2: f64, separation: f64) -> Option<f64> {
        if !pole1.is_finite() || !pole2.is_finite() || !finite_positive(separation) {
            return None;
        }
        let pi = std::f64::consts::PI;
        const MU0: f64 = 1.256_637_061_4e-6;
        Some(MU0 / (4.0 * pi) * pole1 * pole2 / (separation * separation))
    }

    /// Coulomb friction force `F = μ · N` for kinetic (sliding) friction.
    /// Returns a magnitude; sign is set by velocity elsewhere.
    #[allow(dead_code)]
    pub fn coulomb_friction(friction_coefficient: f64, normal_force: f64) -> Option<f64> {
        if !finite_non_negative(friction_coefficient) || !finite_non_negative(normal_force) {
            return None;
        }
        Some(friction_coefficient * normal_force)
    }

    /// Rolling-resistance torque on a wheel of radius `R` with normal load
    /// `N` and coefficient of rolling resistance `c_rr`:
    /// `τ = F_roll · R = c_rr · N · R`.
    #[allow(dead_code)]
    pub fn rolling_resistance_torque(
        rolling_coefficient: f64,
        normal_force: f64,
        radius: f64,
    ) -> Option<f64> {
        if !finite_non_negative(rolling_coefficient)
            || !finite_non_negative(normal_force)
            || !finite_non_negative(radius)
        {
            return None;
        }
        Some(rolling_coefficient * normal_force * radius)
    }
}
