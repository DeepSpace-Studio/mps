//! Archimedes —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "archimedes",
    name: "Archimedes",
    birth_year: Some(-287),
    death_year: Some(-212),
    field_id: "fluid",
    nationality: "Greek",
    contribution: "Buoyancy principle; lever",
    key_constants: "Archimedes' principle",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Buoyancy force (Archimedes principle): F_b = ρ_fluid · g · V_displaced
    #[allow(dead_code)]
    pub fn buoyancy_force(fluid_density: f64, gravity: f64, displaced_volume: f64) -> Option<f64> {
        if !finite_positive(fluid_density)
            || !finite_positive(gravity)
            || !finite_positive(displaced_volume)
        {
            return None;
        }
        Some(fluid_density * gravity * displaced_volume)
    }

    /// Lever balance (law of the lever): F1 · d1 = F2 · d2.
    /// Returns the residual `F1·d1 − F2·d2` (zero when balanced).
    #[allow(dead_code)]
    pub fn lever_balance(
        effort_force: f64,
        effort_arm: f64,
        load_force: f64,
        load_arm: f64,
    ) -> Option<f64> {
        if !finite_positive(effort_force)
            || !finite_positive(effort_arm)
            || !finite_positive(load_force)
            || !finite_positive(load_arm)
        {
            return None;
        }
        Some(effort_force * effort_arm - load_force * load_arm)
    }

    /// Archimedes screw lift per revolution: V = π · (r_o² − r_i²) · pitch
    #[allow(dead_code)]
    pub fn archimedes_screw_lift(outer_radius: f64, inner_radius: f64, pitch: f64) -> Option<f64> {
        if !finite_positive(outer_radius)
            || !finite_non_negative(inner_radius)
            || !finite_positive(pitch)
            || inner_radius >= outer_radius
        {
            return None;
        }
        let pi = std::f64::consts::PI;
        Some(pi * (outer_radius * outer_radius - inner_radius * inner_radius) * pitch)
    }

    /// Displaced volume for a body submerged at a fraction `f` of its total
    /// volume (0 ≤ f ≤ 1).
    #[allow(dead_code)]
    pub fn displaced_volume(total_volume: f64, submersion_fraction: f64) -> Option<f64> {
        if !finite_positive(total_volume) || !(0.0..=1.0).contains(&submersion_fraction) {
            return None;
        }
        Some(total_volume * submersion_fraction)
    }

    /// Specific gravity (relative density): ρ_body / ρ_reference (water=1.0).
    #[allow(dead_code)]
    pub fn specific_gravity(body_density: f64, reference_density: f64) -> Option<f64> {
        if !finite_positive(body_density) || !finite_positive(reference_density) {
            return None;
        }
        Some(body_density / reference_density)
    }
}
