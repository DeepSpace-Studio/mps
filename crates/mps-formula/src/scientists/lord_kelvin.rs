//! William Thomson (Lord Kelvin) —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "lord_kelvin",
    name: "William Thomson (Lord Kelvin)",
    birth_year: Some(1824),
    death_year: Some(1907),
    field_id: "statistical",
    nationality: "British",
    contribution: "Absolute temperature scale; thermodynamics",
    key_constants: "kelvin",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }

    /// Prandtl number: Pr = cp * mu / k

    pub fn prandtl_number(cp: f64, viscosity: f64, conductivity: f64) -> Option<f64> {
        if !finite_4(cp, viscosity, conductivity, 0.0)
            || cp <= 0.0
            || viscosity <= 0.0
            || conductivity <= 0.0
        {
            return None;
        }
        Some(cp * viscosity / conductivity)
    }
}
