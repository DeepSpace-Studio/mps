//! Augustin-Jean Fresnel —— 贡献目录与公式实现。
//!
//! 波动光学奠基人之一；菲涅耳衍射、菲涅耳波带与菲涅耳数为本文件收录的
//! 代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "augustin_fresnel",
    name: "Augustin-Jean Fresnel",
    birth_year: Some(1788),
    death_year: Some(1827),
    field_id: "optics",
    nationality: "French",
    contribution: "Fresnel diffraction, wave theory of light",
    key_constants: "Fresnel number, Fresnel zones",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::{finite_non_negative, finite_positive};

    /// Number of Fresnel half-period zones visible from an observation point
    /// at distance `z`, given wavelength `wavelength` and aperture radius
    /// `max_radius`:
    ///
    /// ```text
    /// N = floor(max_radius^2 / (wavelength * z)) + 1
    /// ```
    pub fn fresnel_zone_count(z: f64, wavelength: f64, max_radius: f64) -> Option<u32> {
        if !finite_positive(z) || !finite_positive(wavelength) || !finite_non_negative(max_radius) {
            return None;
        }
        let n = (max_radius * max_radius / (wavelength * z)).floor() + 1.0;
        if !n.is_finite() || n < 1.0 {
            return None;
        }
        Some(n as u32)
    }

    /// Fresnel number of a circular aperture:
    ///
    /// ```text
    /// N_F = aperture_radius^2 / (wavelength * distance)
    /// ```
    ///
    /// `N_F << 1` → Fraunhofer (far field); `N_F >= 1` → Fresnel (near field).
    pub fn fresnel_number(aperture_radius: f64, wavelength: f64, distance: f64) -> Option<f64> {
        if !finite_positive(aperture_radius)
            || !finite_positive(wavelength)
            || !finite_positive(distance)
        {
            return None;
        }
        Some(aperture_radius * aperture_radius / (wavelength * distance))
    }
}
