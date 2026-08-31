//! Mitchell Jay Feigenbaum —— 贡献目录与公式实现。
//!
//! 倍周期分岔通往混沌的普适常数（Feigenbaum δ）为代表公式。
//! 不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "mitchell_feigenbaum",
    name: "Mitchell Jay Feigenbaum",
    birth_year: Some(1944),
    death_year: Some(2019),
    field_id: "mathphys",
    nationality: "American",
    contribution: "Universal Feigenbaum constants of period-doubling",
    key_constants: "Feigenbaum delta ≈ 4.669",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite;

    /// The first Feigenbaum constant δ — the limiting ratio of successive
    /// period-doubling bifurcation parameter intervals:
    ///
    /// ```text
    /// δ = lim (r_n - r_{n-1}) / (r_{n+1} - r_n) ≈ 4.6692016091…
    /// ```
    pub fn feigenbaum_delta() -> f64 {
        4.669_201_609
    }

    /// Stable fixed point of the logistic map `x_{n+1} = r x_n (1 - x_n)`
    /// for `1 < r ≤ 3`: `x* = 1 - 1/r`. Returns `None` outside that range.
    pub fn logistic_fixed_point(r: f64) -> Option<f64> {
        if !finite(r) || r <= 1.0 || r > 3.0 {
            return None;
        }
        Some(1.0 - 1.0 / r)
    }

    /// Cumulative bifurcation-parameter estimate after `n` period-doubling
    /// steps, given the superstable accumulation point `r_inf` and δ:
    ///
    /// ```text
    /// r_n ≈ r_inf - (r_inf - r_1) / δ^(n-1)
    /// ```
    pub fn bifurcation_parameter(r_1: f64, r_inf: f64, n: u32) -> Option<f64> {
        if !finite(r_1) || !finite(r_inf) || n == 0 {
            return None;
        }
        let d = feigenbaum_delta();
        let denom = d.powi((n - 1) as i32);
        Some(r_inf - (r_inf - r_1) / denom)
    }
}
