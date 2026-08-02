//! 环境扰动力 — 大气阻力、太阳光压等空间环境作用于航天器的力。
//!
//! 复用 `mps_formula::spaceflight` 的纯计算函数，把位置/速度/引力体参数
//! 翻译为对应扰动力（加速度 × 质量）。所有函数返回的是**力**（N），
//! 由 `CosmosWorld::step` 在推进前注入到刚体上。

use mps_formula::celestial_data::{CelestialBody, SOLAR_PRESSURE_AT_1AU};
use mps_formula::ffi::Vec3;
use mps_formula::spaceflight;
use rapier3d::prelude::Vector;

/// 计算大气阻力产生的力（单位 N），返回世界坐标系向量。
///
/// - `body_velocity`：航天器相对惯性系的速度（m/s）
/// - `atmosphere_velocity`：大气在惯性系下的速度（一般为赤道自转分量，0 表示静止大气）
/// - `density`：当地大气密度（kg/m³），可由
///   [`spaceflight::atmospheric_density_scale_height`] 结合天体 `surface_density`/
///   `scale_height` 算出
/// - `drag_coefficient`：阻力系数 Cd（典型 2.2）
/// - `area`：迎风截面积（m²）
/// - `mass`：航天器质量（kg）
///
/// 返回 `None` 表示输入非法（密度、面积、质量非正等）。
pub fn atmospheric_drag_force(
    body_velocity: Vector,
    atmosphere_velocity: Vector,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
) -> Option<Vector> {
    // 早退：密度/面积/质量非正直接返回，避免 FFI 调用 + Vec3 装箱。
    if density <= 0.0 || area <= 0.0 || mass <= 0.0 {
        return None;
    }
    let accel = spaceflight::atmospheric_drag_acceleration(
        to_ffi(body_velocity),
        to_ffi(atmosphere_velocity),
        density,
        drag_coefficient,
        area,
        mass,
    )?;
    Some(scale(accel, mass))
}

/// 基于天体大气模型在给定高度采样的大气密度（kg/m³）。
///
/// 使用天体的 `surface_density`（参考高度处密度）与 `scale_height`，
/// 调用 [`spaceflight::atmospheric_density_scale_height`]。若天体无大气
///（`surface_density==0` 或 `scale_height==0`）则返回 0。
#[inline]
pub fn atmosphere_density_at(body: &CelestialBody, altitude_above_surface: f64) -> f64 {
    if body.surface_density <= 0.0 || body.scale_height <= 0.0 {
        return 0.0;
    }
    spaceflight::atmospheric_density_scale_height(
        body.surface_density,
        altitude_above_surface,
        0.0, // 参考高度取表面
        body.scale_height,
    )
    .unwrap_or(0.0)
}

/// 太阳光压产生的力（N）。
///
/// 模型：`F = -Cr · P · A · ŝ`，其中 `P` 是距太阳 1 AU 处的太阳光压
/// 常数（`SOLAR_PRESSURE_AT_1AU`），按距离平方反比衰减；
/// `ŝ` 是从航天器指向太阳的单位向量。
///
/// - `sun_to_body`：太阳到航天器的位置向量（用于按 1/AU² 衰减）
/// - `sun_direction`：从航天器指向太阳的单位向量
/// - `area`：受光截面积（m²）
/// - `reflectivity`：光压系数 Cr（吸收=1，镜面反射=2，典型 1.3）
/// - `au`：以米为单位的 1 AU（[`mps_formula::celestial_data::AU`]）
pub fn solar_pressure_force(
    sun_to_body: Vector,
    sun_direction: Vector,
    area: f64,
    reflectivity: f64,
    au: f64,
) -> Vector {
    // 早退：area/reflectivity 无意义时直接零，避免 normalize + length 的两次 sqrt。
    if area <= 0.0 || reflectivity <= 0.0 || au <= 0.0 {
        return Vector::ZERO;
    }
    let dir_len = sun_direction.length();
    if dir_len <= 1e-12 {
        return Vector::ZERO;
    }
    // sun_direction 在上游已是 `-sun_to_body/r`，这里再 normalize 容错
    // 非单位输入；用 length_squared 比再次 .length() 省一次 sqrt。
    let dir = sun_direction / dir_len;
    let r2 = sun_to_body.length_squared();
    if r2 < 1.0 {
        return Vector::ZERO;
    }
    // 距离平方反比衰减：实际光压 = P · (1AU / r)²；用 r2 省一次 sqrt。
    let pressure = SOLAR_PRESSURE_AT_1AU * (au * au) / r2;
    -dir * (pressure * area * reflectivity)
}

#[inline]
fn to_ffi(v: Vector) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

#[inline]
fn scale(v: Vec3, s: f64) -> Vector {
    Vector::new(v.x * s, v.y * s, v.z * s)
}

