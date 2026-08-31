//! Ernst Chladni —— 贡献目录与公式实现。
//!
//! 克拉德尼图样（驻波节线）为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "ernst_chladni",
    name: "Ernst Chladni",
    birth_year: Some(1756),
    death_year: Some(1827),
    field_id: "mechanics",
    nationality: "German",
    contribution: "Chladni figures, acoustic standing-wave nodal patterns",
    key_constants: "Chladni plate modes",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Resonant frequency of mode `(m, n)` of a square membrane of side `L`
    /// with wave speed `c`:
    ///
    /// ```text
    /// f(m, n) = (c / 2L) · √(m² + n²)
    /// ```
    ///
    /// The nodal lines of these modes trace the classic Chladni figures.
    pub fn chladni_mode_frequency(
        m: u32,
        n: u32,
        side_length: f64,
        wave_speed: f64,
    ) -> Option<f64> {
        if !finite_positive(side_length) || !finite_positive(wave_speed) {
            return None;
        }
        let mm = m as f64;
        let nn = n as f64;
        Some((wave_speed / (2.0 * side_length)) * (mm * mm + nn * nn).sqrt())
    }

    /// Number of nodal lines (excluding the boundary) for a square-plate
    /// Chladni mode `(m, n)`: `m - 1` horizontal plus `n - 1` vertical.
    pub fn chladni_node_line_count(m: u32, n: u32) -> (u32, u32) {
        (m.saturating_sub(1), n.saturating_sub(1))
    }
}
