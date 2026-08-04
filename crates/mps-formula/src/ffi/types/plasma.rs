use super::core::*;
// ---------------------------------------------------------------------------
// Plasma physics structures
// ---------------------------------------------------------------------------

/// A single macroparticle used in the PIC (particle-in-cell) method.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PicParticle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    /// Charge (C), negative for electrons
    pub charge: f64,
    /// Mass (kg)
    pub mass: f64,
    /// Weight (number of real particles this macroparticle represents)
    pub weight: f64,
}

/// Electromagnetic fields on a 3D grid cell (staggered / Yee-like).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GridField {
    pub ex: f64,
    pub ey: f64,
    pub ez: f64,
    pub bx: f64,
    pub by: f64,
    pub bz: f64,
}

/// Charge density on a grid cell (from particle deposition).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ChargeDensityCell {
    pub rho: f64,
    pub jx: f64,
    pub jy: f64,
    pub jz: f64,
}

/// Debye length and plasma frequency report.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PlasmaParamsReport {
    /// Electron Debye length λ_D = sqrt(ε₀ k_B T_e / (n_e e²))
    pub debye_length: f64,
    /// Electron plasma frequency ω_pe = sqrt(n_e e² / (ε₀ m_e))
    pub plasma_frequency: f64,
    /// Ion plasma frequency ω_pi = sqrt(n_i Z² e² / (ε₀ m_i))
    pub ion_plasma_frequency: f64,
    /// Number of particles in a Debye sphere N_D
    pub debye_sphere_count: f64,
    /// Thermal velocity v_th = sqrt(k_B T_e / m_e)
    pub thermal_velocity: f64,
}

/// Vlasov equation reduced distribution function moment report.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VlasovMomentReport {
    /// Number density n
    pub density: f64,
    /// Bulk velocity u (drift)
    pub ux: f64,
    pub uy: f64,
    pub uz: f64,
    /// Pressure tensor trace / temperature (energy density)
    pub temperature: f64,
    /// Heat flux vector (reduced)
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}

/// Magnetic X-point (reconnection site) report.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MagneticXPoint {
    /// Position of the X-point
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// In-plane magnetic shear angle (radians)
    pub shear_angle: f64,
    /// Reconnection rate estimate (normalised)
    pub reconnection_rate: f64,
    /// Whether this is a valid X-point (B = 0 in the reconnection plane)
    pub valid: Bool,
}

/// PIC simulation step report (self-consistent field solve summary).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PicStepReport {
    pub particle_count: u32,
    pub max_density: f64,
    pub max_electric_field: f64,
    pub max_magnetic_field: f64,
    pub total_kinetic_energy: f64,
    pub total_field_energy: f64,
}

/// Parameters for the Boris particle pusher.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BorisPusherParams {
    pub dt: f64,
    pub charge_to_mass_ratio: f64,
}

impl Default for BorisPusherParams {
    fn default() -> Self {
        Self {
            dt: 1e-12,
            charge_to_mass_ratio: -1.758_820_010e11, // e/m_e for electrons
        }
    }
}
