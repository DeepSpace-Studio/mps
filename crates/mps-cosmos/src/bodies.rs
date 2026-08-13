//! 太空刚体构造 helper。
//!
//! 对常用航天器参数（质量、初始位置/速度、可选姿态）做轻量包装，内部
//! 调用 `rapier3d::prelude::RigidBodyBuilder::dynamic()` + 设置附加质量
//! 与平移/线速度。返回 `RigidBodyBuilder` 仍可链式进一步配置后交给
//! [`crate::CosmosWorld::insert_body`]。

use rapier3d::prelude::{MassProperties, RigidBodyBuilder, Vector};

/// 构造一个动态刚体 builder，给定质量（kg）、初始位置与速度（SI 单位）。
///
/// 惯性张量设为均匀球体 `I = 2/5 · m · r²`，其中 `r` 由 `radius` 给出
/// （仅用于估算惯性，不创建 collider）。设 `radius <= 0` 时惯量取小正值
/// 避免奇异。
pub fn satellite_builder(
    mass: f64,
    position: Vector,
    velocity: Vector,
    radius: f64,
) -> RigidBodyBuilder {
    let inertia = if radius > 0.0 && mass > 0.0 {
        (2.0 / 5.0) * mass * radius * radius
    } else {
        1.0e-3 // 退化：极小但非零惯量
    };
    RigidBodyBuilder::dynamic()
        // rapier 0.35 从\"线速度阈值\"改成\"姿态漂移\"判 sleep——空跑的卫星在
        // 大 `dt` 下会单步触发 sleep 并把 linvel 清零，与项目对\"永恒漂浮天体\"
        // 的语义冲突。对刚发射的卫星永远禁用 sleep，归属它们的速度完全由我们
        // 的积子或 Rapier 力律决定（fixed_body_builder 仍可 sleep，但 fixed 体不
        // 受 sleep 影响速度）。
        .can_sleep(false)
        .additional_mass_properties(MassProperties::new(
            Vector::ZERO,
            mass,
            Vector::splat(inertia),
        ))
        .translation(position)
        .linvel(velocity)
}

/// 构造一个固定（静态）刚体 builder，仅指定位置 —— 适合作为 n-body 引力源
/// 中心（如恒星/行星本体），不参与动态推进。
pub fn fixed_body_builder(position: Vector) -> RigidBodyBuilder {
    RigidBodyBuilder::fixed().translation(position)
}
