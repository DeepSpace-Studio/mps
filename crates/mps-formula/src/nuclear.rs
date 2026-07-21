//! Nuclear physics — radioactive decay, binding energy, fission, fusion, and neutronics formulas.
//!
//! Pure computation only — no access to `WorldHandle`, `RigidBody`, or Rapier state.

use std::f64::consts::PI;

/// Decay constant from half-life: λ = ln(2) / T½
pub fn decay_constant(half_life: f64) -> Option<f64> {
    if !half_life.is_finite() || half_life <= 0.0 { return None; }
    Some(std::f64::consts::LN_2 / half_life)
}

/// Remaining undecayed nuclei after time t: N(t) = N₀ · exp(-λt)
pub fn remaining_nuclei(initial: f64, decay_constant: f64, time: f64) -> Option<f64> {
    if !initial.is_finite() || initial < 0.0 || !decay_constant.is_finite() || decay_constant < 0.0 || !time.is_finite() || time < 0.0 { return None; }
    Some(initial * (-decay_constant * time).exp())
}

/// Activity at time t: A(t) = λ · N(t)
pub fn activity(decay_constant: f64, nuclei: f64) -> Option<f64> {
    if !decay_constant.is_finite() || decay_constant < 0.0 || !nuclei.is_finite() || nuclei < 0.0 { return None; }
    Some(decay_constant * nuclei)
}

/// Half-life from decay constant: T½ = ln(2) / λ
pub fn half_life(decay_constant: f64) -> Option<f64> {
    if !decay_constant.is_finite() || decay_constant <= 0.0 { return None; }
    Some(std::f64::consts::LN_2 / decay_constant)
}

/// Mean lifetime: τ = 1 / λ
pub fn mean_lifetime(decay_constant: f64) -> Option<f64> {
    if !decay_constant.is_finite() || decay_constant <= 0.0 { return None; }
    Some(1.0 / decay_constant)
}

/// Bethe–Weizsäcker semi-empirical mass formula for binding energy.
///
/// B(A, Z) = a_v·A - a_s·A^(2/3) - a_c·Z²/A^(1/3) - a_a·(A-2Z)²/A + δ(A, Z)
/// where δ = +a_p·A^(-1/2) for even-even, 0 for even-odd, -a_p·A^(-1/2) for odd-odd
pub fn bethe_weizsaecker_binding_energy(mass_number: f64, atomic_number: f64) -> Option<f64> {
    if !mass_number.is_finite() || !atomic_number.is_finite() || mass_number < 1.0 || atomic_number < 0.0 || atomic_number > mass_number { return None; }
    let a = mass_number;
    let z = atomic_number;
    let n = a - z;
    let a_v = 15.75;   // volume term (MeV)
    let a_s = 17.80;   // surface term (MeV)
    let a_c = 0.711;   // Coulomb term (MeV)
    let a_a = 23.70;   // asymmetry term (MeV)
    let a_p = 11.18;   // pairing term (MeV)
    let volume = a_v * a;
    let surface = -a_s * a.powf(2.0 / 3.0);
    let coulomb = -a_c * z * z / a.powf(1.0 / 3.0);
    let asymmetry = -a_a * (a - 2.0 * z).powi(2) / a;
    let pairing = if a < 2.0 { 0.0 }
        else if z % 2.0 == 0.0 && n % 2.0 == 0.0 { a_p / a.sqrt() }
        else if z % 2.0 != 0.0 && n % 2.0 != 0.0 { -a_p / a.sqrt() }
        else { 0.0 };
    Some(volume + surface + coulomb + asymmetry + pairing)
}

/// Binding energy per nucleon: B/A
pub fn binding_energy_per_nucleon(mass_number: f64, atomic_number: f64) -> Option<f64> {
    let binding = bethe_weizsaecker_binding_energy(mass_number, atomic_number)?;
    Some(binding / mass_number)
}

/// Q-value of a nuclear reaction: Q = (M_initial - M_final) · c²  (MeV)
pub fn reaction_q_value(initial_mass_u: f64, final_mass_u: f64) -> Option<f64> {
    if !initial_mass_u.is_finite() || initial_mass_u <= 0.0 || !final_mass_u.is_finite() || final_mass_u <= 0.0 { return None; }
    // 1 u = 931.494 MeV/c²
    Some((initial_mass_u - final_mass_u) * 931.494)
}

/// D-T fusion energy release: ²H + ³H → ⁴He + n  (approx 17.6 MeV)
pub fn dt_fusion_energy() -> f64 { 17.6 }

/// D-D fusion branch 1: ²H + ²H → ³H + p  (approx 4.0 MeV)
pub fn dd_fusion_branch1_energy() -> f64 { 4.0 }

/// D-D fusion branch 2: ²H + ²H → ³He + n  (approx 3.3 MeV)
pub fn dd_fusion_branch2_energy() -> f64 { 3.3 }

/// ²³⁵U fission energy (approx 200 MeV per fission, including neutrons)
pub fn u235_fission_energy() -> f64 { 200.0 }

