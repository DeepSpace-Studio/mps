//! Hideki Yukawa —— 贡献目录与公式实现。
//!
//! 汤川势（核力的介子交换理论）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "hideki_yukawa",
    name: "Hideki Yukawa",
    birth_year: Some(1907),
    death_year: Some(1981),
    field_id: "nuclear",
    nationality: "Japanese",
    contribution: "Yukawa potential, meson-exchange nuclear force",
    key_constants: "Yukawa potential V = −(g²/4π)·e^(−mr)/r",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::{finite_non_negative, finite_positive};

    /// Yukawa (screened Coulomb) potential in natural units (ħ = c = 1):
    ///
    /// ```text
    /// V(r) = −(g² / 4π) · e^(−m·r) / r
    /// ```
    ///
    /// `g2` = squared coupling, `mediator_mass` = `m` (range 1/m), `r` = distance.
    pub fn yukawa_potential(g2: f64, mediator_mass: f64, distance: f64) -> Option<f64> {
        if !finite_non_negative(g2)
            || !finite_non_negative(mediator_mass)
            || !finite_positive(distance)
        {
            return None;
        }
        let v = -(g2 / (4.0 * std::f64::consts::PI)) * (-mediator_mass * distance).exp() / distance;
        if !v.is_finite() {
            return None;
        }
        Some(v)
    }
}
