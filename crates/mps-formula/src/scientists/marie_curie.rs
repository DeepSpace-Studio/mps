//! Marie Curie —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "marie_curie",
    name: "Marie Curie",
    birth_year: Some(1867),
    death_year: Some(1934),
    field_id: "nuclear",
    nationality: "Polish/French",
    contribution: "Radioactivity; polonium & radium",
    key_constants: "curie",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {

    /// Specific activity: SA = λ · N_A / A  (Bq/g)
    /// where N_A is Avogadro's number and A is the atomic mass number
    pub fn specific_activity(decay_constant: f64, mass_number: f64) -> Option<f64> {
        if !decay_constant.is_finite()
            || decay_constant <= 0.0
            || !mass_number.is_finite()
            || mass_number <= 0.0
        {
            return None;
        }
        let avogadro = 6.022_140_76e23;
        Some(decay_constant * avogadro / mass_number)
    }

    /// Gamma-ray attenuation (Beer–Lambert): I(x) = I₀ · exp(-μ · x)
    pub fn gamma_attenuation(
        initial_intensity: f64,
        linear_attenuation: f64,
        thickness: f64,
    ) -> Option<f64> {
        if !initial_intensity.is_finite()
            || initial_intensity < 0.0
            || !linear_attenuation.is_finite()
            || linear_attenuation < 0.0
            || !thickness.is_finite()
            || thickness < 0.0
        {
            return None;
        }
        Some(initial_intensity * (-linear_attenuation * thickness).exp())
    }

    /// Half-value layer (HVL): thickness to reduce intensity by half: HVL = ln(2) / μ
    pub fn half_value_layer(linear_attenuation: f64) -> Option<f64> {
        if !linear_attenuation.is_finite() || linear_attenuation <= 0.0 {
            return None;
        }
        Some(std::f64::consts::LN_2 / linear_attenuation)
    }
}
