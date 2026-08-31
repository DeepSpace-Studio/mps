//! Max Born —— 贡献目录与公式实现。
//!
//! 玻恩定则（波函数概率诠释）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "max_born",
    name: "Max Born",
    birth_year: Some(1882),
    death_year: Some(1970),
    field_id: "quantum_mechanics",
    nationality: "German",
    contribution: "Born rule, probability interpretation of ψ",
    key_constants: "|ψ|² probability density",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite;

    /// Born probability density from a complex probability amplitude
    /// `(real, imag)`:
    ///
    /// ```text
    /// P = |ψ|² = real² + imag²
    /// ```
    pub fn born_probability_density(amplitude_real: f64, amplitude_imag: f64) -> Option<f64> {
        if !finite(amplitude_real) || !finite(amplitude_imag) {
            return None;
        }
        let p = amplitude_real * amplitude_real + amplitude_imag * amplitude_imag;
        if p < 0.0 {
            return None;
        }
        Some(p)
    }
}
