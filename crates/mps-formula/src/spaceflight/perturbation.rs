//! `spaceflight::perturbation` submodule — orbital perturbations (J2/J3/J4, atmospheric drag, solar radiation pressure, Gauss variational, SGP4 secular, Sagnac, IGRF tilted dipole, solar activity density correction)
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

pub fn atmospheric_density_scale_height(
    reference_density: f64,
    altitude: f64,
    reference_altitude: f64,
    scale_height: f64,
) -> Option<f64> {
    if !finite(&[
        reference_density,
        altitude,
        reference_altitude,
        scale_height,
    ]) || reference_density < 0.0
        || scale_height <= 0.0
    {
        return None;
    }
    Some(reference_density * (-(altitude - reference_altitude) / scale_height).exp())
}

pub fn atmospheric_drag_acceleration(
    velocity: Vec3,
    atmosphere_velocity: Vec3,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
) -> Option<Vec3> {
    if !vec3_finite(velocity)
        || !vec3_finite(atmosphere_velocity)
        || !finite(&[density, drag_coefficient, area, mass])
        || density < 0.0
        || drag_coefficient < 0.0
        || area < 0.0
        || mass <= 0.0
    {
        return None;
    }
    let rel = vec3_to_rapier(velocity) - vec3_to_rapier(atmosphere_velocity);
    let speed = rel.length();
    let acc = if speed > EPS {
        -rel * (0.5 * density * speed * drag_coefficient * area / mass)
    } else {
        nalgebra::Vector3::<f64>::zeros()
    };
    Some(vec3_from_rapier(acc))
}

/// Gauss variational equations in RSW frame.
/// Returns (da/dt, de/dt, di/dt, dRAAN/dt, domega/dt, dM/dt)
pub fn gauss_variational_equations(
    elements: OrbitalElements,
    mu: f64,
    perturbing_accel_rsw: Vec3,
) -> Option<(f64, f64, f64, f64, f64, f64)> {
    let a = elements.semi_major_axis;
    let e = elements.eccentricity;
    let i = elements.inclination;
    let omega = elements.argument_of_periapsis;
    let nu = elements.true_anomaly;
    if !finite(&[a, e, i, omega, nu, mu])
        || mu <= 0.0
        || a <= 0.0
        || !(0.0..1.0).contains(&e)
        || !vec3_finite(perturbing_accel_rsw)
    {
        return None;
    }
    let ar = perturbing_accel_rsw.x;
    let as_ = perturbing_accel_rsw.y;
    let aw = perturbing_accel_rsw.z;
    let n = (mu / (a * a * a)).sqrt();
    let p = a * (1.0 - e * e);
    if p <= 0.0 {
        return None;
    }
    let r = p / (1.0 + e * nu.cos());
    let h = (mu * p).sqrt();
    let b = a * (1.0 - e * e).sqrt();
    let theta = omega + nu;
    let da_dt = 2.0 * a * a / h * (e * nu.sin() * ar + (1.0 + e * nu.cos()) * as_);
    let de_dt = 1.0 / h * (p * nu.sin() * ar + ((p + r) * nu.cos() + r * e) * as_);
    let di_dt = r * theta.cos() / h * aw;
    let raan_dot = if i.sin().abs() < EPS {
        0.0
    } else {
        r * theta.sin() / (h * i.sin()) * aw
    };
    let domega_dt = if e < EPS {
        0.0
    } else {
        1.0 / (h * e) * (-p * nu.cos() * ar + (p + r) * nu.sin() * as_)
    } - (if i.sin().abs() < EPS {
        0.0
    } else {
        r * theta.sin() * i.cos() / (h * i.sin()) * aw
    });
    let dm_dt = if e < EPS {
        n + b / (h * a) * (-2.0 * r * ar + (p + r) * as_)
    } else {
        n + b / (h * a * e) * ((p * nu.cos() - 2.0 * r * e) * ar - (p + r) * nu.sin() * as_)
    };
    Some((da_dt, de_dt, di_dt, raan_dot, domega_dt, dm_dt))
}

/// Simplified tilted-dipole geomagnetic field in ECEF coordinates.
/// Returns B in Tesla. Uses IGRF-13 approximate dipole coefficients.
pub fn igrf_tilted_dipole(position_ecef: Vec3, epoch_year: f64) -> Option<Vec3> {
    if !vec3_finite(position_ecef)
        || !epoch_year.is_finite()
        || !(1900.0..=2100.0).contains(&epoch_year)
    {
        return None;
    }
    let r = vec3_to_rapier(position_ecef);
    let r_mag = r.length();
    if r_mag < EPS {
        return None;
    }
    let g10 = -29404.5e-9 + 10.5e-9 * (epoch_year - 2020.0);
    let g11 = -1450.7e-9 + 7.7e-9 * (epoch_year - 2020.0);
    let h11 = 4652.9e-9 + (-25.1e-9) * (epoch_year - 2020.0);
    let a_e3 = 6_371_200.0_f64.powi(3);
    let m = nalgebra::Vector3::<f64>::new(g11 * a_e3, h11 * a_e3, g10 * a_e3);
    let r_hat = r / r_mag;
    let m_dot_r = m.dot(r_hat);
    let b = (r_hat * (3.0 * m_dot_r) - m) / (r_mag * r_mag * r_mag);
    Some(vec3_from_rapier(b))
}

