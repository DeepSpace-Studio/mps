//! Lev Landau —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "lev_landau",
    name: "Lev Landau",
    birth_year: Some(1908),
    death_year: Some(1968),
    field_id: "condmat",
    nationality: "Soviet",
    contribution: "Superfluidity; Landau damping; phase transitions",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {

    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }

    /// Plasma beta: β = 2μ₀·n·k·T / B²
    pub fn plasma_beta(density: f64, temperature: f64, magnetic_field: f64) -> Option<f64> {
        let mu0 = 1.25663706212e-6;
        let kb = 1.380649e-23;
        if !density.is_finite()
            || density < 0.0
            || !temperature.is_finite()
            || temperature < 0.0
            || !magnetic_field.is_finite()
            || magnetic_field <= 0.0
        {
            return None;
        }
        Some(2.0 * mu0 * density * kb * temperature / (magnetic_field * magnetic_field))
    }

    /// Cyclotron (gyro) frequency: ω_c = q·B / m
    pub fn gyrofrequency(charge: f64, magnetic_field: f64, mass: f64) -> Option<f64> {
        if !charge.is_finite()
            || !magnetic_field.is_finite()
            || magnetic_field < 0.0
            || !mass.is_finite()
            || mass <= 0.0
        {
            return None;
        }
        Some(charge * magnetic_field / mass)
    }

    /// Larmor radius: r_L = m·v_⟂ / (|q|·B)
    pub fn larmor_radius(
        mass: f64,
        perpendicular_velocity: f64,
        charge: f64,
        magnetic_field: f64,
    ) -> Option<f64> {
        if !mass.is_finite()
            || mass <= 0.0
            || !perpendicular_velocity.is_finite()
            || perpendicular_velocity < 0.0
            || !charge.is_finite()
            || charge == 0.0
            || !magnetic_field.is_finite()
            || magnetic_field <= 0.0
        {
            return None;
        }
        Some(mass * perpendicular_velocity / (charge.abs() * magnetic_field))
    }

    /// Ideal MHD wave speeds: slow, Alfven, fast
    /// Returns (v_slow, v_alfven, v_fast) in m/s.
    pub fn mhd_wave_speeds(
        sound_speed: f64,
        alfven_speed: f64,
        angle_to_b: f64,
    ) -> Option<(f64, f64, f64)> {
        if !sound_speed.is_finite()
            || sound_speed < 0.0
            || !alfven_speed.is_finite()
            || alfven_speed < 0.0
            || !angle_to_b.is_finite()
        {
            return None;
        }
        let ca2 = alfven_speed * alfven_speed;
        let cs2 = sound_speed * sound_speed;
        let sum = ca2 + cs2;
        let cos_theta = angle_to_b.cos();
        let diff2 = (sum * sum - 4.0 * ca2 * cs2 * cos_theta * cos_theta).max(0.0);
        let v_slow = (0.5 * (sum - diff2.sqrt())).max(0.0).sqrt();
        let v_alfven = (ca2 * cos_theta * cos_theta).max(0.0).sqrt();
        let v_fast = (0.5 * (sum + diff2.sqrt())).sqrt();
        Some((v_slow, v_alfven, v_fast))
    }

    /// Tokamak safety factor (cylindrical approximation): q = (r·B_t) / (R·B_p)
    pub fn safety_factor(
        minor_radius: f64,
        toroidal_field: f64,
        major_radius: f64,
        poloidal_field: f64,
    ) -> Option<f64> {
        if !finite_4(minor_radius, toroidal_field, major_radius, poloidal_field)
            || minor_radius <= 0.0
            || toroidal_field <= 0.0
            || major_radius <= 0.0
            || poloidal_field <= 0.0
        {
            return None;
        }
        Some(minor_radius * toroidal_field / (major_radius * poloidal_field))
    }

    /// Landau damping rate (simplified, Maxwellian plasma): γ_L/ω = -sqrt(π/8) · (ω/|k|v_th)³ · exp(-ω²/(2k²v_th²))
    pub fn landau_damping_rate(
        wave_frequency: f64,
        wavenumber: f64,
        thermal_speed: f64,
    ) -> Option<f64> {
        if !wave_frequency.is_finite()
            || wave_frequency <= 0.0
            || !wavenumber.is_finite()
            || wavenumber <= 0.0
            || !thermal_speed.is_finite()
            || thermal_speed <= 0.0
        {
            return None;
        }
        let xi = wave_frequency / (wavenumber * thermal_speed);
        let gamma = -0.626657 * xi * xi * xi * (-0.5 * xi * xi).exp();
        Some(gamma * wave_frequency)
    }

    /// Magnetic mirror ratio: R_m = B_max / B_min
    pub fn mirror_ratio(max_field: f64, min_field: f64) -> Option<f64> {
        if !max_field.is_finite() || max_field <= 0.0 || !min_field.is_finite() || min_field <= 0.0
        {
            return None;
        }
        Some(max_field / min_field)
    }

    /// Mirror loss cone angle: sin²(θ_lc) = B_min / B_max
    pub fn mirror_loss_cone_angle(max_field: f64, min_field: f64) -> Option<f64> {
        if !max_field.is_finite()
            || max_field <= 0.0
            || !min_field.is_finite()
            || min_field <= 0.0
            || min_field > max_field
        {
            return None;
        }
        Some((min_field / max_field).sqrt().asin())
    }
}
