use rapier3d::prelude::Vector;

use super::types::Vec3;

pub fn vec3_to_rapier(value: Vec3) -> Vector {
    Vector::new(value.x, value.y, value.z)
}

pub fn vec3_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

pub fn vec3_from_rapier(value: Vector) -> Vec3 {
    Vec3 { x: value.x, y: value.y, z: value.z }
}

pub fn finite_non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

pub fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

pub fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}
