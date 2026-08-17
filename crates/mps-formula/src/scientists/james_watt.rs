//! James Watt —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "james_watt",
    name: "James Watt",
    birth_year: Some(1736),
    death_year: Some(1819),
    field_id: "statistical",
    nationality: "Scottish",
    contribution: "Improved steam engine; horsepower concept",
    key_constants: "horsepower",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Metric horsepower: `P = (force · velocity) / 75` (one metric hp lifts
    /// 75 kg at 1 m/s, i.e. ≈ 735.49875 W). Use `mass` [kg] and `velocity` [m/s]
    /// under gravity `g` [m/s²] to express the engine's effective output force
    /// as `mass · g`.
    #[allow(dead_code)]
    pub fn horsepower_metric(force_newtons: f64, velocity: f64) -> Option<f64> {
        if !finite_non_negative(force_newtons) || !finite_non_negative(velocity) {
            return None;
        }
        Some(force_newtons * velocity / 735.49875)
    }

    /// Mechanical efficiency: `η = P_out / P_in`. Returns the unitless ratio in
    /// `[0,1]` for valid inputs.
    #[allow(dead_code)]
    pub fn mechanical_efficiency(power_out: f64, power_in: f64) -> Option<f64> {
        if !finite_non_negative(power_out) || !finite_positive(power_in) || power_out > power_in {
            return None;
        }
        Some(power_out / power_in)
    }

    /// Rotational work done by a torque: `W = τ · θ`.
    #[allow(dead_code)]
    pub fn rotational_work(torque: f64, angular_displacement: f64) -> Option<f64> {
        if !finite_non_negative(torque) || !finite_non_negative(angular_displacement) {
            return None;
        }
        Some(torque * angular_displacement)
    }

    /// Centrifugal governor equilibrium rotation rate for a conical pendulum
    /// of arm length `L` whose bob rises to height `h` below the pivot:
    /// `ω² = g / h`. Returns `ω` [rad/s]. (`0 < h ≤ L`.)
    #[allow(dead_code)]
    pub fn governor_speed(gravity: f64, height: f64) -> Option<f64> {
        if !finite_positive(gravity) || !finite_positive(height) {
            return None;
        }
        Some((gravity / height).sqrt())
    }
}
