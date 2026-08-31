//! Edwin Hubble —— 贡献目录与公式实现。
//!
//! 哈勃定律（宇宙膨胀）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "edwin_hubble",
    name: "Edwin Hubble",
    birth_year: Some(1889),
    death_year: Some(1953),
    field_id: "astro",
    nationality: "American",
    contribution: "Hubble's law, expanding universe",
    key_constants: "v = H₀·d",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::{finite_non_negative, finite_positive};

    /// Recessional velocity from Hubble's law, given the Hubble constant `h0`
    /// (km/s/Mpc) and proper distance `d`:
    ///
    /// ```text
    /// v = H₀ · d
    /// ```
    pub fn hubble_velocity(hubble_constant: f64, distance: f64) -> Option<f64> {
        if !finite_non_negative(hubble_constant) || !finite_non_negative(distance) {
            return None;
        }
        Some(hubble_constant * distance)
    }

    /// Hubble constant inferred from a measured recessional velocity `v` and
    /// distance `d`: `H₀ = v / d`.
    pub fn hubble_constant(recessional_velocity: f64, distance: f64) -> Option<f64> {
        if !finite_non_negative(recessional_velocity) || !finite_positive(distance) {
            return None;
        }
        Some(recessional_velocity / distance)
    }
}
