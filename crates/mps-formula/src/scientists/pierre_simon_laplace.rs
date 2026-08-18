//! Pierre-Simon Laplace —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "pierre_simon_laplace",
    name: "Pierre-Simon Laplace",
    birth_year: Some(1749),
    death_year: Some(1827),
    field_id: "astro",
    nationality: "French",
    contribution: "Celestial mechanics; Laplace eq & transform",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::math::*;
    pub const G: f64 = 6.67430e-11;
    const HAWKING_HBAR: f64 = 1.054_571_817e-34;
    const HAWKING_KB: f64 = 1.380_649e-23;
    const METRES_PER_MEGAPARSEC: f64 = 1.0e6 * METRES_PER_PARSEC;
    const METRES_PER_PARSEC: f64 = 3.085_677_581_491_367e16;
    pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

    /// Gravitational lensing Einstein radius for point mass.

    pub fn einstein_radius(
        mass_kg: f64,
        dist_lens: f64,
        dist_source: f64,
        dist_ls: f64,
    ) -> Option<f64> {
        let g = 6.67430e-11;
        let c = 299_792_458.0;
        if !mass_kg.is_finite()
            || mass_kg <= 0.0
            || !dist_lens.is_finite()
            || dist_lens <= 0.0
            || !dist_source.is_finite()
            || dist_source <= 0.0
            || !dist_ls.is_finite()
            || dist_ls <= 0.0
        {
            return None;
        }
        Some((4.0 * g * mass_kg / (c * c) * dist_ls / (dist_lens * dist_source)).sqrt())
    }

    /// Hawking temperature of a Schwarzschild black hole:
    /// T = ħ·c³ / (8·π·G·M·k_B).

    pub fn hawking_temperature(mass: f64, g: f64) -> Option<f64> {
        if !mass.is_finite() || mass <= 0.0 || !g.is_finite() || g <= 0.0 {
            return None;
        }
        Some(
            HAWKING_HBAR * SPEED_OF_LIGHT.powi(3)
                / (8.0 * std::f64::consts::PI * g * mass * HAWKING_KB),
        )
    }

    /// Hubble-law recession velocity: v = H₀·d.

    pub fn hubble_recession_velocity(distance: f64, hubble_constant: f64) -> Option<f64> {
        if !distance.is_finite()
            || distance < 0.0
            || !hubble_constant.is_finite()
            || hubble_constant <= 0.0
        {
            return None;
        }
        Some(hubble_constant * distance)
    }

    /// Hubble-law luminosity distance from redshift (low-z): d = c·z / H₀.

    pub fn hubble_distance(redshift: f64, hubble_constant: f64) -> Option<f64> {
        if !redshift.is_finite()
            || redshift < 0.0
            || !hubble_constant.is_finite()
            || hubble_constant <= 0.0
        {
            return None;
        }
        Some(SPEED_OF_LIGHT * redshift / hubble_constant)
    }

    /// Flat matter-dominated universe lookback time:
    /// t_L = (2/3)·t_H·(1 − 1/√(1+z)), where t_H = 1/H₀ is the Hubble time.

    pub fn flat_universe_lookback_time(redshift: f64, hubble_time: f64) -> Option<f64> {
        if !redshift.is_finite() || redshift < 0.0 || !hubble_time.is_finite() || hubble_time <= 0.0
        {
            return None;
        }
        let factor = 1.0 + redshift;
        Some((2.0 / 3.0) * hubble_time * (1.0 - 1.0 / factor.sqrt()))
    }

    /// Flat-ΛCDM line-of-sight comoving distance [Mpc] for small redshifts using
    /// the z ≪ 1 Hubble approximation `D_C ≈ c · z / H0`.
    ///
    /// - `hubble_constant` in km/s/Mpc (a.k.a. `H0`; typical 67.4 for Planck18,
    ///   70 for the classical "Hubble Key Project" value, 73 for SH0ES)
    /// - `redshift` dimensionless and ≥ 0
    ///
    /// Returns the comoving distance in Mpc.  Inputs are validated for
    /// finiteness and positivity; failures set `ERR_INVALID_ARGUMENT` and return
    /// `None`.

    pub fn friedmann_hubble_distance(hubble_constant: f64, redshift: f64) -> Option<f64> {
        if !finite_positive(hubble_constant) || !finite(redshift) || redshift < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "bad cosmology arguments");
            return None;
        }
        // Convert H0 from km/s/Mpc → 1/s via dimensional analysis:
        //   H0 [km/s/Mpc] · 1000 [m/km] / (Mpc in metres) = H0 in 1/s.
        let h0_si = hubble_constant * 1000.0 / METRES_PER_MEGAPARSEC;
        // D_C = c · z / H0  (metres) → divide back by METRES_PER_MEGAPARSEC.
        let dist_m = SPEED_OF_LIGHT * redshift / h0_si;
        Some(dist_m / METRES_PER_MEGAPARSEC)
    }

    /// Luminosity distance `D_L = (1 + z) · D_C` under the flat-ΛCDM small-z
    /// approximation.  Useful for converting apparent magnitude to absolute
    /// magnitude at low redshift before the proper cosmological integral is
    /// needed.  Inputs in the same units as [`friedmann_hubble_distance`].

    pub fn luminosity_distance_hubble(hubble_constant: f64, redshift: f64) -> Option<f64> {
        let d_c = friedmann_hubble_distance(hubble_constant, redshift)?;
        Some((1.0 + redshift) * d_c)
    }

    /// Einstein-de Sitter (matter-only flat universe) cosmic age:
    /// `t0 = 2 / (3 · H0)`.
    /// `hubble_constant` in km/s/Mpc; returns age in gigayears (1 Gyr = 1e9 yr).

    pub fn einstein_de_sitter_age(hubble_constant: f64) -> Option<f64> {
        if !finite_positive(hubble_constant) {
            set_error(ERR_INVALID_ARGUMENT, "bad H0 for Einstein-de Sitter age");
            return None;
        }
        let h0_si = hubble_constant * 1000.0 / METRES_PER_MEGAPARSEC;
        let age_s = 2.0 / (3.0 * h0_si);
        Some(age_s / 3.15576e16) // → Gyr (1 Gyr ≈ 3.15576e16 s)
    }

    /// Hubble flow recession velocity `v = H0 · D` for sub-luminal small-distance
    /// regime (`D < c / H0`).  Inputs: `hubble_constant` [km/s/Mpc],
    /// `distance_mpc` [Mpc]; returns `v` [km/s].

    pub fn hubble_flow_velocity(hubble_constant: f64, distance_mpc: f64) -> Option<f64> {
        if !finite_positive(hubble_constant) || !finite(distance_mpc) || distance_mpc < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "bad Hubble flow args");
            return None;
        }
        Some(hubble_constant * distance_mpc)
    }
}
