//! Stellar structure and evolution — polytropic models, Cepheid relations,
//! supernova light curves, white-dwarf cooling.
//!
//! Split out as a new physics domain (PHYSICS_EXPANSION_PLAN.md W4).
//! Lane-Emden here complements the single-shot [`lane_emden_first_zero`] in
//! `crate::astrophysics` by returning the full sampled profile plus the
//! dimensionless mass `-ξ₀²·θ'₀` used for stellar-interior scalings.
//!
//! No `extern "C" fn` lives here (pure formula layer).

use crate::error::{ERR_INVALID_ARGUMENT, set_error};
use crate::math::finite;

/// Integrate the Lane-Emden equation
///     d²θ/dξ² + (2/ξ) dθ/dξ + θ^n = 0
/// from near-ξ=0 outward until θ first crosses zero.  Returns
/// `(ξ_first_zero, θ_profile_samples, dimensionless_mass)` on success.
///
/// `polytropic_index` is the n in `P = K·ρ^(1+1/n)`; `samples` is the cap on
/// the inner `Vec` length (caller may take cost of large buffers but is
/// capped at 10_000 to prevent runaway allocations).
///
/// Standard checks: n=0 analytical → ξ_0 = √6 ≈ 2.449; n=1 → π ≈ 3.1416;
/// n=3 → 6.89685 (n=3 textbook ballpark for full radiative stars).
/// For n ≥ 5 (singular isothermal sphere) θ never crosses zero and ξ_0 = ∞;
/// this solver returns `None` when no zero is found below ξ = 200.
///
/// Uses RK4 with step h = 1e-3; near-zero ξ=dxis handled by the series
/// initial condition `θ(0)=1, θ'(0)=0`, θ ≈ 1 - ξ²/6 + ... for small ξ.
pub fn lane_emden_solve(polytropic_index: f64, samples: u32) -> Option<(f64, Vec<f64>, f64)> {
    if !finite(polytropic_index) || polytropic_index < 0.0 || samples == 0 || samples > 10_000 {
        set_error(ERR_INVALID_ARGUMENT, "bad Lane-Emden solve args");
        return None;
    }
    let n = polytropic_index;
    let h = 1.0e-3;

    // Start just inside ξ=0 to avoid the 2/ξ singularity; use the series
    // form θ(ξ) ≈ 1 - ξ²/6 and θ' ≈ -ξ/3 to seed (any n).
    let mut xi = 1.0e-5;
    let mut theta = 1.0 - xi * xi / 6.0;
    let mut dtheta = -xi / 3.0;

    let mut profile = Vec::with_capacity(samples as usize);
    // One profile sample per RK4 step, capped at `samples`; for high-n runs
    // that take more than `samples` steps to reach ξ_0 the returned profile
    // covers only the inner part of the trajectory.
    let stride: u32 = 1;
    let mut i = 0u32;

    while theta > 0.0 && xi < 200.0 {
        // RHS: dθ/dξ = θ', dθ'/dξ = -θ^n - (2/ξ)·θ'.
        let f = |th: f64, dth: f64, x: f64| -> f64 { -th.powf(n) - 2.0 / x * dth };

        let k1 = dtheta;
        let l1 = f(theta, dtheta, xi);
        let k2 = dtheta + 0.5 * h * l1;
        let l2 = f(theta + 0.5 * h * k1, dtheta + 0.5 * h * l1, xi + 0.5 * h);
        let k3 = dtheta + 0.5 * h * l2;
        let l3 = f(theta + 0.5 * h * k2, dtheta + 0.5 * h * l2, xi + 0.5 * h);
        let k4 = dtheta + h * l3;
        let l4 = f(theta + h * k3, dtheta + h * l3, xi + h);

        theta += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        dtheta += h * (l1 + 2.0 * l2 + 2.0 * l3 + l4) / 6.0;
        xi += h;

        i += 1;
        if i.is_multiple_of(stride) && profile.len() < samples as usize {
            profile.push(theta);
        }
    }

    if theta > 0.0 {
        // No zero crossing below ξ = 200 (e.g. n ≥ 5, the singular
        // isothermal sphere, whose ξ_0 = ∞).  No finite solution to report.
        return None;
    }

    // Linearly interpolate the last crossing for a sharp ξ_first_zero:
    // step back from the current (θ < 0) point along the local slope θ'.
    let xi_zero = xi - theta / (dtheta + 1.0e-30);
    let mass_ratio = -xi_zero * xi_zero * dtheta;
    Some((xi_zero, profile, mass_ratio))
}

