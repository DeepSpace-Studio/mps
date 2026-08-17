//! Bernhard Riemann —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "bernhard_riemann",
    name: "Bernhard Riemann",
    birth_year: Some(1826),
    death_year: Some(1866),
    field_id: "mathphys",
    nationality: "German",
    contribution: "Riemann geometry; sums; zeta function",
    key_constants: "ζ",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {

    /// Riemann sum approximation of `∫ₐᵇ f(x) dx` using `n` subintervals of
    /// equal width and the midpoint rule. Caller passes `f` and the bounds.
    #[allow(dead_code)]
    pub fn riemann_sum_integral<F>(a: f64, b: f64, n: usize, f: F) -> Option<f64>
    where
        F: Fn(f64) -> f64,
    {
        if !a.is_finite() || !b.is_finite() || n == 0 {
            return None;
        }
        let h = (b - a) / n as f64;
        let mut acc = 0.0;
        let mut c = a + 0.5 * h;
        for _ in 0..n {
            let y = f(c);
            if !y.is_finite() {
                return None;
            }
            acc += y;
            c += h;
        }
        Some(acc * h)
    }

    /// Riemann-metric line element distance in flat-space identification
    /// with diagonal `g_ii` (scaled): `ds² = Σ g_i · Δx_i²`. Returns `ds`.
    /// Caller passes diagonal coefficients and the corresponding Δx list (any
    /// length).
    #[allow(dead_code)]
    pub fn riemann_metric_distance(g_diag: &[f64], dx: &[f64]) -> Option<f64> {
        if g_diag.len() != dx.len() || g_diag.is_empty() {
            return None;
        }
        let mut acc = 0.0;
        for (g, x) in g_diag.iter().zip(dx.iter()) {
            if !g.is_finite() || !x.is_finite() {
                return None;
            }
            acc += g * x * x;
        }
        if acc < 0.0 {
            return None;
        }
        Some(acc.sqrt())
    }

    /// Christoffel symbol (Levi-Civita connection) for a 2-axis diagonal
    /// metric `g_ii`: `Γ^k_ii = (1 / 2) · g^kk · ∂_k g_ii`. Returns this
    /// standard first-form given `inv_g_kk = 1/g_kk` and the partial
    /// derivative of `g_ii` with respect to coordinate `k`.
    #[allow(dead_code)]
    pub fn christoffel_gamma(inv_g_kk: f64, d_gii_dk: f64) -> Option<f64> {
        if !inv_g_kk.is_finite() || !d_gii_dk.is_finite() {
            return None;
        }
        Some(0.5 * inv_g_kk * d_gii_dk)
    }

    /// Basel-problem identity for `ζ(2) = π² / 6`. Provided both as a stable
    /// closed form (pure); returns `Some(π² / 6)` regardless of input.
    #[allow(dead_code)]
    pub fn riemann_zeta_two(_unused: f64) -> Option<f64> {
        let pi = std::f64::consts::PI;
        Some(pi * pi / 6.0)
    }
}
