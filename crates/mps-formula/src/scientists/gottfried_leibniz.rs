//! Gottfried Wilhelm Leibniz —— 贡献目录与公式实现。
//!
//! 微积分（莱布尼茨级数、乘积法则）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "gottfried_leibniz",
    name: "Gottfried Wilhelm Leibniz",
    birth_year: Some(1646),
    death_year: Some(1716),
    field_id: "mathphys",
    nationality: "German",
    contribution: "Calculus (Leibniz notation), product rule",
    key_constants: "Leibniz series for π/4",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    /// Leibniz series approximation of π/4:
    ///
    /// ```text
    /// π/4 = 1 − 1/3 + 1/5 − 1/7 + … = Σ_{n=0}^{N−1} (−1)^n / (2n+1)
    /// ```
    ///
    /// Converges slowly; `terms` must be ≥ 1. Returns `4 ×` the partial sum.
    pub fn leibniz_pi_approximation(terms: u32) -> Option<f64> {
        if terms == 0 {
            return None;
        }
        let mut sum = 0.0_f64;
        let mut sign = 1.0_f64;
        for n in 0..terms {
            let denom = 2.0 * (n as f64) + 1.0;
            sum += sign / denom;
            sign = -sign;
        }
        Some(4.0 * sum)
    }

    /// Leibniz product rule: the derivative of a product `u·v` is
    ///
    /// ```text
    /// (u·v)' = u'·v + u·v'
    /// ```
    pub fn leibniz_product_rule(u_prime: f64, v: f64, u: f64, v_prime: f64) -> f64 {
        u_prime * v + u * v_prime
    }
}
