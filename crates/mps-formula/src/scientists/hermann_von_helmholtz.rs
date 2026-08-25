//! Hermann von Helmholtz (1821–1894) —— 贡献目录与公式实现。
//!
//! 德国物理学家/生理学家，能量守恒律的早期提出者之一（1847《论力之守恒》），
//! 热力学始祖之一。以其命名 Helmholtz 自由能 F = U − TS（等温-等容过程
//! 自发的判据）。本文件承载其名下公式实现；原 `mps-formula` 域模块
//! （`thermodynamics.rs` / `ludwig_boltzmann.rs` 旧错挂）仅 `pub use` 重导出
//! 以保持 FFI / ABI 不变。不引入 Rapier / `WorldHandle`。
//!
//! 归属性备注：`helmholtz_free_energy` 之前被错挂在
//! `ludwig_boltzmann::formulas`（玻尔兹曼研究与该自由能判据相关，但概念以
//! Helmholtz 1882 年命名为准）；本轮迁至本文件后，旧路径改 `pub use` 转发。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "hermann_von_helmholtz",
    name: "Hermann von Helmholtz",
    birth_year: Some(1821),
    death_year: Some(1894),
    field_id: "statistical",
    nationality: "German",
    contribution: "Energy conservation; Helmholtz free energy; thermodynamics",
    key_constants: "F=U-TS",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Helmholtz free energy: F = U - TS
    /// 等温-等容过程自发性判据。`temperature` 必须 ≥ 0（绝对零度及以上）。
    pub fn helmholtz_free_energy(
        internal_energy: f64,
        temperature: f64,
        entropy: f64,
    ) -> Option<f64> {
        if !finite_5(internal_energy, temperature, entropy, 0.0, 0.0) || temperature < 0.0 {
            return None;
        }
        Some(internal_energy - temperature * entropy)
    }
}
