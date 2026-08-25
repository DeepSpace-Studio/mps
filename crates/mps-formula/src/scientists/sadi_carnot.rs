//! Sadi Carnot —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "sadi_carnot",
    name: "Sadi Carnot",
    birth_year: Some(1796),
    death_year: Some(1832),
    field_id: "statistical",
    nationality: "French",
    contribution: "Carnot cycle; heat-engine efficiency limit",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Carnot efficiency: eta = 1 - T_cold / T_hot
    pub fn carnot_efficiency(t_hot: f64, t_cold: f64) -> Option<f64> {
        if !finite_4(t_hot, t_cold, 0.0, 0.0) || t_hot <= 0.0 || t_cold < 0.0 || t_cold >= t_hot {
            return None;
        }
        Some(1.0 - t_cold / t_hot)
    }

    /// Otto cycle efficiency: eta = 1 - 1 / r^(gamma-1)
    pub fn otto_efficiency(compression_ratio: f64, gamma: f64) -> Option<f64> {
        if !compression_ratio.is_finite()
            || compression_ratio <= 1.0
            || !gamma.is_finite()
            || gamma <= 1.0
        {
            return None;
        }
        Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0))
    }

    /// Diesel cycle efficiency
    pub fn diesel_efficiency(compression_ratio: f64, cutoff_ratio: f64, gamma: f64) -> Option<f64> {
        if !finite_4(compression_ratio, cutoff_ratio, gamma, 0.0)
            || compression_ratio <= 1.0
            || cutoff_ratio <= 1.0
            || gamma <= 1.0
        {
            return None;
        }
        let term = (cutoff_ratio.powf(gamma) - 1.0) / (gamma * (cutoff_ratio - 1.0));
        Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0) * term)
    }

    /// Brayton cycle efficiency: eta = 1 - 1 / r_p^((gamma-1)/gamma)
    pub fn brayton_efficiency(pressure_ratio: f64, gamma: f64) -> Option<f64> {
        if !pressure_ratio.is_finite()
            || pressure_ratio <= 1.0
            || !gamma.is_finite()
            || gamma <= 1.0
        {
            return None;
        }
        Some(1.0 - 1.0 / pressure_ratio.powf((gamma - 1.0) / gamma))
    }

    /// Carnot refrigeration coefficient of performance: COP = Tc / (Th - Tc)
    pub fn carnot_refrigeration_cop(t_cold: f64, t_hot: f64) -> Option<f64> {
        if !finite_5(t_cold, t_hot, 0.0, 0.0, 0.0)
            || t_cold <= 0.0
            || t_hot <= 0.0
            || t_hot <= t_cold
        {
            return None;
        }
        Some(t_cold / (t_hot - t_cold))
    }

    /// Heat pump COP: COP = Th / (Th - Tc)
    pub fn heat_pump_cop(t_cold: f64, t_hot: f64) -> Option<f64> {
        if !finite_5(t_cold, t_hot, 0.0, 0.0, 0.0)
            || t_cold <= 0.0
            || t_hot <= 0.0
            || t_hot <= t_cold
        {
            return None;
        }
        Some(t_hot / (t_hot - t_cold))
    }
}
