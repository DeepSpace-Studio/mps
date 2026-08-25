//! Planetary science — atmospheres, interiors, exoplanets, magma oceans.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W7).
//! Pure-computation module: no `WorldHandle`, no Rapier state.

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::{finite, finite_positive};

const SIGMA_SB: f64 = 5.670374419e-8;

/// Single-layer grey greenhouse equilibrium surface temperature.
///
///   σ · T_eff⁴ = (1 - A) · S / 4
///   T_s = T_eff · (1 + 3 τ/4)^(1/4)
///
/// where `S` is solar irradiance at the planet's orbit (W/m²), `A` is the
/// Bond albedo (dimensionless), and `τ` is the infrared optical depth of
/// the grey atmosphere.
///
/// Inputs:
/// - `solar_irradiance` [W/m²] at the planet's orbit
/// - `albedo`            Bond albedo (must lie in `[0, 1]`)
/// - `infrared_optical_depth` ≥ 0  (1-T_eff scaling uses `3 τ/4`)
///
/// Earth: S=1361, A=0.30, τ≈0.78 → T_s ≈ 288 K (canonical surface temperature).
pub fn greenhouse_simple_temperature(
    solar_irradiance: f64,
    albedo: f64,
    infrared_optical_depth: f64,
) -> Option<f64> {
    if !finite(solar_irradiance)
        || solar_irradiance < 0.0
        || !finite(albedo)
        || !(0.0..=1.0).contains(&albedo)
        || !finite(infrared_optical_depth)
        || infrared_optical_depth < 0.0
    {
        set_error(ERR_INVALID_ARGUMENT, "bad greenhouse temperature args");
        return None;
    }
    let t_eff = ((1.0 - albedo) * solar_irradiance / 4.0 / SIGMA_SB).powf(0.25);
    Some(t_eff * (1.0 + 0.75 * infrared_optical_depth).powf(0.25))
}

/// Kasting (1988) runaway greenhouse threshold — inner edge of the habitable
/// zone where the wet upper atmosphere no longer radiates efficiently and an
/// ocean is photodissociated.
///
/// Returns the threshold solar irradiance above which a planet with an
/// Earth-like atmosphere enters runaway moist greenhouse, approximated as a
/// flux multiple (f_run ≈ 1.1 for Earth today).  Output in W/m².
pub fn runaway_greenhouse_threshold_flux(
    reference_irradiance: f64,
    runaway_factor: f64,
) -> Option<f64> {
    if !finite_positive(reference_irradiance) || !finite_positive(runaway_factor) {
        set_error(ERR_INVALID_ARGUMENT, "bad runaway greenhouse args");
        return None;
    }
    Some(runaway_factor * reference_irradiance)
}

/// Kopparapy et al. (2013) habitable-zone inner/outer boundaries for a planet
/// orbiting a star of luminosity `stellar_luminosity_solar` [L_sun].  Uses a
/// simplified flux-balance approximation `r = sqrt(L / S)` (good within ~5 %
/// of the polynomial fit for Sun-like stars).
///
/// Inputs:
/// - `stellar_luminosity_solar` stellar luminosity in L_sun
/// - `inner_edge_factor` inner edge flux multiple ≈ 1.0 (recent-Venus)
/// - `outer_edge_factor` outer edge flux multiple ≈ 0.36 (early-Mars)
///
/// Returns `(inner_a_au, outer_a_au)` semi-major-axis bounds in AU.
pub fn habitable_zone_separation(
    stellar_luminosity_solar: f64,
    inner_edge_factor: f64,
    outer_edge_factor: f64,
) -> Option<(f64, f64)> {
    if !finite_positive(stellar_luminosity_solar)
        || !finite_positive(inner_edge_factor)
        || !finite_positive(outer_edge_factor)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad habitable zone args");
        return None;
    }
    // flux balance: S = L / r² ⇒ r = sqrt(L / S)
    let inner_au = (stellar_luminosity_solar / inner_edge_factor).sqrt();
    let outer_au = (stellar_luminosity_solar / outer_edge_factor).sqrt();
    Some((inner_au, outer_au))
}

