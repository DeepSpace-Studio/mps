//! Louis de Broglie —— 贡献目录与公式实现。
//!
//! 物质波（德布罗意波长）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "louis_de_broglie",
    name: "Louis de Broglie",
    birth_year: Some(1892),
    death_year: Some(1987),
    field_id: "quantum_mechanics",
    nationality: "French",
    contribution: "Matter waves, de Broglie wavelength",
    key_constants: "λ = h/p",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Planck constant (J·s).
    pub const PLANCK_H: f64 = 6.626_070_15e-34;

    /// de Broglie wavelength of a particle with momentum `p`:
    ///
    /// ```text
    /// λ = h / p
    /// ```
    pub fn de_broglie_wavelength(momentum: f64) -> Option<f64> {
        if !finite_positive(momentum) {
            return None;
        }
        Some(PLANCK_H / momentum)
    }
}
