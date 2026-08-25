//! James Thomson (1822–1892) —— 贡献目录与公式实现。
//!
//! 北爱尔兰工程师 William Thomson 之兄 James Thomson。工程热力学代表
//! 贡献：焦耳–汤姆孙效应（与 J. P. Joule 联名实验，气体节流温度变化）。
//! 本文件承载该效应的两个公式实现；原 `mps-formula` 域模块
//! （`thermodynamics.rs` / `ludwig_boltzmann.rs`）仅 `pub use` 重导出以
//! 保持 FFI / ABI 不变。不引入 Rapier / `WorldHandle`。
//!
//! 归属性备注：历史上焦耳–汤姆孙效应以 Joule 与 *William* Thomson
//! （开尔文勋爵）联名，而非 James Thomson。本工程把他们兄弟二人的
//! 热力学领域条目分开：James 挂该公式，Lord Kelvin 仍持 Prandtl。
//! 若后续需要把这对公式挪到 `lord_kelvin.rs`，仅改 `pub use` 的指向即可。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "james_thomson",
    name: "James Thomson",
    birth_year: Some(1822),
    death_year: Some(1892),
    field_id: "statistical",
    nationality: "British",
    contribution: "Joule-Thomson effect (throttling); thermodynamics",
    key_constants: "mu_JT",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }

    /// Joule-Thomson coefficient: μ_JT = (∂T/∂P)_H
    /// For ideal gas: μ_JT = 0. For Van der Waals: μ_JT ≈ (1/Cp)(2a/RT - b)
    pub fn joule_thomson_coefficient(cp: f64, temperature: f64, a: f64, b: f64) -> Option<f64> {
        if !finite_5(cp, temperature, a, b, 0.0) || cp <= 0.0 || temperature <= 0.0 {
            return None;
        }
        let r = 8.314462618;
        Some((2.0 * a / (r * temperature) - b) / cp)
    }

    /// Joule-Thomson inversion temperature: T_inv = 2a/(Rb)
    pub fn joule_thomson_inversion_temperature(a: f64, b: f64) -> Option<f64> {
        if !finite_5(a, b, 0.0, 0.0, 0.0) || a <= 0.0 || b <= 0.0 {
            return None;
        }
        let r = 8.314462618;
        Some(2.0 * a / (r * b))
    }

    /// 冰再冻（regelation）冰点随压力下降公式 —— Thomson (1849) 关系。
    /// 这是 Clausius–Clapeyron 在固–液相界的特化形式,用于"加压使冰熔点
    /// 下降"这一现象(冰川底部滑动 / 钢丝穿冰实验的物理基础)。James Thomson
    /// 1849 年发表其解析形式,William Thomson(开尔文勋爵)随后做实验确认。
    /// 输入:三相点温度 T(K)、液相密度 ρ_l(kg/m³)、固相密度 ρ_s、相变潜热
    /// L(J/kg, 按单位质量计)。输出:熔点随压力的变化率 dT/dP(K/Pa)。
    /// dT/dP = T · (v_l − v_s) / L = T · (1/ρ_l − 1/ρ_s) / L
    /// 相界次序为"液相减固相":对大部分物质 ρ_l < ρ_s,结果为正(加压升熔点);
    /// 对水 ρ_l > ρ_s,结果为负——加压降低冰点,与一般物质相反,这正是 Thomson
    /// 关系预测的水的反常膨胀效应。
    pub fn regelation_melting_point_slope(
        temperature: f64,
        density_liquid: f64,
        density_solid: f64,
        latent_heat: f64,
    ) -> Option<f64> {
        if !finite_5(temperature, density_liquid, density_solid, latent_heat, 0.0)
            || temperature <= 0.0
            || density_liquid <= 0.0
            || density_solid <= 0.0
            || latent_heat <= 0.0
        {
            return None;
        }
        Some(temperature * (1.0 / density_liquid - 1.0 / density_solid) / latent_heat)
    }

    /// 涡环动能(Kelvin–Helmholtz 涡环能量近似) —— Thomson vortex theorem。
    /// James Thomson 1867 年(与 William Thomson 联名,引用 Helmholtz)给出的
    /// 经典涡环能量命题:截面为圆形的细涡环,环量 Γ、密度 ρ、主半径 R、
    /// 截面半径 a(a << R),其动能为
    ///     E ≈ (1/2) · ρ · Γ² · R · [ln(8R / a) − 7/4]
    /// 此即"Thomson 涡定理"对涡环能量标度的标准估计,后经 Helmholtz 系统化为
    /// 涡动力学。要求 R > 0、a > 0、R > a、ρ > 0、Γ > 0。
    pub fn vortex_ring_kinetic_energy(
        density: f64,
        circulation: f64,
        main_radius: f64,
        core_radius: f64,
    ) -> Option<f64> {
        if !finite_5(density, circulation, main_radius, core_radius, 0.0)
            || density <= 0.0
            || circulation <= 0.0
            || main_radius <= 0.0
            || core_radius <= 0.0
            || main_radius <= core_radius
        {
            return None;
        }
        let log_term = (8.0 * main_radius / core_radius).ln() - 7.0 / 4.0;
        if !log_term.is_finite() {
            return None;
        }
        Some(0.5 * density * circulation * circulation * main_radius * log_term)
    }
}
