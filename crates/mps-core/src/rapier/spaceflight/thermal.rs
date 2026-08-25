//! `spaceflight::thermal` submodule — thermal control (heat balance, heat pipe, single-phase loop, radiator, whipple shield, airlock, SPE oxygen)
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `out_state` must be null or point to a valid, writable `AirlockDepressurization`.
#[unsafe(no_mangle)]
pub extern "C" fn space_airlock_depressurization(
    pressure: f64,
    ambient_pressure: f64,
    volume: f64,
    conductance: f64,
    dt: f64,
    out_state: *mut AirlockDepressurization,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[pressure, ambient_pressure, volume, conductance, dt])
            || volume <= 0.0
            || conductance < 0.0
            || dt < 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid airlock depressurization parameters",
            );
            return Bool::FALSE;
        }
        let rate = -conductance / volume * (pressure - ambient_pressure);
        write_out(
            out_state,
            AirlockDepressurization {
                pressure: ambient_pressure
                    + (pressure - ambient_pressure) * (-conductance * dt / volume).exp(),
                pressure_rate: rate,
            },
        )
    })
}

/// Sums the evaporator, vapor, condenser, and wick thermal resistances of a heat pipe.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_heat_pipe_thermal_resistance(
    evaporator_resistance: f64,
    vapor_resistance: f64,
    condenser_resistance: f64,
    wick_resistance: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[
            evaporator_resistance,
            vapor_resistance,
            condenser_resistance,
            wick_resistance,
        ]) {
            return invalid_nan("invalid heat pipe resistance parameters");
        }
        clear_error();
        evaporator_resistance + vapor_resistance + condenser_resistance + wick_resistance
    })
}

/// # Safety
/// `out_power` must be null or point to a valid, writable `RadiatorPower`.
#[unsafe(no_mangle)]
pub extern "C" fn space_radiator_power(
    area: f64,
    emissivity: f64,
    temperature: f64,
    sink_temperature: f64,
    absorbed_power: f64,
    out_power: *mut RadiatorPower,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
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
            set_error(ERR_INVALID_ARGUMENT, "invalid radiator power parameters");
            return Bool::FALSE;
        }
        let emitted =
            emissivity * SIGMA * area * (temperature.powi(4) - sink_temperature.powi(4)).max(0.0);
        write_out(
            out_power,
            RadiatorPower {
                emitted_power: emitted,
                net_power: emitted - absorbed_power,
            },
        )
    })
}

/// # Safety
/// `out_heat` must be null or point to a valid, writable `FluidLoopHeatTransfer`.
#[unsafe(no_mangle)]
pub extern "C" fn space_single_phase_loop_heat_transfer(
    mass_flow_rate: f64,
    specific_heat: f64,
    inlet_temperature: f64,
    heat_input: f64,
    out_heat: *mut FluidLoopHeatTransfer,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[mass_flow_rate, specific_heat, inlet_temperature, heat_input])
            || mass_flow_rate <= 0.0
            || specific_heat <= 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid single-phase loop heat parameters",
            );
            return Bool::FALSE;
        }
        write_out(
            out_heat,
            FluidLoopHeatTransfer {
                heat_rate: heat_input,
                outlet_temperature: inlet_temperature
                    + heat_input / (mass_flow_rate * specific_heat),
            },
        )
    })
}

/// # Safety
/// `out_rate` must be null or point to a valid, writable `ChemicalReactionRate`.
#[unsafe(no_mangle)]
pub extern "C" fn space_spe_oxygen_rate(
    current: f64,
    cells: f64,
    faraday_efficiency: f64,
    out_rate: *mut ChemicalReactionRate,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[current, cells, faraday_efficiency])
            || current < 0.0
            || cells <= 0.0
            || !(0.0..=1.0).contains(&faraday_efficiency)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid SPE oxygen parameters");
            return Bool::FALSE;
        }
        let faraday = 96_485.332_12;
        let oxygen = current * cells * faraday_efficiency / (4.0 * faraday);
        write_out(
            out_rate,
            ChemicalReactionRate {
                reactant_rate: current * cells / (2.0 * faraday),
                product_rate: oxygen,
            },
        )
    })
}

/// # Safety
/// `out_balance` must be null or point to a valid, writable `ThermalBalance`.
#[unsafe(no_mangle)]
pub extern "C" fn space_thermal_balance(
    absorbed_power: f64,
    internal_power: f64,
    emitted_area: f64,
    emissivity: f64,
    out_balance: *mut ThermalBalance,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[absorbed_power, internal_power, emitted_area, emissivity])
            || emitted_area <= 0.0
            || emissivity <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid thermal balance parameters");
            return Bool::FALSE;
        }
        let net = absorbed_power + internal_power;
        let equilibrium_temperature = if net > 0.0 {
            (net / (emissivity * SIGMA * emitted_area)).powf(0.25)
        } else {
            0.0
        };
        write_out(
            out_balance,
            ThermalBalance {
                net_power: net,
                equilibrium_temperature,
            },
        )
    })
}

/// Computes the critical projectile diameter a Whipple shield can defeat.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_whipple_critical_projectile_diameter(
    bumper_thickness: f64,
    bumper_density: f64,
    projectile_density: f64,
    impact_velocity: f64,
    standoff: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[
            bumper_thickness,
            bumper_density,
            projectile_density,
            impact_velocity,
            standoff,
        ]) || bumper_thickness <= 0.0
            || bumper_density <= 0.0
            || projectile_density <= 0.0
            || impact_velocity <= 0.0
            || standoff <= 0.0
        {
            return invalid_nan("invalid Whipple shield parameters");
        }
        clear_error();
        bumper_thickness
            * (bumper_density / projectile_density).sqrt()
            * (standoff / bumper_thickness).powf(1.0 / 3.0)
            * (7_000.0 / impact_velocity).powf(2.0 / 3.0)
    })
}
