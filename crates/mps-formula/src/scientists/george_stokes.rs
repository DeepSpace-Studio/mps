//! George Gabriel Stokes —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "george_stokes",
    name: "George Gabriel Stokes",
    birth_year: Some(1819),
    death_year: Some(1903),
    field_id: "fluid",
    nationality: "Irish",
    contribution: "Stokes' law; viscous drag",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// 2D point source velocity potential: φ = Q/(2π) · ln(r)
    pub fn source_potential_2d(strength: f64, r: f64) -> Option<f64> {
        if !strength.is_finite() || !r.is_finite() || r <= 0.0 {
            return None;
        }
        Some(strength / (2.0 * std::f64::consts::PI) * r.ln())
    }

    /// 2D doublet stream function: ψ = -κ · sin(θ) / (2π · r)
    pub fn doublet_stream_function_2d(strength: f64, r: f64, theta: f64) -> Option<f64> {
        if !strength.is_finite() || !r.is_finite() || r <= 0.0 || !theta.is_finite() {
            return None;
        }
        Some(-strength * theta.sin() / (2.0 * std::f64::consts::PI * r))
    }

    /// Power-law viscosity: μ_eff = K · γ̇^(n-1)
    pub fn power_law_viscosity(consistency: f64, shear_rate: f64, flow_index: f64) -> Option<f64> {
        if !consistency.is_finite()
            || consistency <= 0.0
            || !shear_rate.is_finite()
            || shear_rate < 0.0
            || !flow_index.is_finite()
        {
            return None;
        }
        if shear_rate <= 1.0e-12 && flow_index < 1.0 {
            return None;
        }
        Some(consistency * shear_rate.powf(flow_index - 1.0))
    }

    /// Bingham plastic: τ = τ_y + μ_p · γ̇
    pub fn bingham_stress(
        yield_stress: f64,
        plastic_viscosity: f64,
        shear_rate: f64,
    ) -> Option<f64> {
        if !yield_stress.is_finite()
            || yield_stress < 0.0
            || !plastic_viscosity.is_finite()
            || plastic_viscosity < 0.0
            || !shear_rate.is_finite()
            || shear_rate < 0.0
        {
            return None;
        }
        Some(yield_stress + plastic_viscosity * shear_rate)
    }

    /// Standard k-epsilon turbulence model: production of TKE.
    /// P_k = nut * S^2 where S = sqrt(2 * S_ij * S_ij)
    pub fn k_epsilon_production(eddy_viscosity: f64, strain_rate_magnitude: f64) -> Option<f64> {
        if !eddy_viscosity.is_finite()
            || eddy_viscosity < 0.0
            || !strain_rate_magnitude.is_finite()
            || strain_rate_magnitude < 0.0
        {
            return None;
        }
        Some(eddy_viscosity * strain_rate_magnitude * strain_rate_magnitude)
    }

    /// Eddy viscosity from k-epsilon: nut = C_mu * k^2 / epsilon
    pub fn k_epsilon_eddy_viscosity(tke: f64, dissipation: f64, c_mu: f64) -> Option<f64> {
        if !tke.is_finite()
            || tke < 0.0
            || !dissipation.is_finite()
            || dissipation <= 0.0
            || !c_mu.is_finite()
            || c_mu <= 0.0
        {
            return None;
        }
        Some(c_mu * tke * tke / dissipation)
    }

    /// k-epsilon: source term for k transport equation.
    /// dk/dt = P_k - epsilon + diffusion
    pub fn k_equation_source(production: f64, dissipation: f64) -> Option<f64> {
        if !production.is_finite() || !dissipation.is_finite() || dissipation < 0.0 {
            return None;
        }
        Some(production - dissipation)
    }

    /// k-epsilon: source term for epsilon transport equation.
    /// depsilon/dt = C_eps1 * P_k * epsilon/k - C_eps2 * epsilon^2/k + diffusion
    pub fn epsilon_equation_source(
        production: f64,
        tke: f64,
        dissipation: f64,
        c_eps1: f64,
        c_eps2: f64,
    ) -> Option<f64> {
        if !finite_5(production, tke, dissipation, c_eps1, c_eps2)
            || tke <= 0.0
            || dissipation < 0.0
            || c_eps1 <= 0.0
            || c_eps2 <= 0.0
        {
            return None;
        }
        Some(c_eps1 * production * dissipation / tke - c_eps2 * dissipation * dissipation / tke)
    }

    /// Standard k-epsilon model constants.
    pub fn k_epsilon_constants() -> (f64, f64, f64, f64, f64) {
        (0.09, 1.44, 1.92, 1.0, 1.3) // C_mu, C_eps1, C_eps2, sigma_k, sigma_eps
    }

    /// Characteristic turbulent length scale: L = C_mu^0.75 * k^1.5 / epsilon
    pub fn turbulent_length_scale(tke: f64, dissipation: f64) -> Option<f64> {
        if !tke.is_finite() || tke < 0.0 || !dissipation.is_finite() || dissipation <= 0.0 {
            return None;
        }
        let cmu: f64 = 0.09;
        Some(cmu.powf(0.75) * tke.powf(1.5) / dissipation)
    }

    /// Turbulent Reynolds number: Re_t = k^2 / (nu * epsilon)
    pub fn turbulent_reynolds(tke: f64, dissipation: f64, kinematic_viscosity: f64) -> Option<f64> {
        if !tke.is_finite()
            || tke < 0.0
            || !dissipation.is_finite()
            || dissipation <= 0.0
            || !kinematic_viscosity.is_finite()
            || kinematic_viscosity <= 0.0
        {
            return None;
        }
        Some(tke * tke / (kinematic_viscosity * dissipation))
    }

    /// Isentropic pressure ratio: P/P₀ = (1 + (γ-1)/2 · M²)^(-γ/(γ-1))
    pub fn isentropic_pressure_ratio(mach: f64, gamma: f64) -> Option<f64> {
        if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some((1.0 + (gamma - 1.0) / 2.0 * mach * mach).powf(-gamma / (gamma - 1.0)))
    }

    /// Isentropic density ratio: ρ/ρ₀ = (1 + (γ-1)/2 · M²)^(-1/(γ-1))
    pub fn isentropic_density_ratio(mach: f64, gamma: f64) -> Option<f64> {
        if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some((1.0 + (gamma - 1.0) / 2.0 * mach * mach).powf(-1.0 / (gamma - 1.0)))
    }

    /// Isentropic temperature ratio: T/T₀ = 1/(1 + (γ-1)/2 · M²)
    pub fn isentropic_temperature_ratio(mach: f64, gamma: f64) -> Option<f64> {
        if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some(1.0 / (1.0 + (gamma - 1.0) / 2.0 * mach * mach))
    }

    /// Area-Mach relation for isentropic flow: A/A* = (1/M) · ((2/(γ+1))·(1+(γ-1)·M²/2))^((γ+1)/(2(γ-1)))
    pub fn area_mach_ratio(mach: f64, gamma: f64) -> Option<f64> {
        if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        let term = (1.0 + (gamma - 1.0) / 2.0 * mach * mach) * 2.0 / (gamma + 1.0);
        Some(1.0 / mach * term.powf((gamma + 1.0) / (2.0 * (gamma - 1.0))))
    }

    /// Normal shock wave: downstream Mach number: M₂² = ((γ-1)M₁² + 2) / (2γ·M₁² - (γ-1))
    pub fn normal_shock_downstream_mach(upstream_mach: f64, gamma: f64) -> Option<f64> {
        if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some(
            ((gamma - 1.0) * upstream_mach * upstream_mach + 2.0)
                / (2.0 * gamma * upstream_mach * upstream_mach - (gamma - 1.0)),
        )
    }

    /// Normal shock pressure ratio: P₂/P₁ = 1 + 2γ/(γ+1) · (M₁² - 1)
    pub fn normal_shock_pressure_ratio(upstream_mach: f64, gamma: f64) -> Option<f64> {
        if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some(1.0 + 2.0 * gamma / (gamma + 1.0) * (upstream_mach * upstream_mach - 1.0))
    }

    /// Normal shock density ratio: ρ₂/ρ₁ = (γ+1)·M₁² / ((γ-1)·M₁² + 2)
    pub fn normal_shock_density_ratio(upstream_mach: f64, gamma: f64) -> Option<f64> {
        if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        Some(
            (gamma + 1.0) * upstream_mach * upstream_mach
                / ((gamma - 1.0) * upstream_mach * upstream_mach + 2.0),
        )
    }

    /// Prandtl-Meyer expansion angle: ν(M) = ((γ+1)/(γ-1))^(1/2) · atan(((γ-1)/(γ+1)·(M²-1))^(1/2)) - atan((M²-1)^(1/2))
    pub fn prandtl_meyer_angle(mach: f64, gamma: f64) -> Option<f64> {
        if !mach.is_finite() || mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 {
            return None;
        }
        let sqrt = ((gamma - 1.0) / (gamma + 1.0) * (mach * mach - 1.0)).sqrt();
        Some(
            ((gamma + 1.0) / (gamma - 1.0)).sqrt() * sqrt.atan()
                - (mach * mach - 1.0).sqrt().atan(),
        )
    }
}
