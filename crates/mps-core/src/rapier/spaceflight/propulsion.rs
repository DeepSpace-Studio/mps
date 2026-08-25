//! `spaceflight::propulsion` submodule — propulsion & power (CO2 mass balance, Hall thruster, Sabatier, solar panel, battery, structural/contact)
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `out_battery` must be null or point to a valid, writable `BatteryEquivalentCircuit`.
#[unsafe(no_mangle)]
pub extern "C" fn space_battery_equivalent_circuit(
    open_circuit_voltage: f64,
    current: f64,
    ohmic_resistance: f64,
    rc_voltage: f64,
    rc_resistance: f64,
    rc_capacitance: f64,
    capacity_coulombs: f64,
    out_battery: *mut BatteryEquivalentCircuit,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
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
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid battery equivalent-circuit parameters",
            );
            return Bool::FALSE;
        }
        write_out(
            out_battery,
            BatteryEquivalentCircuit {
                terminal_voltage: open_circuit_voltage - current * ohmic_resistance - rc_voltage,
                rc_voltage_dot: -rc_voltage / (rc_resistance * rc_capacitance)
                    + current / rc_capacitance,
                state_of_charge_dot: -current / capacity_coulombs,
            },
        )
    })
}

/// # Safety
/// `out_balance` must be null or point to a valid, writable `Co2MassBalance`.
#[unsafe(no_mangle)]
pub extern "C" fn space_co2_mass_balance(
    current_mass: f64,
    generation_rate: f64,
    removal_rate: f64,
    leakage_rate: f64,
    volume: f64,
    dt: f64,
    out_balance: *mut Co2MassBalance,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
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
            set_error(ERR_INVALID_ARGUMENT, "invalid CO2 mass balance parameters");
            return Bool::FALSE;
        }
        let mass_rate = generation_rate - removal_rate - leakage_rate;
        let next_mass = (current_mass + mass_rate * dt).max(0.0);
        write_out(
            out_balance,
            Co2MassBalance {
                mass_rate,
                next_mass,
                concentration_rate: mass_rate / volume,
            },
        )
    })
}

/// # Safety
/// `out_force` must be null or point to a valid, writable `ContactForceModel`.
#[unsafe(no_mangle)]
pub extern "C" fn space_contact_force_hunt_crossley(
    penetration: f64,
    penetration_rate: f64,
    stiffness: f64,
    damping: f64,
    exponent: f64,
    out_force: *mut ContactForceModel,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[penetration, penetration_rate, stiffness, damping, exponent])
            || stiffness < 0.0
            || damping < 0.0
            || exponent <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid contact force parameters");
            return Bool::FALSE;
        }
        let depth = penetration.max(0.0);
        let normal = stiffness * depth.powf(exponent);
        let damping_force = damping * depth.powf(exponent) * penetration_rate.max(0.0);
        write_out(
            out_force,
            ContactForceModel {
                normal_force: normal,
                damping_force,
                total_force: normal + damping_force,
            },
        )
    })
}

/// # Safety
/// `out_performance` must be null or point to a valid, writable `HallThrusterPerformance`.
#[unsafe(no_mangle)]
pub extern "C" fn space_hall_thruster_performance(
    mass_flow_rate: f64,
    exhaust_velocity: f64,
    input_power: f64,
    standard_gravity: f64,
    out_performance: *mut HallThrusterPerformance,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
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
            set_error(ERR_INVALID_ARGUMENT, "invalid Hall thruster parameters");
            return Bool::FALSE;
        }
        let thrust = mass_flow_rate * exhaust_velocity;
        write_out(
            out_performance,
            HallThrusterPerformance {
                thrust,
                specific_impulse: exhaust_velocity / standard_gravity,
                efficiency: 0.5 * mass_flow_rate * exhaust_velocity * exhaust_velocity
                    / input_power,
            },
        )
    })
}

/// # Safety
/// `out_rate` must be null or point to a valid, writable `ChemicalReactionRate`.
#[unsafe(no_mangle)]
pub extern "C" fn space_sabatier_methane_rate(
    co2_molar_rate: f64,
    h2_molar_rate: f64,
    conversion: f64,
    out_rate: *mut ChemicalReactionRate,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[co2_molar_rate, h2_molar_rate, conversion])
            || co2_molar_rate < 0.0
            || h2_molar_rate < 0.0
            || !(0.0..=1.0).contains(&conversion)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid Sabatier parameters");
            return Bool::FALSE;
        }
        let methane = co2_molar_rate.min(h2_molar_rate / 4.0) * conversion;
        write_out(
            out_rate,
            ChemicalReactionRate {
                reactant_rate: methane,
                product_rate: methane,
            },
        )
    })
}

/// # Safety
/// `out_power` must be null or point to a valid, writable `SolarPanelPower`.
#[unsafe(no_mangle)]
pub extern "C" fn space_solar_panel_power(
    solar_flux: f64,
    area: f64,
    efficiency: f64,
    incidence_angle: f64,
    degradation: f64,
    out_power: *mut SolarPanelPower,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[solar_flux, area, efficiency, incidence_angle, degradation])
            || solar_flux < 0.0
            || area < 0.0
            || efficiency < 0.0
            || degradation < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid solar panel parameters");
            return Bool::FALSE;
        }
        let incident = solar_flux * area * incidence_angle.cos().max(0.0);
        write_out(
            out_power,
            SolarPanelPower {
                incident_power: incident,
                electrical_power: incident * efficiency * degradation,
            },
        )
    })
}

/// Computes a structural natural frequency from stiffness, mass, and a mode factor.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_structural_natural_frequency(
    stiffness: f64,
    mass: f64,
    mode_factor: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[stiffness, mass, mode_factor]) || stiffness <= 0.0 || mass <= 0.0 {
            return invalid_nan("invalid structural frequency parameters");
        }
        clear_error();
        mode_factor * (stiffness / mass).sqrt() / TAU
    })
}
