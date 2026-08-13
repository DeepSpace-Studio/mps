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

use crate::gravity::{CelestialSource, NBodySource, celestial_acceleration};
use rapier3d::prelude::{RigidBody, RigidBodyHandle, RigidBodySet, Rotation, Vector};

/// 评估单体加速度所需的全部"环境上下文"——把原来 `total_acceleration`
/// 11 个参数里"与具体被推体无关的共享只读数据"打包，便于在 Verlet 子步里
/// 让多个体的 `accel_fn` 闭包按引用共享同一份（而非每体克隆）。
///
/// `handle` / `mass` / `perturbation` 是 per-body 的，仍作为函数参数传入，
/// 不会进 ctx。`source_positions` 与 `source_rotations` 是子步开头一次性拍下的
/// n-body 源质心位置与姿态快照，子步内不变、所有体共享。`relativistic` 控制
/// 中心天体引力的 1PN/2PN 修正。
#[derive(Clone, Copy)]
pub struct AccelContext<'a> {
    pub celestials: &'a [CelestialSource],
    pub n_body_sources: &'a [NBodySource],
    pub source_positions: &'a [Vector],
    pub source_rotations: &'a [Rotation],
    pub softening_sq: f64,
    pub central_body: Option<&'a mps_formula::celestial_data::CelestialBody>,
    pub sun_position: Vector,
    pub relativistic: crate::world::RelativisticCorrection,
}

/// 不规则质量分布近场分支的姿态 fallback：源刚体被移除或快照表未覆盖某
/// arena index 时按这个 `IDENTITY` 算（在该分支的近场状态下等价于"源没转"）。
pub const DEFAULT_ROT: Rotation = Rotation::IDENTITY;

