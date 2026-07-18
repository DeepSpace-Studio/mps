//! JavaScript-friendly types mirroring mps-core FFI.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }

#[wasm_bindgen]
impl Vec3 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 { Vec3 { x, y, z } }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Quat { pub i: f64, pub j: f64, pub k: f64, pub w: f64 }

#[wasm_bindgen]
impl Quat {
    #[wasm_bindgen(constructor)]
    pub fn new(i: f64, j: f64, k: f64, w: f64) -> Quat { Quat { i, j, k, w } }
    pub fn identity() -> Quat { Quat { i: 0.0, j: 0.0, k: 0.0, w: 1.0 } }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum BodyStatus { Dynamic = 0, Fixed = 1, KinematicPositionBased = 2, KinematicVelocityBased = 3 }

#[wasm_bindgen]
#[derive(Clone)]
pub struct RigidBodyDescriptor {
    pub status: BodyStatus,
    pub translation: Vec3, pub rotation: Quat,
    pub linear_velocity: Vec3, pub angular_velocity: Vec3,
    pub additional_mass: f64,
    pub linear_damping: f64, pub angular_damping: f64,
    pub can_sleep: bool, pub gravity_scale: f64, pub ccd_enabled: bool,
}

#[wasm_bindgen]
impl RigidBodyDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RigidBodyDescriptor {
        RigidBodyDescriptor {
            status: BodyStatus::Dynamic,
            translation: Vec3::new(0.,0.,0.), rotation: Quat::identity(),
            linear_velocity: Vec3::new(0.,0.,0.), angular_velocity: Vec3::new(0.,0.,0.),
            additional_mass: 1.0, linear_damping: 0.0, angular_damping: 0.0,
            can_sleep: true, gravity_scale: 1.0, ccd_enabled: false,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum ShapeType { Ball = 0, Cuboid = 1, Capsule = 2, Cylinder = 3, Cone = 4, Halfspace = 5 }

#[wasm_bindgen]
#[derive(Clone)]
pub struct ColliderDescriptor {
    pub shape_type: ShapeType,
    pub a: f64, pub b: f64, pub c: f64, pub d: f64,
    pub translation: Vec3, pub rotation: Quat,
    pub friction: f64, pub restitution: f64, pub density: f64,
    pub is_sensor: bool,
    pub collision_group: u32, pub collision_mask: u32,
}

#[wasm_bindgen]
impl ColliderDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ColliderDescriptor {
        ColliderDescriptor {
            shape_type: ShapeType::Cuboid,
            a: 0.5, b: 0.5, c: 0.5, d: 0.0,
            translation: Vec3::new(0.,0.,0.), rotation: Quat::identity(),
            friction: 0.5, restitution: 0.0, density: 1.0,
            is_sensor: false, collision_group: 0xFFFF_FFFF, collision_mask: 0xFFFF_FFFF,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct CollisionEvent {
    pub collider1: u64, pub collider2: u64,
    pub started: bool, pub sensor: bool,
}

// Conversions
impl From<mps_core::Vec3> for Vec3 {
    fn from(v: mps_core::Vec3) -> Self { Vec3 { x: v.x, y: v.y, z: v.z } }
}
impl From<Vec3> for mps_core::Vec3 {
    fn from(v: Vec3) -> Self { mps_core::Vec3 { x: v.x, y: v.y, z: v.z } }
}
impl From<mps_core::Quat> for Quat {
    fn from(q: mps_core::Quat) -> Self { Quat { i: q.i, j: q.j, k: q.k, w: q.w } }
}
impl From<Quat> for mps_core::Quat {
    fn from(q: Quat) -> Self { mps_core::Quat { i: q.i, j: q.j, k: q.k, w: q.w } }
}