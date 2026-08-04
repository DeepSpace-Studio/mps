//! `spaceflight::thermal` submodule — thermal control (heat balance, heat pipe, single-phase loop, radiator, reentry peak g-load, Sutton-Graves heat rate)
//!
//! Split out of the original 2040-line `spaceflight.rs` per OPTIMIZATION.md §N8.
//! See [`super`] for the shared helpers (`finite`, `clamp_unit`,
//! `stumpff_functions`) and numeric constants (`EPS`, `SIGMA`,
//! `SPEED_OF_LIGHT`, `PI`, `TAU`).
//!
//! All public functions keep their `pub fn` names and signatures
//! unchanged; the crate-level `pub use` in `super::mod` keeps the
//! downstream `mps-core::rapier::spaceflight::*` path stable.

use super::*;

pub fn heat_pipe_thermal_resistance(
    evaporator_resistance: f64,
    vapor_resistance: f64,
    condenser_resistance: f64,
    wick_resistance: f64,
) -> Option<f64> {
    if !finite(&[
        evaporator_resistance,
        vapor_resistance,
        condenser_resistance,
        wick_resistance,
    ]) {
        return None;
    }
    Some(evaporator_resistance + vapor_resistance + condenser_resistance + wick_resistance)
}

pub fn radiator_power(
    area: f64,
    emissivity: f64,
    temperature: f64,
    sink_temperature: f64,
    absorbed_power: f64,
) -> Option<RadiatorPower> {
    if !finite(&[
        area,
        emissivity,
        temperature,
        sink_temperature,
        absorbed_power,
    ]) || area < 0.0
        || emissivity < 0.0
        || temperature < 0.0
        || sink_temperature < 0.0
    {
        return None;
    }
    let emitted =
        emissivity * SIGMA * area * (temperature.powi(4) - sink_temperature.powi(4)).max(0.0);
    Some(RadiatorPower {
        emitted_power: emitted,
        net_power: emitted - absorbed_power,
    })
}

/// Re-entry deceleration (g-load) for ballistic entry.
pub fn reentry_peak_g_load(
    beta: f64,
    entry_velocity: f64,
    entry_angle: f64,
    scale_height: f64,
) -> Option<f64> {
    if !finite(&[beta, entry_velocity, entry_angle, scale_height])
        || beta <= 0.0
        || entry_velocity <= 0.0
        || scale_height <= 0.0
    {
        return None;
    }
    let sin_gamma = entry_angle.sin().abs();
    if sin_gamma < EPS {
        return None;
    }
    Some(
        entry_velocity * entry_velocity * sin_gamma
            / (2.0 * std::f64::consts::E * beta * 9.80665 * scale_height),
    )
}

pub fn single_phase_loop_heat_transfer(
    mass_flow_rate: f64,
    specific_heat: f64,
    inlet_temperature: f64,
    heat_input: f64,
) -> Option<FluidLoopHeatTransfer> {
    if !finite(&[mass_flow_rate, specific_heat, inlet_temperature, heat_input])
        || mass_flow_rate <= 0.0
        || specific_heat <= 0.0
    {
        return None;
    }
    Some(FluidLoopHeatTransfer {
        heat_rate: heat_input,
        outlet_temperature: inlet_temperature + heat_input / (mass_flow_rate * specific_heat),
    })
}

/// Sutton-Graves convective heating rate (stagnation point).
/// q_dot = k * sqrt(rho / r_n) * V^3  (W/m²)
/// Earth: k ≈ 1.83e-4 (kg^(1/2)/m)
pub fn sutton_graves_heat_rate(
    density: f64,
    velocity: f64,
    nose_radius: f64,
    planet_k: f64,
) -> Option<f64> {
    if !finite(&[density, velocity, nose_radius, planet_k])
        || density < 0.0
        || velocity < 0.0
        || nose_radius <= 0.0
        || planet_k <= 0.0
    {
        return None;
    }
    Some(planet_k * (density / nose_radius).sqrt() * velocity.powi(3))
}

pub fn thermal_balance(
    absorbed_power: f64,
    internal_power: f64,
    emitted_area: f64,
    emissivity: f64,
) -> Option<ThermalBalance> {
    if !finite(&[absorbed_power, internal_power, emitted_area, emissivity])
        || emitted_area <= 0.0
        || emissivity <= 0.0
    {
        return None;
    }
    let net = absorbed_power + internal_power;
    let equilibrium_temperature = if net > 0.0 {
        (net / (emissivity * SIGMA * emitted_area)).powf(0.25)
    } else {
        0.0
    };
    Some(ThermalBalance {
        net_power: net,
        equilibrium_temperature,
    })
}
