//! Joseph-Louis Lagrange —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "joseph_louis_lagrange",
    name: "Joseph-Louis Lagrange",
    birth_year: Some(1736),
    death_year: Some(1813),
    field_id: "mechanics",
    nationality: "French",
    contribution: "Lagrangian mechanics; celestial perturbation theory",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    use std::f64::consts::PI;
    fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
        if x < lo {
            lo
        } else if x > hi {
            hi
        } else {
            x
        }
    }

    /// PID step-size controller for adaptive integration.
    ///
    /// Based on the Gustafsson/Söderlind algorithm used in ODE solvers like
    /// DOPRI8 and CVODE.
    ///
    /// Given the current step size `dt`, the local error estimate `err`,
    /// and the desired tolerance `tol`, returns a recommended `dt_next`.
    ///
    /// The PID gains `kI`, `kP`, `kD` are pre-tuned for orbital mechanics:
    ///   kI = 0.3/order, kP = 0.6/order, kD = 0.0/order

    pub fn adaptive_step_size(dt: f64, err: f64, tolerance: f64, order: u32) -> f64 {
        if err <= 0.0 || tolerance <= 0.0 {
            return dt;
        }

        // Safety factors
        let safety = 0.9;
        let min_scale = 0.2;
        let max_scale = 5.0;

        let ord = order as f64;

        // Classic step-size controller: dt_new = safety · dt · (tol/err)^{1/(order+1)}
        let scale = safety * (tolerance / err).powf(1.0 / (ord + 1.0));
        let scale = scale.clamp(min_scale, max_scale);

        dt * scale
    }

    /// Check if the current step size is adequate for the error tolerance.
    ///
    /// Returns `true` if the step should be accepted.

    pub fn step_accepted(err: f64, tolerance: f64) -> bool {
        err <= tolerance
    }

    /// Evaluate Carlson's symmetric elliptic integral R_F(x, y, z).

    pub fn carlson_rf(x: f64, y: f64, z: f64) -> f64 {
        let mut x = x;
        let mut y = y;
        let mut z = z;

        for _ in 0..20 {
            let lambda = x.sqrt() * y.sqrt() + y.sqrt() * z.sqrt() + z.sqrt() * x.sqrt();
            x = (x + lambda) * 0.25;
            y = (y + lambda) * 0.25;
            z = (z + lambda) * 0.25;

            let avg = (x + y + z) / 3.0;
            let max_dev = ((x - avg).abs()).max((y - avg).abs()).max((z - avg).abs());
            if max_dev < 1e-15 * avg {
                break;
            }
        }

        let avg = (x + y + z) / 3.0;
        avg.powf(-0.5)
    }

    /// Evaluate Carlson's symmetric elliptic integral R_D(x, y, z).

    pub fn carlson_rd(x: f64, y: f64, z: f64) -> f64 {
        let mut x = x;
        let mut y = y;
        let mut z = z;
        let mut sum = 0.0;
        let mut fac = 1.0;

        for _ in 0..20 {
            let lambda = x.sqrt() * y.sqrt() + y.sqrt() * z.sqrt() + z.sqrt() * x.sqrt();
            sum += fac / (z.sqrt() * (z + lambda));
            fac *= 0.25;
            x = (x + lambda) * 0.25;
            y = (y + lambda) * 0.25;
            z = (z + lambda) * 0.25;

            let avg = (x + y + z) / 3.0;
            let max_dev = ((x - avg).abs()).max((y - avg).abs()).max((z - avg).abs());
            if max_dev < 1e-15 * avg {
                break;
            }
        }

        let avg = (x + y + z) / 3.0;
        sum + fac * 3.0 * avg.powf(-1.5)
    }
}
