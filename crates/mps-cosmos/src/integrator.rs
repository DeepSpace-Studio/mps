//! 轨道积分器 —— 把天体引力 + n-body 互引力的推进从 rapier 的力注入
//! 路径里抽出来，独立用 velocity-Verlet（leapfrog）子步积分。
//!
//! ## 为什么单独抽出来
//!
//! rapier 的 `add_force` 路径是 semi-implicit Euler（`v += a·dt` 后 `x += v·dt`），
//! 对轨道力学这种长弧、保守力场会累积相位误差：1s 步长一圈 LEO 即漂数百公里。
//! velocity-Verlet 是二阶辛积分，能量有界、相位误差随 `dt²` 收敛，适合轨道。
//!
//! ## 与 rapier 的分工
//!
//! - 轨道积分（本模块）：位置/线速度由显式 Verlet 推进，绕过 rapier 的力律。
//! - rapier `step`：只跑碰撞检测、约束求解、姿态/角速度积分。为此 bodies 设
//!   `gravity_scale=0` 且不注入 `user_force`，由本模块在 step 前后直接写回
//!   `translation`/`linvel`。
//!
//! 阻力/光压等耗散力并到 Verlet 的加速度函数里（它们在一步内变化缓慢，用
//! 当前位置/速度来评估足够），保持单条积分路径，避免半步力再喂回 rapier。

#[cfg(test)]
use crate::gravity::gm_from_mass;
use crate::gravity::{CelestialSource, NBodySource, celestial_acceleration, n_body_acceleration};
use rapier3d::prelude::{RigidBody, RigidBodyHandle, RigidBodySet, Vector};

/// 评估单体加速度所需的全部"环境上下文"——把原来 `total_acceleration`
/// 11 个参数里"与具体被推体无关的共享只读数据"打包，便于在 Verlet 子步里
/// 让多个体的 `accel_fn` 闭包按引用共享同一份（而非每体克隆）。
///
/// `handle` / `mass` / `perturbation` 是 per-body 的，仍作为函数参数传入，
/// 不进 ctx。`source_positions` 是子步开头一次性拍下的 n-body 源位置快照，
/// 子步内不变，所有体共享。
#[derive(Clone)]
pub struct AccelContext<'a> {
    pub celestials: &'a [CelestialSource],
    pub n_body_sources: &'a [NBodySource],
    pub source_positions: &'a [Vector],
    pub softening_sq: f64,
    pub central_body: Option<&'a mps_formula::celestial_data::CelestialBody>,
    pub sun_position: Vector,
}

/// 对单体的总加速度评估（天体引力 + n-body + 环境扰动），返回加速度 (m/s²)。
///
/// 一处统一把"力"换算成"加速度"：阻力/光压的 `*_force` 返回 N，除以 `mass`。
/// 共享环境数据走 [`AccelContext`]；per-body 的 `handle`/`mass`/`perturbation`
/// 仍按值/引用传入。
pub fn total_acceleration(
    position: Vector,
    velocity: Vector,
    mass: f64,
    handle: RigidBodyHandle,
    ctx: &AccelContext,
    perturbation: Option<&crate::world::PerturbationConfig>,
) -> Vector {
    let AccelContext {
        celestials,
        n_body_sources,
        source_positions,
        softening_sq,
        central_body,
        sun_position,
    } = *ctx;
    let mut acc = Vector::ZERO;

    // 天体引力（纯位置函数）
    for src in celestials {
        acc += celestial_acceleration(position, src);
    }

    // n-body 互引力（位置函数，跳过自身）
    if !n_body_sources.is_empty() {
        acc += n_body_acceleration(
            position,
            n_body_sources,
            handle,
            |h| {
                let idx = h.into_raw_parts().0 as usize;
                source_positions.get(idx).copied().unwrap_or(Vector::ZERO)
            },
            softening_sq,
        );
    }

    // 环境扰动（阻力依赖速度，光压依赖位置）；力 → 加速度
    if let Some(cfg) = perturbation {
        if mass > 0.0 {
            if cfg.enable_drag {
                if let Some(central) = central_body {
                    let altitude = position.length() - central.equatorial_radius;
                    let density = crate::perturbation::atmosphere_density_at(central, altitude);
                    if density > 0.0 {
                        let atmosphere_vel = angular_velocity_of(central).cross(position);
                        if let Some(f) = crate::perturbation::atmospheric_drag_force(
                            velocity,
                            atmosphere_vel,
                            density,
                            cfg.drag_coefficient,
                            cfg.area,
                            mass,
                        ) {
                            acc += f / mass;
                        }
                    }
                }
            }
            if cfg.enable_solar && cfg.optical_area > 0.0 {
                let sun_to_body = position - sun_position;
                let r = sun_to_body.length();
                let sun_dir = if r > 1e-9 {
                    -sun_to_body / r
                } else {
                    Vector::ZERO
                };
                let f = crate::perturbation::solar_pressure_force(
                    sun_to_body,
                    sun_dir,
                    cfg.optical_area,
                    cfg.reflectivity,
                    mps_formula::celestial_data::AU,
                );
                acc += f / mass;
            }
        }
    }

    acc
}

