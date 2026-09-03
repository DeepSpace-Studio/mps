//! Johannes Kepler —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "johannes_kepler",
    name: "Johannes Kepler",
    birth_year: Some(1571),
    death_year: Some(1630),
    field_id: "astro",
    nationality: "German",
    contribution: "Kepler's l of planetary motion",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::ffi::*;

    /// Compute the osculating Keplerian orbital elements.
    ///
    /// Returns (semi_major_axis, eccentricity, inclination, RAAN, arg_periapsis, true_anomaly)
    /// or zeros for invalid orbits.
    pub fn keplerian_elements(
        position: Vec3,
        velocity: Vec3,
        gm: f64,
    ) -> (f64, f64, f64, f64, f64, f64) {
        let r = vec3_to_rapier(position);
        let v = vec3_to_rapier(velocity);
        let r_mag = r.length();

        if r_mag < 1e-12 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let v2 = v.x * v.x + v.y * v.y + v.z * v.z;

        // Specific angular momentum
        let h_vec = r.cross(v);
        let h = h_vec.length();
        if h < 1e-20 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Semi-major axis from vis-viva: a = 1 / (2/r - v²/GM)
        let a = 1.0 / (2.0 / r_mag - v2 / gm);
        if !a.is_finite() || a <= 0.0 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Eccentricity vector: e = (v × h)/GM - r̂
        let e_vec = (v.cross(h_vec)) / gm - r / r_mag;
        let e = e_vec.length();

        // Inclination: cos i = h_z / h
        let inc = (h_vec.z / h).acos();

        // Node vector: n = k̂ × h
        let n_vec = nalgebra::Vector3::<f64>::new(-h_vec.y, h_vec.x, 0.0);
        let n = n_vec.length();

        // RAAN
        let raan = if n > 1e-20 {
            let mut om = n_vec.x.acos() / n;
            if n_vec.y < 0.0 {
                om = 2.0 * std::f64::consts::PI - om;
            }
            om
        } else {
            0.0
        };

        // Argument of periapsis
        let argp = if n > 1e-20 && e > 1e-12 {
            let mut w = (n_vec.dot(e_vec) / (n * e)).acos();
            if e_vec.z < 0.0 {
                w = 2.0 * std::f64::consts::PI - w;
            }
            w
        } else {
            0.0
        };

        // True anomaly
        let nu = if e > 1e-12 {
            let mut f = (e_vec.dot(r) / (e * r_mag)).acos();
            if r.dot(v) < 0.0 {
                f = 2.0 * std::f64::consts::PI - f;
            }
            f
        } else {
            // Circular orbit: use argument of latitude
            let mut u = (n_vec.dot(r) / (n * r_mag)).acos();
            if r.z < 0.0 {
                u = 2.0 * std::f64::consts::PI - u;
            }
            u
        };

        (a, e, inc, raan, argp, nu)
    }

    /// Kepler's third law (scalar): T² = 4π² a³ / (GM)
    /// Returns orbital period `T` given semi-major axis `a` and central mass `M`.
    pub fn kepler_period(semi_major_axis: f64, mass: f64) -> Option<f64> {
        if !semi_major_axis.is_finite()
            || !mass.is_finite()
            || semi_major_axis <= 0.0
            || mass <= 0.0
        {
            return None;
        }
        const G: f64 = 6.674_30e-11;
        let period = 2.0 * std::f64::consts::PI * (semi_major_axis.powi(3) / (G * mass)).sqrt();
        if !period.is_finite() {
            return None;
        }
        Some(period)
    }
}
