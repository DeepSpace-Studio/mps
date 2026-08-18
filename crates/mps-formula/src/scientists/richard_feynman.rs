//! Richard Feynman —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "richard_feynman",
    name: "Richard Feynman",
    birth_year: Some(1918),
    death_year: Some(1988),
    field_id: "quantum_mechanics",
    nationality: "American",
    contribution: "Path-integral; Feynman diagrams",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    const PI: f64 = std::f64::consts::PI;
    pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;

    /// 2×2 degenerate perturbation matrix solution.
    /// Given the perturbation matrix elements in the degenerate subspace:
    /// H'_11, H'_12, H'_21 (=H'_12 for Hermitian), H'_22
    /// Returns the two first-order energy corrections.

    pub fn degenerate_perturbation_2x2(h11: f64, h12: f64, h22: f64) -> Option<(f64, f64)> {
        if !h11.is_finite() || !h12.is_finite() || !h22.is_finite() {
            return None;
        }
        let trace = h11 + h22;
        let det = h11 * h22 - h12 * h12;
        let discriminant = trace * trace - 4.0 * det;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_disc = discriminant.sqrt();
        Some(((trace + sqrt_disc) / 2.0, (trace - sqrt_disc) / 2.0))
    }

    /// Born approximation — differential scattering cross-section for Yukawa potential.
    /// dσ/dΩ = (2m/ħ²)² · (A/(q²+μ²))²

    pub fn born_yukawa_cross_section(
        mass: f64,
        amplitude: f64,
        screening: f64,
        scattering_angle: f64,
        incident_energy: f64,
    ) -> Option<f64> {
        let hbar = REDUCED_PLANCK;
        if !mass.is_finite()
            || !amplitude.is_finite()
            || !screening.is_finite()
            || !scattering_angle.is_finite()
            || !incident_energy.is_finite()
        {
            return None;
        }
        if mass <= 0.0 || incident_energy <= 0.0 {
            return None;
        }
        let k = (2.0 * mass * incident_energy * 1.602_176_634e-19).sqrt() / hbar; // incident wavenumber (convert eV→J)
        let q = 2.0 * k * (scattering_angle / 2.0).sin(); // momentum transfer
        let factor = 2.0 * mass / (hbar * hbar) * amplitude / (q * q + screening * screening);
        Some(factor * factor)
    }

    /// Variational method — estimate ground state energy upper bound.
    /// E_var = ⟨ψ_α|H|ψ_α⟩ / ⟨ψ_α|ψ_α⟩
    /// For hydrogen with trial wavefunction exp(-αr): E(α) = ħ²α²/(2m) - ke²α

    pub fn variational_hydrogen_energy(alpha: f64) -> Option<f64> {
        let hbar = REDUCED_PLANCK;
        let mass_e = 9.109_383_701_5e-31;
        let e_charge = 1.602_176_634e-19;
        let epsilon0 = 8.854_187_812_8e-12;
        if !alpha.is_finite() || alpha <= 0.0 {
            return None;
        }
        let kinetic = hbar * hbar * alpha * alpha / (2.0 * mass_e);
        let potential = -e_charge * e_charge * alpha / (4.0 * PI * epsilon0);
        Some(kinetic + potential)
    }

    /// Optimal variational parameter for hydrogen: α_opt = m e² / (4π ε₀ ħ²) = 1/a₀

    pub fn variational_hydrogen_optimal_alpha() -> f64 {
        let hbar = REDUCED_PLANCK;
        let mass_e = 9.109_383_701_5e-31;
        let e_charge = 1.602_176_634e-19;
        let epsilon0 = 8.854_187_812_8e-12;
        mass_e * e_charge * e_charge / (4.0 * PI * epsilon0 * hbar * hbar)
    }

    /// Spin-orbit coupling energy for hydrogen-like atoms: E_SO = (Z·α)² · E_n / (2n) · [j(j+1)-l(l+1)-s(s+1)] / [l(l+1/2)(l+1)]

    pub fn spin_orbit_energy(n: f64, l: f64, j: f64, atomic_number: f64) -> Option<f64> {
        if !n.is_finite() || !l.is_finite() || !j.is_finite() || !atomic_number.is_finite() {
            return None;
        }
        if n <= 0.0
            || l < 0.0
            || l >= n
            || j < (l - 0.5).abs()
            || j > l + 0.5
            || atomic_number <= 0.0
        {
            return None;
        }
        let alpha = 1.0 / 137.036; // fine structure constant
        let e_n = -13.605_693 * atomic_number * atomic_number / (n * n); // eV
        let numerator = j * (j + 1.0) - l * (l + 1.0) - 0.75; // s(s+1) = 3/4
        let denominator = l * (l + 0.5) * (l + 1.0);
        if denominator <= 0.0 {
            return None;
        }
        Some((atomic_number * alpha).powi(2) * e_n / n * numerator / denominator)
    }
}
