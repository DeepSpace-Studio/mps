//! Galactic dynamics & interstellar medium.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W8).
//! Pure-computation module: no `WorldHandle` and no Rapier state.  Distances
//! use parsec (pc) conventions; masses use solar masses (Msun); velocities
//! use km/s.

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::{finite, finite_positive};

/// Gravitational constant `G` in galactic units: (km/s)² pc / Msun.
/// SI: G = 6.67430e-11 m³ kg^-1 s^-2.
/// Convert: 1 pc = 3.0856775814913673e16 m, 1 Msun = 1.98847e30 kg,
/// 1 km/s = 1000 m/s.  ⇒ G_units = G · Msun / (pc · (km/s)²) ≈ 4.302e-3.
const G_UNITS: f64 = 4.302e-3;

/// Toomre Q stability parameter for a rotating thin disk:
///     Q = σ · κ / (π · G · Σ)
/// `Q < 1` ⇒ local disk unstable to axisymmetric spiral perturbations,
/// `Q ≈ 2` ⇒ marginal stability (Milky Way solar vicinity), `Q ≫ 1` ⇒ stable.
///
/// Inputs:
/// - `velocity_dispersion` [km/s]               — gas or stellar velocity
///   dispersion in the radial
///   direction
/// - `epicyclic_freq`     [(km/s)/pc]           — κ = (4 Ω² + R · dΩ²/dR)^1/2;
///   must use the same length
///   unit as `G_UNITS` (pc) for Q
///   to be dimensionless.  Solar
///   neighborhood κ ≈ 37
///   (km/s)/kpc = 0.037 (km/s)/pc.
/// - `surface_density`    [Msun/pc²]            — Σ of the disk component
pub fn toomre_q(
    velocity_dispersion: f64,
    epicyclic_freq: f64,
    surface_density: f64,
) -> Option<f64> {
    if !finite_positive(velocity_dispersion)
        || !finite_positive(epicyclic_freq)
        || !finite_positive(surface_density)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Toomre Q args");
        return None;
    }
    Some(velocity_dispersion * epicyclic_freq / (core::f64::consts::PI * G_UNITS * surface_density))
}

/// Dynamical friction (Chandrasekhar 1943) deceleration of a massive body `M`
/// moving at velocity `v` through a background of light tracer particles of
/// density `ρ`:
///     a_df = -(4π G² M ρ / v²) · ln Λ · v_hat
/// where `ln Λ` is the Coulomb logarithm (≈2-10 in galactic contexts).
///
/// Inputs:
/// - `mass_kg`             — satellite mass in kg
/// - `background_density_kg_m3` — ρ in kg/m³
/// - `velocity_ms`         — |v| in m/s
/// - `coulomb_log`         — ln Λ, dimensionless, typical 2-10
///
/// Returns the magnitude of the deceleration in m/s².
pub fn chandrasekhar_dynamical_friction(
    mass_kg: f64,
    background_density_kg_m3: f64,
    velocity_ms: f64,
    coulomb_log: f64,
) -> Option<f64> {
    const G: f64 = 6.67430e-11;
    if !finite_positive(mass_kg)
        || !finite_positive(background_density_kg_m3)
        || !finite_positive(velocity_ms)
        || !finite(coulomb_log)
        || coulomb_log <= 0.0
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Chandrasekhar friction args");
        return None;
    }
    Some(
        4.0 * core::f64::consts::PI * G * G * mass_kg * background_density_kg_m3 * coulomb_log
            / (velocity_ms * velocity_ms),
    )
}

/// MOND (Milgrom 1983) acceleration: at accelerations below the MOND scale
/// `a_0 ≈ 1.2e-10 m/s²`, the Newtonian acceleration `a_N` is boosted to
///     a_MOND = sqrt(a_N · a_0)
/// producing flat galaxy rotation curves without invoking dark matter in
/// the kinematic regime.
///
/// Inputs:
/// - `newtonian_acceleration` — a_N in m/s² (must be > 0)
/// - `mond_a_zero`            — a_0 in m/s² (typical 1.2e-10)
///
/// Returns the MOND-corrected acceleration in m/s².
pub fn mond_acceleration(newtonian_acceleration: f64, mond_a_zero: f64) -> Option<f64> {
    if !finite_positive(newtonian_acceleration) || !finite_positive(mond_a_zero) {
        set_error(ERR_INVALID_ARGUMENT, "bad MOND accel args");
        return None;
    }
    Some((newtonian_acceleration * mond_a_zero).sqrt())
}

/// Free-fall (dynamical) timescale `t_ff = sqrt(3π / (32 G ρ))` of a uniform
/// density cloud undergoing gravitational collapse.  Inputs:
/// - `density_kg_m3` — ρ in kg/m³
///   Returns seconds until collapse.
pub fn free_fall_timescale(density_kg_m3: f64) -> Option<f64> {
    const G: f64 = 6.67430e-11;
    if !finite_positive(density_kg_m3) {
        set_error(ERR_INVALID_ARGUMENT, "bad free-fall density");
        return None;
    }
    Some((3.0 * core::f64::consts::PI / (32.0 * G * density_kg_m3)).sqrt())
}

/// HII region Strömgren radius — the equilibrium radius at which the
/// ionisation front of an O/B star stalls against recombinations:
///     R_S = (3 · ṅ_ion / (4π · α_B · n_H²))^(1/3)
/// Inputs:
/// - `ionising_photon_rate` — ṅ_ion (s^-1; O star ~ 1e49 s^-1)
/// - `recombination_coeff` — α_B (m³ s^-1; case B ≈ 2.6e-19 at 1e4 K)
/// - `hydrogen_density`    — n_H (m^-3)
///   Returns the Strömgren sphere radius in metres.
pub fn stromgren_radius(
    ionising_photon_rate: f64,
    recombination_coeff: f64,
    hydrogen_density: f64,
) -> Option<f64> {
    if !finite_positive(ionising_photon_rate)
        || !finite_positive(recombination_coeff)
        || !finite_positive(hydrogen_density)
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Strömgren radius args");
        return None;
    }
    let r3 = 3.0 * ionising_photon_rate
        / (4.0 * core::f64::consts::PI * recombination_coeff * hydrogen_density * hydrogen_density);
    Some(r3.cbrt())
}
