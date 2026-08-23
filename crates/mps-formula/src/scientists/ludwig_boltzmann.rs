//! Ludwig Boltzmann —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "ludwig_boltzmann",
    name: "Ludwig Boltzmann",
    birth_year: Some(1844),
    death_year: Some(1906),
    field_id: "statistical",
    nationality: "Austrian",
    contribution: "Statistical mechanics; S=k·ln W",
    key_constants: "k_B",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    pub const G: f64 = 6.67430e-11;
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Ideal gas law: PV = nRT. Returns pressure (Pa).
    pub fn ideal_gas_pressure(volume: f64, moles: f64, temperature: f64) -> Option<f64> {
        if !volume.is_finite()
            || volume <= 0.0
            || !moles.is_finite()
            || moles < 0.0
            || !temperature.is_finite()
            || temperature < 0.0
        {
            return None;
        }
        Some(moles * 8.314462618 * temperature / volume)
    }

    /// Returns volume from ideal gas law.
    pub fn ideal_gas_volume(pressure: f64, moles: f64, temperature: f64) -> Option<f64> {
        if !pressure.is_finite()
            || pressure < 0.0
            || !moles.is_finite()
            || moles < 0.0
            || !temperature.is_finite()
            || temperature < 0.0
        {
            return None;
        }
        Some(moles * 8.314462618 * temperature / pressure)
    }

    /// Returns temperature from ideal gas law.
    pub fn ideal_gas_temperature(pressure: f64, volume: f64, moles: f64) -> Option<f64> {
        if !pressure.is_finite()
            || pressure < 0.0
            || !volume.is_finite()
            || volume <= 0.0
            || !moles.is_finite()
            || moles <= 0.0
        {
            return None;
        }
        Some(pressure * volume / (moles * 8.314462618))
    }

    /// Polytropic process: P2 = P1 * (V1/V2)^gamma
    pub fn polytropic_pressure(p1: f64, v1: f64, v2: f64, gamma: f64) -> Option<f64> {
        if !finite_4(p1, v1, v2, gamma) || p1 < 0.0 || v1 <= 0.0 || v2 <= 0.0 || gamma <= 0.0 {
            return None;
        }
        Some(p1 * (v1 / v2).powf(gamma))
    }

    /// Polytropic work: W = (P2*V2 - P1*V1) / (1 - gamma)
    pub fn polytropic_work(p1: f64, v1: f64, p2: f64, v2: f64, gamma: f64) -> Option<f64> {
        if !finite_4(p1, v1, gamma, 0.0)
            || !finite_4(p2, v2, gamma, 0.0)
            || p1 < 0.0
            || v1 <= 0.0
            || p2 < 0.0
            || v2 <= 0.0
            || gamma <= 0.0
        {
            return None;
        }
        if (gamma - 1.0).abs() < 1.0e-12 {
            return None;
        }
        Some((p2 * v2 - p1 * v1) / (1.0 - gamma))
    }

    /// Reynolds number: Re = rho * v * L / mu
    pub fn reynolds_number(
        density: f64,
        velocity: f64,
        char_length: f64,
        viscosity: f64,
    ) -> Option<f64> {
        if !finite_4(density, velocity, char_length, viscosity)
            || density < 0.0
            || velocity < 0.0
            || char_length <= 0.0
            || viscosity <= 0.0
        {
            return None;
        }
        Some(density * velocity * char_length / viscosity)
    }

    /// Carnot efficiency: eta = 1 - T_cold / T_hot
    pub fn carnot_efficiency(t_hot: f64, t_cold: f64) -> Option<f64> {
        if !finite_4(t_hot, t_cold, 0.0, 0.0) || t_hot <= 0.0 || t_cold < 0.0 || t_cold >= t_hot {
            return None;
        }
        Some(1.0 - t_cold / t_hot)
    }

    /// Otto cycle efficiency: eta = 1 - 1 / r^(gamma-1)
    pub fn otto_efficiency(compression_ratio: f64, gamma: f64) -> Option<f64> {
        if !compression_ratio.is_finite()
            || compression_ratio <= 1.0
            || !gamma.is_finite()
            || gamma <= 1.0
        {
            return None;
        }
        Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0))
    }

    /// Diesel cycle efficiency
    pub fn diesel_efficiency(compression_ratio: f64, cutoff_ratio: f64, gamma: f64) -> Option<f64> {
        if !finite_4(compression_ratio, cutoff_ratio, gamma, 0.0)
            || compression_ratio <= 1.0
            || cutoff_ratio <= 1.0
            || gamma <= 1.0
        {
            return None;
        }
        let term = (cutoff_ratio.powf(gamma) - 1.0) / (gamma * (cutoff_ratio - 1.0));
        Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0) * term)
    }

    /// Brayton cycle efficiency: eta = 1 - 1 / r_p^((gamma-1)/gamma)
    pub fn brayton_efficiency(pressure_ratio: f64, gamma: f64) -> Option<f64> {
        if !pressure_ratio.is_finite()
            || pressure_ratio <= 1.0
            || !gamma.is_finite()
            || gamma <= 1.0
        {
            return None;
        }
        Some(1.0 - 1.0 / pressure_ratio.powf((gamma - 1.0) / gamma))
    }

    /// Clausius-Clapeyron: ln(P2/P1) = -(L/R) * (1/T2 - 1/T1)
    pub fn clausius_clapeyron_pressure(p1: f64, t1: f64, t2: f64, latent_heat: f64) -> Option<f64> {
        if !finite_4(p1, t1, t2, latent_heat)
            || p1 <= 0.0
            || t1 <= 0.0
            || t2 <= 0.0
            || latent_heat < 0.0
        {
            return None;
        }
        Some(p1 * (-latent_heat / 8.314462618 * (1.0 / t2 - 1.0 / t1)).exp())
    }

    pub fn entropy_change_constant_volume(moles: f64, cv: f64, t1: f64, t2: f64) -> Option<f64> {
        if !finite_4(moles, cv, t1, t2) || moles < 0.0 || cv <= 0.0 || t1 <= 0.0 || t2 <= 0.0 {
            return None;
        }
        Some(moles * cv * (t2 / t1).ln())
    }

    pub fn entropy_change_constant_pressure(moles: f64, cp: f64, t1: f64, t2: f64) -> Option<f64> {
        if !finite_4(moles, cp, t1, t2) || moles < 0.0 || cp <= 0.0 || t1 <= 0.0 || t2 <= 0.0 {
            return None;
        }
        Some(moles * cp * (t2 / t1).ln())
    }

    /// Van der Waals pressure: P = RT/(V-b) - a/V²
    pub fn van_der_waals_pressure(
        temperature: f64,
        molar_volume: f64,
        a: f64,
        b: f64,
    ) -> Option<f64> {
        if !finite_5(temperature, molar_volume, a, b, 0.0)
            || temperature <= 0.0
            || molar_volume <= 0.0
            || a < 0.0
            || b < 0.0
        {
            return None;
        }
        let r = 8.314462618;
        Some(r * temperature / (molar_volume - b) - a / (molar_volume * molar_volume))
    }

    /// Van der Waals critical point: Tc = 8a/(27Rb), Pc = a/(27b²), Vc = 3b
    pub fn van_der_waals_critical_point(a: f64, b: f64) -> Option<(f64, f64, f64)> {
        if !finite_5(a, b, 0.0, 0.0, 0.0) || a <= 0.0 || b <= 0.0 {
            return None;
        }
        let r = 8.314462618;
        let tc = 8.0 * a / (27.0 * r * b);
        let pc = a / (27.0 * b * b);
        let vc = 3.0 * b;
        Some((tc, pc, vc))
    }

    /// Maxwell relation 1: (∂T/∂V)_S = -(∂P/∂S)_V
    pub fn maxwell_relation_1(
        _temperature: f64,
        _volume: f64,
        _entropy: f64,
        _pressure: f64,
    ) -> f64 {
        0.0 // stub — analytical form depends on the specific EOS; use as reminder of the identity
    }

    /// Helmholtz free energy — 见上文注释（re-export 用于路径稳定）。
    pub use crate::scientists::hermann_von_helmholtz::formulas::helmholtz_free_energy;
    /// Enthalpy / Helmholtz / Gibbs free energy — 实现迁至
    /// `scientists::willard_gibbs::formulas` 与
    /// `scientists::hermann_von_helmholtz::formulas`；此处 `pub use` 重导出
    /// 以保持 `mps_formula::scientists::ludwig_boltzmann::formulas::*` 路径稳定。
    pub use crate::scientists::willard_gibbs::formulas::enthalpy;
    /// Gibbs free energy — 见上文注释（re-export 用于路径稳定）。
    pub use crate::scientists::willard_gibbs::formulas::gibbs_free_energy;

    /// Joule-Thomson effect — implementation moved to
    /// `scientists::james_thomson::formulas`; re-exported here to preserve the
    /// existing `mps_formula::scientists::ludwig_boltzmann::formulas::*` path.
    pub use crate::scientists::james_thomson::formulas::joule_thomson_coefficient;
    pub use crate::scientists::james_thomson::formulas::joule_thomson_inversion_temperature;

    /// Debye heat capacity — 实现迁至 `scientists::peter_debye::formulas`，
    /// 此处 `pub use` 重导出以保持
    /// `mps_formula::scientists::ludwig_boltzmann::formulas::*` 路径稳定。
    pub use crate::scientists::peter_debye::formulas::debye_heat_capacity_low_t;

    /// Einstein heat capacity — 实现迁至
    /// `scientists::albert_einstein::formulas`，此处 `pub use` 重导出以保持
    /// `mps_formula::scientists::ludwig_boltzmann::formulas::*` 路径稳定。
    pub use crate::scientists::albert_einstein::formulas::einstein_heat_capacity;

    /// Carnot refrigeration coefficient of performance: COP = Tc / (Th - Tc)
    pub fn carnot_refrigeration_cop(t_cold: f64, t_hot: f64) -> Option<f64> {
        if !finite_5(t_cold, t_hot, 0.0, 0.0, 0.0)
            || t_cold <= 0.0
            || t_hot <= 0.0
            || t_hot <= t_cold
        {
            return None;
        }
        Some(t_cold / (t_hot - t_cold))
    }

    /// Heat pump COP: COP = Th / (Th - Tc)
    pub fn heat_pump_cop(t_cold: f64, t_hot: f64) -> Option<f64> {
        if !finite_5(t_cold, t_hot, 0.0, 0.0, 0.0)
            || t_cold <= 0.0
            || t_hot <= 0.0
            || t_hot <= t_cold
        {
            return None;
        }
        Some(t_hot / (t_hot - t_cold))
    }
}
