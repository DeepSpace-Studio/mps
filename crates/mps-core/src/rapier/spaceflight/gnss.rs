//! `spaceflight::gnss` submodule — GNSS pseudorange/double-difference, Friis link budget, radar range rate
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `out_link` must be null or point to a valid, writable `FriisLink`.
#[unsafe(no_mangle)]
pub extern "C" fn space_friis_link(
    transmit_power: f64,
    transmit_gain: f64,
    receive_gain: f64,
    wavelength: f64,
    range: f64,
    system_loss: f64,
    out_link: *mut FriisLink,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[
            transmit_power,
            transmit_gain,
            receive_gain,
            wavelength,
            range,
            system_loss,
        ]) || transmit_power < 0.0
            || transmit_gain < 0.0
            || receive_gain < 0.0
            || wavelength <= 0.0
            || range <= 0.0
            || system_loss <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid Friis link parameters");
            return Bool::FALSE;
        }
        let path_gain = (wavelength / (4.0 * PI * range)).powi(2);
        let path_loss = 1.0 / path_gain;
        write_out(
            out_link,
            FriisLink {
                received_power: transmit_power * transmit_gain * receive_gain * path_gain
                    / system_loss,
                path_loss,
            },
        )
    })
}

/// Converts a frequency to the corresponding free-space wavelength.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_friis_wavelength_from_frequency(frequency: f64) -> f64 {
    ffi_guard(0.0, || {
        if !frequency.is_finite() || frequency <= 0.0 {
            return invalid_nan("invalid Friis frequency");
        }
        clear_error();
        SPEED_OF_LIGHT / frequency
    })
}

/// Computes the GNSS double-difference carrier phase observable in cycles.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_gnss_double_difference_carrier_phase(
    range_rover_sat_a: f64,
    range_rover_sat_b: f64,
    range_base_sat_a: f64,
    range_base_sat_b: f64,
    wavelength: f64,
    ambiguity: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[
            range_rover_sat_a,
            range_rover_sat_b,
            range_base_sat_a,
            range_base_sat_b,
            wavelength,
            ambiguity,
        ]) || wavelength <= 0.0
        {
            return invalid_nan("invalid double-difference carrier phase parameters");
        }
        clear_error();
        ((range_rover_sat_a - range_rover_sat_b) - (range_base_sat_a - range_base_sat_b))
            / wavelength
            + ambiguity
    })
}

/// # Safety
/// `out_observation` must be null or point to a valid, writable `GnssObservation`.
#[unsafe(no_mangle)]
pub extern "C" fn space_gnss_pseudorange(
    receiver: Vec3,
    satellite: Vec3,
    receiver_clock_bias: f64,
    satellite_clock_bias: f64,
    ionosphere_delay: f64,
    troposphere_delay: f64,
    out_observation: *mut GnssObservation,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(receiver)
            || !vec3_finite(satellite)
            || !finite(&[
                receiver_clock_bias,
                satellite_clock_bias,
                ionosphere_delay,
                troposphere_delay,
            ])
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid GNSS pseudorange parameters");
            return Bool::FALSE;
        }
        let range = (vec3_to_rapier(satellite) - vec3_to_rapier(receiver)).length();
        write_out(
            out_observation,
            GnssObservation {
                value: range
                    + SPEED_OF_LIGHT * (receiver_clock_bias - satellite_clock_bias)
                    + ionosphere_delay
                    + troposphere_delay,
                geometric_range: range,
            },
        )
    })
}

/// # Safety
/// `out_measurement` must be null or point to a valid, writable `RadarMeasurement`.
#[unsafe(no_mangle)]
pub extern "C" fn space_radar_range_rate(
    radar_position: Vec3,
    target_position: Vec3,
    radar_velocity: Vec3,
    target_velocity: Vec3,
    out_measurement: *mut RadarMeasurement,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(radar_position)
            || !vec3_finite(target_position)
            || !vec3_finite(radar_velocity)
            || !vec3_finite(target_velocity)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid radar measurement parameters");
            return Bool::FALSE;
        }
        let line = vec3_to_rapier(target_position) - vec3_to_rapier(radar_position);
        let range = line.length();
        if range <= EPS {
            set_error(ERR_INVALID_ARGUMENT, "radar range is zero");
            return Bool::FALSE;
        }
        let rel_v = vec3_to_rapier(target_velocity) - vec3_to_rapier(radar_velocity);
        write_out(
            out_measurement,
            RadarMeasurement {
                range,
                range_rate: rel_v.dot(line / range),
            },
        )
    })
}
