//! Albert Einstein —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "albert_einstein",
    name: "Albert Einstein",
    birth_year: Some(1879),
    death_year: Some(1955),
    field_id: "relativity",
    nationality: "German/Swiss/American",
    contribution: "Special & general relativity; E=mc²",
    key_constants: "E=mc²",
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

    /// Kerr metric horizon radii.
    /// Returns (r_outer, r_inner) where r = GM/c^2 +- sqrt((GM/c^2)^2 - a^2)
    pub fn kerr_horizon_radii(mass: f64, spin_parameter: f64, g: f64) -> Option<(f64, f64)> {
        let c = 299_792_458.0;
        if !mass.is_finite()
            || mass <= 0.0
            || !spin_parameter.is_finite()
            || spin_parameter < 0.0
            || !g.is_finite()
            || g <= 0.0
        {
            return None;
        }
        let m = g * mass / (c * c); // gravitational radius
        if spin_parameter > m {
            return None;
        } // naked singularity
        let r = (m * m - spin_parameter * spin_parameter).sqrt();
        Some((m + r, m - r))
    }

    /// Kerr ergosphere radius (outer): r_E = m + sqrt(m^2 - a^2 * cos^2(theta))
    pub fn kerr_ergosphere_radius(
        mass: f64,
        spin_parameter: f64,
        polar_angle: f64,
        g: f64,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite()
            || mass <= 0.0
            || !spin_parameter.is_finite()
            || spin_parameter < 0.0
            || !polar_angle.is_finite()
            || !g.is_finite()
            || g <= 0.0
        {
            return None;
        }
        let m = g * mass / (c * c);
        if spin_parameter > m {
            return None;
        }
        let r = (m * m - spin_parameter * spin_parameter * polar_angle.cos().powi(2)).sqrt();
        Some(m + r)
    }

    /// Frame-dragging angular velocity at radius r (the Lense-Thirring effect):
    /// omega = 2 * m * a * r / Sigma^2  where Sigma = r^2 + a^2 * cos^2(theta)
    pub fn kerr_frame_dragging_frequency(
        mass: f64,
        spin_parameter: f64,
        r: f64,
        theta: f64,
        g: f64,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite()
            || mass <= 0.0
            || !spin_parameter.is_finite()
            || spin_parameter < 0.0
            || !r.is_finite()
            || r <= 0.0
            || !theta.is_finite()
            || !g.is_finite()
            || g <= 0.0
        {
            return None;
        }
        let m = g * mass / (c * c);
        let cos2 = theta.cos().powi(2);
        let sigma = r * r + spin_parameter * spin_parameter * cos2;
        Some(2.0 * m * spin_parameter * r / (sigma * sigma))
    }

    /// Innermost Stable Circular Orbit (ISCO) for Schwarzschild: 6 M (3 R_s)
    pub fn schwarzschild_isco(mass: f64, g: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite() || mass <= 0.0 || !g.is_finite() || g <= 0.0 {
            return None;
        }
        Some(6.0 * g * mass / (c * c))
    }

    /// ISCO radius for Kerr (prograde orbit): r_isco/M depends on spin
    /// r_isco(prograde) = M * (3 + Z2 - sqrt((3 - Z1)(3 + Z1 + 2*Z2)))
    /// where Z1 = 1 + (1 - a^2)^(1/3) * ((1+a)^(1/3) + (1-a)^(1/3))
    /// Z2 = sqrt(3*a^2 + Z1^2)
    pub fn kerr_isco(mass: f64, spin_parameter: f64, g: f64, prograde: bool) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite()
            || mass <= 0.0
            || !spin_parameter.is_finite()
            || spin_parameter < 0.0
            || !g.is_finite()
            || g <= 0.0
        {
            return None;
        }
        let m = g * mass / (c * c);
        let a = if prograde {
            spin_parameter.min(m)
        } else {
            -spin_parameter.min(m)
        };
        let a_norm = if m > 0.0 {
            a / m
        } else {
            return None;
        };
        let z1 = 1.0
            + (1.0 - a_norm * a_norm).powf(1.0 / 3.0)
                * ((1.0 + a_norm).powf(1.0 / 3.0) + (1.0 - a_norm).powf(1.0 / 3.0));
        let z2 = (3.0 * a_norm * a_norm + z1 * z1).sqrt();
        let z3 = ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt();
        Some(m * (3.0 + z2 - z3))
    }

    /// Gravitational redshift: z = 1 / sqrt(1 - R_s / r) - 1
    pub fn gravitational_redshift(mass: f64, radius: f64, g: f64) -> Option<f64> {
        let c = 299_792_458.0;
        if !mass.is_finite() || mass <= 0.0 || !radius.is_finite() || !g.is_finite() || g <= 0.0 {
            return None;
        }
        let rs = 2.0 * g * mass / (c * c);
        if radius <= rs {
            return None;
        } // inside horizon
        Some(1.0 / (1.0 - rs / radius).sqrt() - 1.0)
    }

    /// Reissner-Nordstrom horizon radii: r = m +- sqrt(m^2 - Q^2)
    pub fn reissner_nordstrom_horizons(mass: f64, charge: f64, g: f64) -> Option<(f64, f64)> {
        let c = 299_792_458.0;
        let k = 8.9875517923e9;
        if !mass.is_finite()
            || mass <= 0.0
            || !charge.is_finite()
            || charge < 0.0
            || !g.is_finite()
            || g <= 0.0
        {
            return None;
        }
        let m = g * mass / (c * c);
        let q2 = k * g * charge * charge / (c * c * c * c);
        let disc = m * m - q2;
        if disc < 0.0 {
            return None;
        }
        let r = disc.sqrt();
        Some((m + r, m - r))
    }

    /// Characteristic GW strain amplitude from compact binary.
    /// h = (4/d) · (G M_c / c²)^(5/3) · (πf)^(2/3)
    pub fn gw_strain_amplitude(
        distance: f64,
        chirp_mass_kg: f64,
        orbital_frequency: f64,
    ) -> Option<f64> {
        let g = 6.67430e-11;
        let c = 299_792_458.0;
        if !distance.is_finite()
            || distance <= 0.0
            || !chirp_mass_kg.is_finite()
            || chirp_mass_kg <= 0.0
            || !orbital_frequency.is_finite()
            || orbital_frequency <= 0.0
        {
            return None;
        }
        let pi_f = std::f64::consts::PI * orbital_frequency;
        let gm = g * chirp_mass_kg / (c * c);
        Some(4.0 / distance * gm.powf(5.0 / 3.0) * pi_f.powf(2.0 / 3.0))
    }

    /// Chirp mass: M_c = (m₁·m₂)^(3/5) / (m₁+m₂)^(1/5)
    pub fn chirp_mass(mass1: f64, mass2: f64) -> Option<f64> {
        if !mass1.is_finite() || mass1 <= 0.0 || !mass2.is_finite() || mass2 <= 0.0 {
            return None;
        }
        Some((mass1 * mass2).powf(0.6) / (mass1 + mass2).powf(0.2))
    }

    /// GW frequency evolution: df/dt ∝ f^(11/3)
    pub fn gw_frequency_derivative(frequency: f64, chirp_mass_kg: f64) -> Option<f64> {
        let g = 6.67430e-11;
        let c = 299_792_458.0;
        if !frequency.is_finite()
            || frequency <= 0.0
            || !chirp_mass_kg.is_finite()
            || chirp_mass_kg <= 0.0
        {
            return None;
        }
        let mc = g * chirp_mass_kg / (c * c * c);
        Some(
            96.0 / 5.0
                * std::f64::consts::PI.powf(8.0 / 3.0)
                * mc.powf(5.0 / 3.0)
                * frequency.powf(11.0 / 3.0),
        )
    }

    /// Relativistic longitudinal Doppler shift.
    pub fn relativistic_doppler_longitudinal(
        source_frequency: f64,
        relative_velocity: f64,
        approaching: bool,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !source_frequency.is_finite()
            || source_frequency <= 0.0
            || !relative_velocity.is_finite()
            || relative_velocity < 0.0
            || relative_velocity >= c
        {
            return None;
        }
        let beta = relative_velocity / c;
        let shift = ((1.0 - beta) / (1.0 + beta)).sqrt();
        Some(if approaching {
            source_frequency / shift
        } else {
            source_frequency * shift
        })
    }

    /// Relativistic transverse Doppler: f' = f/γ
    pub fn relativistic_doppler_transverse(
        source_frequency: f64,
        relative_velocity: f64,
    ) -> Option<f64> {
        let c = 299_792_458.0;
        if !source_frequency.is_finite()
            || source_frequency <= 0.0
            || !relative_velocity.is_finite()
            || relative_velocity < 0.0
            || relative_velocity >= c
        {
            return None;
        }
        let gamma = 1.0 / (1.0 - (relative_velocity / c).powi(2)).sqrt();
        Some(source_frequency / gamma)
    }

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

    /// Cosmological redshift: z = 1/a - 1
    pub fn cosmological_redshift(scale_factor: f64) -> Option<f64> {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return None;
        }
        Some(1.0 / scale_factor - 1.0)
    }

    /// Redshift from wavelengths: z = (λ_obs - λ_em) / λ_em
    pub fn redshift_from_wavelengths(observed: f64, emitted: f64) -> Option<f64> {
        if !observed.is_finite() || !emitted.is_finite() || emitted <= 0.0 {
            return None;
        }
        Some(observed / emitted - 1.0)
    }

    /// Lense-Thirring frame dragging angular frequency at polar orbit.
    pub fn lense_thirring_angular_frequency(
        mass_kg: f64,
        spin_parameter: f64,
        orbital_radius: f64,
    ) -> Option<f64> {
        let g = 6.67430e-11;
        let c = 299_792_458.0;
        if !mass_kg.is_finite()
            || mass_kg <= 0.0
            || !spin_parameter.is_finite()
            || !orbital_radius.is_finite()
            || orbital_radius <= 0.0
        {
            return None;
        }
        let j = spin_parameter * mass_kg * c;
        Some(2.0 * g * j / (c * c * orbital_radius * orbital_radius * orbital_radius))
    }

    /// Schwarzschild effective potential: V_eff = (1 - r_s/r)(1 + L²/r²)
    pub fn schwarzschild_effective_potential(
        r: f64,
        rs: f64,
        angular_momentum: f64,
    ) -> Option<f64> {
        if !r.is_finite()
            || r <= 0.0
            || !rs.is_finite()
            || rs <= 0.0
            || r <= rs
            || !angular_momentum.is_finite()
        {
            return None;
        }
        Some((1.0 - rs / r) * (1.0 + angular_momentum * angular_momentum / (r * r)))
    }

    /// Matched-filter signal-to-noise ratio for a circular compact-binary
    /// inspiral against a flat (single-sided) noise PSD.  Order-of-magnitude
    /// estimate under the stationary-phase approximation:
    ///
    /// ```text
    /// ρ² = 4 · ∫|h̃(f)|² / S_n(f) df
    /// ρ ≈ h_rss · sqrt(Δf / S_n)
    /// ```
    ///
    /// Inputs:
    /// - `strain_rss` — root-sum-square strain amplitude `h_rss` [1/sqrt(Hz)]
    /// - `f_min`      — lower band edge [Hz] (e.g. 20 Hz for LIGO O4)
    /// - `f_max`      — upper band edge [Hz] (e.g. 400 Hz for post-merger cutoff)
    /// - `noise_psd`  — flat single-sided noise PSD `S_n` at the band centre
    ///   [Hz^-1] (e.g. 1e-46 for early-aLIGO at 100 Hz)
    ///
    /// Returns the matched-filter SNR (dimensionless).  Note: real LIGO uses a
    /// shaped PSD curve, not a single number; this is a compact closed-form
    /// estimate that generalises naturally when integrated against an actual
    /// PSD curve later (see PHYSICS_EXPANSION_PLAN.md W6 follow-up).
    pub fn gw_inspiral_snr(strain_rss: f64, f_min: f64, f_max: f64, noise_psd: f64) -> Option<f64> {
        if !finite_positive(strain_rss)
            || !finite_positive(f_min)
            || !finite_positive(f_max)
            || f_max <= f_min
            || !finite_positive(noise_psd)
        {
            set_error(ERR_INVALID_ARGUMENT, "bad GW inspiral SNR args");
            return None;
        }
        let bandwidth = f_max - f_min;
        // ρ ≈ h_rss · sqrt(Δf / S_n) — root-sum-square convention; for
        // root-power spectral density conventions this collapses the integral.
        Some(strain_rss * (bandwidth / noise_psd).sqrt())
    }

    /// inspiral-time-to-coalescence for a circular binary in the quadrupole
    /// approximation (Peters & Mathews 1963, leading order):
    ///
    /// ```text
    /// t_c = (5/256) · (c⁵ / G³) · (M_c⁵ / f⁵) · (π · f)^(-8/3)
    /// ```
    ///
    /// Simplified (geometric units dropped back to SI): use chirp mass and the
    /// gravitational-wave frequency `f_gw` (twice the orbital frequency) to
    /// compute remaining inspiral time to coalescence.
    ///
    /// Inputs:
    /// - `chirp_mass_kg` — M_c (pulsar-mass-plus-pulsar-mass-derived chirp mass)
    /// - `f_gw_hz`       — current gravitational-wave frequency [Hz]
    ///
    /// Returns seconds until coalescence.  For reference, a 1.4 Msun + 1.4 Msun
    /// binary at f_gw = 100 Hz has t_c ≈ 2.2 s.
    pub fn gw_inspiral_time_to_coalescence(chirp_mass_kg: f64, f_gw_hz: f64) -> Option<f64> {
        const G: f64 = 6.67430e-11;
        if !finite_positive(chirp_mass_kg) || !finite_positive(f_gw_hz) {
            set_error(ERR_INVALID_ARGUMENT, "bad GW t_c args");
            return None;
        }
        // t_c = (5/256) · c^5 / (G^(5/3) · π^(8/3) · f_gw^(8/3) · M_c^(5/3))
        // Standard inspiral formula from leading-order quadrupole radiation.
        let f_pow = f_gw_hz.powf(8.0 / 3.0);
        let m_pow = chirp_mass_kg.powf(5.0 / 3.0);
        let numerator = 5.0 / 256.0 * SPEED_OF_LIGHT.powi(5);
        let denominator = G.powf(5.0 / 3.0) * std::f64::consts::PI.powf(8.0 / 3.0) * f_pow * m_pow;
        Some(numerator / denominator)
    }

    /// Relativistic total energy: E = γ·m·c².
    pub fn relativistic_total_energy(rest_mass: f64, lorentz_factor: f64) -> Option<f64> {
        if !rest_mass.is_finite()
            || rest_mass < 0.0
            || !lorentz_factor.is_finite()
            || lorentz_factor < 1.0
        {
            return None;
        }
        Some(lorentz_factor * rest_mass * SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Relativistic momentum magnitude: p = γ·m·v.
    pub fn relativistic_momentum(rest_mass: f64, speed: f64) -> Option<f64> {
        if !rest_mass.is_finite()
            || rest_mass < 0.0
            || !speed.is_finite()
            || !(0.0..SPEED_OF_LIGHT).contains(&speed)
        {
            return None;
        }
        let beta = speed / SPEED_OF_LIGHT;
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        Some(gamma * rest_mass * speed)
    }

    /// Energy–momentum relation (inverse of invariant mass): E = √(m²c⁴ + p²c²).
    pub fn relativistic_energy_from_momentum(rest_mass: f64, momentum: f64) -> Option<f64> {
        if !rest_mass.is_finite() || rest_mass < 0.0 || !momentum.is_finite() || momentum < 0.0 {
            return None;
        }
        let c2 = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
        Some((rest_mass * rest_mass * c2 * c2 + momentum * momentum * c2).sqrt())
    }

    /// Relativistic aberration of light: cos θ' = (cos θ − β) / (1 − β·cos θ).
    pub fn relativistic_aberration(cos_theta: f64, beta: f64) -> Option<f64> {
        if !cos_theta.is_finite() || !beta.is_finite() || beta.abs() >= 1.0 {
            return None;
        }
        let denom = 1.0 - beta * cos_theta;
        if denom.abs() < 1.0e-12 {
            return None;
        }
        Some(((cos_theta - beta) / denom).clamp(-1.0, 1.0))
    }

    /// Relativistic Doppler beaming (boost) factor: δ = 1 / [γ·(1 − β·cos θ)].
    pub fn relativistic_doppler_beaming_factor(beta: f64, cos_theta: f64) -> Option<f64> {
        if !beta.is_finite() || !(0.0..1.0).contains(&beta) || !cos_theta.is_finite() {
            return None;
        }
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        let denom = gamma * (1.0 - beta * cos_theta);
        if denom.abs() < 1.0e-12 {
            return None;
        }
        Some(1.0 / denom)
    }

    /// Photon-sphere radius (Schwarzschild): r_ph = 1.5·r_s = 3·G·M/c².
    pub fn photon_sphere_radius(mass: f64, g: f64) -> Option<f64> {
        if !mass.is_finite() || mass <= 0.0 || !g.is_finite() || g <= 0.0 {
            return None;
        }
        Some(3.0 * g * mass / (SPEED_OF_LIGHT * SPEED_OF_LIGHT))
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

    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Einstein heat capacity: C_V = 3Nk_B (θ_E/T)² e^{θ_E/T} / (e^{θ_E/T} - 1)²
    pub fn einstein_heat_capacity(
        temperature: f64,
        einstein_temperature: f64,
        n_atoms: f64,
    ) -> Option<f64> {
        if !finite_5(temperature, einstein_temperature, n_atoms, 0.0, 0.0)
            || temperature <= 0.0
            || einstein_temperature <= 0.0
            || n_atoms <= 0.0
        {
            return None;
        }
        let r = 8.314462618;
        let x = einstein_temperature / temperature;
        let ex = x.exp();
        if ex <= 1.0 {
            return None;
        }
        Some(3.0 * n_atoms * r * x * x * ex / (ex - 1.0).powi(2))
    }

    const ALBERT_PLANCK_H: f64 = 6.62607015e-34;

    /// Photoelectric threshold frequency: f₀ = W / h, where W is the work function.
    /// Einstein (1905)——对光电效应的解释：光子能量必须超过金属功函数
    /// 才能打出电子，阈值频率由 W = h·f₀ 决定。
    pub fn photoelectric_threshold(work_function: f64) -> Option<f64> {
        if !work_function.is_finite() || work_function <= 0.0 {
            return None;
        }
        Some(work_function / ALBERT_PLANCK_H)
    }

    /// Photoelectric maximum kinetic energy (Einstein): K_max = h·f − W.
    /// Returns 0 when the photon energy is below the work function (no emission).
    pub fn photoelectric_max_kinetic(frequency: f64, work_function: f64) -> Option<f64> {
        if !frequency.is_finite()
            || frequency < 0.0
            || !work_function.is_finite()
            || work_function < 0.0
        {
            return None;
        }
        let k = ALBERT_PLANCK_H * frequency - work_function;
        Some(k.max(0.0))
    }
}
