//! Rudolf Clausius —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "rudolf_clausius",
    name: "Rudolf Clausius",
    birth_year: Some(1822),
    death_year: Some(1888),
    field_id: "statistical",
    nationality: "German",
    contribution: "Second law of thermodynamics; entropy",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }

    /// Clausius-Clapeyron: ln(P2/P1) = -(L/R) * (1/T2 - 1/T1)
    pub fn clausius_clapeyron_pressure(p1: f64, t1: f64, t2: f64, latent_heat: f64) -> Option<f64> {
        if !finite_4(p1, t1, t2, latent_heat)
            || p1 <= 0.0
            || t1 <= 0.0
            || t2 <= 0.0
            || latent_heat < 0.0
        {
            return None;
        }
        Some(p1 * (-latent_heat / 8.314462618 * (1.0 / t2 - 1.0 / t1)).exp())
    }
}