/// Tidal dissipation heating inside a synchronously-rotating satellite (e.g.
/// Io, Enceladus):
///     Ė = (63/4) · (G · M_p² · R_s⁵ · n · e²) / (Q · a⁶)
/// where `M_p` is the primary mass [kg], `R_s` the secondary radius [m],
/// `n` the mean motion [rad/s], `e` the orbital eccentricity, `Q` the
/// dissipation function (typically 10-1000), and `a` the orbital semi-major
/// axis [m].  Returns heating power in W.
pub fn tidal_heating_power(
    primary_mass_kg: f64,
    satellite_radius_m: f64,
    mean_motion: f64,
    eccentricity: f64,
    dissipation_q: f64,
    semi_major_axis_m: f64,
) -> Option<f64> {
    const G: f64 = 6.67430e-11;
    if !finite_positive(primary_mass_kg)
        || !finite_positive(satellite_radius_m)
        || !finite_positive(mean_motion)
        || !finite(eccentricity)
        || !(0.0..1.0).contains(&eccentricity)
        || !finite_positive(dissipation_q)
        || !finite_positive(semi_major_axis_m)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad tidal heating args");
        return None;
    }
    let r5 = satellite_radius_m.powi(5);
    let a6 = semi_major_axis_m.powi(6);
    Some(
        (63.0 / 4.0)
            * G
            * primary_mass_kg
            * primary_mass_kg
            * r5
            * mean_motion
            * eccentricity
            * eccentricity
            / (dissipation_q * a6),
    )
}

/// Magma-ocean solidification timescale (Solomatov 2007 simplified): the
/// cooling time for a fully molten silicate mantle of thickness `D` to
/// solidify while radiating from the surface.
///
/// The energy to remove per unit surface area is the sensible heat of the
/// temperature drop plus the latent heat of crystallisation:
///     E/A = ρ · D · (c_p · ΔT + L_lat)          [J/m²]
/// and the net radiative loss is
///     F   = σ · (T_surf⁴ - T_eq⁴)               [W/m²]
/// so `t_solidify ≈ (E/A) / F`.
///
/// Inputs:
/// - `mantle_density`        ρ [kg/m³]
/// - `specific_heat`         c_p [J/(kg·K)]
/// - `latent_heat_per_mass`  L_lat [J/kg]
/// - `mantle_thickness_m`    D [m]
/// - `temperature_drop_k`    ΔT, mantle temperature decrease over
///   solidification [K]
/// - `surface_temperature_k` T_surf [K] (must exceed `equilibrium_temp_k`)
/// - `equilibrium_temp_k`    T_eq [K]
///
/// Returns seconds.  This is a heavily simplified estimate; real magma
/// oceans convect and crystalise heterogeneously.
pub fn magma_ocean_solidification_timescale(
    mantle_density: f64,
    specific_heat: f64,
    latent_heat_per_mass: f64,
    mantle_thickness_m: f64,
    temperature_drop_k: f64,
    surface_temperature_k: f64,
    equilibrium_temp_k: f64,
) -> Option<f64> {
    if !finite_positive(mantle_density)
        || !finite_positive(specific_heat)
        || !finite_positive(latent_heat_per_mass)
        || !finite_positive(mantle_thickness_m)
        || !finite_positive(temperature_drop_k)
        || !finite_positive(surface_temperature_k)
        || !finite_positive(equilibrium_temp_k)
        || surface_temperature_k <= equilibrium_temp_k
    {
        set_error(ERR_INVALID_ARGUMENT, "bad magma ocean args");
        return None;
    }
    // Sensible heat (c_p·ΔT) and latent heat (L_lat) are energies per unit
    // mass — they add; the areal energy density is ρ·D·(…).
    let energy = mantle_density
        * mantle_thickness_m
        * (specific_heat * temperature_drop_k + latent_heat_per_mass);
    let radiative = SIGMA_SB * (surface_temperature_k.powi(4) - equilibrium_temp_k.powi(4));
    Some(energy / radiative)
}
