//! James Clerk Maxwell —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "james_clerk_maxwell",
    name: "James Clerk Maxwell",
    birth_year: Some(1831),
    death_year: Some(1879),
    field_id: "electromagnetism",
    nationality: "British",
    contribution: "Maxwell's equations; EM wave & circuit theory",
    key_constants: "c",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    pub const G: f64 = 6.67430e-11;
    const PI: f64 = std::f64::consts::PI;
    fn finite_6(v: &[f64; 6]) -> bool {
        v.iter().all(|x| x.is_finite())
    }

    /// Biot-Savart law: dB = (mu0/4pi) * I * dl x r_hat / r^2
    /// Returns the magnetic field contribution at `point` from a current element.

    pub fn biot_savart_element(
        current: f64,
        dl: Vec3,
        position: Vec3,
        point: Vec3,
    ) -> Option<Vec3> {
        let mu0 = 1.25663706212e-6;
        if !vec3_finite(dl) || !vec3_finite(position) || !vec3_finite(point) || !current.is_finite()
        {
            return None;
        }
        let r_vec = vec3_to_rapier(point) - vec3_to_rapier(position);
        let r = r_vec.length();
        if r < 1.0e-12 {
            return None;
        }
        let r_hat = r_vec / r;
        let cross = vec3_to_rapier(dl).cross(r_hat);
        let factor = mu0 * current / (4.0 * PI * r * r);
        Some(vec3_from_rapier(cross * factor))
    }

    /// Biot-Savart law for a finite straight wire segment.
    /// Returns B at `point` from wire from `p1` to `p2` carrying current.

    pub fn biot_savart_wire_segment(current: f64, p1: Vec3, p2: Vec3, point: Vec3) -> Option<Vec3> {
        let mu0 = 1.25663706212e-6;
        if !finite_6(&[current, p1.x, p1.y, p1.z, p2.x, p2.y]) || !vec3_finite(point) {
            return None;
        }
        let a = vec3_to_rapier(p1);
        let b = vec3_to_rapier(p2);
        let p = vec3_to_rapier(point);
        let l = b - a;
        let l_len = l.length();
        if l_len < 1.0e-12 {
            return None;
        }
        let r1 = p - a;
        let r2 = p - b;
        let r1_len = r1.length();
        let r2_len = r2.length();
        if r1_len < 1.0e-12 || r2_len < 1.0e-12 {
            return None;
        }
        let l_hat = l / l_len;
        let sin_theta1 = l_hat.cross(r1 / r1_len).length();
        let sin_theta2 = l_hat.cross(r2 / r2_len).length();
        let direction = l_hat.cross(r1 / r1_len).try_normalize()?;
        let factor = mu0 * current / (4.0 * PI);
        let term = (1.0 / r1_len + 1.0 / r2_len) * (sin_theta1 + sin_theta2) / 2.0;
        Some(vec3_from_rapier(direction * factor * term))
    }

    /// Poynting vector: S = E x H (W/m^2) where H = B / mu0

    pub fn poynting_vector(e: Vec3, b: Vec3) -> Option<Vec3> {
        let mu0 = 1.25663706212e-6;
        if !vec3_finite(e) || !vec3_finite(b) {
            return None;
        }
        let e_v = vec3_to_rapier(e);
        let b_v = vec3_to_rapier(b);
        let s = e_v.cross(b_v) / mu0;
        Some(vec3_from_rapier(s))
    }

    /// Poynting vector magnitude for plane wave: |S| = |E|^2 / (mu0 * c)

    pub fn poynting_magnitude_plane_wave(e_field_magnitude: f64) -> Option<f64> {
        let c = 299_792_458.0;
        let mu0 = 1.25663706212e-6;
        if !e_field_magnitude.is_finite() || e_field_magnitude < 0.0 {
            return None;
        }
        Some(e_field_magnitude * e_field_magnitude / (mu0 * c))
    }

    /// Phase velocity in medium: v = c / n

    pub fn phase_velocity(refractive_index: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !refractive_index.is_finite() || refractive_index <= 0.0 {
            return None;
        }
        Some(c / refractive_index)
    }

    /// Wavelength: lambda = c / (n * f)

    pub fn wavelength_in_medium(frequency: f64, refractive_index: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !frequency.is_finite()
            || frequency <= 0.0
            || !refractive_index.is_finite()
            || refractive_index <= 0.0
        {
            return None;
        }
        Some(c / (refractive_index * frequency))
    }

    /// Intrinsic impedance of medium: eta = sqrt(mu / epsilon)

    pub fn intrinsic_impedance(permeability: f64, permittivity: f64) -> Option<f64> {
        if !permeability.is_finite()
            || permeability <= 0.0
            || !permittivity.is_finite()
            || permittivity <= 0.0
        {
            return None;
        }
        Some((permeability / permittivity).sqrt())
    }

    /// Skin depth: delta = 1 / sqrt(pi * f * mu * sigma)

    pub fn skin_depth(frequency: f64, permeability: f64, conductivity: f64) -> Option<f64> {
        if !frequency.is_finite()
            || frequency <= 0.0
            || !permeability.is_finite()
            || permeability <= 0.0
            || !conductivity.is_finite()
            || conductivity <= 0.0
        {
            return None;
        }
        Some(1.0 / (PI * frequency * permeability * conductivity).sqrt())
    }

    /// EM wave vacuum wavelength: lambda = c / f

    pub fn vacuum_wavelength(frequency: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !frequency.is_finite() || frequency <= 0.0 {
            return None;
        }
        Some(c / frequency)
    }

    /// EM wave frequency: f = c / lambda

    pub fn wave_frequency(wavelength: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !wavelength.is_finite() || wavelength <= 0.0 {
            return None;
        }
        Some(c / wavelength)
    }

    /// Radiation resistance of a short dipole: R_r = 80π² (L/λ)²

    pub fn dipole_radiation_resistance(dipole_length: f64, wavelength: f64) -> Option<f64> {
        if !dipole_length.is_finite()
            || !wavelength.is_finite()
            || dipole_length < 0.0
            || wavelength <= 0.0
        {
            return None;
        }
        Some(
            80.0 * std::f64::consts::PI
                * std::f64::consts::PI
                * (dipole_length / wavelength).powi(2),
        )
    }

    /// Half-wave dipole directivity: D = 1.64

    pub fn half_wave_dipole_directivity() -> f64 {
        1.64
    }

    /// Effective aperture from gain: A_e = G · λ² / (4π)

    pub fn effective_aperture(gain_linear: f64, wavelength: f64) -> Option<f64> {
        if !gain_linear.is_finite()
            || gain_linear <= 0.0
            || !wavelength.is_finite()
            || wavelength <= 0.0
        {
            return None;
        }
        Some(gain_linear * wavelength * wavelength / (4.0 * std::f64::consts::PI))
    }

    /// Far-field distance (Fraunhofer): r = 2D²/λ where D is the largest antenna dimension.

    pub fn far_field_distance(antenna_size: f64, wavelength: f64) -> Option<f64> {
        if !antenna_size.is_finite()
            || antenna_size <= 0.0
            || !wavelength.is_finite()
            || wavelength <= 0.0
        {
            return None;
        }
        Some(2.0 * antenna_size * antenna_size / wavelength)
    }

    /// Friis transmission equation (power): P_r = P_t · G_t · G_r · (λ/(4πR))²

    pub fn friis_power_received(
        transmit_power: f64,
        tx_gain: f64,
        rx_gain: f64,
        wavelength: f64,
        range: f64,
    ) -> Option<f64> {
        if !transmit_power.is_finite()
            || !tx_gain.is_finite()
            || !rx_gain.is_finite()
            || !wavelength.is_finite()
            || !range.is_finite()
        {
            return None;
        }
        if transmit_power < 0.0
            || tx_gain < 0.0
            || rx_gain < 0.0
            || wavelength <= 0.0
            || range <= 0.0
        {
            return None;
        }
        Some(
            transmit_power
                * tx_gain
                * rx_gain
                * (wavelength / (4.0 * std::f64::consts::PI * range)).powi(2),
        )
    }

    /// Reflection coefficient: Γ = (Z_L - Z_0) / (Z_L + Z_0)

    pub fn reflection_coefficient(
        load_impedance: f64,
        characteristic_impedance: f64,
    ) -> Option<f64> {
        if !load_impedance.is_finite()
            || !characteristic_impedance.is_finite()
            || characteristic_impedance <= 0.0
        {
            return None;
        }
        let gamma = (load_impedance - characteristic_impedance)
            / (load_impedance + characteristic_impedance);
        Some(gamma)
    }

    /// Voltage standing wave ratio: VSWR = (1+|Γ|)/(1-|Γ|)

    pub fn vswr(reflection_coeff: f64) -> Option<f64> {
        if !reflection_coeff.is_finite() || reflection_coeff.abs() >= 1.0 {
            return None;
        }
        Some((1.0 + reflection_coeff.abs()) / (1.0 - reflection_coeff.abs()))
    }

    /// Return loss: RL = -20 log₁₀ |Γ| (dB)

    pub fn return_loss(reflection_coeff: f64) -> Option<f64> {
        if !reflection_coeff.is_finite()
            || reflection_coeff.abs() <= 0.0
            || reflection_coeff.abs() >= 1.0
        {
            return None;
        }
        Some(-20.0 * reflection_coeff.abs().log10())
    }

    /// Quarter-wave transformer impedance: Z_q = sqrt(Z_0 · Z_L)

    pub fn quarter_wave_transformer(z0: f64, z_load: f64) -> Option<f64> {
        if !z0.is_finite() || z0 <= 0.0 || !z_load.is_finite() || z_load <= 0.0 {
            return None;
        }
        Some((z0 * z_load).sqrt())
    }

    /// Transmission line input impedance: Z_in = Z_0 · (Z_L + j·Z_0·tan(βl)) / (Z_0 + j·Z_L·tan(βl))
    /// Returns (real, imag) for lossless case.

    pub fn transmission_line_input_impedance(
        z0: f64,
        z_load_real: f64,
        z_load_imag: f64,
        phase_constant: f64,
        length: f64,
    ) -> Option<(f64, f64)> {
        if !z0.is_finite()
            || z0 <= 0.0
            || !z_load_real.is_finite()
            || !z_load_imag.is_finite()
            || !phase_constant.is_finite()
            || !length.is_finite()
        {
            return None;
        }
        let tan_bl = (phase_constant * length).tan();
        let num_real = z_load_real;
        let num_imag = z_load_imag + z0 * tan_bl;
        let den_real = z0 - z_load_imag * tan_bl;
        let den_imag = z_load_real * tan_bl;
        let den_sq = den_real * den_real + den_imag * den_imag;
        if den_sq <= 0.0 {
            return None;
        }
        let z_in_real = z0 * (num_real * den_real + num_imag * den_imag) / den_sq;
        let z_in_imag = z0 * (num_imag * den_real - num_real * den_imag) / den_sq;
        Some((z_in_real, z_in_imag))
    }

    /// Coaxial cable characteristic impedance: Z_0 = (60/√ε_r) · ln(D/d)

    pub fn coaxial_impedance(
        inner_diameter: f64,
        outer_diameter: f64,
        relative_permittivity: f64,
    ) -> Option<f64> {
        if !inner_diameter.is_finite()
            || !outer_diameter.is_finite()
            || !relative_permittivity.is_finite()
        {
            return None;
        }
        if inner_diameter <= 0.0 || outer_diameter <= inner_diameter || relative_permittivity <= 0.0
        {
            return None;
        }
        Some(60.0 / relative_permittivity.sqrt() * (outer_diameter / inner_diameter).ln())
    }

    /// Coaxial cable cutoff frequency (TE11 mode): f_c ≈ c/(π·(D+d)/2 · √ε_r)

    pub fn coaxial_cutoff_frequency(
        inner_diameter: f64,
        outer_diameter: f64,
        relative_permittivity: f64,
    ) -> Option<f64> {
        if !inner_diameter.is_finite()
            || !outer_diameter.is_finite()
            || !relative_permittivity.is_finite()
        {
            return None;
        }
        if inner_diameter <= 0.0 || outer_diameter <= inner_diameter || relative_permittivity <= 0.0
        {
            return None;
        }
        let c = 299_792_458.0;
        let mean_diameter = 0.5 * (inner_diameter + outer_diameter);
        Some(c / (std::f64::consts::PI * mean_diameter * relative_permittivity.sqrt()))
    }

    /// Rayleigh scattering cross-section for a small dielectric sphere.
    /// σ_s = (8π³/3) · ((n²-1)/(n²+2))² · (d/2)⁶ / λ⁴

    pub fn rayleigh_scattering_cross_section(
        refractive_index: f64,
        diameter: f64,
        wavelength: f64,
    ) -> Option<f64> {
        if !refractive_index.is_finite() || !diameter.is_finite() || !wavelength.is_finite() {
            return None;
        }
        if refractive_index <= 0.0 || diameter <= 0.0 || wavelength <= 0.0 {
            return None;
        }
        let r = diameter / 2.0;
        let polarizability = (refractive_index * refractive_index - 1.0)
            / (refractive_index * refractive_index + 2.0);
        Some(
            8.0 * std::f64::consts::PI.powi(3) / 3.0 * polarizability.powi(2) * r.powi(6)
                / wavelength.powi(4),
        )
    }

    /// Faraday rotation angle: θ = V · B · L
    /// V = Verdet constant (rad/(T·m)), B = magnetic field along path (T), L = path length (m)

    pub fn faraday_rotation(
        verdet_constant: f64,
        magnetic_field: f64,
        path_length: f64,
    ) -> Option<f64> {
        if !verdet_constant.is_finite() || !magnetic_field.is_finite() || !path_length.is_finite() {
            return None;
        }
        Some(verdet_constant * magnetic_field * path_length)
    }
}
