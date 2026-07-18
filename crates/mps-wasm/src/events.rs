use wasm_bindgen::prelude::*;
use crate::ffi;
use crate::types::*;
use super::PhysicsWorld;

#[wasm_bindgen]
impl PhysicsWorld {
    pub fn get_collision_event_count(&self) -> u32 {
        unsafe { ffi::world_collision_event_count(self.handle) }
    }

    pub fn get_collision_event(&self, index: u32) -> CollisionEvent {
        unsafe {
            let raw = ffi::world_get_collision_event(self.handle, index);
            CollisionEvent { collider1: raw.collider1, collider2: raw.collider2, started: raw.started.0 != 0, sensor: raw.sensor.0 != 0 }
        }
    }

    pub fn get_collision_events(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for i in 0..self.get_collision_event_count() {
            let evt = self.get_collision_event(i);
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"collider1".into(), &evt.collider1.into()).ok();
            js_sys::Reflect::set(&obj, &"collider2".into(), &evt.collider2.into()).ok();
            js_sys::Reflect::set(&obj, &"started".into(), &evt.started.into()).ok();
            arr.push(&obj);
        }
        arr
    }

    pub fn clear_events(&mut self) { unsafe { ffi::world_clear_events(self.handle); } }
}