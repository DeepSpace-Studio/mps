//! Erwin Schrödinger —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "erwin_schrodinger",
    name: "Erwin Schrödinger",
    birth_year: Some(1887),
    death_year: Some(1961),
    field_id: "quantum_mechanics",
    nationality: "Austrian",
    contribution: "Schrödinger equation; wave mechanics",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    const PI: f64 = std::f64::consts::PI;
    pub const PLANCK: f64 = 6.62607015e-34;
    pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;

    /// Free-particle plane wave: psi(x, t) = A * exp(i(kx - omega t))
    /// Returns (real, imag) components.
    pub fn free_particle_wave_function(
        amplitude: f64,
        wave_number: f64,
        x: f64,
        time: f64,
    ) -> (f64, f64) {
        if !amplitude.is_finite()
            || amplitude < 0.0
            || !wave_number.is_finite()
            || !x.is_finite()
            || !time.is_finite()
        {
            return (0.0, 0.0);
        }
        let energy =
            REDUCED_PLANCK * REDUCED_PLANCK * wave_number * wave_number / (2.0 * 9.1093837e-31);
        let phase = wave_number * x - energy * time / REDUCED_PLANCK;
        (amplitude * phase.cos(), amplitude * phase.sin())
    }

    /// Free particle energy: E = (hbar * k)^2 / (2m)
    pub fn free_particle_energy(wave_number: f64, mass: f64) -> Option<f64> {
        if !wave_number.is_finite() || !mass.is_finite() || mass <= 0.0 {
            return None;
        }
        Some(REDUCED_PLANCK * REDUCED_PLANCK * wave_number * wave_number / (2.0 * mass))
    }

    /// De Broglie wavelength: λ = h / p = h / (m·v).
    /// Shared with other quantum-mechanics founders; defined once in
    /// `crate::scientists::other::formulas`.
    pub use crate::scientists::other::formulas::de_broglie_wavelength;

    /// Infinite square well energy levels: E_n = n^2 * pi^2 * hbar^2 / (2 * m * L^2)
    pub fn infinite_well_energy(quantum_number: u32, mass: f64, well_width: f64) -> Option<f64> {
        if quantum_number == 0
            || !mass.is_finite()
            || mass <= 0.0
            || !well_width.is_finite()
            || well_width <= 0.0
        {
            return None;
        }
        let n = quantum_number as f64;
        Some(
            n * n * PI * PI * REDUCED_PLANCK * REDUCED_PLANCK
                / (2.0 * mass * well_width * well_width),
        )
    }

    /// Infinite square well wave function at position x: psi_n(x) = sqrt(2/L) * sin(n*pi*x/L)
    pub fn infinite_well_wave_function(
        quantum_number: u32,
        well_width: f64,
        x: f64,
    ) -> Option<f64> {
        if quantum_number == 0
            || !well_width.is_finite()
            || well_width <= 0.0
            || !x.is_finite()
            || x < 0.0
            || x > well_width
        {
            return None;
        }
        let n = quantum_number as f64;
        Some((2.0 / well_width).sqrt() * (n * PI * x / well_width).sin())
    }

    /// Probability density at position x in infinite well.
    pub fn infinite_well_probability_density(
        quantum_number: u32,
        well_width: f64,
        x: f64,
    ) -> Option<f64> {
        let psi = infinite_well_wave_function(quantum_number, well_width, x)?;
        Some(psi * psi)
    }

    /// Time evolution phase factor for an energy eigenstate: exp(-iEt/ħ)
    /// Returns (cos_term, sin_term) — real and imaginary parts.
    pub fn time_evolution_phase(energy: f64, time: f64) -> Option<(f64, f64)> {
        let hbar = REDUCED_PLANCK;
        if !energy.is_finite() || !time.is_finite() {
            return None;
        }
        let omega = energy / hbar;
        Some(((-omega * time).cos(), (-omega * time).sin()))
    }

    /// Coherent state amplitude from position/momentum expectation.
    pub fn coherent_state_alpha(
        mean_position: f64,
        mean_momentum: f64,
        mass: f64,
        frequency: f64,
    ) -> Option<f64> {
        let hbar = REDUCED_PLANCK;
        if !mean_position.is_finite()
            || !mean_momentum.is_finite()
            || !mass.is_finite()
            || !frequency.is_finite()
        {
            return None;
        }
        if mass <= 0.0 || frequency <= 0.0 {
            return None;
        }
        let alpha = (mass * frequency / (2.0 * hbar)).sqrt() * mean_position
            + (1.0 / (2.0 * mass * frequency * hbar)).sqrt() * mean_momentum;
        Some(alpha)
    }

    /// Poisson probability for measuring n photons in a coherent state: P(n) = |α|^(2n) exp(-|α|²) / n!
    pub fn coherent_state_photon_probability(alpha_squared: f64, n: u32) -> Option<f64> {
        if !alpha_squared.is_finite() || alpha_squared < 0.0 {
            return None;
        }
        if alpha_squared == 0.0 {
            return if n == 0 { Some(1.0) } else { Some(0.0) };
        }
        let mut factorial = 1.0;
        for i in 1..=n {
            if i > 170 {
                return None;
            } // avoid overflow
            factorial *= i as f64;
        }
        Some(alpha_squared.powi(n as i32) * (-alpha_squared).exp() / factorial)
    }

    /// Spherical harmonic Y_lm(θ, φ) — real-valued combinations (l ≤ 2).
    /// Returns Y_lm for the given angles.
    pub fn spherical_harmonic_real(l: i32, m: i32, theta: f64, phi: f64) -> Option<f64> {
        if !theta.is_finite() || !phi.is_finite() {
            return None;
        }
        let sqrt_4pi_inv = 1.0 / (4.0 * PI).sqrt();
        let sqrt_3_4pi = (3.0 / (4.0 * PI)).sqrt();
        let sqrt_15_4pi = (15.0 / (4.0 * PI)).sqrt();
        let sqrt_15_16pi = (15.0 / (16.0 * PI)).sqrt();
        let sqrt_5_16pi = (5.0 / (16.0 * PI)).sqrt();
        match (l, m) {
            (0, 0) => Some(sqrt_4pi_inv),
            (1, -1) => Some(sqrt_3_4pi * theta.sin() * phi.sin()),
            (1, 0) => Some(sqrt_3_4pi * theta.cos()),
            (1, 1) => Some(sqrt_3_4pi * theta.sin() * phi.cos()),
            (2, -2) => Some(sqrt_15_16pi * theta.sin().powi(2) * (2.0 * phi).sin()),
            (2, -1) => Some(sqrt_15_4pi * theta.sin() * theta.cos() * phi.sin()),
            (2, 0) => Some(sqrt_5_16pi * (3.0 * theta.cos().powi(2) - 1.0)),
            (2, 1) => Some(sqrt_15_4pi * theta.sin() * theta.cos() * phi.cos()),
            (2, 2) => Some(sqrt_15_16pi * theta.sin().powi(2) * (2.0 * phi).cos()),
            _ => None,
        }
    }

    /// Angular momentum quantum numbers: J²|jm⟩ = ħ²·j(j+1)|jm⟩
    pub fn angular_momentum_squared(j: f64) -> Option<f64> {
        if !j.is_finite() || j < 0.0 {
            return None;
        }
        let hbar = REDUCED_PLANCK;
        Some(hbar * hbar * j * (j + 1.0))
    }

    /// Quantum harmonic oscillator energy levels (Schrödinger equation solution):
    /// `E_n = ħ·ω·(n + ½)`.
    ///
    /// `quantum_number` is a non-negative integer; `angular_frequency` is ω.
    /// Returns `None` for non-finite or non-positive `angular_frequency`.
    pub fn harmonic_oscillator_energy(quantum_number: u32, angular_frequency: f64) -> Option<f64> {
        if !angular_frequency.is_finite() || angular_frequency <= 0.0 {
            return None;
        }
        let n = quantum_number as f64;
        Some(REDUCED_PLANCK * angular_frequency * (n + 0.5))
    }

    /// Probability current density for a 1-D wavefunction ψ = ψ_R + i·ψ_I:
    /// `j = (ħ / m) · (ψ_R · ∂ψ_I/∂x − ψ_I · ∂ψ_R/∂x)`.
    ///
    /// `grad_psi_real`/`grad_psi_imag` are the spatial derivatives of the real
    /// and imaginary parts. Returns `None` for non-finite or non-positive `mass`.
    pub fn probability_current(
        psi_real: f64,
        psi_imag: f64,
        grad_psi_real: f64,
        grad_psi_imag: f64,
        mass: f64,
    ) -> Option<f64> {
        if !psi_real.is_finite()
            || !psi_imag.is_finite()
            || !grad_psi_real.is_finite()
            || !grad_psi_imag.is_finite()
            || !mass.is_finite()
            || mass <= 0.0
        {
            return None;
        }
        Some((REDUCED_PLANCK / mass) * (psi_real * grad_psi_imag - psi_imag * grad_psi_real))
    }
}
