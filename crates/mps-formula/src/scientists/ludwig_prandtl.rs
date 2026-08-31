//! Ludwig Prandtl —— 贡献目录与公式实现。
//!
//! 边界层理论、普朗特数为代表公式。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "ludwig_prandtl",
    name: "Ludwig Prandtl",
    birth_year: Some(1875),
    death_year: Some(1953),
    field_id: "fluid",
    nationality: "German",
    contribution: "Boundary-layer theory, Prandtl number",
    key_constants: "Prandtl number Pr = ν/α",
};

/// 该科学家名下的公式实现。
pub mod formulas {
    use crate::math::finite_positive;

    /// Blasius boundary-layer thickness at downstream distance `x` for a
    /// flat plate in a stream of speed `u_inf` and kinematic viscosity `nu`:
    ///
    /// ```text
    /// δ(x) ≈ 5.0 · x / √(Re_x),   Re_x = u_inf · x / nu
    /// ```
    pub fn blasius_boundary_layer_thickness(
        x: f64,
        free_stream_velocity: f64,
        kinematic_viscosity: f64,
    ) -> Option<f64> {
        if !finite_positive(x)
            || !finite_positive(free_stream_velocity)
            || !finite_positive(kinematic_viscosity)
        {
            return None;
        }
        let re_x = free_stream_velocity * x / kinematic_viscosity;
        if !re_x.is_finite() || re_x <= 0.0 {
            return None;
        }
        Some(5.0 * x / re_x.sqrt())
    }

    /// Prandtl number — ratio of momentum diffusivity to thermal diffusivity:
    ///
    /// ```text
    /// Pr = nu / alpha
    /// ```
    pub fn prandtl_number(momentum_diffusivity: f64, thermal_diffusivity: f64) -> Option<f64> {
        if !finite_positive(momentum_diffusivity) || !finite_positive(thermal_diffusivity) {
            return None;
        }
        Some(momentum_diffusivity / thermal_diffusivity)
    }
}
