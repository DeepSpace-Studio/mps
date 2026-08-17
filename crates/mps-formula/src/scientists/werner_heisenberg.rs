//! Werner Heisenberg —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "werner_heisenberg",
    name: "Werner Heisenberg",
    birth_year: Some(1901),
    death_year: Some(1976),
    field_id: "quantum_mechanics",
    nationality: "German",
    contribution: "Uncertainty principle; matrix mechanics",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;

    /// Heisenberg uncertainty principle check: Delta_x * Delta_p >= hbar/2

    pub fn heisenberg_uncertainty_satisfied(delta_x: f64, delta_p: f64) -> Option<bool> {
        if !delta_x.is_finite() || delta_x < 0.0 || !delta_p.is_finite() || delta_p < 0.0 {
            return None;
        }
        Some(delta_x * delta_p >= REDUCED_PLANCK / 2.0 - 1.0e-15)
    }

    /// Minimum uncertainty product: hbar/2

    pub fn minimum_uncertainty_product() -> f64 {
        REDUCED_PLANCK / 2.0
    }
}
