//! 环境扰动力 — 大气阻力、太阳光压等空间环境作用于航天器的力。
//!
//! 复用 `mps_formula::spaceflight` 的纯计算函数，把位置/速度/引力体参数
//! 翻译为对应扰动力（加速度 × 质量）。所有函数返回的是**力**（N），
//! 由 `CosmosWorld::step` 在推进前注入到刚体上。

use mps_formula::celestial_data::{CelestialBody, SOLAR_PRESSURE_AT_1AU};
use mps_formula::ffi::Vec3;
use mps_formula::galactic_dynamics as gd;
use mps_formula::heliophysics as hph;
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

/// 太阳风等离子体动压产生的力（N），沿风向推航天器。
///
/// 模型：`F = P_sw · A_eff · d̂`，其中 `P_sw = ρ · v_rel²` 是太阳风（质子数
/// 密度 × m_p × 整体速度²）作用于物体的动压，由
/// [`heliophysics::solar_wind_dynamic_pressure`] 计算（返回 nPa，本函数换算到
/// Pa 后乘以有效面积以得 N）。`d̂` 取自太阳指向航天器的方向——即风自太阳
/// 向外辐射的世界方向，与 `solar_pressure_force` 共用同一几何约定。
///
/// 物体自身速度沿风向的分量被减去，得到相对风压；物体顺风而行
///（v_rel ≤ 0）感受不到力。
///
/// 接入 `mps-formula` 的 [`heliophysics::solar_wind_dynamic_pressure`]。
///
/// Inputs:
/// - `sun_to_body`        — 太阳指物体的位置向量（决定风向与距离）
/// - `body_velocity`      — 物体在惯性系的速度（m/s）
/// - `proton_density`     — 太阳风质子数密度（n / m³）
/// - `solar_wind_speed`   — 太阳风整体速度（m/s，世界系）
/// - `effective_area`     — 迎风有效面积（m²）
pub fn solar_wind_pressure_force(
    sun_to_body: Vector,
    body_velocity: Vector,
    proton_density: f64,
    solar_wind_speed: f64,
    effective_area: f64,
) -> Option<Vector> {
    // 早退：参数无意义时直接零。
    if proton_density <= 0.0 || solar_wind_speed <= 0.0 || effective_area <= 0.0 {
        return None;
    }
    let r2 = sun_to_body.length_squared();
    if r2 < 1.0 || !r2.is_finite() {
        return None;
    }
    let r = r2.sqrt();
    let dir = sun_to_body / r; // 太阳 → 物体 的单位向量（风自太阳辐射的方向）

    // 沿风向的相对速度：v_rel = v_sw - v_body·d̂。物体顺风而行则压为 0。
    let v_rel = solar_wind_speed - body_velocity.dot(dir);
    if v_rel <= 0.0 || !v_rel.is_finite() {
        return None;
    }
    // 公式要求 km/s。
    let v_rel_kms = v_rel * 1.0e-3;
    // nPa → Pa。
    let pressure_pa = hph::solar_wind_dynamic_pressure(proton_density, v_rel_kms)? * 1.0e-9;
    Some(dir * (pressure_pa * effective_area))
}

/// Chandrasekhar 动力学摩擦产生的力（N）。当一颗卫星穿过背景弥散介质（如星
/// 系盘中的暗物质晕或星际介质）时，介质粒子的引力拖尾对卫星施加反向减速力：
/// `F = -m_sat · a_df · v̂`，其中
/// `a_df = 4π G² M ρ ln Λ / v²` 由
/// [`galactic_dynamics::chandrasekhar_dynamical_friction`] 给出。
///
/// 与 mps-core 的 `DynamicalFrictionForceLaw` 等价，但 cosmos 自行复用
/// mps-formula 纯函数（不介入 mps-core 的 C ABI / 力律登记表）。
///
/// Inputs:
/// - `body_mass`          — 卫星质量（kg）；a_df 是**自身质量无关**，
///   故力 = m·a_df，质量进入合力系数
/// - `background_density` — 背景介质密度 ρ（kg/m³）
/// - `velocity`           — 卫星相对背景介质的速度（m/s）
/// - `coulomb_log`        — 库仑对数 ln Λ（典型 2–10）
pub fn dynamical_friction_force(
    body_mass: f64,
    background_density: f64,
    velocity: Vector,
    coulomb_log: f64,
) -> Option<Vector> {
    if body_mass <= 0.0 || background_density <= 0.0 || coulomb_log <= 0.0 {
        return None;
    }
    let speed = velocity.length();
    if speed <= 0.0 || !speed.is_finite() {
        // 速度为零 / 退化时摩擦无定义（公式 1/v² 发散），返回 None。
        return None;
    }
    let a_mag =
        gd::chandrasekhar_dynamical_friction(body_mass, background_density, speed, coulomb_log)?;
    // 反向于速度方向施加。
    Some(velocity / speed * (-a_mag) * body_mass)
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
