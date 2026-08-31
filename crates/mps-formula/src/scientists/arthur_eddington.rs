//! Arthur Eddington —— 贡献目录与公式实现。
//!
//! 爱丁顿光度极限（恒星辐射上限）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "arthur_eddington",
    name: "Arthur Eddington",
    birth_year: Some(1882),
    death_year: Some(1944),
    field_id: "astro",
    nationality: "British",
    contribution: "Eddington luminosity, stellar structure",
    key_constants: "L_Edd = 4π G M m_p c / σ_T",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Gravitational constant (m³·kg⁻¹·s⁻²).
    pub const G: f64 = 6.674_30e-11;
    /// Speed of light (m/s).
    pub const C: f64 = 299_792_458.0;
    /// Proton mass (kg).
    pub const PROTON_MASS: f64 = 1.672_621_9e-27;
    /// Thomson cross-section (m²).
    pub const THOMSON_SIGMA: f64 = 6.652_458_7e-29;

    /// Eddington luminosity — the maximum luminosity a body of mass `m` can
    /// radiate while its outer layers remain in hydrostatic equilibrium against
    /// outward radiation pressure:
    ///
    /// ```text
    /// L_Edd = 4π G M m_p c / σ_T
    /// ```
    pub fn eddington_luminosity(mass: f64) -> Option<f64> {
        if !finite_positive(mass) {
            return None;
        }
        let l = 4.0 * std::f64::consts::PI * G * mass * PROTON_MASS * C / THOMSON_SIGMA;
        if !l.is_finite() {
            return None;
        }
        Some(l)
    }
}
