//! Edward Norton Lorenz —— 贡献目录与公式实现。
//!
//! 混沌理论先驱；洛伦兹吸引子与确定性非周期流为代表公式。
//! 不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "edward_lorenz",
    name: "Edward Norton Lorenz",
    birth_year: Some(1917),
    death_year: Some(2008),
    field_id: "mathphys",
    nationality: "American",
    contribution: "Lorenz attractor, deterministic chaos",
    key_constants: "Lorenz system (sigma, rho, beta)",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite;

    /// Right-hand side of the Lorenz system:
    ///
    /// ```text
    /// dx/dt = sigma * (y - x)
    /// dy/dt = x * (rho - z) - y
    /// dz/dt = x * y - beta * z
    /// ```
    ///
    /// Returns `Some((dx, dy, dz))` when all inputs are finite.
    pub fn lorenz_derivative(
        sigma: f64,
        rho: f64,
        beta: f64,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<(f64, f64, f64)> {
        if !finite(sigma) || !finite(rho) || !finite(beta) || !finite(x) || !finite(y) || !finite(z)
        {
            return None;
        }
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;
        Some((dx, dy, dz))
    }

    /// Equilibrium (fixed) points of the Lorenz system. For `rho > 1` there
    /// are two non-trivial fixed points `C± = (±√(β(rho−1)), ±√(β(rho−1)), rho−1)`
    /// in addition to the origin. Returns the non-trivial `C+` when `rho > 1`.
    pub fn lorenz_fixed_point_positive(sigma: f64, rho: f64, beta: f64) -> Option<(f64, f64, f64)> {
        if !finite(sigma) || !finite(rho) || !finite(beta) {
            return None;
        }
        if rho <= 1.0 || beta <= 0.0 {
            return None;
        }
        let s = (beta * (rho - 1.0)).sqrt();
        Some((s, s, rho - 1.0))
    }
}
