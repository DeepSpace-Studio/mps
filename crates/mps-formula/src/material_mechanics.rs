//! Material mechanics — elasticity, plasticity, fracture, fatigue, and structural formulas.
//!
//! Pure computation only — no access to `WorldHandle`, `RigidBody`, or Rapier state.

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Linear elasticity
// ---------------------------------------------------------------------------

/// Hooke's law (uniaxial): σ = E · ε
pub fn hookes_law_uniaxial(stress: f64, youngs_modulus: f64) -> Option<f64> {
    if !stress.is_finite() || !youngs_modulus.is_finite() || youngs_modulus <= 0.0 { return None; }
    Some(stress / youngs_modulus)
}

/// Hooke's law (stress from strain): σ = E · ε
pub fn stress_from_strain(youngs_modulus: f64, strain: f64) -> Option<f64> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !strain.is_finite() { return None; }
    Some(youngs_modulus * strain)
}

/// Shear modulus from Young's modulus and Poisson's ratio:
/// G = E / (2 · (1 + ν))
pub fn shear_modulus(youngs_modulus: f64, poisson_ratio: f64) -> Option<f64> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !poisson_ratio.is_finite() || poisson_ratio < -1.0 || poisson_ratio > 0.5 { return None; }
    Some(youngs_modulus / (2.0 * (1.0 + poisson_ratio)))
}

/// Bulk modulus: K = E / (3 · (1 - 2ν))
pub fn bulk_modulus(youngs_modulus: f64, poisson_ratio: f64) -> Option<f64> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !poisson_ratio.is_finite() || poisson_ratio >= 0.5 { return None; }
    Some(youngs_modulus / (3.0 * (1.0 - 2.0 * poisson_ratio)))
}

/// Lamé's first parameter: λ = E · ν / ((1+ν)(1-2ν))
pub fn lame_lambda(youngs_modulus: f64, poisson_ratio: f64) -> Option<f64> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !poisson_ratio.is_finite() || poisson_ratio < -1.0 || poisson_ratio >= 0.5 { return None; }
    Some(youngs_modulus * poisson_ratio / ((1.0 + poisson_ratio) * (1.0 - 2.0 * poisson_ratio)))
}

/// Lamé's second parameter (= shear modulus): μ = G
pub fn lame_mu(shear_modulus: f64) -> f64 { shear_modulus }

/// Plane stress stiffness matrix components (2D isotropic):
/// Q₁₁ = E/(1-ν²), Q₁₂ = ν·E/(1-ν²), Q₆₆ = G
pub fn plane_stress_stiffness(youngs_modulus: f64, poisson_ratio: f64) -> Option<(f64, f64, f64)> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !poisson_ratio.is_finite() || poisson_ratio < -1.0 || poisson_ratio >= 1.0 { return None; }
    let g = youngs_modulus / (2.0 * (1.0 + poisson_ratio));
    let factor = youngs_modulus / (1.0 - poisson_ratio * poisson_ratio);
    Some((factor, factor * poisson_ratio, g))
}

// ---------------------------------------------------------------------------
// Yield criteria
// ---------------------------------------------------------------------------

/// von Mises equivalent stress: σ_vm = sqrt(σ_x² + σ_y² + σ_z² - σ_xσ_y - σ_yσ_z - σ_zσ_x + 3(τ_xy² + τ_yz² + τ_zx²))
pub fn von_mises_stress(
    sx: f64, sy: f64, sz: f64,
    txy: f64, tyz: f64, tzx: f64,
) -> Option<f64> {
    if !finite_6(&[sx, sy, sz, txy, tyz, tzx]) { return None; }
    let val = sx*sx + sy*sy + sz*sz - sx*sy - sy*sz - sz*sx + 3.0*(txy*txy + tyz*tyz + tzx*tzx);
    if val < 0.0 { return None; }
    Some(val.sqrt())
}

/// von Mises yield criterion: σ_vm ≥ σ_yield → yield occurs
pub fn von_mises_yield_check(von_mises_stress: f64, yield_stress: f64) -> Option<f64> {
    if !von_mises_stress.is_finite() || von_mises_stress < 0.0 || !yield_stress.is_finite() || yield_stress <= 0.0 { return None; }
    Some(von_mises_stress / yield_stress) // ratio < 1 = elastic, >= 1 = yield
}

/// Tresca (maximum shear stress) criterion: τ_max = (σ₁ - σ₃)/2 ≥ σ_yield/2
/// σ₁ = first principal stress, σ₃ = third principal stress
pub fn tresca_shear_stress(sigma_1: f64, sigma_3: f64) -> Option<f64> {
    if !sigma_1.is_finite() || !sigma_3.is_finite() { return None; }
    Some(0.5 * (sigma_1 - sigma_3).abs())
}

