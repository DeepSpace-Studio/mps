//! Ernst Mach —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献,并承载其名下的公式实现
//! (从原 `mps-formula` 域模块迁移而来;实现在此,域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变)。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "ernst_mach",
    name: "Ernst Mach",
    birth_year: Some(1838),
    death_year: Some(1916),
    field_id: "fluid",
    nationality: "Austrian",
    contribution: "Mach number; supersonic flow",
    key_constants: "Ma",
};

/// 该科学家名下的公式实现 (从各域模块迁移而来)。
pub mod formulas {
    use crate::math::*;

    /// Mach angle (Mach cone): μ = arcsin(1 / Ma), valid for Ma ≥ 1.
    #[allow(dead_code)]
    pub fn mach_angle(mach_number: f64) -> Option<f64> {
        if !finite_positive(mach_number) || mach_number < 1.0 {
            return None;
        }
        Some((1.0 / mach_number).asin())
    }

    /// Mach number (simple): Ma = v / c.
    #[allow(dead_code)]
    pub fn mach_number_simple(flow_velocity: f64, speed_of_sound: f64) -> Option<f64> {
        if !finite_non_negative(flow_velocity) || !finite_positive(speed_of_sound) {
            return None;
        }
        Some(flow_velocity / speed_of_sound)
    }

    /// Mach line angle for an oblique shock (truncation of Prandtl-Meyer):
    /// `θ = atan(1 / sqrt(Ma² − 1))`, valid for Ma > 1.
    #[allow(dead_code)]
    pub fn mach_line_angle(mach_number: f64) -> Option<f64> {
        if !finite_positive(mach_number) || mach_number <= 1.0 {
            return None;
        }
        Some((1.0 / (mach_number * mach_number - 1.0).sqrt()).atan())
    }

    /// Mach area-Mach relation (isentropic 1D flow). Returns the A/A* ratio
    /// for a given Mach and γ:
    /// `A/A* = (1/Ma) · [(2/(γ+1))(1 + (γ−1)/2 · Ma²)]^((γ+1)/(2·(γ−1)))`.
    #[allow(dead_code)]
    pub fn mach_area(mach_number: f64, gamma: f64) -> Option<f64> {
        if !finite_positive(mach_number) || gamma <= 1.0 {
            return None;
        }
        let g = gamma;
        let m2 = mach_number * mach_number;
        let exp = (g + 1.0) / (2.0 * (g - 1.0));
        let term = (2.0 / (g + 1.0)) * (1.0 + (g - 1.0) / 2.0 * m2);
        Some((1.0 / mach_number) * term.powf(exp))
    }

    /// Supersonic shock (oblique) angle θ for a turning angle δ — the
    /// weak-shock root satisfying `0 < θ < π/2`, found by bisection over the
    /// shock polar relation
    /// `tan δ = 2 · cot(θ) · (Ma² · sin²θ − 1) / (Ma² · (γ + cos 2θ) + 2)`.
    #[allow(dead_code)]
    pub fn supersonic_shock_angle(
        mach_number: f64,
        gamma: f64,
        deflection_angle: f64,
    ) -> Option<f64> {
        if !finite_positive(mach_number)
            || mach_number <= 1.0
            || gamma <= 1.0
            || !finite_non_negative(deflection_angle)
            || deflection_angle >= std::f64::consts::FRAC_PI_2
        {
            return None;
        }
        let m = mach_number;
        let g = gamma;
        let m2 = m * m;
        let tan_delta = deflection_angle.tan();
        let eval = |theta: f64| -> f64 {
            let s = theta.sin();
            let c2t = (2.0 * theta).cos();
            (2.0 / theta.tan()) * (m2 * s * s - 1.0) / (m2 * (g + c2t) + 2.0) - tan_delta
        };
        let mu = (1.0 / m).asin();
        let mut lo = mu;
        let mut hi = std::f64::consts::FRAC_PI_2 * 0.999_999;
        let sign_lo = eval(lo).signum();
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if (hi - lo).abs() < 1.0e-12 {
                break;
            }
            if eval(mid).signum() == sign_lo {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(0.5 * (lo + hi))
    }
}
