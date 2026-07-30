//! 天体重力与 n-body 互引力计算。
//!
//! `celestial_acceleration` 复刻 `mps-core/src/rapier/interaction.rs` 的
//! `CelestialGravityForceLaw` 自适应模型分支（<2R 椭球、<10R 球谐、
//! <100R J2–J6 带谐、>100R 点质量），但作为独立实现直接调用
//! `mps_formula::gravitational_models`，不依赖 `mps-core`。
//!
//! `n_body_acceleration` 实现经典 n-body 互引力，可加 softening 防奇点。

use mps_formula::celestial_data::{CelestialBody, G};
use mps_formula::ffi::Vec3;
use mps_formula::gravitational_models::{
    ellipsoid_gravity, spherical_harmonics_acceleration, zonal_harmonics_acceleration,
};
use rapier3d::prelude::{RigidBodyHandle, Vector};

/// 一个注册到世界中的天体引力源。
///
/// `max_sh_degree` 限制球谐展开的最高阶；实际还会被 `body.max_degree` 约束。
#[derive(Clone, Copy)]
pub struct CelestialSource {
    pub body: &'static CelestialBody,
    /// 球谐模型最高阶（受 `body.max_degree` 上限约束）。
    pub max_sh_degree: u32,
    /// 是否启用本天体的引力（关闭则贡献为 0）。
    pub enabled: bool,
}

impl CelestialSource {
    pub fn new(body: &'static CelestialBody, max_sh_degree: u32) -> Self {
        Self {
            body,
            max_sh_degree,
            enabled: true,
        }
    }
}

/// 一个参与 n-body 互引力的质点源。
///
/// 通常对应场景中一个已插入的动态刚体：`handle` 指向其 `RigidBodySet` 句柄，
/// `gm = G·mass` 是其引力参数。受力刚体不应对自身引用，`n_body_acceleration`
/// 通过 `exclude_handle` 跳过自身。
#[derive(Clone, Copy)]
pub struct NBodySource {
    pub handle: RigidBodyHandle,
    pub gm: f64,
}

/// 自适应选择最匹配的引力模型并返回在该 `position` 处的引力加速度。
///
/// 选择规则（[`mps-core`] 的 `CelestialGravityForceLaw` 参考）：
///
/// | 归一化高度 r/R_eq | 椭率>0 且有 SH | 选用的模型 |
/// |---|---|---|
/// | <2   | 椭率>0       | 椭球 `ellipsoid_gravity` |
/// | 2–10 | max_sh≥2 且有 SH 系数 | 球谐 `spherical_harmonics_acceleration` |
/// | 10–100 | 任意       | J2–J6 带谐 `zonal_harmonics_acceleration` |
/// | >100 | 任意         | 点质量 + J2 |
pub fn celestial_acceleration(position: Vector, source: &CelestialSource) -> Vector {
    if !source.enabled {
        return Vector::ZERO;
    }

    let gm = source.body.gm;
    let r_eq = source.body.equatorial_radius;
    if r_eq <= 0.0 {
        // 退化天体：点质量
        return point_mass_acceleration(position, gm);
    }

    let r = position.length();
    if r < 1.0 {
        return Vector::ZERO; // 位于天体内部母点：无意义
    }

    let normalized_altitude = r / r_eq;
    let pos = Vec3 {
        x: position.x,
        y: position.y,
        z: position.z,
    };

    let accel = if normalized_altitude < 2.0 && source.body.flattening > 0.0 {
        ellipsoid_gravity(pos, source.body)
    } else if normalized_altitude < 10.0
        && source.max_sh_degree >= 2
        && !source.body.c_coeffs.is_empty()
    {
        spherical_harmonics_acceleration(pos, source.body, source.max_sh_degree)
    } else if normalized_altitude < 100.0 {
        let jn: [f64; 5] = [
            source.body.j2,
            source.body.j3,
            source.body.j4,
            source.body.j5,
            source.body.j6,
        ];
        let jn_filtered: Vec<f64> = jn.iter().copied().filter(|&j| j != 0.0).collect();
        zonal_harmonics_acceleration(pos, gm, r_eq, &jn_filtered)
    } else {
        let jn = if source.body.j2 != 0.0 {
            vec![source.body.j2]
        } else {
            vec![]
        };
        zonal_harmonics_acceleration(pos, gm, r_eq, &jn)
    };

    Vector::new(accel.x, accel.y, accel.z)
}

/// 点质量引力加速度 `a = -GM · r̂ / r²`。
pub fn point_mass_acceleration(position: Vector, gm: f64) -> Vector {
    let r2 = position.length_squared();
    if r2 < 1.0 {
        return Vector::ZERO;
    }
    let r = r2.sqrt();
    -position * (gm / (r2 * r))
}

/// 对位于 `position`（世界坐标）的质点，求所有 n-body 源贡献的互引力加速度。
///
/// 跳过 `exclude_handle`（通常是质点自身对应的刚体），以避免自吸引奇点。
/// `softening` 平方项加在分母上防止两体无限接近时发散（典型取几公里量级
/// 的平方，0 表示不加 soften）。
pub fn n_body_acceleration(
    position: Vector,
    sources: &[NBodySource],
    exclude_handle: RigidBodyHandle,
    source_positions: impl Fn(RigidBodyHandle) -> Vector,
    softening_sq: f64,
) -> Vector {
    let mut acc = Vector::ZERO;
    for src in sources {
        if src.handle == exclude_handle || src.gm <= 0.0 {
            continue;
        }
        let r_j = source_positions(src.handle);
        let d = r_j - position;
        let dist_sq = d.length_squared() + softening_sq;
        if dist_sq < 1.0 {
            continue;
        }
        let dist = dist_sq.sqrt();
        // a = G·m_j · (r_j - r) / |r_j - r|³  =  gm · d / dist³
        acc += d * (src.gm / (dist_sq * dist));
    }
    acc
}

/// 由质量恢复引力参数 `gm = G · mass`（与 `mps_formula::celestial_data::G` 一致）。
pub fn gm_from_mass(mass: f64) -> f64 {
    G * mass
}

