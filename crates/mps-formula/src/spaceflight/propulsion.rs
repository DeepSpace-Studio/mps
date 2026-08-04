//! `spaceflight::propulsion` submodule — propulsion & power (CO2 mass balance, Hall thruster, Sabatier, solar panel, battery, SPE oxygen)
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

pub fn battery_equivalent_circuit(
    open_circuit_voltage: f64,
    current: f64,
    ohmic_resistance: f64,
    rc_voltage: f64,
    rc_resistance: f64,
    rc_capacitance: f64,
    capacity_coulombs: f64,
) -> Option<BatteryEquivalentCircuit> {
    if !finite(&[
        open_circuit_voltage,
        current,
        ohmic_resistance,
        rc_voltage,
        rc_resistance,
        rc_capacitance,
        capacity_coulombs,
    ]) || ohmic_resistance < 0.0
        || rc_resistance <= 0.0
        || rc_capacitance <= 0.0
        || capacity_coulombs <= 0.0
    {
        return None;
    }
    Some(BatteryEquivalentCircuit {
        terminal_voltage: open_circuit_voltage - current * ohmic_resistance - rc_voltage,
        rc_voltage_dot: -rc_voltage / (rc_resistance * rc_capacitance) + current / rc_capacitance,
        state_of_charge_dot: -current / capacity_coulombs,
    })
}

pub fn co2_mass_balance(
    current_mass: f64,
    generation_rate: f64,
    removal_rate: f64,
    leakage_rate: f64,
    volume: f64,
    dt: f64,
) -> Option<Co2MassBalance> {
    if !finite(&[
        current_mass,
        generation_rate,
        removal_rate,
        leakage_rate,
        volume,
        dt,
    ]) || volume <= 0.0
        || dt < 0.0
    {
        return None;
    }
    let mass_rate = generation_rate - removal_rate - leakage_rate;
    let next_mass = (current_mass + mass_rate * dt).max(0.0);
    Some(Co2MassBalance {
        mass_rate,
        next_mass,
        concentration_rate: mass_rate / volume,
    })
}

pub fn hall_thruster_performance(
    mass_flow_rate: f64,
    exhaust_velocity: f64,
    input_power: f64,
    standard_gravity: f64,
) -> Option<HallThrusterPerformance> {
    if !finite(&[
        mass_flow_rate,
        exhaust_velocity,
        input_power,
        standard_gravity,
    ]) || mass_flow_rate < 0.0
        || exhaust_velocity < 0.0
        || input_power <= 0.0
        || standard_gravity <= 0.0
    {
        return None;
    }
    let thrust = mass_flow_rate * exhaust_velocity;
    Some(HallThrusterPerformance {
        thrust,
        specific_impulse: exhaust_velocity / standard_gravity,
        efficiency: 0.5 * mass_flow_rate * exhaust_velocity * exhaust_velocity / input_power,
    })
}

pub fn sabatier_methane_rate(
    co2_molar_rate: f64,
    h2_molar_rate: f64,
    conversion: f64,
) -> Option<ChemicalReactionRate> {
    if !finite(&[co2_molar_rate, h2_molar_rate, conversion])
        || co2_molar_rate < 0.0
        || h2_molar_rate < 0.0
        || !(0.0..=1.0).contains(&conversion)
    {
        return None;
    }
    let methane = co2_molar_rate.min(h2_molar_rate / 4.0) * conversion;
    Some(ChemicalReactionRate {
        reactant_rate: methane,
        product_rate: methane,
    })
}

pub fn solar_panel_power(
    solar_flux: f64,
    area: f64,
    efficiency: f64,
    incidence_angle: f64,
    degradation: f64,
) -> Option<SolarPanelPower> {
    if !finite(&[solar_flux, area, efficiency, incidence_angle, degradation])
        || solar_flux < 0.0
        || area < 0.0
        || efficiency < 0.0
        || degradation < 0.0
    {
        return None;
    }
    let incident = solar_flux * area * incidence_angle.cos().max(0.0);
    Some(SolarPanelPower {
        incident_power: incident,
        electrical_power: incident * efficiency * degradation,
    })
}

pub fn spe_oxygen_rate(
    current: f64,
    cells: f64,
    faraday_efficiency: f64,
) -> Option<ChemicalReactionRate> {
    if !finite(&[current, cells, faraday_efficiency])
        || current < 0.0
        || cells <= 0.0
        || !(0.0..=1.0).contains(&faraday_efficiency)
    {
        return None;
    }
    let faraday = 96_485.332_12;
    let oxygen = current * cells * faraday_efficiency / (4.0 * faraday);
    Some(ChemicalReactionRate {
        reactant_rate: current * cells / (2.0 * faraday),
        product_rate: oxygen,
    })
}
