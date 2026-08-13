//! 轨道诊断 — 从刚体状态提取轨道根数、能量、角动量。
//!
//! 只读 API，复用 `mps_formula::spaceflight::state_to_elements` 与
//! `mps_formula::integrators` 的纯计算函数。`Body` 参数以平移/速度向量
//! 传入，避免与 `RigidBody` 生命周期耦合。

use mps_formula::ffi::OrbitalElements;
use mps_formula::integrators::{specific_angular_momentum, specific_energy};
use mps_formula::spaceflight;
use rapier3d::prelude::Vector;

/// 刚体的瞬时位置/速度切片，用于轨道诊断。
#[derive(Clone, Copy, Debug, Default)]
pub struct BodyState {
    pub position: Vector,
    pub velocity: Vector,
}

impl BodyState {
    pub fn new(position: Vector, velocity: Vector) -> Self {
        Self { position, velocity }
    }

    fn to_ffi(self) -> mps_formula::ffi::StateVector {
        mps_formula::ffi::StateVector {
            position: mps_formula::ffi::Vec3 {
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
            },
            velocity: mps_formula::ffi::Vec3 {
                x: self.velocity.x,
                y: self.velocity.y,
                z: self.velocity.z,
            },
        }
    }
}

/// 计算瞬时 osculating 轨道根数。返回 `None` 表示轨道退化（径向/无中心质量）。
pub fn elements_of(state: BodyState, central_gm: f64) -> Option<OrbitalElements> {
    spaceflight::state_to_elements(state.to_ffi(), central_gm)
}

/// 比机械能 `E = ½v² - GM/r`。<0 绑定，=0 抛物，>0 双曲。
pub fn energy_of(state: BodyState, central_gm: f64) -> f64 {
    specific_energy(to_ffi(state.position), to_ffi(state.velocity), central_gm)
}

/// 比角动量 `h = r × v`。
pub fn angular_momentum_of(state: BodyState) -> Vector {
    let h = specific_angular_momentum(to_ffi(state.position), to_ffi(state.velocity));
    Vector::new(h.x, h.y, h.z)
}

/// 过赤道的高度（假定赤道在 xy 平面，正 z 为北极）。
pub fn height_above_equator(state: BodyState, body_radius: f64) -> f64 {
    state.position.length() - body_radius
}

fn to_ffi(v: Vector) -> mps_formula::ffi::Vec3 {
    mps_formula::ffi::Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}
