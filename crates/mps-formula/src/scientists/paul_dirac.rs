//! Paul Dirac —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "paul_dirac",
    name: "Paul Dirac",
    birth_year: Some(1902),
    death_year: Some(1984),
    field_id: "quantum_mechanics",
    nationality: "British",
    contribution: "Dirac equation; QFT; delta function",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    const COMPTON_C: f64 = 299_792_458.0;
    const COMPTON_M_E: f64 = 9.109_383_701_5e-31;
    pub const PLANCK: f64 = 6.62607015e-34;

    /// De Broglie wavelength: λ = h / p = h / (m·v).
    /// Shared with other quantum-mechanics founders; defined once in
    /// `crate::scientists::other::formulas`.
    pub use crate::scientists::other::formulas::de_broglie_wavelength;

    /// Compton wavelength shift: Δλ = (h / m_e c)·(1 − cos θ).
    pub fn compton_wavelength_shift(scattering_angle: f64) -> Option<f64> {
        if !scattering_angle.is_finite() {
            return None;
        }
        let compton_wavelength = PLANCK / (COMPTON_M_E * COMPTON_C);
        Some(compton_wavelength * (1.0 - scattering_angle.cos()))
    }

    /// Compton scattered wavelength: λ' = λ + Δλ.
    pub fn compton_scattered_wavelength(lambda: f64, scattering_angle: f64) -> Option<f64> {
        if !lambda.is_finite() || lambda < 0.0 || !scattering_angle.is_finite() {
            return None;
        }
        let shift = compton_wavelength_shift(scattering_angle)?;
        Some(lambda + shift)
    }

    /// Two-level Rabi oscillation excitation probability:
    /// P = [Ω² / (Ω² + δ²)] · sin²(½·√(Ω² + δ²)·t)
    /// where Ω is the (generalized) Rabi frequency and δ the detuning.
    pub fn rabi_oscillation_probability(
        rabi_frequency: f64,
        detuning: f64,
        time: f64,
    ) -> Option<f64> {
        if !rabi_frequency.is_finite()
            || rabi_frequency < 0.0
            || !detuning.is_finite()
            || !time.is_finite()
            || time < 0.0
        {
            return None;
        }
        let omega = (rabi_frequency * rabi_frequency + detuning * detuning).sqrt();
        if omega == 0.0 {
            return Some(0.0);
        }
        let p = (rabi_frequency * rabi_frequency / (omega * omega))
            * (0.5 * omega * time).sin().powi(2);
        Some(p.clamp(0.0, 1.0))
    }

    /// Clebsch–Gordan coupling overlap check.
    /// Returns 1.0 when (j1, j2, j3, m1, m2, m3) satisfy both the triangle
    /// inequality |j1−j2| ≤ j3 ≤ j1+j2 and the projection sum m1+m2 = m3;
    /// otherwise 0.0. (This is the *selection rule*, not the full CG coefficient.)
    pub fn clebsch_gordan_allowed(
        j1: f64,
        j2: f64,
        j3: f64,
        m1: f64,
        m2: f64,
        m3: f64,
    ) -> Option<f64> {
        if !j1.is_finite()
            || !j2.is_finite()
            || !j3.is_finite()
            || !m1.is_finite()
            || !m2.is_finite()
            || !m3.is_finite()
        {
            return None;
        }
        let triangle = (j1 - j2).abs() <= j3 + 1.0e-9 && j3 <= j1 + j2 + 1.0e-9;
        let msum = (m1 + m2 - m3).abs() < 1.0e-9;
        Some(if triangle && msum { 1.0 } else { 0.0 })
    }

    /// Relativistic energy–momentum relation from the Dirac equation:
    /// `E = ±√(p²·c² + m²·c⁴)`.
    ///
    /// Returns the positive-energy (particle) root `√(p²c² + m²c⁴)`.
    /// `momentum` and `rest_mass` must be finite and non-negative.
    pub fn dirac_equation_energy(momentum: f64, rest_mass: f64) -> Option<f64> {
        if !momentum.is_finite() || momentum < 0.0 || !rest_mass.is_finite() || rest_mass < 0.0 {
            return None;
        }
        let mc2 = rest_mass * COMPTON_C * COMPTON_C;
        let pc = momentum * COMPTON_C;
        Some((pc * pc + mc2 * mc2).sqrt())
    }
}
