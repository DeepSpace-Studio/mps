//! Georg Ohm —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "georg_ohm",
    name: "Georg Ohm",
    birth_year: Some(1789),
    death_year: Some(1854),
    field_id: "electromagnetism",
    nationality: "German",
    contribution: "Ohm's law; electrical resistance",
    key_constants: "Ω",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Ohm's law solved for voltage: `V = I · R`.
    #[allow(dead_code)]
    pub fn ohms_law_voltage(current: f64, resistance: f64) -> Option<f64> {
        if !current.is_finite() || !finite_non_negative(resistance) {
            return None;
        }
        Some(current * resistance)
    }

    /// Electrical conductance: `G = 1 / R` (Siemens).
    #[allow(dead_code)]
    pub fn electrical_conductance(resistance: f64) -> Option<f64> {
        if !finite_positive(resistance) {
            return None;
        }
        Some(1.0 / resistance)
    }

    /// Electrical power via Ohm's law: `P = V · I = I² · R`.
    #[allow(dead_code)]
    pub fn ohms_law_power(current: f64, resistance: f64) -> Option<f64> {
        if !finite_non_negative(current) || !finite_non_negative(resistance) {
            return None;
        }
        Some(current * current * resistance)
    }

    /// Voltage divider output across `R2` from a series `R1+R2` driven by
    /// `V_in`: `V_out = V_in · R2 / (R1 + R2)`.
    #[allow(dead_code)]
    pub fn voltage_divider(input_voltage: f64, resistance1: f64, resistance2: f64) -> Option<f64> {
        if !input_voltage.is_finite()
            || !finite_non_negative(resistance1)
            || !finite_non_negative(resistance2)
            || (resistance1 + resistance2) <= 0.0
        {
            return None;
        }
        Some(input_voltage * resistance2 / (resistance1 + resistance2))
    }
}
