//! Hendrik Antoon Lorentz —— 贡献目录与公式实现。
//!
//! 洛伦兹力、洛伦兹因子与洛伦兹收缩为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "hendrik_lorentz",
    name: "Hendrik Antoon Lorentz",
    birth_year: Some(1853),
    death_year: Some(1928),
    field_id: "electromagnetism",
    nationality: "Dutch",
    contribution: "Lorentz force, Lorentz transformations",
    key_constants: "Lorentz factor γ = 1/√(1−v²/c²)",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_non_negative;

    /// Lorentz factor for an object moving at speed `v` (with `c` the speed
    /// of light):
    ///
    /// ```text
    /// γ = 1 / √(1 − v²/c²)
    /// ```
    ///
    /// Returns `None` when `v ≥ c` (or non-finite / negative inputs).
    pub fn lorentz_factor(velocity: f64, speed_of_light: f64) -> Option<f64> {
        if !finite_non_negative(velocity)
            || !finite_non_negative(speed_of_light)
            || speed_of_light <= 0.0
        {
            return None;
        }
        if velocity >= speed_of_light {
            return None;
        }
        let beta2 = velocity * velocity / (speed_of_light * speed_of_light);
        Some(1.0 / (1.0 - beta2).sqrt())
    }

    /// Length contraction: a rod of proper length `proper_length` measured in
    /// a frame where it moves with Lorentz factor `gamma` appears shortened:
    ///
    /// ```text
    /// L = proper_length / gamma
    /// ```
    pub fn lorentz_length_contraction(proper_length: f64, gamma: f64) -> Option<f64> {
        if !finite_non_negative(proper_length) || !gamma.is_finite() || gamma < 1.0 {
            return None;
        }
        Some(proper_length / gamma)
    }
}
