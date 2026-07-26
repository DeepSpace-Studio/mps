// C ABI entry points validate raw pointers at the boundary (length/null checks
// plus `ffi_guard`), so the safe-fn-raw-pointer lint is noise here — same
// pattern as mps-core.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub extern crate rapier3d;
pub mod acoustics;
pub mod aerodynamics;
pub mod astrophysics;
pub mod biomechanics;
pub mod celestial_data;
pub mod chaos;
pub mod continuum;
pub mod control_theory;
pub mod electromagnetism;
pub mod error;
pub mod ffi;
pub mod fluid;
pub mod gravitational_models;
pub mod integrators;
pub mod material_mechanics;
pub mod math;
pub mod molecular;
pub mod nuclear;
pub mod physchem;
pub mod plasma;
pub mod quantum;
pub mod relativity;
pub mod softbody;
pub mod spaceflight;
pub mod superfluidity;
pub mod thermodynamics;
pub mod topology;
pub mod trajectory;
pub mod transmission;
pub mod wave_optics;
