use wasm_bindgen::prelude::*;
use crate::ffi;
use crate::types::*;
use super::PhysicsWorld;

#[wasm_bindgen]
impl PhysicsWorld {
    pub fn insert_collider(&mut self, desc: &ColliderDescriptor, parent: u64) -> u64 {
        unsafe {
            let shape = ffi::ShapeDesc { shape_type: desc.shape_type as u32, a: desc.a, b: desc.b, c: desc.c, d: desc.d };
            let b = ffi::collider_builder_create_ex(shape);
            ffi::collider_builder_set_translation(b, desc.translation.into());
            ffi::collider_builder_set_pose(b, desc.translation.into(), desc.rotation.into());
            ffi::collider_builder_set_friction(b, desc.friction);
            ffi::collider_builder_set_restitution(b, desc.restitution);
            ffi::collider_builder_set_density(b, desc.density);
            if desc.is_sensor { ffi::collider_builder_set_sensor(b, ffi::Bool::TRUE); }
            let groups = ffi::InteractionGroupsDesc { memberships: desc.collision_group, filter: desc.collision_mask };
            ffi::collider_builder_set_collision_groups(b, groups);
            ffi::world_insert_collider_with_parent(self.handle, ffi::collider_builder_build(b), parent)
        }
    }

    pub fn remove_collider(&mut self, h: u64) {
        unsafe { ffi::world_remove_collider(self.handle, h, ffi::Bool::TRUE); }
    }

    pub fn create_ground_plane(&mut self, nx: f64, ny: f64, nz: f64, dist: f64) -> u64 {
        let d = RigidBodyDescriptor { status: BodyStatus::Fixed, additional_mass: 0.0, ..RigidBodyDescriptor::new() };
        let body = self.insert_rigid_body(&d);
        let c = ColliderDescriptor {
            shape_type: ShapeType::Halfspace, a: nx, b: ny, c: nz, d: dist,
            friction: 0.5, restitution: 0.1, density: 0.0, ..ColliderDescriptor::new()
        };
        self.insert_collider(&c, body);
        body
    }

    pub fn create_dynamic_box(&mut self, px: f64, py: f64, pz: f64, hx: f64, hy: f64, hz: f64, mass: f64) -> u64 {
        let d = RigidBodyDescriptor {
            status: BodyStatus::Dynamic, translation: Vec3::new(px, py, pz), additional_mass: mass,
            ..RigidBodyDescriptor::new()
        };
        let body = self.insert_rigid_body(&d);
        let c = ColliderDescriptor {
            shape_type: ShapeType::Cuboid, a: hx, b: hy, c: hz, d: 0.0,
            friction: 0.5, restitution: 0.1, density: 1.0, ..ColliderDescriptor::new()
        };
        self.insert_collider(&c, body);
        body
    }

    pub fn create_dynamic_sphere(&mut self, px: f64, py: f64, pz: f64, r: f64, mass: f64) -> u64 {
        let d = RigidBodyDescriptor {
            status: BodyStatus::Dynamic, translation: Vec3::new(px, py, pz), additional_mass: mass,
            ..RigidBodyDescriptor::new()
        };
        let body = self.insert_rigid_body(&d);
        let c = ColliderDescriptor {
            shape_type: ShapeType::Ball, a: r, b: 0.0, c: 0.0, d: 0.0,
            friction: 0.5, restitution: 0.3, density: 1.0, ..ColliderDescriptor::new()
        };
        self.insert_collider(&c, body);
        body
    }
}