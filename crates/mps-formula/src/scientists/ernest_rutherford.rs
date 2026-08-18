//! Ernest Rutherford —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "ernest_rutherford",
    name: "Ernest Rutherford",
    birth_year: Some(1871),
    death_year: Some(1937),
    field_id: "nuclear",
    nationality: "NZ/British",
    contribution: "Nuclear model of the atom; radioactivity",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {

    /// Q-value of a nuclear reaction: Q = (M_initial - M_final) · c²  (MeV)

    pub fn reaction_q_value(initial_mass_u: f64, final_mass_u: f64) -> Option<f64> {
        if !initial_mass_u.is_finite()
            || initial_mass_u <= 0.0
            || !final_mass_u.is_finite()
            || final_mass_u <= 0.0
        {
            return None;
        }
        // 1 u = 931.494 MeV/c²
        Some((initial_mass_u - final_mass_u) * 931.494)
    }

    /// D-T fusion energy release: ²H + ³H → ⁴He + n  (approx 17.6 MeV)

    pub fn dt_fusion_energy() -> f64 {
        17.6
    }

    /// D-D fusion branch 1: ²H + ²H → ³H + p  (approx 4.0 MeV)

    pub fn dd_fusion_branch1_energy() -> f64 {
        4.0
    }

    /// D-D fusion branch 2: ²H + ²H → ³He + n  (approx 3.3 MeV)

    pub fn dd_fusion_branch2_energy() -> f64 {
        3.3
    }

    /// ²³⁵U fission energy (approx 200 MeV per fission, including neutrons)

    pub fn u235_fission_energy() -> f64 {
        200.0
    }

    /// Atomic mass from mass number: m ≈ A · u  (with binding energy correction)
    /// Returns mass in atomic mass units (u).

    pub fn atomic_mass_approx(mass_number: f64, binding_energy_mev: f64) -> Option<f64> {
        if !mass_number.is_finite() || mass_number <= 0.0 || !binding_energy_mev.is_finite() {
            return None;
        }
        Some(mass_number - binding_energy_mev / 931.494)
    }
}
