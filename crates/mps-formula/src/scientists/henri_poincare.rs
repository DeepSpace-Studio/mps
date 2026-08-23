//! Henri Poincaré —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "henri_poincare",
    name: "Henri Poincaré",
    birth_year: Some(1854),
    death_year: Some(1912),
    field_id: "mathphys",
    nationality: "French",
    contribution: "Foundations of chaos theory; relativity precursor",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

    /// Relativistic longitudinal Doppler shift.
    pub fn relativistic_doppler_longitudinal(
        source_frequency: f64,
        relative_velocity: f64,
        approaching: bool,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !source_frequency.is_finite()
            || source_frequency <= 0.0
            || !relative_velocity.is_finite()
            || relative_velocity < 0.0
            || relative_velocity >= c
        {
            return None;
        }
        let beta = relative_velocity / c;
        let shift = ((1.0 - beta) / (1.0 + beta)).sqrt();
        Some(if approaching {
            source_frequency / shift
        } else {
            source_frequency * shift
        })
    }

    /// Relativistic transverse Doppler: f' = f/γ
    pub fn relativistic_doppler_transverse(
        source_frequency: f64,
        relative_velocity: f64,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !source_frequency.is_finite()
            || source_frequency <= 0.0
            || !relative_velocity.is_finite()
            || relative_velocity < 0.0
            || relative_velocity >= c
        {
            return None;
        }
        let gamma = 1.0 / (1.0 - (relative_velocity / c).powi(2)).sqrt();
        Some(source_frequency / gamma)
    }

    /// Relativistic total energy: E = γ·m·c².
    pub fn relativistic_total_energy(rest_mass: f64, lorentz_factor: f64) -> Option<f64> {
        if !rest_mass.is_finite()
            || rest_mass < 0.0
            || !lorentz_factor.is_finite()
            || lorentz_factor < 1.0
        {
            return None;
        }
        Some(lorentz_factor * rest_mass * SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Relativistic momentum magnitude: p = γ·m·v.
    pub fn relativistic_momentum(rest_mass: f64, speed: f64) -> Option<f64> {
        if !rest_mass.is_finite()
            || rest_mass < 0.0
            || !speed.is_finite()
            || !(0.0..SPEED_OF_LIGHT).contains(&speed)
        {
            return None;
        }
        let beta = speed / SPEED_OF_LIGHT;
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        Some(gamma * rest_mass * speed)
    }

    /// Energy–momentum relation (inverse of invariant mass): E = √(m²c⁴ + p²c²).
    pub fn relativistic_energy_from_momentum(rest_mass: f64, momentum: f64) -> Option<f64> {
        if !rest_mass.is_finite() || rest_mass < 0.0 || !momentum.is_finite() || momentum < 0.0 {
            return None;
        }
        let c2 = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
        Some((rest_mass * rest_mass * c2 * c2 + momentum * momentum * c2).sqrt())
    }

    /// Relativistic aberration of light: cos θ' = (cos θ − β) / (1 − β·cos θ).
    pub fn relativistic_aberration(cos_theta: f64, beta: f64) -> Option<f64> {
        if !cos_theta.is_finite() || !beta.is_finite() || beta.abs() >= 1.0 {
            return None;
        }
        let denom = 1.0 - beta * cos_theta;
        if denom.abs() < 1.0e-12 {
            return None;
        }
        Some(((cos_theta - beta) / denom).clamp(-1.0, 1.0))
    }

    /// Relativistic Doppler beaming (boost) factor: δ = 1 / [γ·(1 − β·cos θ)].
    pub fn relativistic_doppler_beaming_factor(beta: f64, cos_theta: f64) -> Option<f64> {
        if !beta.is_finite() || !(0.0..1.0).contains(&beta) || !cos_theta.is_finite() {
            return None;
        }
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        let denom = gamma * (1.0 - beta * cos_theta);
        if denom.abs() < 1.0e-12 {
            return None;
        }
        Some(1.0 / denom)
    }

    /// Gravitational redshift: z = 1 / sqrt(1 - R_s / r) - 1
    pub fn gravitational_redshift(mass: f64, radius: f64, g: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite() || mass <= 0.0 || !radius.is_finite() || !g.is_finite() || g <= 0.0 {
            return None;
        }
        let rs = 2.0 * g * mass / (c * c);
        if radius <= rs {
            return None;
        } // inside horizon
        Some(1.0 / (1.0 - rs / radius).sqrt() - 1.0)
    }

    /// Cosmological redshift: z = 1/a - 1
    pub fn cosmological_redshift(scale_factor: f64) -> Option<f64> {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return None;
        }
        Some(1.0 / scale_factor - 1.0)
    }

    /// Redshift from wavelengths: z = (λ_obs - λ_em) / λ_em
    pub fn redshift_from_wavelengths(observed: f64, emitted: f64) -> Option<f64> {
        if !observed.is_finite() || !emitted.is_finite() || emitted <= 0.0 {
            return None;
        }
        Some(observed / emitted - 1.0)
    }
}