/// 对单体的总加速度评估（天体引力 + n-body + 环境扰动），返回加速度 (m/s²)。
///
/// 一处统一把"力"换算成"加速度"：阻力/光压的 `*_force` 返回 N，除以 `mass`。
/// 共享环境数据走 [`AccelContext`]；per-body 的 `handle`/`mass`/`perturbation`
/// 仍按值/引用传入。
#[inline]
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
        source_rotations,
        softening_sq,
        central_body,
        sun_position,
        relativistic,
    } = *ctx;
    let mut acc = Vector::ZERO;

    // 天体引力（纯位置函数）。空请求是常见配置（只挂中心天体一项），加早退；
    // 中心天体另叠加相对论后牛顿修正。
    if celestials.is_empty() {
        // 无天体源：跳过整个循环，含相对论修正也无处叠加。
    } else {
        let has_relativistic = !matches!(relativistic, crate::world::RelativisticCorrection::None);
        for src in celestials {
            acc += celestial_acceleration(position, src);
            if has_relativistic
                && let Some(central) = central_body
                && std::ptr::eq(src.body as *const _, central as *const _)
            {
                acc += relativistic_acceleration(position, velocity, central.gm, relativistic);
            }
        }
    }

    // n-body 互引力（位置函数，跳过自身）
    // n_body_sources 为空时跳过：不构造闭包、不走循环，是单体的快路径。
    // 非空时直接用 slice 索引取源位置（替代闭包闭包 + .get(idx).copied()
    // 的虚调用路径）；位置快照已按 arena 容量建好，索引必在界内。
    // 不规则质量分布分支：当源带 `points` 且 near field（dist ≤
    // `src.near_field_threshold()`）时按 Σ G·mᵢ·dᵢ/|dᵢ|³ 求和，把 `local_offset`
    // 经源姿态变到世界坐标；远场/无 points → 单 monopole。
    if !n_body_sources.is_empty() {
        let exclude = handle.into_raw_parts().0 as usize;
        let mut acc_nb = Vector::ZERO;
        for src in n_body_sources {
            let src_idx = src.handle.into_raw_parts().0 as usize;
            if src_idx == exclude || src.gm <= 0.0 {
                continue;
            }
            // 源快照按 arena index 建，索引必在界内；防御性 fallback ZERO。
            let r_j = source_positions
                .get(src_idx)
                .copied()
                .unwrap_or(Vector::ZERO);
            let d = r_j - position;
            let dist_sq = d.length_squared() + softening_sq;
            if dist_sq < 1.0 {
                continue;
            }
            let dist = dist_sq.sqrt();
            let near_threshold = src.near_field_threshold();
            if !src.points.is_empty() && near_threshold > 0.0 && dist <= near_threshold {
                let rot = source_rotations
                    .get(src_idx)
                    .copied()
                    .unwrap_or(DEFAULT_ROT);
                for mp in &src.points {
                    if mp.gm <= 0.0 {
                        continue;
                    }
                    let world = r_j + rot * mp.local_offset;
                    let d_i = world - position;
                    let dist_sq_i = d_i.length_squared() + softening_sq;
                    if dist_sq_i < 1.0 {
                        continue;
                    }
                    let dist_i = dist_sq_i.sqrt();
                    acc_nb += d_i * (mp.gm / (dist_sq_i * dist_i));
                }
            } else {
                acc_nb += d * (src.gm / (dist_sq * dist));
            }
        }
        acc += acc_nb;
    }

    // 环境扰动（阻力依赖速度，光压依赖位置）；力 → 加速度
    if let Some(cfg) = perturbation
        && mass > 0.0
    {
        if cfg.enable_drag
            && let Some(central) = central_body
        {
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

    acc
}

/// 中心天体引力的相对论后牛顿修正加速度。
///
/// 仅对 `central_body` 叠加 1PN/2PN 项；n-body 与扰动不修正。`gm` 取自中心
/// 天体。`None` 模式返回零。所有路径走 `mps_formula::integrators` 的现成实现。
fn relativistic_acceleration(
    position: Vector,
    velocity: Vector,
    gm: f64,
    mode: crate::world::RelativisticCorrection,
) -> Vector {
    use crate::world::RelativisticCorrection as R;
    let pos = ffi_vec3(position);
    let vel = ffi_vec3(velocity);
    let pred = match mode {
        R::None => return Vector::ZERO,
        R::OnePN => mps_formula::integrators::post_newtonian_1pn(pos, vel, gm, 0.0),
        R::TwoPN => mps_formula::integrators::post_newtonian_2pn(pos, vel, gm),
        R::Full => mps_formula::integrators::post_newtonian_full(pos, vel, gm),
    };
    rapier_vec(pred)
}

/// `rapier3d::Vector` ↔ `mps_formula::ffi::Vec3` 双向转换 helper。
#[inline]
pub fn ffi_vec3_pub(v: Vector) -> mps_formula::ffi::Vec3 {
    ffi_vec3(v)
}
#[inline]
fn ffi_vec3(v: Vector) -> mps_formula::ffi::Vec3 {
    mps_formula::ffi::Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}
#[inline]
fn rapier_vec(v: mps_formula::ffi::Vec3) -> Vector {
    Vector::new(v.x, v.y, v.z)
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
/// 构造 `O(n)`，之后 n-body 互引力查询 `O(1)`，替代
/// `Vec<(handle,pos)> + find` 的 `O(n²)`。
///
/// 仍保留为测试与外部诊断入口；`CosmosWorld` 内部热路径已改用复用缓冲
/// （`scratch_source_positions`）直接写入，不经过本函数。
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

// ---------------------------------------------------------------------------
// 高阶辛积子包装（Yoshida4 / Forest-Ruth8 / 各自 Kahan 版）
// ---------------------------------------------------------------------------

/// 构造一个 `Fn(mps_formula::ffi::Vec3) -> mps_formula::ffi::Vec3` 的纯位置型
/// 加速度闭包，供 `mps_formula::integrators::yoshida4_step` /
/// `forest_ruth8_step` 这类**位置导数型**辛积子使用。
///
/// 速度依赖项（大气阻力）在一步内变化缓慢，用**闭包构造时刻冻结的 `v0`**
/// 评估，作为"伪位置依赖"——一步内变化缓慢，足够准（与 Verlet 路径里
/// `v_half` 近似的精度同阶，辛积子阶数主导误差）。这样不破坏辛结构。
///
/// 闭包 `move` 捕获 `ctx` 引用 + per-body 的 `handle`/`mass`/`perturbation`/
/// `v0`，对每个 `(w·dt)` 子级评估时统一从这同一份环境取数据，无克隆。
fn accel_fn_positional<'a>(
    ctx: &'a AccelContext<'a>,
    handle: RigidBodyHandle,
    mass: f64,
    v0: Vector,
    perturbation: Option<crate::world::PerturbationConfig>,
) -> impl Fn(mps_formula::ffi::Vec3) -> mps_formula::ffi::Vec3 + 'a {
    move |p: mps_formula::ffi::Vec3| {
        let pos = rapier_vec(p);
        let a = total_acceleration(pos, v0, mass, handle, ctx, perturbation.as_ref());
        ffi_vec3(a)
    }
}

