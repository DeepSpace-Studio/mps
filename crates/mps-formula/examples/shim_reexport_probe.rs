//! Ad-hoc probe: verify that after rewriting
//! crates/mps-formula/src/{fluid,nuclear,aerodynamics,cosmology}.rs
//! as pure `pub use crate::scientists::*::formulas::*;` shims, the
//! existing call paths `mps_formula::<shim>::<fn>` still resolve and
//! produce the same numerical results as the scientists' definitions.
//!
//! NOT a substitute for the full test suite — a targeted behavioral probe
//! of the 4 changed shim files + the new `pub mod scientists; pub mod
//! disciplines;` hookup in lib.rs.
//!
//! Run: cargo run --example shim_reexport_probe --release -p mps-formula

use mps_formula::aerodynamics;
use mps_formula::cosmology;
use mps_formula::ffi::{AeroSurface, Vec3};
use mps_formula::fluid;
use mps_formula::nuclear;

fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let mut checks = 0u32;

    // ===== fluid.rs shim → scientists::{leonhard_euler, daniel_bernoulli, george_stokes, claude_louis_navier} =====
    {
        // bernoulli_pressure (daniel_bernoulli): 5 args, returns f64.
        // P = total_pressure + 0.5 * density * velocity^2 + density * gravity * elevation
        let got = fluid::bernoulli_pressure(101_325.0, 1.225, 10.0, 9.81, 0.0);
        let want = 101_325.0 + 0.5 * 1.225 * 100.0 + 1.225 * 9.81 * 0.0;
        checks += 1;
        if !approx_eq(got, want, 1.0e-6) {
            failures.push(format!("fluid::bernoulli_pressure: got {got}, want {want}"));
        }
    }
    {
        // re_n (daniel_bernoulli): Re = rho*v*D/mu
        let got = fluid::re_n(1000.0, 1.0, 0.1, 1.0e-3).unwrap();
        let want = 100_000.0;
        checks += 1;
        if !approx_eq(got, want, 1.0e-3) {
            failures.push(format!("fluid::re_n: got {got}, want {want}"));
        }
    }
    {
        // sph_poly6_kernel (leonhard_euler): at distance 0, value = 315/(64*pi)
        let h = 1.0_f64;
        let got = fluid::sph_poly6_kernel(0.0, h);
        let want = 315.0 / (64.0 * std::f64::consts::PI) * h.powi(9);
        checks += 1;
        if !approx_eq(got, want, 1.0e-6) {
            failures.push(format!("fluid::sph_poly6_kernel: got {got}, want {want}"));
        }
    }
    {
        // isentropic_pressure_ratio (george_stokes): (1 + 0.2*M^2)^(3.5)
        let got = fluid::isentropic_pressure_ratio(1.0, 1.4).unwrap();
        let want = 1.2_f64.powf(3.5);
        checks += 1;
        if !approx_eq(got, want, 1.0e-4) {
            failures.push(format!("fluid::isentropic_pressure_ratio: got {got}, want {want}"));
        }
    }
    {
        // isentropic_density_ratio (george_stokes): (1 + 0.2*M^2)^(2.5)
        let got = fluid::isentropic_density_ratio(1.0, 1.4).unwrap();
        let want = 1.2_f64.powf(2.5);
        checks += 1;
        if !approx_eq(got, want, 1.0e-4) {
            failures.push(format!("fluid::isentropic_density_ratio: got {got}, want {want}"));
        }
    }
    {
        // atwood_number (claude_louis_navier): Option<f64> = (rho_heavy - rho_light) / (rho_heavy + rho_light)
        let got = fluid::atwood_number(2.0, 1.0).unwrap();
        let want = 1.0 / 3.0;
        checks += 1;
        if !approx_eq(got, want, 1.0e-9) {
            failures.push(format!("fluid::atwood_number: got {got}, want {want}"));
        }
    }

    // ===== nuclear.rs shim → scientists::{enrico_fermi, ernest_rutherford} =====
    {
        // decay_constant (enrico_fermi): lambda = ln2 / T_half
        let t_half = 5730.0 * 365.25 * 86400.0;
        let got = nuclear::decay_constant(t_half).unwrap();
        let want = std::f64::consts::LN_2 / t_half;
        checks += 1;
        if !approx_eq(got, want, 1.0e-20) {
            failures.push(format!("nuclear::decay_constant: got {got}, want {want}"));
        }
    }
    {
        // remaining_nuclei (enrico_fermi): N(t) = N0 * exp(-lambda*t)
        let lambda = std::f64::consts::LN_2 / 100.0;
        let got = nuclear::remaining_nuclei(1_000_000.0, lambda, 100.0).unwrap();
        let want = 500_000.0; // one half-life
        checks += 1;
        if !approx_eq(got, want, 1.0e-3) {
            failures.push(format!("nuclear::remaining_nuclei: got {got}, want {want}"));
        }
    }
    {
        // reaction_q_value (ernest_rutherford): 2 args, Option<f64>; Q = (m_initial - m_final) * 931.5 MeV
        let got = nuclear::reaction_q_value(5.0, 4.0);  // 931.5 MeV positive
        checks += 1;
        if got.is_none() || got.unwrap() <= 0.0 {
            failures.push(format!("nuclear::reaction_q_value: got {:?}, want finite+", got));
        }
    }
    {
        // specific_activity (ambiguous enrico_fermi / marie_curie; shim points to enrico_fermi)
        let got = nuclear::specific_activity(1.0e-6, 60.0).unwrap();
        let want = 1.0e-6 * 6.022_140_76e23 / 60.0;
        checks += 1;
        if !approx_eq(got, want, 1.0e3) {
            failures.push(format!("nuclear::specific_activity: got {got}, want {want}"));
        }
    }
    {
        // half_value_layer (ambiguous; shim points to enrico_fermi)
        let got = nuclear::half_value_layer(0.5).unwrap();
        let want = std::f64::consts::LN_2 / 0.5;
        checks += 1;
        if !approx_eq(got, want, 1.0e-9) {
            failures.push(format!("nuclear::half_value_layer: got {got}, want {want}"));
        }
    }
    {
        // dt_fusion_energy (ernest_rutherford): finite positive
        let got = nuclear::dt_fusion_energy();
        checks += 1;
        if !got.is_finite() || got <= 0.0 {
            failures.push(format!("nuclear::dt_fusion_energy: got {got}, want finite positive"));
        }
    }

    // ===== aerodynamics.rs shim → scientists::daniel_bernoulli =====
    // compute_surface_force(surface, linvel, angvel, center, wind, density)
    // estimate_surface_force(linvel, angvel, center, wind, density, surface)
    {
        let surface = AeroSurface {
            point: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            normal: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            area: 1.0,
            drag_coefficient: 0.5,
            lift_coefficient: 0.4,
        };
        let linvel = Vec3 { x: 10.0, y: 0.0, z: 0.0 };
        let angvel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let center = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let wind = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let got = aerodynamics::compute_surface_force(surface, linvel, angvel, center, wind, 1.225);
        checks += 1;
        if got.is_none() {
            failures.push("aerodynamics::compute_surface_force: got None, want Some".into());
        }
        let got2 = aerodynamics::estimate_surface_force(linvel, angvel, center, wind, 1.225, surface);
        checks += 1;
        if got2.is_none() {
            failures.push("aerodynamics::estimate_surface_force: got None, want Some".into());
        }
    }

    // ===== cosmology.rs shim → scientists::albert_einstein =====
    // friedmann_hubble_distance(hubble_constant, redshift)
    // hubble_flow_velocity(hubble_constant, distance_mpc)
    // einstein_de_sitter_age(hubble_constant)
    // luminosity_distance_hubble(hubble_constant, redshift)
    {
        let got = cosmology::friedmann_hubble_distance(299_792.458, 0.01).unwrap_or(0.0);
        checks += 1;
        if !got.is_finite() || got <= 0.0 {
            failures.push(format!("cosmology::friedmann_hubble_distance: got {got}, want finite positive"));
        }
    }
    {
        let got = cosmology::hubble_flow_velocity(70.0, 100.0).unwrap_or(0.0);
        let want = 7000.0; // km/s
        checks += 1;
        if !approx_eq(got, want, 1.0e-6) {
            failures.push(format!("cosmology::hubble_flow_velocity: got {got}, want {want}"));
        }
    }
    {
        let got = cosmology::einstein_de_sitter_age(70.0).unwrap_or(0.0);
        let want = 2.0 / (3.0 * 70.0);
        checks += 1;
        if !approx_eq(got, want, 1.0e-6) {
            failures.push(format!("cosmology::einstein_de_sitter_age: got {got}, want {want}"));
        }
    }
    {
        let got = cosmology::luminosity_distance_hubble(70.0, 0.5).unwrap_or(0.0);
        checks += 1;
        if !got.is_finite() || got <= 0.0 {
            failures.push(format!("cosmology::luminosity_distance_hubble: got {got}, want finite positive"));
        }
    }

    println!("=== ad-hoc shim reexport probe ===");
    println!("checks run: {checks}");
    if failures.is_empty() {
        println!("SHIM-REEXPORT-PROBE-OK");
        std::process::exit(0);
    } else {
        println!("FAIL: {} check(s) failed:", failures.len());
        for f in &failures {
            println!("  - {f}");
        }
        std::process::exit(1);
    }
}
