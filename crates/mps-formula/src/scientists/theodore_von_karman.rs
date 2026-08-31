//! Theodore von Kármán —— 贡献目录与公式实现。
//!
//! 卡门涡街与斯特劳哈尔数为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "theodore_von_karman",
    name: "Theodore von Kármán",
    birth_year: Some(1881),
    death_year: Some(1963),
    field_id: "fluid",
    nationality: "Hungarian-American",
    contribution: "Kármán vortex street, aerodynamics",
    key_constants: "Strouhal number St = f·L/U",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Strouhal number of vortex shedding behind a bluff body:
    ///
    /// ```text
    /// St = f · L / U
    /// ```
    ///
    /// where `f` = shedding frequency, `L` = characteristic length, `U` = flow speed.
    pub fn strouhal_number(
        vortex_shedding_freq: f64,
        characteristic_length: f64,
        flow_velocity: f64,
    ) -> Option<f64> {
        if !finite_positive(vortex_shedding_freq)
            || !finite_positive(characteristic_length)
            || !finite_positive(flow_velocity)
        {
            return None;
        }
        Some(vortex_shedding_freq * characteristic_length / flow_velocity)
    }

    /// Vortex shedding frequency from the Strouhal number (inverted above):
    ///
    /// ```text
    /// f = St · U / L
    /// ```
    pub fn von_karman_vortex_shedding_freq(
        strouhal: f64,
        characteristic_length: f64,
        flow_velocity: f64,
    ) -> Option<f64> {
        if !finite_positive(strouhal)
            || !finite_positive(characteristic_length)
            || !finite_positive(flow_velocity)
        {
            return None;
        }
        Some(strouhal * flow_velocity / characteristic_length)
    }
}
