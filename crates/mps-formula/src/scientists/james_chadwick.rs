//! James Chadwick —— 贡献目录与公式实现。
//!
//! 中子质量亏损与结合能（质能等价）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "james_chadwick",
    name: "James Chadwick",
    birth_year: Some(1891),
    death_year: Some(1974),
    field_id: "nuclear",
    nationality: "British",
    contribution: "Discovery of the neutron, mass defect",
    key_constants: "mass defect Δm",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::{finite_non_negative, finite_positive};

    /// Mass defect of a nucleus: the difference between the sum of the free
    /// constituent masses and the bound nucleus mass:
    ///
    /// ```text
    /// Δm = constituent_mass_sum − nucleus_mass
    /// ```
    pub fn mass_defect(constituent_mass_sum: f64, nucleus_mass: f64) -> Option<f64> {
        if !finite_non_negative(constituent_mass_sum) || !finite_non_negative(nucleus_mass) {
            return None;
        }
        let dm = constituent_mass_sum - nucleus_mass;
        if dm < 0.0 {
            return None;
        }
        Some(dm)
    }

    /// Nuclear binding energy from a mass defect via `E = Δm·c²`:
    ///
    /// ```text
    /// E = delta_mass · c²
    /// ```
    pub fn binding_energy(delta_mass: f64, speed_of_light: f64) -> Option<f64> {
        if !finite_non_negative(delta_mass) || !finite_positive(speed_of_light) {
            return None;
        }
        Some(delta_mass * speed_of_light * speed_of_light)
    }
}
