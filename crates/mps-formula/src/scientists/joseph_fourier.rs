//! Joseph Fourier —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "joseph_fourier",
    name: "Joseph Fourier",
    birth_year: Some(1768),
    death_year: Some(1830),
    field_id: "mathphys",
    nationality: "French",
    contribution: "Fourier series; heat conduction equation",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    pub const C: f64 = 299_792_458.0;
    const PI: f64 = std::f64::consts::PI;
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Newton's law of cooling: Q = h * A * (T_surface - T_fluid)

    pub fn convective_heat_flux(h: f64, area: f64, t_surface: f64, t_fluid: f64) -> Option<f64> {
        if !finite_4(h, area, t_surface, t_fluid) || h < 0.0 || area < 0.0 {
            return None;
        }
        Some(h * area * (t_surface - t_fluid))
    }

    /// Nusselt number (Dittus-Boelter): Nu = 0.023 * Re^0.8 * Pr^n

    pub fn dittus_boelter_nusselt(reynolds: f64, prandtl: f64, heating: bool) -> Option<f64> {
        if !reynolds.is_finite() || reynolds < 0.0 || !prandtl.is_finite() || prandtl < 0.0 {
            return None;
        }
        if reynolds < 10000.0 {
            return None;
        }
        let n = if heating { 0.4 } else { 0.3 };
        Some(0.023 * reynolds.powf(0.8) * prandtl.powf(n))
    }

    /// Prandtl number: Pr = cp * mu / k

    pub fn prandtl_number(cp: f64, viscosity: f64, conductivity: f64) -> Option<f64> {
        if !finite_4(cp, viscosity, conductivity, 0.0)
            || cp <= 0.0
            || viscosity <= 0.0
            || conductivity <= 0.0
        {
            return None;
        }
        Some(cp * viscosity / conductivity)
    }

    /// Heat transfer coefficient from Nusselt: h = Nu * k / L

    pub fn htc_from_nusselt(nusselt: f64, conductivity: f64, char_length: f64) -> Option<f64> {
        if !finite_4(nusselt, conductivity, char_length, 0.0)
            || nusselt < 0.0
            || conductivity <= 0.0
            || char_length <= 0.0
        {
            return None;
        }
        Some(nusselt * conductivity / char_length)
    }

    /// Log-mean temperature difference for counter-flow heat exchanger.

    pub fn lmtd_counter_flow(
        t_hot_in: f64,
        t_hot_out: f64,
        t_cold_in: f64,
        t_cold_out: f64,
    ) -> Option<f64> {
        if !finite_5(t_hot_in, t_hot_out, t_cold_in, t_cold_out, 0.0)
            || t_hot_in < t_cold_out
            || t_hot_out < t_cold_in
        {
            return None;
        }
        let d1 = t_hot_in - t_cold_out;
        let d2 = t_hot_out - t_cold_in;
        if d1 <= 0.0 || d2 <= 0.0 {
            return None;
        }
        Some((d1 - d2) / (d1 / d2).ln())
    }

    /// Log-mean temperature difference for parallel-flow heat exchanger.

    pub fn lmtd_parallel_flow(
        t_hot_in: f64,
        t_hot_out: f64,
        t_cold_in: f64,
        t_cold_out: f64,
    ) -> Option<f64> {
        if !finite_5(t_hot_in, t_hot_out, t_cold_in, t_cold_out, 0.0) {
            return None;
        }
        let d1 = t_hot_in - t_cold_in;
        let d2 = t_hot_out - t_cold_out;
        if d1 <= 0.0 || d2 <= 0.0 {
            return None;
        }
        Some((d1 - d2) / (d1 / d2).ln())
    }

    /// NTU-epsilon effectiveness for counter-flow heat exchanger.

    pub fn ntu_epsilon_counter_flow(ntu: f64, c_r: f64) -> Option<f64> {
        if !finite_5(ntu, c_r, 0.0, 0.0, 0.0) || ntu < 0.0 || c_r < 0.0 {
            return None;
        }
        let epsilon = if c_r >= 1.0 {
            ntu / (1.0 + ntu) // c_r = 1 limiting case
        } else {
            let exp = (-ntu * (1.0 - c_r)).exp();
            (1.0 - exp) / (1.0 - c_r * exp)
        };
        Some(epsilon)
    }

    /// Number of transfer units: NTU = UA / C_min

    pub fn ntu(overall_htc: f64, area: f64, c_min: f64) -> Option<f64> {
        if !finite_5(overall_htc, area, c_min, 0.0, 0.0)
            || overall_htc <= 0.0
            || area <= 0.0
            || c_min <= 0.0
        {
            return None;
        }
        Some(overall_htc * area / c_min)
    }

    /// Heat capacity rate: C = m_dot * cp

    pub fn heat_capacity_rate(mass_flow: f64, specific_heat: f64) -> Option<f64> {
        if !finite_5(mass_flow, specific_heat, 0.0, 0.0, 0.0)
            || mass_flow <= 0.0
            || specific_heat <= 0.0
        {
            return None;
        }
        Some(mass_flow * specific_heat)
    }

    /// View factor for two parallel coaxial disks.
    /// R1 = r1/d, R2 = r2/d where d is the separation distance.

    pub fn view_factor_coaxial_disks(radius_ratio_1: f64, radius_ratio_2: f64) -> Option<f64> {
        if !finite_5(radius_ratio_1, radius_ratio_2, 0.0, 0.0, 0.0)
            || radius_ratio_1 < 0.0
            || radius_ratio_2 < 0.0
        {
            return None;
        }
        let x = 1.0 + (1.0 + radius_ratio_2 * radius_ratio_2) / (radius_ratio_1 * radius_ratio_1);
        Some(0.5 * (x - (x * x - 4.0 * (radius_ratio_2 / radius_ratio_1).powi(2)).sqrt()))
    }

    /// View factor for two parallel, equal rectangles.
    /// X = a/d, Y = b/d where a, b are side lengths and d is the separation.

    pub fn view_factor_parallel_rectangles(x: f64, y: f64) -> Option<f64> {
        if !finite_5(x, y, 0.0, 0.0, 0.0) || x <= 0.0 || y <= 0.0 {
            return None;
        }
        let f = 2.0 / (std::f64::consts::PI * x * y)
            * ((x * x * (1.0 + y * y) / (1.0 + x * x + y * y)).ln().sqrt()
                + (y * y * (1.0 + x * x) / (1.0 + x * x + y * y)).ln().sqrt()
                + x * (1.0 + y * y).atan() / (x * x + y * y + x * x * y * y).sqrt()
                + y * (1.0 + x * x).atan() / (x * x + y * y + y * y * x * x).sqrt()
                - x * x.atan()
                - y * y.atan());
        Some(f)
    }

    /// Second virial coefficient for Lennard-Jones gas (simplified).
    /// B(T) = b₀ - a₀/RT, where b₀ = 2πN_A σ³/3, a₀ = 2πN_A² ε σ³

    pub fn virial_second_coefficient(temperature: f64, sigma: f64, epsilon: f64) -> Option<f64> {
        if !finite_5(temperature, sigma, epsilon, 0.0, 0.0)
            || temperature <= 0.0
            || sigma <= 0.0
            || epsilon < 0.0
        {
            return None;
        }
        let r = 8.314462618;
        let avogadro = 6.022_140_76e23;
        let b0 = 2.0 * std::f64::consts::PI * avogadro * sigma.powi(3) / 3.0;
        let a0 = 2.0 * std::f64::consts::PI * avogadro * avogadro * epsilon * sigma.powi(3);
        Some(b0 - a0 / (r * temperature))
    }

    /// Quality (vapor mass fraction): x = m_vapor / (m_vapor + m_liquid)

    pub fn quality(vapor_mass: f64, liquid_mass: f64) -> Option<f64> {
        if !finite_5(vapor_mass, liquid_mass, 0.0, 0.0, 0.0)
            || vapor_mass < 0.0
            || liquid_mass < 0.0
        {
            return None;
        }
        let total = vapor_mass + liquid_mass;
        if total <= 0.0 {
            return None;
        }
        Some(vapor_mass / total)
    }

    /// Homogeneous void fraction: α = 1 / (1 + (1-x)/x * ρ_v/ρ_l)

    pub fn homogeneous_void_fraction(quality: f64, rho_vapor: f64, rho_liquid: f64) -> Option<f64> {
        if !finite_5(quality, rho_vapor, rho_liquid, 0.0, 0.0)
            || !(0.0..=1.0).contains(&quality)
            || rho_vapor <= 0.0
            || rho_liquid <= 0.0
        {
            return None;
        }
        if quality <= 0.0 || quality >= 1.0 {
            return Some(quality);
        }
        Some(1.0 / (1.0 + (1.0 - quality) / quality * rho_vapor / rho_liquid))
    }
}
