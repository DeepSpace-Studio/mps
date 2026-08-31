//! Aleksandr Mikhailovich Lyapunov —— 贡献目录与公式实现。
//!
//! 李雅普诺夫稳定性与 Lyapunov 指数为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "aleksandr_lyapunov",
    name: "Aleksandr Mikhailovich Lyapunov",
    birth_year: Some(1857),
    death_year: Some(1918),
    field_id: "mathphys",
    nationality: "Russian",
    contribution: "Lyapunov stability, Lyapunov exponents",
    key_constants: "Lyapunov exponent λ",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_non_negative;

    /// Maximal Lyapunov exponent from two initially separated trajectories:
    ///
    /// ```text
    /// λ = ln(d(t) / d0) / t
    /// ```
    ///
    /// `λ > 0` ⇒ chaotic divergence; `λ < 0` ⇒ convergence. Returns `None`
    /// on non-positive separations or non-finite input.
    pub fn lyapunov_exponent(
        initial_separation: f64,
        final_separation: f64,
        time: f64,
    ) -> Option<f64> {
        if !finite_non_negative(initial_separation)
            || initial_separation <= 0.0
            || !finite_non_negative(final_separation)
            || !finite_non_negative(time)
            || time <= 0.0
        {
            return None;
        }
        Some((final_separation / initial_separation).ln() / time)
    }

    /// Stability classification from a Lyapunov exponent:
    /// `0` = asymptotically stable (λ < 0), `1` = marginally stable (λ = 0),
    /// `2` = unstable (λ > 0).
    pub fn lyapunov_stability(lambda: f64) -> u8 {
        if !lambda.is_finite() {
            return 1;
        }
        if lambda < 0.0 {
            0
        } else if lambda > 0.0 {
            2
        } else {
            1
        }
    }
}
