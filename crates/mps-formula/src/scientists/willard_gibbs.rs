//! J. Willard Gibbs (1839–1903) —— 贡献目录与公式实现。
//!
//! 美国物理化学家，统计力学与化学热力学奠基人之一。以其命名 Gibbs
//! 自由能 G = H − TS = U + PV − TS（等温-等压过程自发性判据），他 1873
//! 起《关于热力学量之几何表示》等论文系统提出焓 H = U + PV 与吉布斯
//! 自由能。本文件承载其名下公式实现；原 `mps-formula` 域模块
//! （`thermodynamics.rs` / `ludwig_boltzmann.rs` 旧错挂）仅 `pub use` 重导出
//! 以保持 FFI / ABI 不变。不引入 Rapier / `WorldHandle`。
//!
//! 归属性备注：`gibbs_free_energy` 与 `enthalpy` 之前错挂在
//! `ludwig_boltzmann::formulas`；本轮迁至本文件后，旧路径改 `pub use` 转发。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "willard_gibbs",
    name: "J. Willard Gibbs",
    birth_year: Some(1839),
    death_year: Some(1903),
    field_id: "statistical",
    nationality: "American",
    contribution: "Gibbs free energy; enthalpy; statistical mechanics foundations",
    key_constants: "G=H-TS",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Enthalpy: H = U + PV
    /// Gibbs 1873 系统提出的等压热函；后人在 20 世纪将其命名为 "enthalpy"。
    pub fn enthalpy(internal_energy: f64, pressure: f64, volume: f64) -> Option<f64> {
        if !finite_5(internal_energy, pressure, volume, 0.0, 0.0) {
            return None;
        }
        Some(internal_energy + pressure * volume)
    }

    /// Gibbs free energy: G = H - TS = U + PV - TS
    /// 等温-等压过程自发性判据；`temperature` 必须 ≥ 0。
    pub fn gibbs_free_energy(
        internal_energy: f64,
        pressure: f64,
        volume: f64,
        temperature: f64,
        entropy: f64,
    ) -> Option<f64> {
        if !finite_5(internal_energy, pressure, volume, temperature, entropy) || temperature < 0.0 {
            return None;
        }
        Some(internal_energy + pressure * volume - temperature * entropy)
    }

    /// Gibbs phase rule: F = C - P + 2
    /// C = number of components, P = number of phases, F = degrees of freedom.
    pub fn gibbs_phase_rule(components: u32, phases: u32) -> Option<i32> {
        if components == 0 || phases == 0 {
            return None;
        }
        let f = components as i32 - phases as i32 + 2;
        if f < 0 {
            return None;
        }
        Some(f)
    }
}
