//! Christiaan Huygens —— 贡献目录与公式实现。
//!
//! 惠更斯原理（次波包络构造波前）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "christiaan_huygens",
    name: "Christiaan Huygens",
    birth_year: Some(1629),
    death_year: Some(1695),
    field_id: "optics",
    nationality: "Dutch",
    contribution: "Huygens' principle, wavefront construction",
    key_constants: "Huygens secondary wavelet",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_non_negative;

    /// Radius of a Huygens secondary-wavelet envelope after time `t`:
    ///
    /// ```text
    /// r = wave_speed * t
    /// ```
    ///
    /// Every point on a wavefront acts as a source of spherical secondary
    /// wavelets; their common tangent reconstructs the advanced wavefront.
    pub fn huygens_wavelet_radius(wave_speed: f64, time: f64) -> Option<f64> {
        if !finite_non_negative(wave_speed) || !finite_non_negative(time) {
            return None;
        }
        Some(wave_speed * time)
    }

    /// Advance a planar wavefront by distance `advance` given the propagation
    /// speed `wave_speed` and elapsed `time` (consistency check:
    /// `advance == wave_speed * time`):
    ///
    /// ```text
    /// advance = wave_speed * time
    /// ```
    pub fn huygens_wavefront_advance(wave_speed: f64, time: f64) -> Option<f64> {
        if !finite_non_negative(wave_speed) || !finite_non_negative(time) {
            return None;
        }
        Some(wave_speed * time)
    }
}