pub fn j2_acceleration(position: Vec3, mu: f64, equatorial_radius: f64, j2: f64) -> Option<Vec3> {
    if !vec3_finite(position)
        || !finite(&[mu, equatorial_radius, j2])
        || mu <= 0.0
        || equatorial_radius <= 0.0
    {
        return None;
    }
    let r = vec3_to_rapier(position);
    let radius = r.length();
    if radius <= EPS {
        return None;
    }
    let z2_r2 = (r.z * r.z) / (radius * radius);
    let factor = 1.5 * j2 * mu * equatorial_radius * equatorial_radius / radius.powi(5);
    Some(Vec3 {
        x: factor * r.x * (5.0 * z2_r2 - 1.0),
        y: factor * r.y * (5.0 * z2_r2 - 1.0),
        z: factor * r.z * (5.0 * z2_r2 - 3.0),
    })
}

pub fn j2_j3_j4_acceleration(
    position: Vec3,
    mu: f64,
    equatorial_radius: f64,
    j2: f64,
    j3: f64,
    j4: f64,
) -> Option<Vec3> {
    let a_j2 = j2_acceleration(position, mu, equatorial_radius, j2)?;
    let a_j3 = j3_acceleration(position, mu, equatorial_radius, j3)?;
    let a_j4 = j4_acceleration(position, mu, equatorial_radius, j4)?;
    Some(Vec3 {
        x: a_j2.x + a_j3.x + a_j4.x,
        y: a_j2.y + a_j3.y + a_j4.y,
        z: a_j2.z + a_j3.z + a_j4.z,
    })
}

pub fn j3_acceleration(position: Vec3, mu: f64, equatorial_radius: f64, j3: f64) -> Option<Vec3> {
    if !vec3_finite(position)
        || !finite(&[mu, equatorial_radius, j3])
        || mu <= 0.0
        || equatorial_radius <= 0.0
    {
        return None;
    }
    let r = vec3_to_rapier(position);
    let radius = r.length();
    if radius <= EPS {
        return None;
    }
    let z_r = r.z / radius;
    let re_r = equatorial_radius / radius;
    let factor = 0.5 * j3 * mu * re_r.powi(4) / (radius * radius);
    Some(Vec3 {
        x: factor * r.x / radius * z_r * (7.5 * z_r * z_r - 1.5),
        y: factor * r.y / radius * z_r * (7.5 * z_r * z_r - 1.5),
        z: factor * (3.0 - z_r * z_r * 5.0 * (7.0 - 3.5 * z_r * z_r)),
    })
}

pub fn j4_acceleration(position: Vec3, mu: f64, equatorial_radius: f64, j4: f64) -> Option<Vec3> {
    if !vec3_finite(position)
        || !finite(&[mu, equatorial_radius, j4])
        || mu <= 0.0
        || equatorial_radius <= 0.0
    {
        return None;
    }
    let r = vec3_to_rapier(position);
    let radius = r.length();
    if radius <= EPS {
        return None;
    }
    let z2_r2 = (r.z * r.z) / (radius * radius);
    let z4_r4 = z2_r2 * z2_r2;
    let factor = 0.625 * j4 * mu * equatorial_radius.powi(4) / radius.powi(7);
    Some(Vec3 {
        x: factor * r.x * (3.0 - 42.0 * z2_r2 + 63.0 * z4_r4),
        y: factor * r.y * (3.0 - 42.0 * z2_r2 + 63.0 * z4_r4),
        z: factor * r.z * (15.0 - 70.0 * z2_r2 + 63.0 * z4_r4),
    })
}

pub fn sagnac_phase_rate(area: f64, angular_rate: f64, wavelength: f64) -> Option<f64> {
    if !finite(&[area, angular_rate, wavelength]) || wavelength <= 0.0 {
        return None;
    }
    Some(8.0 * PI * area * angular_rate / (wavelength * SPEED_OF_LIGHT))
}

pub fn sgp4_j2_secular_rates(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    mean_motion: f64,
    equatorial_radius: f64,
    j2: f64,
) -> Option<Sgp4SecularRates> {
    if !finite(&[
        semi_major_axis,
        eccentricity,
        inclination,
        mean_motion,
        equatorial_radius,
        j2,
    ]) || semi_major_axis <= 0.0
        || !(0.0..1.0).contains(&eccentricity)
    {
        return None;
    }
    let p = semi_major_axis * (1.0 - eccentricity * eccentricity);
    let factor = 1.5 * j2 * mean_motion * (equatorial_radius / p).powi(2);
    Some(Sgp4SecularRates {
        mean_motion_dot: 0.0,
        raan_dot: -factor * inclination.cos(),
        argument_of_perigee_dot: 0.5 * factor * (5.0 * inclination.cos().powi(2) - 1.0),
    })
}

/// Atmospheric density correction for solar activity (F10.7 proxy).
/// Simplified: density = rho_0 * exp(alpha * (F10.7 - F10.7_ref))
pub fn solar_activity_density_correction(
    base_density: f64,
    f107: f64,
    f107_ref: f64,
    alpha: f64,
) -> Option<f64> {
    if !finite(&[base_density, f107, f107_ref, alpha]) || base_density < 0.0 {
        return None;
    }
    Some(base_density * (alpha * (f107 - f107_ref)).exp())
}

pub fn solar_radiation_pressure_acceleration(
    sun_direction: Vec3,
    solar_flux: f64,
    reflectivity: f64,
    area: f64,
    mass: f64,
) -> Option<Vec3> {
    if !vec3_finite(sun_direction)
        || !finite(&[solar_flux, reflectivity, area, mass])
        || solar_flux < 0.0
        || reflectivity < 0.0
        || area < 0.0
        || mass <= 0.0
    {
        return None;
    }
    let dir = vec3_to_rapier(sun_direction).try_normalize()?;
    Some(vec3_from_rapier(
        dir * (solar_flux / SPEED_OF_LIGHT * reflectivity * area / mass),
    ))
}