/// 动态读取当前 `vel` 的位置型加速度闭包——闭包持 `&mut vel`，每次评估前
/// 取其当前值作为"速度依赖项"的输入。这样高阶辛积子各子级更新速度后，
/// 下一级加速度评估即用更新后速度（保辛结构的前提下让阻力逐级重估）。
/// 对纯保守引力（无阻力）等价于 `accel_fn_positional`。
///
/// **借用限制**：`run_highorder` 也需 `&mut vel`，二者不可同时持可变借用。
/// 故动态速度路径目前未接入（保留函数便于后续以 `Rc<RefCell<Vec3>>` 等重构）；
/// 现路径用冻结 `v0` 的 `accel_fn_positional`，对纯保守引力无损。
#[allow(dead_code)]
fn accel_fn_positional_dyn<'a>(
    ctx: &'a AccelContext<'a>,
    handle: RigidBodyHandle,
    mass: f64,
    vel_ref: &'a mut mps_formula::ffi::Vec3,
    perturbation: Option<crate::world::PerturbationConfig>,
) -> impl Fn(mps_formula::ffi::Vec3) -> mps_formula::ffi::Vec3 + 'a {
    move |p: mps_formula::ffi::Vec3| {
        let pos = rapier_vec(p);
        let v_now = rapier_vec(*vel_ref);
        let a = total_acceleration(pos, v_now, mass, handle, ctx, perturbation.as_ref());
        ffi_vec3(a)
    }
}

/// 按 `mode` 调度一次高阶辛积子推进（位置/速度 `Vec3` 原地更新）。
/// 把 dispatch 写成内联 match，避免函数指针签名里 `impl Fn` 不被允许的问题。
fn run_highorder(
    mode: crate::world::OrbitIntegration,
    pos: &mut mps_formula::ffi::Vec3,
    vel: &mut mps_formula::ffi::Vec3,
    dt: f64,
    accel_fn: impl Fn(mps_formula::ffi::Vec3) -> mps_formula::ffi::Vec3,
) {
    use mps_formula::integrators::{forest_ruth8_step, yoshida4_step};
    match mode {
        crate::world::OrbitIntegration::Yoshida4
        | crate::world::OrbitIntegration::Yoshida4Kahan => {
            yoshida4_step(pos, vel, dt, accel_fn);
        }
        crate::world::OrbitIntegration::ForestRuth8
        | crate::world::OrbitIntegration::ForestRuth8Kahan => {
            forest_ruth8_step(pos, vel, dt, accel_fn);
        }
        _ => unreachable!("run_highorder 只接受 Yoshida4 / ForestRuth8 系"),
    }
}