/// Bateman equation: abundance of Nth nuclide in a decay chain.
/// Simplified for a chain λ₁ → λ₂ → ... → λ_N with only initial parent N₁(0) non-zero.
/// N_n(t) = N₁(0) · Σ_{i=1}^{n} ( Π_{j=1}^{n-1} λ_j / Π_{j≠i, j=1}^{n} (λ_j - λ_i) ) · exp(-λ_i · t)
///
/// Returns the abundance of the n-th nuclide at time t.
pub fn bateman_abundance(
    parent_initial: f64,
    decay_constants: &[f64],
    n: usize,
    time: f64,
) -> Option<f64> {
    if !parent_initial.is_finite() || parent_initial < 0.0 || !time.is_finite() || time < 0.0 || n == 0 || n > decay_constants.len() { return None; }
    let lambdas = decay_constants;
    let n = n - 1; // 0-indexed
    if n == 0 { return Some(parent_initial * (-lambdas[0] * time).exp()); }

    // For n-th member, sum over i=0..n of c_i * exp(-λ_i * t)
    // where c_i = parent_initial * Π_{j=0}^{n-1} λ_j / Π_{j≠i, j=0}^{n} (λ_j - λ_i)
    let product_lambdas: f64 = (0..n).map(|j| lambdas[j]).product();
    let mut sum = 0.0;
    for i in 0..=n {
        let mut denom = 1.0;
        for j in 0..=n {
            if i == j { continue; }
            denom *= lambdas[j] - lambdas[i];
        }
        if denom.abs() < 1.0e-30 { return None; } // degenerate eigenvalues
        sum += product_lambdas / denom * (-lambdas[i] * time).exp();
    }
    Some(parent_initial * sum)
}

/// Neutron diffusion equation — simplified one-group flux in a sphere.
/// φ(r) = S / (4π·D·R) · sin(B·r) / r  for critical reactor with buckling B²
pub fn neutron_flux_sphere(radius: f64, diffusion_coefficient: f64, source_strength: f64, r: f64) -> Option<f64> {
    if !radius.is_finite() || radius <= 0.0 || !diffusion_coefficient.is_finite() || diffusion_coefficient <= 0.0 || !source_strength.is_finite() || source_strength < 0.0 || !r.is_finite() || r < 0.0 || r > radius { return None; }
    let b = PI / radius; // geometric buckling for sphere
    if r <= 1.0e-12 {
        return Some(source_strength / (4.0 * PI * diffusion_coefficient * radius));
    }
    Some(source_strength / (4.0 * PI * diffusion_coefficient * radius) * (b * r).sin() / r)
}

/// Four-factor formula for thermal reactor criticality:
/// k_eff = η · ε · p · f
/// where η = neutrons per fission, ε = fast fission factor, p = resonance escape, f = thermal utilization
pub fn four_factor_formula(eta: f64, epsilon: f64, p: f64, f: f64) -> Option<f64> {
    if !eta.is_finite() || eta <= 0.0 || !epsilon.is_finite() || epsilon <= 0.0 || !p.is_finite() || !(0.0..=1.0).contains(&p) || !f.is_finite() || !(0.0..=1.0).contains(&f) { return None; }
    Some(eta * epsilon * p * f)
}

/// Macroscopic cross-section: Σ = N · σ
pub fn macroscopic_cross_section(number_density: f64, microscopic_cross_section: f64) -> Option<f64> {
    if !number_density.is_finite() || number_density < 0.0 || !microscopic_cross_section.is_finite() || microscopic_cross_section < 0.0 { return None; }
    Some(number_density * microscopic_cross_section)
}

/// Reaction rate: R = Σ · φ (reactions per unit volume per second)
pub fn reaction_rate(macroscopic_cross_section: f64, neutron_flux: f64) -> Option<f64> {
    if !macroscopic_cross_section.is_finite() || macroscopic_cross_section < 0.0 || !neutron_flux.is_finite() || neutron_flux < 0.0 { return None; }
    Some(macroscopic_cross_section * neutron_flux)
}

/// Atomic mass from mass number: m ≈ A · u  (with binding energy correction)
/// Returns mass in atomic mass units (u).
pub fn atomic_mass_approx(mass_number: f64, binding_energy_mev: f64) -> Option<f64> {
    if !mass_number.is_finite() || mass_number <= 0.0 || !binding_energy_mev.is_finite() { return None; }
    Some(mass_number - binding_energy_mev / 931.494)
}

/// Specific activity: SA = λ · N_A / A  (Bq/g)
/// where N_A is Avogadro's number and A is the atomic mass number
pub fn specific_activity(decay_constant: f64, mass_number: f64) -> Option<f64> {
    if !decay_constant.is_finite() || decay_constant <= 0.0 || !mass_number.is_finite() || mass_number <= 0.0 { return None; }
    let avogadro = 6.022_140_76e23;
    Some(decay_constant * avogadro / mass_number)
}

/// Gamma-ray attenuation (Beer–Lambert): I(x) = I₀ · exp(-μ · x)
pub fn gamma_attenuation(initial_intensity: f64, linear_attenuation: f64, thickness: f64) -> Option<f64> {
    if !initial_intensity.is_finite() || initial_intensity < 0.0 || !linear_attenuation.is_finite() || linear_attenuation < 0.0 || !thickness.is_finite() || thickness < 0.0 { return None; }
    Some(initial_intensity * (-linear_attenuation * thickness).exp())
}

/// Half-value layer (HVL): thickness to reduce intensity by half: HVL = ln(2) / μ
pub fn half_value_layer(linear_attenuation: f64) -> Option<f64> {
    if !linear_attenuation.is_finite() || linear_attenuation <= 0.0 { return None; }
    Some(std::f64::consts::LN_2 / linear_attenuation)
}

/// Q-value for D-T fusion from mass defect (exact)
pub fn dt_fusion_q_value() -> f64 {
    // D (2.014102 u) + T (3.016049 u) → He-4 (4.002603 u) + n (1.008665 u)
    // Δm = 2.014102 + 3.016049 - 4.002603 - 1.008665 = 0.018883 u
    // Q = 0.018883 × 931.494 = 17.59 MeV
    17.59
}