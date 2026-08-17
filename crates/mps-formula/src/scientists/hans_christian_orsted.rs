//! Hans Christian Ørsted —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "hans_christian_orsted",
    name: "Hans Christian Ørsted",
    birth_year: Some(1777),
    death_year: Some(1851),
    field_id: "electromagnetism",
    nationality: "Danish",
    contribution: "Magnetic effect of electric current",
    key_constants: "B",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    const MU0: f64 = 1.256_637_061_4e-6;

    /// Magnetic field around a long straight current `I` (Ørsted observation,
    /// SI form): `B = μ0 · I / (2π · r)`.
    #[allow(dead_code)]
    pub fn magnetic_field_straight_current(current: f64, distance: f64) -> Option<f64> {
        if !current.is_finite() || !finite_positive(distance) {
            return None;
        }
        let pi = std::f64::consts::PI;
        Some(MU0 * current / (2.0 * pi * distance))
    }

    /// Magnetic (Lorentz) force on a moving charge q with velocity v moving
    /// perpendicular to a uniform magnetic field B: `F = q · v · B`.
    /// (Perpendicular case; general θ handled by caller via `|sin θ|`.)
    #[allow(dead_code)]
    pub fn magnetic_force_on_moving_charge(
        charge: f64,
        velocity: f64,
        magnetic_field: f64,
    ) -> Option<f64> {
        if !charge.is_finite() || !finite_non_negative(velocity) || !magnetic_field.is_finite() {
            return None;
        }
        Some(charge.abs() * velocity * magnetic_field.abs())
    }

    /// Torque on a magnetic dipole `m` of moment in a uniform field `B` when
    /// their angle is θ: `τ = m · B · sin θ`. Callers pass `m`, `B`, and the
    /// precomputed `sin_theta` for stability.
    #[allow(dead_code)]
    pub fn magnetic_dipole_torque(
        magnetic_moment: f64,
        magnetic_field: f64,
        sin_theta: f64,
    ) -> Option<f64> {
        if !finite_non_negative(magnetic_moment)
            || !finite_non_negative(magnetic_field)
            || !(0.0..=1.0).contains(&sin_theta)
        {
            return None;
        }
        Some(magnetic_moment * magnetic_field * sin_theta)
    }

    /// Precession (Larmor) angular frequency of a magnetic moment in field `B`:
    /// `ω = γ · B`, where `γ` is the gyromagnetic ratio.
    #[allow(dead_code)]
    pub fn orbit_precession_magnetic(gyromagnetic_ratio: f64, magnetic_field: f64) -> Option<f64> {
        if !gyromagnetic_ratio.is_finite() || !magnetic_field.is_finite() {
            return None;
        }
        Some(gyromagnetic_ratio.abs() * magnetic_field.abs())
    }
}