/// 对单个动态刚体执行一步高阶辛积子（Yoshida4 或 Forest-Ruth8）。
///
/// 与 [`verlet_step`] 同样的"绕过 rapier 力律、直接写回 translation/linvel"约定。
/// `mode` 决定阶数；`accel_fn` 在本步内冻结 `body.linvel()` 为 `v0`，整步内
/// 不更新速度（阻力用 `v0` 评估，伪位置依赖）。
pub fn explicit_highorder_step(
    body: &mut RigidBody,
    mass: f64,
    handle: RigidBodyHandle,
    perturbation: Option<crate::world::PerturbationConfig>,
    ctx: &AccelContext,
    dt: f64,
    mode: crate::world::OrbitIntegration,
) {
    let r0 = body.translation();
    let v0 = body.linvel();
    let mut pos = ffi_vec3(r0);
    let mut vel = ffi_vec3(v0);
    // 加速度评估冻结 v0：速度依赖项（阻力）在整步内用 v0 估算。对纯保守引力
    // （n-body + 中心引力，无阻力）v0 不进入加速度，无影响；阻力场景下相位精度
    // 退化为 O(dt·τ_drag)，但阻力本身非保守，辛性本就被破，可接受。需要阻力高精
    // 长弧应改走 Verlet 路径或后续接入动态速度闭包。
    let accel_fn = accel_fn_positional(ctx, handle, mass, v0, perturbation);
    run_highorder(mode, &mut pos, &mut vel, dt, accel_fn);
    body.set_translation(rapier_vec(pos), false);
    body.set_linvel(rapier_vec(vel), false);
}

/// 高阶辛积子 + Kahan 补偿位置/速度累加版。
///
/// `state` 是 per-body 的 `(pos_accum, vel_accum)` Kahan 累加态，由调用方在
/// `world` 里按 arena index 缓存。入口把 rapier body 的当前位姿同步进累加态
/// （首次或被外部改动后），出口把补偿后的位姿写回 rapier body。
///
/// 高阶积子的 `v += a·dt` / `r += v·dt` 仍是普通 `+=`；这里把"积子内部生成的
/// 增量"喂给 Kahan 累加器——即每步前快照 `(r0,v0)`，积子算完拿到 `(r1,v1)`，
/// 用 Kahan 累加 `Δr=r1-r0`、`Δv=v1-v0`，再写回 body。数值上逼近 Kahan 全程
/// 累加，且不破坏辛结构。
pub fn explicit_highorder_kahan_step(
    body: &mut RigidBody,
    state: &mut (mps_formula::math::KahanVec3, mps_formula::math::KahanVec3),
    mass: f64,
    handle: RigidBodyHandle,
    perturbation: Option<crate::world::PerturbationConfig>,
    ctx: &AccelContext,
    dt: f64,
    mode: crate::world::OrbitIntegration,
) {
    let r0 = body.translation();
    let v0 = body.linvel();
    // 把外部可能直接写入 body 的位姿增量并入累加态，再以累加态为基础向前推。
    let mut pos = state.0.value();
    let mut vel = state.1.value();
    // 若 body 与累加态不一致（外部 set_translation 等），以 body 为准重置累加态。
    let rapier_pos = ffi_vec3(r0);
    let rapier_vel = ffi_vec3(v0);
    let pos_drift =
        (pos.x - rapier_pos.x).abs() + (pos.y - rapier_pos.y).abs() + (pos.z - rapier_pos.z).abs();
    let vel_drift =
        (vel.x - rapier_vel.x).abs() + (vel.y - rapier_vel.y).abs() + (vel.z - rapier_vel.z).abs();
    if pos_drift > 1e-9 || vel_drift > 1e-12 {
        state.0 = mps_formula::math::KahanVec3::new(rapier_pos);
        state.1 = mps_formula::math::KahanVec3::new(rapier_vel);
        pos = rapier_pos;
        vel = rapier_vel;
    }

    let accel_fn = accel_fn_positional(ctx, handle, mass, v0, perturbation);
    run_highorder(mode, &mut pos, &mut vel, dt, accel_fn);

    let delta_pos = mps_formula::ffi::Vec3 {
        x: pos.x - state.0.value().x,
        y: pos.y - state.0.value().y,
        z: pos.z - state.0.value().z,
    };
    let delta_vel = mps_formula::ffi::Vec3 {
        x: vel.x - state.1.value().x,
        y: vel.y - state.1.value().y,
        z: vel.z - state.1.value().z,
    };
    state.0.add(delta_pos);
    state.1.add(delta_vel);

    body.set_translation(state.0.value_vec(), false);
    body.set_linvel(state.1.value_vec(), false);
}
