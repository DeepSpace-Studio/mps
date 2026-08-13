//! High-energy astrophysics — pulsars, magnetars, X-ray binaries, AGN, GRBs.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W5).
//! Pure-formula layer: no `WorldHandle` and no Rapier state.

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::finite_positive;

const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Characteristic spin-down age of an isolated pulsar `τ = P / (2 · Ṗ)`,
/// i.e. the inferred age if the current spin is fully attributable to
/// magnetic-dipole braking from an initial period ≪ P (a limit that's not
/// quite Crab-true but used universally for order-of-magnitude estimates).
///
/// Inputs:
/// - `period_ms`        — pulse period P in ms (e.g. Crab ≈ 33 ms)
/// - `period_derivative` — period derivative Ṗ in s/s (Crab ≈ 4.2e-13)
///
/// Returns age in years (1 yr ≈ 3.15576e7 s).
pub fn pulsar_characteristic_age(period_ms: f64, period_derivative: f64) -> Option<f64> {
    if !finite_positive(period_ms) || !finite_positive(period_derivative) {
        set_error(ERR_INVALID_ARGUMENT, "bad pulsar age args");
        return None;
    }
    let p_s = period_ms * 1.0e-3;
    let age_s = p_s / (2.0 * period_derivative);
    Some(age_s / 3.15576e7)
}

/// Pulsar spin-down luminosity (a.k.a. spin-down power) radiated by magnetic
/// dipole braking:
///     Ė = 4π² · I · Ṗ / P³
/// where I is the star's moment of inertia (default 1.0e38 kg·m² for a
/// canonical 1.4 Msun neutron star).
///
/// Inputs:
/// - `moment_of_inertia` — I in kg·m² (canonical 1e38)
/// - `period_ms`        — P in ms
/// - `period_derivative` — Ṗ in s/s
///
/// Returns spin-down power in watts (J/s).
pub fn pulsar_spin_down_luminosity(
    moment_of_inertia: f64,
    period_ms: f64,
    period_derivative: f64,
) -> Option<f64> {
    if !finite_positive(moment_of_inertia)
        || !finite_positive(period_ms)
        || !finite_positive(period_derivative)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad pulsar spin-down args");
        return None;
    }
    let p_s = period_ms * 1.0e-3;
    let p_cubed = p_s * p_s * p_s;
    Some(
        4.0 * core::f64::consts::PI * core::f64::consts::PI * moment_of_inertia * period_derivative
            / p_cubed,
    )
}

/// Magnetic-dipole braking surface B-field strength inferred from pulsar
/// spin-down.  Assuming a vacuum orthogonal rotator, the surface equatorial
/// field in SI units is:
///     B² = 3 μ₀ c³ I P Ṗ / (32 π³ R⁶)
/// (Equating the SI dipole-radiation power L = μ₀ m² ω⁴ / (6π c³) with the
/// rotational energy loss 4π² I Ṗ / P³, with m = 4π R³ B / μ₀.)
/// Inputs:
/// - `moment_of_inertia` [kg·m²]
/// - `radius_m`          neutron-star radius in metres (canonical 1e4 m →
///   10 km radius)
/// - `period_ms`         P in ms
/// - `period_derivative` Ṗ in s/s
///   Returns B in Tesla at the magnetic equator.  Matches the canonical
///   B_s = 3.2e19·√(P·Ṗ) G to <0.1 %.  Typical neutron-star
///   fields: 1e4 T (recycled millisecond pulsars) → 1e8 T (young pulsars) →
///   1e10-1e11 T (magnetars).
pub fn pulsar_surface_b_field(
    moment_of_inertia: f64,
    radius_m: f64,
    period_ms: f64,
    period_derivative: f64,
) -> Option<f64> {
    if !finite_positive(moment_of_inertia)
        || !finite_positive(radius_m)
        || !finite_positive(period_ms)
        || !finite_positive(period_derivative)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad pulsar B-field args");
        return None;
    }
    const MU0: f64 = 4.0e-7 * core::f64::consts::PI; // vacuum permeability [N/A²]
    let p_s = period_ms * 1.0e-3;
    let r6 = radius_m.powi(6);
    let b_sq = 3.0 * MU0 * SPEED_OF_LIGHT.powi(3) * moment_of_inertia * p_s * period_derivative
        / (32.0 * core::f64::consts::PI.powi(3) * r6);
    Some(b_sq.sqrt())
}

/// Eddington-limited bolometric luminosity for a spherically symmetric
/// accretor (Thorne 1974 Kerr correction not applied here):
///     L_Edd = 4π G M c / κ
/// Returns the Eddington luminosity in watts.
///
/// Inputs:
/// - `mass_kg`    — accretor mass in kg (e.g. 10 Msun = 1.99e31 kg)
/// - `opacity`    — opacity κ in m²/kg (electron scattering ≈ 0.034 m²/kg
///   for fully ionised H; 0.034·(1+X) for general composition)
pub fn eddington_limited_luminosity(mass_kg: f64, opacity: f64) -> Option<f64> {
    if !finite_positive(mass_kg) || !finite_positive(opacity) {
        set_error(ERR_INVALID_ARGUMENT, "bad Eddington luminosity args");
        return None;
    }
    const G: f64 = 6.67430e-11;
    Some(4.0 * core::f64::consts::PI * G * mass_kg * SPEED_OF_LIGHT / opacity)
}

/// GRB afterglow flux at observer time `t_days` from an isotropic-equivalent
/// blast-wave model with energy `e_iso_erg` and circumburst density `n_cm3`.
/// Returns flux in erg/s (very rough closure-relation approximation, intended
/// only for order-of-magnitude work — a complete afterglow needs multiple
/// spectral segments and synchrotron self-absorption).
pub fn grb_afterglow_flux_simple(t_days: f64, e_iso_erg: f64, n_cm3: f64) -> Option<f64> {
    if !finite_positive(t_days) || !finite_positive(e_iso_erg) || !finite_positive(n_cm3) {
        set_error(ERR_INVALID_ARGUMENT, "bad GRB afterglow args");
        return None;
    }
    // Simplified constant-density-medium afterglow: F ∝ E · n / t
    Some(e_iso_erg * n_cm3 / t_days)
}

/// X-ray binary disc bolometric luminosity of a soft-state multi-temperature
/// disc (Shakura-Sunyaev) with inner-edge temperature `kT_eff` [keV] and
/// inner radius `r_in` [km], corrected for spectral hardening `f_col`.
/// Returns luminosity in solar luminosities.
pub fn xray_disc_bolometric_luminosity(
    k_t_eff_kev: f64,
    r_in_km: f64,
    spectral_hardening: f64,
) -> Option<f64> {
    const KEV_TO_K: f64 = 1.160451812e7; // 1 keV/k_B in K
    const SIGMA_SB: f64 = 5.670374419e-8;
    const L_SUN: f64 = 3.828e26;
    if !finite_positive(k_t_eff_kev)
        || !finite_positive(r_in_km)
        || !finite_positive(spectral_hardening)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad X-ray disc luminosity args");
        return None;
    }
    let t_kelvin = k_t_eff_kev * KEV_TO_K * spectral_hardening;
    let r_m = r_in_km * 1000.0;
    let l_w = 4.0 * core::f64::consts::PI * r_m * r_m * SIGMA_SB * t_kelvin.powi(4);
    Some(l_w / L_SUN)
}