/// Cepheid period-luminosity relation (Madore & Freedman 1991, LMC-calibrated):
///     M_V = -2.76 · (log10 P - 1.0) - 4.16   (P in days, M_V absolute V-band)
/// Returns the absolute V magnitude for a Cepheid with pulsation period
/// `period_days`.
pub fn cepheid_period_luminosity(period_days: f64) -> Option<f64> {
    if !finite(period_days) || period_days <= 0.0 {
        set_error(ERR_INVALID_ARGUMENT, "bad Cepheid period");
        return None;
    }
    Some(-2.76 * (period_days.log10() - 1.0) - 4.16)
}

/// White-dwarf Mestel cooling luminosity (1952): a hot WD with no further
/// nuclear burning cools as `L ≈ L0 · (t / t0)^(-7/5)` (electron conduction
/// dominates the deep interior; photon opacity from non-degenerate envelope).
/// Inputs: `t_cool_gyr` cosmic age of the WD since hot birth, `t0_gyr`
/// normalisation constant (~0.001-0.01 typical scale), `l0_solar` initial
/// luminosity in solar units.  Returns the luminosity in solar luminosities.
pub fn white_dwarf_mestel_luminosity(t_cool_gyr: f64, t0_gyr: f64, l0_solar: f64) -> Option<f64> {
    if !finite(t_cool_gyr)
        || t_cool_gyr <= 0.0
        || !finite(t0_gyr)
        || t0_gyr <= 0.0
        || !finite(l0_solar)
        || l0_solar <= 0.0
    {
        set_error(ERR_INVALID_ARGUMENT, "bad Mestel cooling args");
        return None;
    }
    Some(l0_solar * (t_cool_gyr / t0_gyr).powf(-7.0 / 5.0))
}

/// Supernova Arnett (1979) ^56Ni → ^56Co → ^56Fe decay light-curve bolometric
/// luminosity at time `t_days` since explosion:
///     L(t) = M_Ni · (exp(-t/τ_Ni) - exp(-t/τ_Co)) / (τ_Ni · (1/τ_Co - 1/τ_Ni))
/// plus the more slowly fading Co tail term transitioning to ^56Fe.
///
/// `m_ni_solar` is the synthesised ^56Ni mass in solar masses (typical
/// Type Ia ≈ 0.6 Msun).  Returns the bolometric luminosity in solar
/// luminosities at `t_days`.
pub fn sn_arnett_lightcurve(t_days: f64, m_ni_solar: f64) -> Option<f64> {
    // Decay timescales (in days) for 56Ni → 56Co → 56Fe decay chain.
    const TAU_NI_DAYS: f64 = 8.8;
    const TAU_CO_DAYS: f64 = 111.3;
    // Effective luminosity produced per solar mass of 56Ni at t=0.
    // Approximately 6.45e43 erg/s = 1.68e10 Lsun when fully depositing.
    const L_PER_MSUN: f64 = 1.68e10;
    if !finite(t_days) || t_days < 0.0 || !finite(m_ni_solar) || m_ni_solar <= 0.0 {
        set_error(ERR_INVALID_ARGUMENT, "bad Arnett LC args");
        return None;
    }
    if t_days == 0.0 {
        return Some(0.0);
    }
    let e_ni = (-t_days / TAU_NI_DAYS).exp();
    let e_co = (-t_days / TAU_CO_DAYS).exp();
    let denom = TAU_NI_DAYS * (1.0 / TAU_CO_DAYS - 1.0 / TAU_NI_DAYS);
    Some(m_ni_solar * L_PER_MSUN * (e_ni - e_co) / denom)
}