/// 对单个动态刚体执行一步 velocity-Verlet。
///
/// `a0` 是 `r0` 处的加速度（调用方按 r0 算好，避免对所有体重复算一遍）。
/// `ctx` 是与具体被推体无关的共享只读环境（天体源、n-body 源、源位置快照、
/// 软化平方、中心天体、太阳位置），所有体共享一份 —— 闭包按引用 capture，
/// 不再每体克隆。`mass`/`handle`/`perturbation` 是 per-body 的，作为闭包
/// owned 值带进 `accel_fn`。
///
/// 推进后直接写回 rapier body 的 `translation`/`linvel`，**不唤醒**。
pub fn verlet_step(
    body: &mut RigidBody,
    a0: Vector,
    ctx: &AccelContext,
    mass: f64,
    handle: RigidBodyHandle,
    perturbation: Option<crate::world::PerturbationConfig>,
    dt: f64,
) {
    let r0 = body.translation();
    let v0 = body.linvel();

    // half kick
    let v_half = v0 + a0 * (0.5 * dt);
    // full drift
    let r1 = r0 + v_half * dt;
    // new acceleration at r1（速度用 v_half 近似新位置处的速度）
    let a1 = total_acceleration(r1, v_half, mass, handle, ctx, perturbation.as_ref());
    // half kick
    let v1 = v_half + a1 * (0.5 * dt);

    body.set_translation(r1, false);
    body.set_linvel(v1, false);
}

/// 按 n-body 源句柄拉一张位置快照表，索引 = arena index。
///
/// 构造 `O(n)`，之后 [`n_body_acceleration`] 的 `source_positions` 闭包
/// 查询 `O(1)`，替代 `Vec<(handle,pos)> + find` 的 `O(n²)`。
pub fn snapshot_source_positions(
    bodies: &RigidBodySet,
    n_body_sources: &[NBodySource],
) -> Vec<Vector> {
    let cap = bodies.len();
    let mut positions = vec![Vector::ZERO; cap];
    for s in n_body_sources {
        let idx = s.handle.into_raw_parts().0 as usize;
        if idx < cap {
            positions[idx] = bodies
                .get(s.handle)
                .map(|b| b.translation())
                .unwrap_or(Vector::ZERO);
        }
    }
    positions
}

/// 天体自转角速度矢量（假设自转轴沿 +z）。与 `world.rs` 内同名逻辑同义，
/// 抽到本模块避免循环依赖。
fn angular_velocity_of(body: &mps_formula::celestial_data::CelestialBody) -> Vector {
    Vector::new(0.0, 0.0, body.rotation_rate)
}

