//! Karl Schwarzschild —— 贡献目录与公式实现。
//!
//! 史瓦西半径（黑洞事件视界）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "karl_schwarzschild",
    name: "Karl Schwarzschild",
    birth_year: Some(1873),
    death_year: Some(1916),
    field_id: "relativity",
    nationality: "German",
    contribution: "Schwarzschild solution, event horizon",
    key_constants: "r_s = 2GM/c²",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Gravitational constant (m³·kg⁻¹·s⁻²).
    pub const G: f64 = 6.674_30e-11;
    /// Speed of light (m/s).
    pub const C: f64 = 299_792_458.0;

    /// Schwarzschild radius of a mass `m` (the radius of its event horizon):
    ///
    /// ```text
    /// r_s = 2 G m / c²
    /// ```
    pub fn schwarzschild_radius(mass: f64) -> Option<f64> {
        if !finite_positive(mass) {
            return None;
        }
        Some(2.0 * G * mass / (C * C))
    }
}
