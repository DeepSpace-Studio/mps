use crate::error::{ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, set_error};
use crate::ffi::{
    Bool, QuantumBarrier, QuantumOscillatorReport, QuantumTunnelingReport, QuantumWaveFunction,
};

use crate::math::{finite_non_negative, finite_positive};
use std::f64::consts::PI;

const EPSILON: f64 = 1.0e-12;
pub const REDUCED_PLANCK: f64 = 1.054_571_817e-34;
pub const PLANCK: f64 = 6.62607015e-34;

fn effective_hbar(reduced_planck: f64) -> f64 {
    if reduced_planck == 0.0 {
        REDUCED_PLANCK
    } else {
        reduced_planck
    }
}

fn wave_function_valid(wave: QuantumWaveFunction) -> bool {
    wave.amplitude_real.is_finite() && wave.amplitude_imag.is_finite()
}

fn barrier_valid(barrier: QuantumBarrier) -> bool {
    let hbar = effective_hbar(barrier.reduced_planck);
    finite_non_negative(barrier.particle_energy)
        && finite_non_negative(barrier.barrier_potential)
        && finite_non_negative(barrier.barrier_width)
        && finite_positive(barrier.particle_mass)
        && finite_positive(hbar)
}

fn compute_tunneling(barrier: QuantumBarrier) -> Option<QuantumTunnelingReport> {
    if !barrier_valid(barrier) {
        return None;
    }
    let hbar = effective_hbar(barrier.reduced_planck);
    let mass = barrier.particle_mass;
    let energy = barrier.particle_energy;
    let potential = barrier.barrier_potential;

    if barrier.barrier_width <= EPSILON || energy >= potential {
        let wave_number = (2.0 * mass * energy.max(0.0)).sqrt() / hbar;
        return Some(QuantumTunnelingReport {
            wave_number,
            decay_constant: 0.0,
            exponent: 0.0,
            transmission_coefficient: 1.0,
            reflection_coefficient: 0.0,
        });
    }

    let delta = potential - energy;
    let decay_constant = (2.0 * mass * delta).sqrt() / hbar;
    let exponent = 2.0 * decay_constant * barrier.barrier_width;
    let transmission = (-exponent).exp().clamp(0.0, 1.0);
    Some(QuantumTunnelingReport {
        wave_number: (2.0 * mass * energy.max(0.0)).sqrt() / hbar,
        decay_constant,
        exponent,
        transmission_coefficient: transmission,
        reflection_coefficient: 1.0 - transmission,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_reduced_planck_constant() -> f64 {
    REDUCED_PLANCK
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_wave_probability_density(wave: QuantumWaveFunction) -> f64 {
    if !wave_function_valid(wave) {
        return f64::NAN;
    }
    wave.amplitude_real * wave.amplitude_real + wave.amplitude_imag * wave.amplitude_imag
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_wave_normalize(
    wave: QuantumWaveFunction,
    out_wave: *mut QuantumWaveFunction,
) -> Bool {
    if !wave_function_valid(wave) {
        set_error(ERR_INVALID_ARGUMENT, "invalid quantum wave function");
        return Bool::FALSE;
    }
    let density = quantum_wave_probability_density(wave);
    if density <= EPSILON {
        set_error(ERR_INVALID_ARGUMENT, "quantum wave function has zero norm");
        return Bool::FALSE;
    }
    let Some(out_wave) = (unsafe { out_wave.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "quantum wave output is null");
        return Bool::FALSE;
    };
    let norm = density.sqrt();
    *out_wave = QuantumWaveFunction {
        amplitude_real: wave.amplitude_real / norm,
        amplitude_imag: wave.amplitude_imag / norm,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_wkb_transmission(action_integral: f64, reduced_planck: f64) -> f64 {
    let hbar = effective_hbar(reduced_planck);
    if !finite_non_negative(action_integral) || !finite_positive(hbar) {
        return f64::NAN;
    }
    (-2.0 * action_integral / hbar).exp().clamp(0.0, 1.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_rectangular_barrier_tunneling(
    barrier: QuantumBarrier,
    out_report: *mut QuantumTunnelingReport,
) -> Bool {
    let Some(report) = compute_tunneling(barrier) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid quantum tunneling barrier");
        return Bool::FALSE;
    };
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "quantum tunneling output is null");
        return Bool::FALSE;
    };
    *out_report = report;
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_rectangular_barrier_probability(barrier: QuantumBarrier) -> f64 {
    compute_tunneling(barrier)
        .map(|report| report.transmission_coefficient)
        .unwrap_or(f64::NAN)
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_zero_point_energy(angular_frequency: f64, reduced_planck: f64) -> f64 {
    let hbar = effective_hbar(reduced_planck);
    if !finite_non_negative(angular_frequency) || !finite_positive(hbar) {
        return f64::NAN;
    }
    0.5 * hbar * angular_frequency
}

#[unsafe(no_mangle)]
pub extern "C" fn quantum_harmonic_oscillator_report(
    angular_frequency: f64,
    reduced_planck: f64,
    out_report: *mut QuantumOscillatorReport,
) -> Bool {
    let hbar = effective_hbar(reduced_planck);
    if !finite_non_negative(angular_frequency) || !finite_positive(hbar) {
        set_error(
            ERR_INVALID_ARGUMENT,
            "invalid quantum oscillator parameters",
        );
        return Bool::FALSE;
    }
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "quantum oscillator output is null");
        return Bool::FALSE;
    };
    let level_spacing = hbar * angular_frequency;
    *out_report = QuantumOscillatorReport {
        angular_frequency,
        zero_point_energy: 0.5 * level_spacing,
        first_excited_energy: 1.5 * level_spacing,
        level_spacing,
    };
    clear_error();
    Bool::TRUE
}





// ---------------------------------------------------------------------------
// Schrodinger equation basics
// ---------------------------------------------------------------------------




/// Free-particle plane wave: psi(x, t) = A * exp(i(kx - omega t))
/// Returns (real, imag) components.
pub fn free_particle_wave_function(amplitude: f64, wave_number: f64, x: f64, time: f64) -> (f64, f64) {
    if !amplitude.is_finite() || amplitude < 0.0 || !wave_number.is_finite() || !x.is_finite() || !time.is_finite() { return (0.0, 0.0); }
    let energy = REDUCED_PLANCK * REDUCED_PLANCK * wave_number * wave_number / (2.0 * 9.1093837e-31);
    let phase = wave_number * x - energy * time / REDUCED_PLANCK;
    (amplitude * phase.cos(), amplitude * phase.sin())
}

/// Free particle energy: E = (hbar * k)^2 / (2m)
pub fn free_particle_energy(wave_number: f64, mass: f64) -> Option<f64> {
    if !wave_number.is_finite() || !mass.is_finite() || mass <= 0.0 { return None; }
    Some(REDUCED_PLANCK * REDUCED_PLANCK * wave_number * wave_number / (2.0 * mass))
}

/// De Broglie wavelength: lambda = h / p = h / (m * v)
pub fn de_broglie_wavelength(mass: f64, velocity: f64) -> Option<f64> {
    if !mass.is_finite() || mass <= 0.0 || !velocity.is_finite() || velocity <= 0.0 { return None; }
    Some(PLANCK / (mass * velocity))
}

// ---------------------------------------------------------------------------
// Infinite square well
// ---------------------------------------------------------------------------

/// Infinite square well energy levels: E_n = n^2 * pi^2 * hbar^2 / (2 * m * L^2)
pub fn infinite_well_energy(quantum_number: u32, mass: f64, well_width: f64) -> Option<f64> {
    if quantum_number == 0 || !mass.is_finite() || mass <= 0.0 || !well_width.is_finite() || well_width <= 0.0 { return None; }
    let n = quantum_number as f64;
    Some(n * n * PI * PI * REDUCED_PLANCK * REDUCED_PLANCK / (2.0 * mass * well_width * well_width))
}

/// Infinite square well wave function at position x: psi_n(x) = sqrt(2/L) * sin(n*pi*x/L)
pub fn infinite_well_wave_function(quantum_number: u32, well_width: f64, x: f64) -> Option<f64> {
    if quantum_number == 0 || !well_width.is_finite() || well_width <= 0.0 || !x.is_finite() || x < 0.0 || x > well_width { return None; }
    let n = quantum_number as f64;
    Some((2.0 / well_width).sqrt() * (n * PI * x / well_width).sin())
}

/// Probability density at position x in infinite well.
pub fn infinite_well_probability_density(quantum_number: u32, well_width: f64, x: f64) -> Option<f64> {
    let psi = infinite_well_wave_function(quantum_number, well_width, x)?;
    Some(psi * psi)
}

// ---------------------------------------------------------------------------
// Hydrogen atom (Bohr model)
// ---------------------------------------------------------------------------

/// Bohr radius: a0 = 4*pi*eps0 * hbar^2 / (m_e * e^2)
pub fn bohr_radius() -> f64 { 5.29177210903e-11 }

/// Hydrogen energy levels (Bohr model): E_n = -13.6 eV / n^2
pub fn hydrogen_energy_level(quantum_number: u32) -> Option<f64> {
    if quantum_number == 0 { return None; }
    Some(-13.59844 / (quantum_number as f64 * quantum_number as f64))
}

/// Hydrogen orbital radius (Bohr): r_n = n^2 * a0
pub fn hydrogen_orbital_radius(quantum_number: u32) -> Option<f64> {
    if quantum_number == 0 { return None; }
    Some(quantum_number as f64 * quantum_number as f64 * bohr_radius())
}

/// Hydrogen transition wavelength: 1/lambda = R * (1/n1^2 - 1/n2^2) where R = 1.097e7
pub fn hydrogen_transition_wavelength(n1: u32, n2: u32) -> Option<f64> {
    if n1 == 0 || n2 == 0 || n1 >= n2 { return None; }
    let rydberg = 1.0973731568160e7;
    let n1f = n1 as f64; let n2f = n2 as f64;
    let inv_lambda = rydberg * (1.0 / (n1f * n1f) - 1.0 / (n2f * n2f));
    if inv_lambda <= 0.0 { return None; }
    Some(1.0 / inv_lambda)
}

// ---------------------------------------------------------------------------
// Uncertainty principle
// ---------------------------------------------------------------------------

/// Heisenberg uncertainty principle check: Delta_x * Delta_p >= hbar/2
pub fn heisenberg_uncertainty_satisfied(delta_x: f64, delta_p: f64) -> Option<bool> {
    if !delta_x.is_finite() || delta_x < 0.0 || !delta_p.is_finite() || delta_p < 0.0 { return None; }
    Some(delta_x * delta_p >= REDUCED_PLANCK / 2.0 - 1.0e-15)
}

/// Minimum uncertainty product: hbar/2
pub fn minimum_uncertainty_product() -> f64 { REDUCED_PLANCK / 2.0 }

// ---------------------------------------------------------------------------
// Pauli matrices
// ---------------------------------------------------------------------------

/// Pauli sigma_x matrix-vector multiply.
pub fn pauli_sigma_x(_spinor: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    ((0.0, 1.0), (1.0, 0.0))
}

/// Pauli sigma_y matrix-vector multiply.
pub fn pauli_sigma_y(spinor: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    // ((0, -i), (i, 0))
    ((-spinor.1, spinor.0), (spinor.1, -spinor.0))
}

/// Pauli sigma_z matrix-vector multiply.
pub fn pauli_sigma_z(spinor: (f64, f64)) -> (f64, f64) {
    (spinor.0, -spinor.1)
}

/// Spin expectation value in direction n from spinor.
pub fn spin_expectation(spinor: (f64, f64)) -> (f64, f64, f64) {
    let (a, b) = spinor;
    let norm2 = a*a + b*b;
    if norm2 < 1.0e-15 { return (0.0, 0.0, 0.0); }
    let sx = 2.0 * (a * b) / norm2;
    let sy = 0.0; // simplified
    let sz = (a*a - b*b) / norm2;
    (sx, sy, sz)
}

fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
    a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
}

// ---------------------------------------------------------------------------
// Degenerate perturbation theory
// ---------------------------------------------------------------------------

/// 2×2 degenerate perturbation matrix solution.
/// Given the perturbation matrix elements in the degenerate subspace:
/// H'_11, H'_12, H'_21 (=H'_12 for Hermitian), H'_22
/// Returns the two first-order energy corrections.
pub fn degenerate_perturbation_2x2(h11: f64, h12: f64, h22: f64) -> Option<(f64, f64)> {
    if !h11.is_finite() || !h12.is_finite() || !h22.is_finite() { return None; }
    let trace = h11 + h22;
    let det = h11 * h22 - h12 * h12;
    let discriminant = trace * trace - 4.0 * det;
    if discriminant < 0.0 { return None; }
    let sqrt_disc = discriminant.sqrt();
    Some(((trace + sqrt_disc) / 2.0, (trace - sqrt_disc) / 2.0))
}

// ---------------------------------------------------------------------------
// Fermi's golden rule
// ---------------------------------------------------------------------------

/// Fermi's golden rule — transition rate: Γ_fi = 2π/ħ · |⟨f|H'|i⟩|² · ρ(E_f)
pub fn fermi_golden_rule_linear(matrix_element2: f64, density_of_states: f64) -> Option<f64> {
    let hbar = REDUCED_PLANCK;
    if !matrix_element2.is_finite() || matrix_element2 < 0.0 || !density_of_states.is_finite() || density_of_states < 0.0 { return None; }
    Some(2.0 * PI / hbar * matrix_element2 * density_of_states)
}

/// Fermi's golden rule — emission rate into continuum (single mode).
pub fn fermi_golden_rule_cavity(coupling_strength: f64, cavity_linewidth: f64, detuning: f64) -> Option<f64> {
    if !coupling_strength.is_finite() || coupling_strength < 0.0 || !cavity_linewidth.is_finite() || cavity_linewidth <= 0.0 || !detuning.is_finite() { return None; }
    Some(coupling_strength * cavity_linewidth / (detuning * detuning + 0.25 * cavity_linewidth * cavity_linewidth))
}

// ---------------------------------------------------------------------------
// Spin-orbit coupling
// ---------------------------------------------------------------------------

/// Spin-orbit coupling energy for hydrogen-like atoms: E_SO = (Z·α)² · E_n / (2n) · [j(j+1)-l(l+1)-s(s+1)] / [l(l+1/2)(l+1)]
pub fn spin_orbit_energy(n: f64, l: f64, j: f64, atomic_number: f64) -> Option<f64> {
    if !n.is_finite() || !l.is_finite() || !j.is_finite() || !atomic_number.is_finite() { return None; }
    if n <= 0.0 || l < 0.0 || l >= n || j < (l - 0.5).abs() || j > l + 0.5 || atomic_number <= 0.0 { return None; }
    let alpha = 1.0 / 137.036; // fine structure constant
    let e_n = -13.605_693 * atomic_number * atomic_number / (n * n); // eV
    let numerator = j * (j + 1.0) - l * (l + 1.0) - 0.75; // s(s+1) = 3/4
    let denominator = l * (l + 0.5) * (l + 1.0);
    if denominator <= 0.0 { return None; }
    Some((atomic_number * alpha).powi(2) * e_n / n * numerator / denominator)
}

/// Fine structure constant: α ≈ 1/137.036
pub fn fine_structure_constant() -> f64 { 1.0 / 137.035_999_084 }

// ---------------------------------------------------------------------------
// Born approximation (scattering)
// ---------------------------------------------------------------------------

/// Born approximation — differential scattering cross-section for Yukawa potential.
/// dσ/dΩ = (2m/ħ²)² · (A/(q²+μ²))²
pub fn born_yukawa_cross_section(mass: f64, amplitude: f64, screening: f64, scattering_angle: f64, incident_energy: f64) -> Option<f64> {
    let hbar = REDUCED_PLANCK;
    if !mass.is_finite() || !amplitude.is_finite() || !screening.is_finite() || !scattering_angle.is_finite() || !incident_energy.is_finite() { return None; }
    if mass <= 0.0 || incident_energy <= 0.0 { return None; }
    let k = (2.0 * mass * incident_energy * 1.602_176_634e-19).sqrt() / hbar; // incident wavenumber (convert eV→J)
    let q = 2.0 * k * (scattering_angle / 2.0).sin(); // momentum transfer
    let factor = 2.0 * mass / (hbar * hbar) * amplitude / (q * q + screening * screening);
    Some(factor * factor)
}

// ---------------------------------------------------------------------------
// Variational method
// ---------------------------------------------------------------------------

/// Variational method — estimate ground state energy upper bound.
/// E_var = ⟨ψ_α|H|ψ_α⟩ / ⟨ψ_α|ψ_α⟩
/// For hydrogen with trial wavefunction exp(-αr): E(α) = ħ²α²/(2m) - ke²α
pub fn variational_hydrogen_energy(alpha: f64) -> Option<f64> {
    let hbar = REDUCED_PLANCK;
    let mass_e = 9.109_383_701_5e-31;
    let e_charge = 1.602_176_634e-19;
    let epsilon0 = 8.854_187_812_8e-12;
    if !alpha.is_finite() || alpha <= 0.0 { return None; }
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

// ---------------------------------------------------------------------------
// Time evolution
// ---------------------------------------------------------------------------

/// Time evolution phase factor for an energy eigenstate: exp(-iEt/ħ)
/// Returns (cos_term, sin_term) — real and imaginary parts.
pub fn time_evolution_phase(energy: f64, time: f64) -> Option<(f64, f64)> {
    let hbar = REDUCED_PLANCK;
    if !energy.is_finite() || !time.is_finite() { return None; }
    let omega = energy / hbar;
    Some(((-omega * time).cos(), (-omega * time).sin()))
}

// ---------------------------------------------------------------------------
// Coherent states
// ---------------------------------------------------------------------------

/// Coherent state amplitude from position/momentum expectation.
pub fn coherent_state_alpha(mean_position: f64, mean_momentum: f64, mass: f64, frequency: f64) -> Option<f64> {
    let hbar = REDUCED_PLANCK;
    if !mean_position.is_finite() || !mean_momentum.is_finite() || !mass.is_finite() || !frequency.is_finite() { return None; }
    if mass <= 0.0 || frequency <= 0.0 { return None; }
    let alpha = (mass * frequency / (2.0 * hbar)).sqrt() * mean_position +
                (1.0 / (2.0 * mass * frequency * hbar)).sqrt() * mean_momentum;
    Some(alpha)
}

/// Poisson probability for measuring n photons in a coherent state: P(n) = |α|^(2n) exp(-|α|²) / n!
pub fn coherent_state_photon_probability(alpha_squared: f64, n: u32) -> Option<f64> {
    if !alpha_squared.is_finite() || alpha_squared < 0.0 { return None; }
    if alpha_squared == 0.0 { return if n == 0 { Some(1.0) } else { Some(0.0) }; }
    let mut factorial = 1.0;
    for i in 1..=n {
        if i > 170 { return None; } // avoid overflow
        factorial *= i as f64;
    }
    Some(alpha_squared.powi(n as i32) * (-alpha_squared).exp() / factorial)
}

// ---------------------------------------------------------------------------
// Angular momentum
// ---------------------------------------------------------------------------

/// Spherical harmonic Y_lm(θ, φ) — real-valued combinations (l ≤ 2).
/// Returns Y_lm for the given angles.
pub fn spherical_harmonic_real(l: i32, m: i32, theta: f64, phi: f64) -> Option<f64> {
    if !theta.is_finite() || !phi.is_finite() { return None; }
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
    if !j.is_finite() || j < 0.0 { return None; }
    let hbar = REDUCED_PLANCK;
    Some(hbar * hbar * j * (j + 1.0))
}