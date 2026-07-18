use wasm_bindgen::prelude::*;
use std::ptr;
use crate::ffi;
use crate::types::*;

#[inline]
fn as_bool(b: bool) -> ffi::Bool { if b { ffi::Bool::TRUE } else { ffi::Bool::FALSE } }

#[wasm_bindgen]
pub struct PhysicsWorld { pub(crate) handle: *mut ffi::WorldHandle }

#[wasm_bindgen]
impl PhysicsWorld {
    #[wasm_bindgen(constructor)]
    pub fn new(gx: f64, gy: f64, gz: f64) -> PhysicsWorld {
        let handle = unsafe { ffi::world_create(ffi::Vec3 { x: gx, y: gy, z: gz }) };
        PhysicsWorld { handle }
    }

    pub fn step(&mut self, dt: f64) { unsafe { ffi::world_step(self.handle, dt); } }
    pub fn set_gravity(&mut self, x: f64, y: f64, z: f64) {
        unsafe { ffi::world_set_gravity(self.handle, ffi::Vec3 { x, y, z }); }
    }

    pub fn get_body_count(&self) -> i32 {
        unsafe { ffi::world_get_rigid_body_set_size(self.handle) }
    }

    pub fn insert_rigid_body(&mut self, desc: &RigidBodyDescriptor) -> u64 {
        unsafe {
            let b = ffi::rigid_body_builder_create(desc.status as u32);
            ffi::rigid_body_builder_set_pose(b, desc.translation.into(), desc.rotation.into());
            ffi::rigid_body_builder_set_linvel(b, desc.linear_velocity.into());
            ffi::rigid_body_builder_set_angvel(b, desc.angular_velocity.into());
            ffi::rigid_body_builder_set_additional_mass(b, desc.additional_mass);
            ffi::rigid_body_builder_set_linear_damping(b, desc.linear_damping);
            ffi::rigid_body_builder_set_angular_damping(b, desc.angular_damping);
            ffi::world_insert_rigid_body(self.handle, ffi::rigid_body_builder_build(b))
        }
    }

    pub fn remove_rigid_body(&mut self, h: u64, remove_col: bool) {
        unsafe { ffi::world_remove_rigid_body(self.handle, h, as_bool(remove_col)); }
    }

    pub fn get_body_translation(&self, h: u64) -> Vec3 {
        unsafe { ffi::rigid_body_get_translation(self.handle, h).into() }
    }

    pub fn get_body_rotation(&self, h: u64) -> Quat {
        unsafe { ffi::rigid_body_get_rotation(self.handle, h).into() }
    }

    pub fn get_body_linear_velocity(&self, h: u64) -> Vec3 {
        unsafe { ffi::rigid_body_get_linvel(self.handle, h).into() }
    }

    pub fn get_body_angular_velocity(&self, h: u64) -> Vec3 {
        unsafe { ffi::rigid_body_get_angvel(self.handle, h).into() }
    }

    pub fn add_force(&mut self, h: u64, fx: f64, fy: f64, fz: f64, wake: bool) {
        unsafe { ffi::rigid_body_add_force(self.handle, h, ffi::Vec3{x:fx,y:fy,z:fz}, as_bool(wake)); }
    }

    pub fn add_torque(&mut self, h: u64, tx: f64, ty: f64, tz: f64, wake: bool) {
        unsafe { ffi::rigid_body_add_torque(self.handle, h, ffi::Vec3{x:tx,y:ty,z:tz}, as_bool(wake)); }
    }

    pub fn apply_impulse(&mut self, h: u64, ix: f64, iy: f64, iz: f64, wake: bool) {
        unsafe { ffi::rigid_body_apply_impulse(self.handle, h, ffi::Vec3{x:ix,y:iy,z:iz}, as_bool(wake)); }
    }

    pub fn wake_up(&mut self, h: u64) {
        unsafe { ffi::rigid_body_wake_up(self.handle, h, ffi::Bool::TRUE); }
    }

    pub fn get_body_snapshot(&self) -> js_sys::Float64Array {
        let count = self.get_body_count() as usize;
        let stride: usize = 13;
        let cap = (count * stride).max(13);
        let mut data: Vec<f64> = vec![0.0; cap];
        let mut handles: Vec<u64> = vec![0u64; count.max(1)];
        unsafe {
            ffi::world_body_snapshot(self.handle, handles.as_mut_ptr(), data.as_mut_ptr(), count as u32);
        }
        unsafe { js_sys::Float64Array::view(&data) }
    }

    pub fn destroy(&mut self) {
        if !self.handle.is_null() { unsafe { ffi::world_destroy(self.handle); } self.handle = ptr::null_mut(); }
    }
}

impl Drop for PhysicsWorld {
    fn drop(&mut self) {
        if !self.handle.is_null() { unsafe { ffi::world_destroy(self.handle); } self.handle = ptr::null_mut(); }
    }
}