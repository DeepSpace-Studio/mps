//! Chen-Ning Yang (杨振宁) —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现。
//! 不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "chen_ning_yang",
    name: "Chen-Ning Yang",
    birth_year: Some(1922),
    death_year: None,
    field_id: "quantum_mechanics",
    nationality: "Chinese",
    contribution: "Yang–Mills non-Abelian gauge theory; parity violation; Yang–Baxter",
    key_constants: "g",
};

/// 该科学家名下的公式实现。
pub mod formulas {

    /// 普朗克常数 h = 6.62607015e-34 J·s（2019 SI 精确值）。
    pub const PLANCK: f64 = 6.626_070_15e-34;
    /// 元电荷 e = 1.602176634e-19 C（2019 SI 精确值）。
    pub const E_CHARGE: f64 = 1.602_176_634e-19;

    /// Yang–Mills non-Abelian gauge coupling strength `g`.
    ///
    /// In a non-Abelian gauge theory (Yang–Mills, 1954, with R. L. Mills) the
    /// interaction vertex carries a dimensionful/ dimensionless coupling `g`
    /// that replaces the single electric charge of electromagnetism. This is
    /// the entry point: it returns the supplied coupling (finite, non-negative)
    /// unchanged, so callers can treat `g` as the theory's single free
    /// parameter. Returns `None` for non-finite or negative `g`.
    pub fn yang_mills_coupling(coupling: f64) -> Option<f64> {
        if !coupling.is_finite() || coupling < 0.0 {
            return None;
        }
        Some(coupling)
    }

    /// Parity-violation asymmetry coefficient for longitudinally polarized
    /// electrons in weak β-decay (Wu experiment, 1957; Yang & Lee, 1956).
    ///
    /// The angular distribution of emitted electrons is
    /// `1 + α·P·cos θ`, where `P` is the polarization and the maximum
    /// asymmetry for a pure V–A interaction is `α = 1/3`.
    /// Given the polarization `p` and emission angle `theta`, this returns the
    /// asymmetry factor `α·P·cos θ` with `α = 1/3`.
    /// Returns `None` for non-finite inputs or `|p| > 1`.
    pub fn weak_parity_asymmetry(polarization: f64, theta: f64) -> Option<f64> {
        if !polarization.is_finite()
            || !theta.is_finite()
            || polarization.abs() > 1.0
            || theta.abs() > std::f64::consts::PI
        {
            return None;
        }
        let alpha = 1.0 / 3.0;
        Some(alpha * polarization * theta.cos())
    }

    /// Scalar R-matrix weight of the Yang–Baxter equation (Yang, 1967), the
    /// integrability condition for exactly-solvable 1-D many-body / lattice
    /// models. For a spectral parameter `lambda` and anisotropy `q` the
    /// alternating (trigonometric) weight is
    /// `w(λ) = (λ - q) / (λ - q⁻¹)`.
    ///
    /// Returns `None` for non-finite inputs or when the denominator vanishes
    /// (λ = q⁻¹, a pole of the R-matrix).
    pub fn yang_baxter_weight(lambda: f64, q: f64) -> Option<f64> {
        if !lambda.is_finite() || !q.is_finite() || q == 0.0 {
            return None;
        }
        let denom = lambda - q.recip();
        if denom == 0.0 || !denom.is_finite() {
            return None;
        }
        Some((lambda - q) / denom)
    }

    /// Magnetic flux quantum `Φ₀ = h / (2·e)` (≈ 2.067833848e-15 Wb), the
    /// quantized unit of magnetic flux through a superconducting ring
    /// (flux quantization, explained with N. Byers, 1961).
    pub fn superconducting_flux_quantum() -> f64 {
        PLANCK / (2.0 * E_CHARGE)
    }
}
