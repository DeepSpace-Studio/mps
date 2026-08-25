//! Galileo Galilei —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "galileo_galilei",
    name: "Galileo Galilei",
    birth_year: Some(1564),
    death_year: Some(1642),
    field_id: "mechanics",
    nationality: "Italian",
    contribution: "Kinematics; projectile motion; pendulum",
    key_constants: "Galilean relativity",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Free-fall distance under constant acceleration: `s = ½ · g · t²`
    /// (rest assumed; initial velocity zero).
    #[allow(dead_code)]
    pub fn free_fall_distance(gravity: f64, time: f64) -> Option<f64> {
        if !finite_positive(gravity) || !finite_non_negative(time) {
            return None;
        }
        Some(0.5 * gravity * time * time)
    }

    /// Free-fall velocity after time t: `v = g · t` (initial velocity zero).
    #[allow(dead_code)]
    pub fn free_fall_velocity(gravity: f64, time: f64) -> Option<f64> {
        if !finite_positive(gravity) || !finite_non_negative(time) {
            return None;
        }
        Some(gravity * time)
    }

    /// Speed gained sliding down a frictionless incline from height h:
    /// `v = sqrt(2 · g · h)`.
    #[allow(dead_code)]
    pub fn inclined_plane_speed(gravity: f64, height: f64) -> Option<f64> {
        if !finite_positive(gravity) || !finite_non_negative(height) {
            return None;
        }
        Some((2.0 * gravity * height).sqrt())
    }

    /// Projectile range for launch speed v at angle θ (radians) over flat
    /// ground with no air drag: `R = v² · sin(2θ) / g`.
    #[allow(dead_code)]
    pub fn projectile_range(launch_speed: f64, launch_angle: f64, gravity: f64) -> Option<f64> {
        if !finite_positive(launch_speed) || !launch_angle.is_finite() || !finite_positive(gravity)
        {
            return None;
        }
        Some(launch_speed * launch_speed * (2.0 * launch_angle).sin() / gravity)
    }

    /// Simple-pendulum small-angle period: `T = 2π · sqrt(L / g)`.
    /// Valid for small amplitudes (typically < ~10°).
    #[allow(dead_code)]
    pub fn pendulum_period_simple(length: f64, gravity: f64) -> Option<f64> {
        if !finite_positive(length) || !finite_positive(gravity) {
            return None;
        }
        let pi = std::f64::consts::PI;
        Some(2.0 * pi * (length / gravity).sqrt())
    }
}
