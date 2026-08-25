//! Niels Bohr —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "niels_bohr",
    name: "Niels Bohr",
    birth_year: Some(1885),
    death_year: Some(1962),
    field_id: "quantum_mechanics",
    nationality: "Danish",
    contribution: "Bohr model of the atom; complementarity",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {

    /// Bohr radius: a0 = 4*pi*eps0 * hbar^2 / (m_e * e^2)
    pub fn bohr_radius() -> f64 {
        5.29177210903e-11
    }

    /// Hydrogen energy levels (Bohr model): E_n = -13.6 eV / n^2
    pub fn hydrogen_energy_level(quantum_number: u32) -> Option<f64> {
        if quantum_number == 0 {
            return None;
        }
        Some(-13.59844 / (quantum_number as f64 * quantum_number as f64))
    }

    /// Hydrogen orbital radius (Bohr): r_n = n^2 * a0
    pub fn hydrogen_orbital_radius(quantum_number: u32) -> Option<f64> {
        if quantum_number == 0 {
            return None;
        }
        Some(quantum_number as f64 * quantum_number as f64 * bohr_radius())
    }

    /// Hydrogen transition wavelength: 1/lambda = R * (1/n1^2 - 1/n2^2) where R = 1.097e7
    pub fn hydrogen_transition_wavelength(n1: u32, n2: u32) -> Option<f64> {
        if n1 == 0 || n2 == 0 || n1 >= n2 {
            return None;
        }
        let rydberg = 1.0973731568160e7;
        let n1f = n1 as f64;
        let n2f = n2 as f64;
        let inv_lambda = rydberg * (1.0 / (n1f * n1f) - 1.0 / (n2f * n2f));
        if inv_lambda <= 0.0 {
            return None;
        }
        Some(1.0 / inv_lambda)
    }

    // ----- Bohr's own core additional contributions -----

    /// Rydberg energy (Hartree-like): `E_R = 13.6 eV`, the ionization-energy
    /// magnitude of the hydrogen ground state in the Bohr model.
    pub fn rydberg_energy() -> f64 {
        13.605_693_122_994
    }

    /// Bohr magneton (atomic unit of magnetic moment):
    /// `μ_B = e·ħ / (2·m_e) = 9.2740100783e-24 J/T`.
    /// Electron charge e = 1.602176634e-19 C (exact SI), reduced Planck ħ,
    /// electron rest mass m_e = 9.1093837015e-31 kg.
    pub fn bohr_magneton() -> f64 {
        const E_CHARGE: f64 = 1.602_176_634e-19;
        const E_MASS: f64 = 9.109_383_701_5e-31;
        const HBAR: f64 = 1.054_571_817e-34;
        E_CHARGE * HBAR / (2.0 * E_MASS)
    }

    /// Bohr's quantization of angular momentum for stationary orbits:
    /// `L_n = n·ħ` (n = 1, 2, 3, …).
    pub fn angular_momentum_quantum(quantum_number: u32) -> Option<f64> {
        if quantum_number == 0 {
            return None;
        }
        Some(1.054_571_817e-34 * quantum_number as f64)
    }
}
