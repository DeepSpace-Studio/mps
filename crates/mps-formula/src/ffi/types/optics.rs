use super::core::*;
// ---------------------------------------------------------------------------
// Wave optics / diffraction structures
// ---------------------------------------------------------------------------

/// Complex wave amplitude at a point.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ComplexAmplitude {
    pub real: f64,
    pub imag: f64,
    /// Intensity I = |E|²
    pub intensity: f64,
}

/// Parameters for a monochromatic plane wave.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlaneWaveParams {
    /// Wavenumber k = 2π/λ
    pub wavenumber: f64,
    /// Wavelength λ
    pub wavelength: f64,
    /// Initial amplitude A₀
    pub amplitude: f64,
    /// Initial phase φ₀
    pub phase_offset: f64,
}

impl Default for PlaneWaveParams {
    fn default() -> Self {
        Self {
            wavenumber: 2.0 * std::f64::consts::PI / 500e-9,
            wavelength: 500e-9,
            amplitude: 1.0,
            phase_offset: 0.0,
        }
    }
}

/// Huygens–Fresnel diffraction from an aperture (single point).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DiffractionPoint {
    /// Coordinates in the observation plane
    pub x: f64,
    pub y: f64,
    /// Complex amplitude at this point
    pub amplitude: ComplexAmplitude,
}

/// A single point source used in Huygens–Fresnel superposition.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PointSource {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Initial phase at this source point
    pub phase: f64,
    /// Amplitude scaling factor
    pub amplitude: f64,
}

/// Fresnel diffraction zone plate / Fresnel zone parameters.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FresnelZoneReport {
    /// Radius of the n-th Fresnel zone
    pub zone_radius: f64,
    /// Zone index
    pub zone_index: u32,
    /// Phase contribution from this zone
    pub zone_phase: f64,
    /// Whether the zone is constructive (phase within ±π/2 of centre)
    pub constructive: Bool,
}

/// Thin-film interference report (single layer).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ThinFilmInterferenceReport {
    /// Optical path difference
    pub opd: f64,
    /// Phase difference from path
    pub phase_difference: f64,
    /// Reflection coefficient magnitude
    pub reflection_coefficient: f64,
    /// Interference intensity (normalised)
    pub intensity: f64,
    /// Whether half-wave loss occurs (n_film > n_substrate or similar)
    pub half_wave_loss: Bool,
    /// Wavelength for which this report was computed
    pub wavelength: f64,
}

/// Parameters for a thin film.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ThinFilmParams {
    /// Film thickness (m)
    pub thickness: f64,
    /// Film refractive index
    pub n_film: f64,
    /// Substrate refractive index
    pub n_substrate: f64,
    /// Incident medium refractive index (typically 1.0 for air)
    pub n_incident: f64,
    /// Angle of incidence (radians)
    pub incidence_angle: f64,
}

impl Default for ThinFilmParams {
    fn default() -> Self {
        Self {
            thickness: 500e-9,
            n_film: 1.5,
            n_substrate: 1.0,
            n_incident: 1.0,
            incidence_angle: 0.0,
        }
    }
}

/// Fresnel–Kirchhoff diffraction integral result for a single observation point.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct KirchhoffDiffractionPoint {
    pub x: f64,
    pub y: f64,
    pub amplitude: ComplexAmplitude,
    /// Obliquity (inclination) factor cosθ
    pub obliquity_factor: f64,
}

/// A single spherical wave emitted from a point source.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SphericalWavePoint {
    pub amplitude: ComplexAmplitude,
    /// Distance from source
    pub radius: f64,
    /// 1/r amplitude decay factor
    pub decay_factor: f64,
}

/// Describes a planar aperture for diffraction calculations.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ApertureDesc {
    /// Half-width in x (m)
    pub half_width_x: f64,
    /// Half-width in y (m)
    pub half_width_y: f64,
    /// Centre position in the aperture plane
    pub center_x: f64,
    pub center_y: f64,
    /// Transmission coefficient (0=opaque, 1=fully transparent)
    pub transmission: f64,
}

/// Two-slit (Young's) interference pattern at a point.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct YoungSlitPoint {
    pub x: f64,
    pub y: f64,
    /// Phase difference between slits
    pub phase_difference: f64,
    /// Path difference in metres
    pub path_difference: f64,
    /// Interference intensity
    pub intensity: f64,
    /// Envelope (single-slit diffraction) factor
    pub envelope_factor: f64,
}
