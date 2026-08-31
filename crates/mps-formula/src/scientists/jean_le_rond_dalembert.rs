//! Jean le Rond d'Alembert —— 贡献目录与公式实现。
//!
//! 达朗贝尔原理与达朗贝尔佯谬为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "jean_le_rond_dalembert",
    name: "Jean le Rond d'Alembert",
    birth_year: Some(1717),
    death_year: Some(1783),
    field_id: "fluid",
    nationality: "French",
    contribution: "d'Alembert's principle, d'Alembert's paradox",
    key_constants: "d'Alembert operator □",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_non_negative;

    /// d'Alembert's paradox: the drag force on a body moving steadily through
    /// an *inviscid* (ideal) incompressible fluid is zero. This helper returns
    /// that idealised drag so callers can compare against real (viscous) drag.
    pub fn dalembert_paradox_drag() -> f64 {
        0.0
    }

    /// d'Alembert inertial force for a body of `mass` under acceleration `a`
    /// (the fictitious force introduced to keep Newton's law valid in an
    /// accelerating frame):
    ///
    /// ```text
    /// F_inertial = -mass · a
    /// ```
    pub fn dalembert_inertial_force(mass: f64, acceleration: f64) -> Option<f64> {
        if !finite_non_negative(mass) {
            return None;
        }
        Some(-mass * acceleration)
    }
}
