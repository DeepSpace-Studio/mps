//! John William Strutt, 3rd Baron Rayleigh —— 贡献目录与公式实现。
//!
//! 瑞利散射、瑞利商与瑞利判据为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "lord_rayleigh",
    name: "John William Strutt, 3rd Baron Rayleigh",
    birth_year: Some(1842),
    death_year: Some(1919),
    field_id: "mechanics",
    nationality: "British",
    contribution: "Rayleigh scattering, Rayleigh quotient",
    key_constants: "Rayleigh scattering ∝ 1/λ⁴",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::{finite_non_negative, finite_positive};

    /// Relative Rayleigh scattering intensity at wavelength `lambda`
    /// normalised to a reference wavelength `lambda_ref`:
    ///
    /// ```text
    /// I(λ) / I(λ_ref) = (λ_ref / λ)^4
    /// ```
    ///
    /// Governs why the sky is blue (shorter wavelengths scatter more).
    pub fn rayleigh_scattering(lambda: f64, lambda_ref: f64) -> Option<f64> {
        if !finite_positive(lambda) || !finite_positive(lambda_ref) {
            return None;
        }
        Some((lambda_ref / lambda).powi(4))
    }

    /// Rayleigh quotient of a symmetric 2×2 matrix `A` w.r.t. vector `x`:
    ///
    /// ```text
    /// λ = (xᵀ A x) / (xᵀ x)
    /// ```
    ///
    /// Equals the eigenvalue for an eigenvector `x`; bounds the spectrum otherwise.
    pub fn rayleigh_quotient(
        a11: f64,
        a12: f64,
        a22: f64,
        x1: f64,
        x2: f64,
    ) -> Option<f64> {
        if !finite_non_negative(a11) || !a11.is_finite() || !a12.is_finite() || !a22.is_finite()
            || !x1.is_finite() || !x2.is_finite()
        {
            return None;
        }
        let num = a11 * x1 * x1 + 2.0 * a12 * x1 * x2 + a22 * x2 * x2;
        let den = x1 * x1 + x2 * x2;
        if den <= 0.0 {
            return None;
        }
        Some(num / den)
    }
}
