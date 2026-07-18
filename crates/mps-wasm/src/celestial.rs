use wasm_bindgen::prelude::*;
use crate::ffi;
use super::PhysicsWorld;

#[wasm_bindgen]
pub enum CelestialBody { Sun=0, Mercury=1, Venus=2, Earth=3, Moon=4, Mars=5, Jupiter=6, Saturn=7, Uranus=8, Neptune=9 }

#[wasm_bindgen]
#[derive(Clone)]
pub struct CelestialParams {
    pub gm: f64, pub equatorial_radius: f64, pub flattening: f64, pub rotation_rate: f64,
    pub j2: f64, pub j3: f64, pub j4: f64, pub j5: f64, pub j6: f64,
    pub max_degree: u32, pub ref_radius: f64,
}

#[wasm_bindgen]
impl PhysicsWorld {
    pub fn register_celestial_gravity(&mut self, body_id: u32, degree: u32) -> u64 {
        unsafe { ffi::world_register_celestial_gravity(self.handle, body_id, degree) }
    }

    pub fn get_celestial_params(&self, body_id: u32) -> Option<CelestialParams> {
        let mut gm=0.0; let mut er=0.0; let mut fl=0.0; let mut rr=0.0;
        let mut jj=[0.0f64;5]; let mut md=0u32; let mut rf=0.0; let mut sd=0.0; let mut sh=0.0;
        unsafe {
            let ok = ffi::celestial_get_body(body_id, &mut gm, &mut er, &mut fl, &mut rr, jj.as_mut_ptr(), &mut md, &mut rf, &mut sd, &mut sh);
            if ok.0 == 0 { return None; }
        }
        Some(CelestialParams { gm, equatorial_radius: er, flattening: fl, rotation_rate: rr, j2: jj[0], j3: jj[1], j4: jj[2], j5: jj[3], j6: jj[4], max_degree: md, ref_radius: rf })
    }
}