/// Tresca yield check: τ_max · 2 ≥ σ_yield
pub fn tresca_yield_check(sigma_1: f64, sigma_3: f64, yield_stress: f64) -> Option<f64> {
    if !sigma_1.is_finite() || !sigma_3.is_finite() || !yield_stress.is_finite() || yield_stress <= 0.0 { return None; }
    Some((sigma_1 - sigma_3).abs() / yield_stress)
}

/// Principal stresses from 3D stress tensor (roots of cubic).
/// Returns (σ₁, σ₂, σ₃) sorted descending.
pub fn principal_stresses(
    sx: f64, sy: f64, sz: f64,
    txy: f64, tyz: f64, tzx: f64,
) -> Option<(f64, f64, f64)> {
    // Stress invariants
    let i1 = sx + sy + sz;
    let i2 = sx*sy + sy*sz + sz*sx - txy*txy - tyz*tyz - tzx*tzx;
    let i3 = sx*sy*sz + 2.0*txy*tyz*tzx - sx*tyz*tyz - sy*tzx*tzx - sz*txy*txy;
    // Depressed cubic: y³ - p·y - q = 0 where y = σ - I₁/3
    let p = i1*i1/3.0 - i2;
    let q = 2.0*i1*i1*i1/27.0 - i1*i2/3.0 + i3;
    if p < 0.0 { return None; } // should not happen for real stress tensors
    let r = (p/3.0).sqrt();
    let phi = (q / (2.0 * p * r)).clamp(-1.0, 1.0).acos();
    let s1 = i1/3.0 + 2.0*r * phi.cos();
    let s2 = i1/3.0 + 2.0*r * (phi - 2.0*PI/3.0).cos();
    let s3 = i1/3.0 + 2.0*r * (phi + 2.0*PI/3.0).cos();
    // Sort descending
    let mut v = [s1, s2, s3];
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    Some((v[0], v[1], v[2]))
}

// ---------------------------------------------------------------------------
// Fracture mechanics
// ---------------------------------------------------------------------------

/// Mode I stress intensity factor for infinite plate with center crack:
/// K_I = σ · sqrt(π · a)
pub fn ki_center_crack(stress: f64, crack_half_length: f64) -> Option<f64> {
    if !stress.is_finite() || !crack_half_length.is_finite() || crack_half_length <= 0.0 { return None; }
    Some(stress * (PI * crack_half_length).sqrt())
}

/// Mode I stress intensity factor for edge crack:
/// K_I = 1.12 · σ · sqrt(π · a)
pub fn ki_edge_crack(stress: f64, crack_length: f64) -> Option<f64> {
    if !stress.is_finite() || !crack_length.is_finite() || crack_length <= 0.0 { return None; }
    Some(1.12 * stress * (PI * crack_length).sqrt())
}

/// Fracture toughness check: K_I ≥ K_IC → fracture
pub fn fracture_check(stress_intensity: f64, fracture_toughness: f64) -> Option<f64> {
    if !stress_intensity.is_finite() || stress_intensity < 0.0 || !fracture_toughness.is_finite() || fracture_toughness <= 0.0 { return None; }
    Some(stress_intensity / fracture_toughness)
}

/// Critical crack length for fracture: a_c = K_IC² / (π · σ²)
pub fn critical_crack_length(stress: f64, fracture_toughness: f64) -> Option<f64> {
    if !stress.is_finite() || stress <= 0.0 || !fracture_toughness.is_finite() || fracture_toughness <= 0.0 { return None; }
    Some(fracture_toughness * fracture_toughness / (PI * stress * stress))
}

// ---------------------------------------------------------------------------
// Fatigue
// ---------------------------------------------------------------------------

/// S-N curve (Basquin relation): σ_a = σ_f' · (2N_f)^b
/// where σ_a = stress amplitude, N_f = cycles to failure,
/// σ_f' = fatigue strength coefficient, b = fatigue strength exponent
pub fn basquin_stress_amplitude(cycles_to_failure: f64, fatigue_strength_coefficient: f64, fatigue_exponent: f64) -> Option<f64> {
    if !cycles_to_failure.is_finite() || cycles_to_failure < 1.0 || !fatigue_strength_coefficient.is_finite() || fatigue_strength_coefficient <= 0.0 || !fatigue_exponent.is_finite() { return None; }
    Some(fatigue_strength_coefficient * (2.0 * cycles_to_failure).powf(fatigue_exponent))
}

/// Cycles to failure from Basquin relation: N_f = 0.5 · (σ_a/σ_f')^(1/b)
pub fn basquin_cycles_to_failure(stress_amplitude: f64, fatigue_strength_coefficient: f64, fatigue_exponent: f64) -> Option<f64> {
    if !stress_amplitude.is_finite() || stress_amplitude <= 0.0 || !fatigue_strength_coefficient.is_finite() || fatigue_strength_coefficient <= 0.0 || !fatigue_exponent.is_finite() || fatigue_exponent >= 0.0 { return None; }
    Some(0.5 * (stress_amplitude / fatigue_strength_coefficient).powf(1.0 / fatigue_exponent))
}

