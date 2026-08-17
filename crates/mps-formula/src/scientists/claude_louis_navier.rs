//! Claude-Louis Navier —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "claude_louis_navier",
    name: "Claude-Louis Navier",
    birth_year: Some(1785),
    death_year: Some(1836),
    field_id: "fluid",
    nationality: "French",
    contribution: "Navier-Stokes momentum equation (viscous flow)",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// KH instability growth rate for two inviscid fluids with velocity shear.

    pub fn kelvin_helmholtz_growth_rate(
        k: f64,
        rho1: f64,
        rho2: f64,
        v1: f64,
        v2: f64,
    ) -> Option<f64> {
        if !finite_5(k, rho1, rho2, v1, v2) || k <= 0.0 || rho1 <= 0.0 || rho2 <= 0.0 {
            return None;
        }
        let dv = v1 - v2;
        Some(k * (rho1 * rho2).sqrt() * dv.abs() / (rho1 + rho2))
    }

    /// RT instability growth rate: ω = sqrt(At · g · k)

    pub fn rayleigh_taylor_growth_rate(atuood_number: f64, gravity: f64, k: f64) -> Option<f64> {
        if !atuood_number.is_finite()
            || atuood_number < 0.0
            || !gravity.is_finite()
            || gravity < 0.0
            || !k.is_finite()
            || k <= 0.0
        {
            return None;
        }
        Some((atuood_number * gravity * k).sqrt())
    }

    /// Atwood number: At = (ρ₂ - ρ₁)/(ρ₂ + ρ₁)

    pub fn atwood_number(density_heavy: f64, density_light: f64) -> Option<f64> {
        if !density_heavy.is_finite()
            || density_heavy < 0.0
            || !density_light.is_finite()
            || density_light < 0.0
        {
            return None;
        }
        let sum = density_heavy + density_light;
        if sum <= 0.0 {
            return None;
        }
        Some((density_heavy - density_light) / sum)
    }

    /// Minor loss pressure drop: ΔP = K · ½ρV²

    pub fn minor_loss_pressure_drop(
        loss_coefficient: f64,
        density: f64,
        velocity: f64,
    ) -> Option<f64> {
        if !loss_coefficient.is_finite()
            || loss_coefficient < 0.0
            || !density.is_finite()
            || density < 0.0
            || !velocity.is_finite()
            || velocity < 0.0
        {
            return None;
        }
        Some(loss_coefficient * 0.5 * density * velocity * velocity)
    }

    /// Joukowsky pressure surge: ΔP = ρ · c · ΔV

    pub fn water_hammer_pressure_surge(
        density: f64,
        wave_speed: f64,
        velocity_change: f64,
    ) -> Option<f64> {
        if !density.is_finite()
            || density <= 0.0
            || !wave_speed.is_finite()
            || wave_speed <= 0.0
            || !velocity_change.is_finite()
        {
            return None;
        }
        Some(density * wave_speed * velocity_change.abs())
    }
}
