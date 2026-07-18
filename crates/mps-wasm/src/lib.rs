use wasm_bindgen::prelude::*;

mod types;
mod world;
mod collider;
mod events;
mod celestial;

pub use types::*;
pub use world::*;
pub use collider::*;
pub use events::*;
pub use celestial::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Internal FFI aliases
pub(crate) mod ffi {
    pub use mps_core::rapier::world::*;
    pub use mps_core::rapier::rigid_body::*;
    pub use mps_core::rapier::collider::*;
    pub use mps_core::rapier::events::*;
    pub use mps_core::rapier::query::*;
    pub use mps_core::rapier::celestial_data::*;
    pub use mps_core::rapier::ffi::*;
    pub use mps_core::WorldHandle;
    pub use mps_core::Vec3;
    pub use mps_core::Quat;
    pub use mps_core::Bool;
    pub use mps_core::ShapeDesc;
    pub use mps_core::InteractionGroupsDesc;
    pub use mps_core::CollisionEventRecord;
    pub use mps_core::ContactForceEventRecord;
    pub use mps_core::QueryFilterDesc;
    pub use mps_core::AabbDesc;
    pub use mps_core::RayHit;
}