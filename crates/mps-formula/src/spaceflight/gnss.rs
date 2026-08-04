//! `spaceflight::gnss` submodule — GNSS pseudorange/double-difference, Friis link budget, radar range rate
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

pub fn friis_link(
    transmit_power: f64,
    transmit_gain: f64,
    receive_gain: f64,
    wavelength: f64,
    range: f64,
    system_loss: f64,
) -> Option<FriisLink> {
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
        return None;
    }
    let path_gain = (wavelength / (4.0 * PI * range)).powi(2);
    let path_loss = 1.0 / path_gain;
    Some(FriisLink {
        received_power: transmit_power * transmit_gain * receive_gain * path_gain / system_loss,
        path_loss,
    })
}

pub fn friis_wavelength_from_frequency(frequency: f64) -> Option<f64> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return None;
    }
    Some(SPEED_OF_LIGHT / frequency)
}

pub fn gnss_double_difference_carrier_phase(
    range_rover_sat_a: f64,
    range_rover_sat_b: f64,
    range_base_sat_a: f64,
    range_base_sat_b: f64,
    wavelength: f64,
    ambiguity: f64,
) -> Option<f64> {
    if !finite(&[
        range_rover_sat_a,
        range_rover_sat_b,
        range_base_sat_a,
        range_base_sat_b,
        wavelength,
        ambiguity,
    ]) || wavelength <= 0.0
    {
        return None;
    }
    Some(
        ((range_rover_sat_a - range_rover_sat_b) - (range_base_sat_a - range_base_sat_b))
            / wavelength
            + ambiguity,
    )
}

pub fn gnss_pseudorange(
    receiver: Vec3,
    satellite: Vec3,
    receiver_clock_bias: f64,
    satellite_clock_bias: f64,
    ionosphere_delay: f64,
    troposphere_delay: f64,
) -> Option<GnssObservation> {
    if !vec3_finite(receiver)
        || !vec3_finite(satellite)
        || !finite(&[
            receiver_clock_bias,
            satellite_clock_bias,
            ionosphere_delay,
            troposphere_delay,
        ])
    {
        return None;
    }
    let range = (vec3_to_rapier(satellite) - vec3_to_rapier(receiver)).length();
    Some(GnssObservation {
        value: range
            + SPEED_OF_LIGHT * (receiver_clock_bias - satellite_clock_bias)
            + ionosphere_delay
            + troposphere_delay,
        geometric_range: range,
    })
}

pub fn radar_range_rate(
    radar_position: Vec3,
    target_position: Vec3,
    radar_velocity: Vec3,
    target_velocity: Vec3,
) -> Option<RadarMeasurement> {
    if !vec3_finite(radar_position)
        || !vec3_finite(target_position)
        || !vec3_finite(radar_velocity)
        || !vec3_finite(target_velocity)
    {
        return None;
    }
    let line = vec3_to_rapier(target_position) - vec3_to_rapier(radar_position);
    let range = line.length();
    if range <= EPS {
        return None;
    }
    let rel_v = vec3_to_rapier(target_velocity) - vec3_to_rapier(radar_velocity);
    Some(RadarMeasurement {
        range,
        range_rate: rel_v.dot(line / range),
    })
}
