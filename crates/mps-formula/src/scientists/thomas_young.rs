//! Thomas Young —— 贡献目录与公式实现。
//!
//! 杨氏双缝干涉实验证实光的波动性；双缝条纹间距与单缝包络为代表公式。
//! 不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "thomas_young",
    name: "Thomas Young",
    birth_year: Some(1773),
    death_year: Some(1829),
    field_id: "optics",
    nationality: "British",
    contribution: "Young's double-slit interference, wave nature of light",
    key_constants: "Young's fringe spacing",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;
    use std::f64::consts::PI;

    /// Fringe spacing of Young's double-slit experiment:
    ///
    /// ```text
    /// Δy = wavelength * L / slit_separation
    /// ```
    pub fn young_double_slit_fringe_spacing(
        slit_separation: f64,
        wavelength: f64,
        screen_distance: f64,
    ) -> Option<f64> {
        if !finite_positive(slit_separation)
            || !finite_positive(wavelength)
            || !finite_positive(screen_distance)
        {
            return None;
        }
        Some(wavelength * screen_distance / slit_separation)
    }

    /// Normalised intensity of Young's double-slit pattern at screen
    /// coordinate `y`, including the single-slit envelope (sinc²):
    ///
    /// ```text
    /// I(y) = cos²(π d y / (λ L)) · sinc²(π a y / (λ L))
    /// ```
    ///
    /// where `d` = slit separation, `a` = single-slit width, `L` = screen
    /// distance. Returns `I ∈ [0, 1]`; `None` on invalid input.
    pub fn young_double_slit_intensity(
        slit_separation: f64,
        slit_width: f64,
        wavelength: f64,
        screen_distance: f64,
        y: f64,
    ) -> Option<f64> {
        if !finite_positive(slit_separation)
            || !finite_positive(slit_width)
            || !finite_positive(wavelength)
            || !finite_positive(screen_distance)
            || !y.is_finite()
        {
            return None;
        }
        let beta = PI * slit_separation * y / (wavelength * screen_distance);
        let alpha = PI * slit_width * y / (wavelength * screen_distance);
        let interference = beta.cos() * beta.cos();
        let sinc = if alpha.abs() < 1.0e-12 {
            1.0
        } else {
            let s = alpha.sin() / alpha;
            s * s
        };
        Some(interference * sinc)
    }
}