/// Coffin-Manson (low-cycle fatigue): ε_pa = ε_f' · (2N_f)^c
/// where ε_pa = plastic strain amplitude, ε_f' = fatigue ductility coefficient, c = fatigue ductility exponent
pub fn coffin_manson_strain_amplitude(cycles_to_failure: f64, ductility_coefficient: f64, ductility_exponent: f64) -> Option<f64> {
    if !cycles_to_failure.is_finite() || cycles_to_failure < 1.0 || !ductility_coefficient.is_finite() || ductility_coefficient <= 0.0 || !ductility_exponent.is_finite() { return None; }
    Some(ductility_coefficient * (2.0 * cycles_to_failure).powf(ductility_exponent))
}

/// Miner's linear damage rule: D = Σ (n_i / N_fi)
pub fn miners_damage(cycle_ratios: &[f64]) -> Option<f64> {
    if cycle_ratios.is_empty() { return Some(0.0); }
    for &r in cycle_ratios {
        if !r.is_finite() || r < 0.0 { return None; }
    }
    Some(cycle_ratios.iter().sum())
}

/// Goodman mean stress correction: σ_a = σ_flip · (1 - σ_m/σ_uts)
pub fn goodman_correction(stress_amplitude: f64, mean_stress: f64, ultimate_tensile: f64) -> Option<f64> {
    if !stress_amplitude.is_finite() || stress_amplitude < 0.0 || !mean_stress.is_finite() || !ultimate_tensile.is_finite() || ultimate_tensile <= 0.0 { return None; }
    Some(stress_amplitude / (1.0 - mean_stress / ultimate_tensile))
}

// ---------------------------------------------------------------------------
// Creep
// ---------------------------------------------------------------------------

/// Norton creep law: ε_dot = A · σ^n · exp(-Q/(R·T))
pub fn norton_creep_rate(stress: f64, temperature: f64, a: f64, n: f64, activation_energy: f64, gas_constant: f64) -> Option<f64> {
    if !stress.is_finite() || stress < 0.0 || !temperature.is_finite() || temperature <= 0.0 || !a.is_finite() || a <= 0.0 || !n.is_finite() || !activation_energy.is_finite() || !gas_constant.is_finite() || gas_constant <= 0.0 { return None; }
    Some(a * stress.powf(n) * (-activation_energy / (gas_constant * temperature)).exp())
}

// ---------------------------------------------------------------------------
// Beam theory
// ---------------------------------------------------------------------------

/// Euler-Bernoulli beam: maximum bending stress at section: σ = M · c / I
pub fn beam_bending_stress(bending_moment: f64, distance_from_neutral_axis: f64, area_moment_of_inertia: f64) -> Option<f64> {
    if !bending_moment.is_finite() || !distance_from_neutral_axis.is_finite() || distance_from_neutral_axis < 0.0 || !area_moment_of_inertia.is_finite() || area_moment_of_inertia <= 0.0 { return None; }
    Some(bending_moment * distance_from_neutral_axis / area_moment_of_inertia)
}

/// Beam deflection (simply supported, point load at center): δ = P·L³/(48·E·I)
pub fn beam_deflection_center_point_load(load: f64, span: f64, youngs_modulus: f64, moment_of_inertia: f64) -> Option<f64> {
    if !load.is_finite() || load < 0.0 || !span.is_finite() || span <= 0.0 || !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !moment_of_inertia.is_finite() || moment_of_inertia <= 0.0 { return None; }
    Some(load * span * span * span / (48.0 * youngs_modulus * moment_of_inertia))
}

/// Euler critical buckling load: P_cr = π² · E · I / (K·L)²
pub fn euler_buckling_load(youngs_modulus: f64, moment_of_inertia: f64, effective_length_factor: f64, column_length: f64) -> Option<f64> {
    if !youngs_modulus.is_finite() || youngs_modulus <= 0.0 || !moment_of_inertia.is_finite() || moment_of_inertia <= 0.0 || !effective_length_factor.is_finite() || effective_length_factor <= 0.0 || !column_length.is_finite() || column_length <= 0.0 { return None; }
    Some(PI * PI * youngs_modulus * moment_of_inertia / (effective_length_factor * column_length).powi(2))
}

/// Slenderness ratio: λ = K·L / r (where r = sqrt(I/A) = radius of gyration)
pub fn slenderness_ratio(effective_length_factor: f64, column_length: f64, radius_of_gyration: f64) -> Option<f64> {
    if !effective_length_factor.is_finite() || effective_length_factor <= 0.0 || !column_length.is_finite() || column_length <= 0.0 || !radius_of_gyration.is_finite() || radius_of_gyration <= 0.0 { return None; }
    Some(effective_length_factor * column_length / radius_of_gyration)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn finite_6(v: &[f64; 6]) -> bool {
    v.iter().all(|x| x.is_finite())
}