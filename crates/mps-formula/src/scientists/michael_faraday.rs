//! Michael Faraday —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "michael_faraday",
    name: "Michael Faraday",
    birth_year: Some(1791),
    death_year: Some(1867),
    field_id: "electromagnetism",
    nationality: "British",
    contribution: "Electromagnetic induction; field concept",
    key_constants: "farad",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {

    /// Faraday rotation angle: θ = V · B · L
    /// V = Verdet constant (rad/(T·m)), B = magnetic field along path (T), L = path length (m)
    pub fn faraday_rotation(
        verdet_constant: f64,
        magnetic_field: f64,
        path_length: f64,
    ) -> Option<f64> {
        if !verdet_constant.is_finite() || !magnetic_field.is_finite() || !path_length.is_finite() {
            return None;
        }
        Some(verdet_constant * magnetic_field * path_length)
    }
}
