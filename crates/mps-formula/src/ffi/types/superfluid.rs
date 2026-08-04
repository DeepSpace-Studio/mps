use super::core::*;
// ---------------------------------------------------------------------------
// Superfluidity / quantum vortex structures
// ---------------------------------------------------------------------------

/// A single quantum vortex segment (straight line in 3D).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VortexSegment {
    pub start: Vec3,
    pub end: Vec3,
    /// Circulation quantum number (integer)
    pub circulation_quantum: i32,
    /// Core radius (healing length)
    pub core_radius: f64,
}

/// Velocity induced by a vortex segment at a field point (Biot–Savart kernel).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BiotSavartVelocity {
    pub velocity: Vec3,
    pub magnitude: f64,
    /// Distance from segment to field point
    pub distance: f64,
}

/// Gross–Pitaevskii order parameter (condensate wavefunction) at a point.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpOrderParameter {
    pub amplitude: f64,
    pub phase: f64,
    /// Superfluid density n = |ψ|²
    pub density: f64,
}

/// Gross–Pitaevskii chemical potential / energy density report.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpEnergyDensity {
    pub kinetic_density: f64,
    pub interaction_density: f64,
    pub trapping_density: f64,
    pub total_density: f64,
    pub chemical_potential: f64,
}

/// State of a single quantum vortex ring (circular vortex line).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VortexRing {
    pub center: Vec3,
    /// Radius of the ring
    pub radius: f64,
    /// Circulation quantum number
    pub circulation_quantum: i32,
    /// Orientation axis (unit vector)
    pub axis: Vec3,
    /// Translational velocity (self-induced)
    pub velocity: Vec3,
}

/// Report from a vortex reconnection event.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VortexReconnectionReport {
    /// Distance between two segments before reconnection
    pub closest_approach: f64,
    /// Whether a reconnection occurred
    pub reconnected: Bool,
    /// Post-reconnection segment 1 start
    pub seg1_start: Vec3,
    pub seg1_end: Vec3,
    /// Post-reconnection segment 2 start
    pub seg2_start: Vec3,
    pub seg2_end: Vec3,
    /// Energy dissipated during reconnection
    pub energy_dissipated: f64,
}

/// Quantised circulation around a closed loop.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct QuantisedCirculation {
    /// Circulation κ = n × h/m
    pub circulation: f64,
    /// Quantum number n
    pub quantum_number: i32,
    /// Circulation quantum h/m
    pub circulation_quantum: f64,
    /// Whether the circulation is consistent with quantisation
    pub quantised: Bool,
}

/// Parameters for time-dependent Gross–Pitaevskii integration.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GpTimeEvolutionParams {
    /// Healing length ξ
    pub healing_length: f64,
    /// Speed of sound c
    pub sound_speed: f64,
    /// Chemical potential μ
    pub chemical_potential: f64,
    /// Nonlinear coupling constant g
    pub coupling_constant: f64,
    /// Time step dt
    pub dt: f64,
}

impl Default for GpTimeEvolutionParams {
    fn default() -> Self {
        Self {
            healing_length: 1.0,
            sound_speed: 1.0,
            chemical_potential: 1.0,
            coupling_constant: 1.0,
            dt: 0.01,
        }
    }
}

/// Vortex filament network: a collection of vortex segments forming a tangle.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VortexTangleStats {
    pub segment_count: u32,
    pub total_length: f64,
    pub average_curvature: f64,
    pub total_kinetic_energy: f64,
    pub vortex_line_density: f64,
}

/// A single point in a 2D cross-section of the GP wavefunction (for visualisation).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpGridPoint {
    pub x: f64,
    pub y: f64,
    pub amplitude: f64,
    pub phase: f64,
    pub density: f64,
}
