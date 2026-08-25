//! Peter Debye (1884–1966) —— 贡献目录与公式实现。
//!
//! 荷兰裔美国物理化学家，1936 年诺贝尔化学奖。固体热学代表贡献：
//! Debye 模型（德拜温度 θ_D、T³ 低温热容定律）、Debye 屏蔽长度、
//! Debye–Hückel 电解质理论。本文件承载其名下公式实现；原
//! `mps-formula` 域模块（`thermodynamics.rs` / `ludwig_boltzmann.rs`）
//! 仅 `pub use` 重导出以保持 FFI / ABI 不变。不引入 Rapier / `WorldHandle`。
//!
//! 归属性备注：`debye_heat_capacity_low_t` 之前被错挂在
//! `ludwig_boltzmann::formulas`（玻尔兹曼与该公式无直接关联）；本轮迁至
//! 本文件后，旧路径改 `pub use` 转发，无行为变化。Debye 屏蔽长度相关
//! 计算（`plasma.rs` 中 `plasma_parameters` / `pl_debye_length`）目前仍是
//! FFI 入口且与 plasma 报告结构强耦合，未随纯公式迁移——后续若拆出纯
//! 函数版可再补入本文件。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "peter_debye",
    name: "Peter Debye",
    birth_year: Some(1884),
    death_year: Some(1966),
    field_id: "statistical",
    nationality: "Dutch-American",
    contribution: "Debye model (heat capacity); Debye shielding; electrolytes",
    key_constants: "theta_D",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Debye heat capacity: C_V = 9Nk_B (T/θ_D)³ ∫₀^{θ_D/T} x⁴ eˣ/(eˣ-1)² dx
    /// Simplified low-T limit: C_V ≈ 12π⁴/5 Nk_B (T/θ_D)³
    pub fn debye_heat_capacity_low_t(
        temperature: f64,
        debye_temperature: f64,
        n_atoms: f64,
    ) -> Option<f64> {
        if !finite_5(temperature, debye_temperature, n_atoms, 0.0, 0.0)
            || temperature <= 0.0
            || debye_temperature <= 0.0
            || n_atoms <= 0.0
        {
            return None;
        }
        let r = 8.314462618;
        let ratio = temperature / debye_temperature;
        Some(12.0 * std::f64::consts::PI.powi(4) / 5.0 * n_atoms * r * ratio.powi(3))
    }
}
