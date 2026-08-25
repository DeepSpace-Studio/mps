//! Osborne Reynolds —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "osborne_reynolds",
    name: "Osborne Reynolds",
    birth_year: Some(1842),
    death_year: Some(1912),
    field_id: "fluid",
    nationality: "British",
    contribution: "Reynolds number; turbulence transition",
    key_constants: "Re",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Reynolds number for pipe flow: Re = ρ · v · D / μ.
    #[allow(dead_code)]
    pub fn reynolds_number_pipe(
        density: f64,
        velocity: f64,
        diameter: f64,
        dynamic_viscosity: f64,
    ) -> Option<f64> {
        if !finite_positive(density)
            || !finite_non_negative(velocity)
            || !finite_positive(diameter)
            || !finite_positive(dynamic_viscosity)
        {
            return None;
        }
        Some(density * velocity * diameter / dynamic_viscosity)
    }

    /// Critical velocity for transition to turbulence given Re_c (typically
    /// ~2300): v_c = Re_c · μ / (ρ · D).
    #[allow(dead_code)]
    pub fn critical_velocity(
        critical_reynolds: f64,
        dynamic_viscosity: f64,
        density: f64,
        diameter: f64,
    ) -> Option<f64> {
        if !finite_positive(critical_reynolds)
            || !finite_positive(dynamic_viscosity)
            || !finite_positive(density)
            || !finite_positive(diameter)
        {
            return None;
        }
        Some(critical_reynolds * dynamic_viscosity / (density * diameter))
    }

    /// Darcy-Weisbach pressure drop: `Δp = f · (L/D) · (ρ v² / 2)`.
    #[allow(dead_code)]
    pub fn darcy_weisbach_pressure_drop(
        friction_factor: f64,
        length: f64,
        diameter: f64,
        density: f64,
        velocity: f64,
    ) -> Option<f64> {
        if !finite_positive(friction_factor)
            || !finite_positive(length)
            || !finite_positive(diameter)
            || !finite_positive(density)
            || !finite_non_negative(velocity)
        {
            return None;
        }
        Some(friction_factor * (length / diameter) * 0.5 * density * velocity * velocity)
    }

    /// Turbulent shear stress at a wall (Reynolds-stress approximation):
    /// `τ_w = ρ · u_*²`, with `u_*` the friction velocity.
    #[allow(dead_code)]
    pub fn turbulent_shear_stress(density: f64, friction_velocity: f64) -> Option<f64> {
        if !finite_positive(density) || !finite_non_negative(friction_velocity) {
            return None;
        }
        Some(density * friction_velocity * friction_velocity)
    }
}
