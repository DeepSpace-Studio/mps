//! Max Planck —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "max_planck",
    name: "Max Planck",
    birth_year: Some(1858),
    death_year: Some(1947),
    field_id: "quantum_mechanics",
    nationality: "German",
    contribution: "Quantum hypothesis; E=h·ν; blackbody",
    key_constants: "h",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    const COMPTON_C: f64 = 299_792_458.0;
    const EINSTEIN_EPS0: f64 = 8.854_187_812_8e-12;
    const PI: f64 = std::f64::consts::PI;
    pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;

    /// Landau energy level (non-relativistic, spinless): E_n = (n + ½)·(q·B/m)·ħ

    pub fn landau_level(
        quantum_number: i32,
        magnetic_field: f64,
        charge: f64,
        mass: f64,
    ) -> Option<f64> {
        if quantum_number < 0
            || !magnetic_field.is_finite()
            || !charge.is_finite()
            || !mass.is_finite()
            || mass <= 0.0
        {
            return None;
        }
        let n = quantum_number as f64;
        Some((n + 0.5) * (charge * magnetic_field / mass) * REDUCED_PLANCK)
    }

    /// Einstein A (spontaneous emission) coefficient for an electric-dipole
    /// transition: A = ω³·|d|² / (3·π·ε₀·ħ·c³), with ω = 2π·f.

    pub fn einstein_a_coefficient(transition_frequency: f64, dipole_moment: f64) -> Option<f64> {
        if !transition_frequency.is_finite()
            || transition_frequency < 0.0
            || !dipole_moment.is_finite()
            || dipole_moment < 0.0
        {
            return None;
        }
        let omega = 2.0 * PI * transition_frequency;
        Some(
            omega.powi(3) * dipole_moment * dipole_moment
                / (3.0 * PI * EINSTEIN_EPS0 * REDUCED_PLANCK * COMPTON_C.powi(3)),
        )
    }

    /// Fine structure constant: α ≈ 1/137.036

    pub fn fine_structure_constant() -> f64 {
        1.0 / 137.035_999_084
    }

    // ----- Planck's own core contributions (blackbody & natural units) -----

    /// Planck constant h = ħ·2π = 6.62607015e-34 J·s (exact SI definition since 2019).
    const PLANCK_H: f64 = 6.626_070_15e-34;
    /// Boltzmann constant k_B = 1.380649e-23 J/K (exact SI).
    const BOLTZMANN_K: f64 = 1.380_649e-23;
    /// Newton's gravitational constant G = 6.67430e-11 m³/(kg·s²).
    const G_NEWTON: f64 = 6.674_30e-11;

    /// Planck–Einstein relation for the energy of a photon (Planck's quantum
    /// hypothesis, 1900): `E = h · ν`.
    ///
    /// `frequency` is the photon frequency in Hz. Returns `None` for
    /// non-finite or negative `frequency`.
    pub fn planck_energy(frequency: f64) -> Option<f64> {
        if !frequency.is_finite() || frequency < 0.0 {
            return None;
        }
        Some(PLANCK_H * frequency)
    }

    /// Planck mass `m_p = √(ħ·c/G)` ≈ 2.176434e-8 kg, the mass scale at which
    /// quantum-gravitational effects become order-unity.
    pub fn planck_mass() -> f64 {
        (REDUCED_PLANCK * COMPTON_C / G_NEWTON).sqrt()
    }

    /// Planck length `l_p = √(ħ·G/c³)` ≈ 1.616255e-35 m, the smallest length
    /// scale where classical general relativity remains self-consistent.
    pub fn planck_length() -> f64 {
        (REDUCED_PLANCK * G_NEWTON / COMPTON_C.powi(3)).sqrt()
    }

    /// Planck time `t_p = √(ħ·G/c⁵)` ≈ 5.391247e-44 s, the earliest time after
    /// the Big Bang for which the classical Big-Bang model is meaningful.
    pub fn planck_time() -> f64 {
        (REDUCED_PLANCK * G_NEWTON / COMPTON_C.powi(5)).sqrt()
    }

    /// Planck's blackbody spectral radiance density (energy per unit volume per
    /// unit frequency) at frequency ν and temperature T:
    /// `u(ν, T) = 8π·h·ν³ / c³ / (exp(h·ν / (k_B·T)) - 1)`.
    ///
    /// Both `frequency` and `temperature` must be finite and non-negative.
    /// Returns `None` for invalid inputs or for the limiting special-case
    /// `ν = 0` (where the formula is mathematically `0`); for `T = 0` the
    /// result is `0` (no thermal photons), returned as `Some(0.0)`.
    pub fn planck_radiation_spectral_density(frequency: f64, temperature: f64) -> Option<f64> {
        if !frequency.is_finite()
            || frequency < 0.0
            || !temperature.is_finite()
            || temperature < 0.0
        {
            return None;
        }
        if frequency == 0.0 {
            return Some(0.0);
        }
        if temperature == 0.0 {
            return Some(0.0);
        }
        let x = PLANCK_H * frequency / (BOLTZMANN_K * temperature);
        // Guard against overflow in exp for very large x (Rayleigh–Jeans tail
        // vanishes): if x is huge, the exponential dominates and the result is 0.
        if x > 700.0 {
            return Some(0.0);
        }
        let denom = x.exp() - 1.0;
        if denom <= 0.0 || !denom.is_finite() {
            return None;
        }
        let prefactor = 8.0 * PI * PLANCK_H * frequency.powi(3) / COMPTON_C.powi(3);
        Some(prefactor / denom)
    }
}
