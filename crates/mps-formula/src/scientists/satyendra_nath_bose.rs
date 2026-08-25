//! Satyendra Nath Bose —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "satyendra_nath_bose",
    name: "Satyendra Nath Bose",
    birth_year: Some(1894),
    death_year: Some(1974),
    field_id: "quantum_mechanics",
    nationality: "Indian",
    contribution: "Bose-Einstein statistics",
    key_constants: "n_B(E)",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    const BOLTZMANN_K: f64 = 1.380_649e-23;
    const PLANCK_HBAR: f64 = 1.054_571_817e-34;

    /// Bose-Einstein occupation number per mode: `n_B(E) = 1 / (exp((E − μ)/kT) − 1)`.
    /// Returns `None` when `(E − μ) ≤ 0` (compressible ground-mode divergence
    /// in the ideal-gas treatment) or when any argument is non-finite.
    #[allow(dead_code)]
    pub fn bose_einstein_distribution(
        energy: f64,
        chemical_potential: f64,
        temperature: f64,
    ) -> Option<f64> {
        if !energy.is_finite() || !chemical_potential.is_finite() || !finite_positive(temperature) {
            return None;
        }
        let x = (energy - chemical_potential) / (BOLTZMANN_K * temperature);
        if x <= 0.0 {
            return None;
        }
        let denom = x.exp() - 1.0;
        if denom <= 0.0 {
            return None;
        }
        Some(1.0 / denom)
    }

    /// Bose-Einstein condensation (BEC) critical temperature for an ideal 3D
    /// Bose gas of number density `n` and mass `m`:
    /// `T_c = (2πℏ² / m·k) · (n / ζ(3/2))^(2/3)`, with `ζ(3/2) ≈ 2.6124`.
    #[allow(dead_code)]
    pub fn bose_einstein_critical_temperature(number_density: f64, mass: f64) -> Option<f64> {
        if !finite_positive(number_density) || !finite_positive(mass) {
            return None;
        }
        let pi = std::f64::consts::PI;
        let zeta_3halves = 2.612_375_348_685_488;
        let pref = 2.0 * pi * PLANCK_HBAR * PLANCK_HBAR / (mass * BOLTZMANN_K);
        Some(pref * (number_density / zeta_3halves).powf(2.0 / 3.0))
    }

    /// Bose number density at temperature `T` and chemical potential `μ = 0`
    /// (saturated equilibrium phase): `n = (ζ(3/2) / λ_T³)` with the thermal
    /// de Broglie wavelength `λ_T = h / sqrt(2π·m·kT)`.
    #[allow(dead_code)]
    pub fn bose_number_density(temperature: f64, mass: f64) -> Option<f64> {
        if !finite_positive(temperature) || !finite_positive(mass) {
            return None;
        }
        let pi = std::f64::consts::PI;
        let planck_h = 2.0 * pi * PLANCK_HBAR;
        let lambda_t = planck_h / (2.0 * pi * mass * BOLTZMANN_K * temperature).sqrt();
        Some(2.612_375_348_685_488 / lambda_t.powi(3))
    }

    /// Phonon (acoustic mode) thermal de Broglie wavelength at temperature `T`
    /// for a phonon mode of effective mass `m_eff` (or pseudo-particle sound
    /// wavepacket model): `λ_ph = h / sqrt(2π · m_eff · kT)`.
    #[allow(dead_code)]
    pub fn phonon_thermal_wavelength(temperature: f64, effective_mass: f64) -> Option<f64> {
        if !finite_positive(temperature) || !finite_positive(effective_mass) {
            return None;
        }
        let pi = std::f64::consts::PI;
        let planck_h = 2.0 * pi * PLANCK_HBAR;
        Some(planck_h / (2.0 * pi * effective_mass * BOLTZMANN_K * temperature).sqrt())
    }
}
