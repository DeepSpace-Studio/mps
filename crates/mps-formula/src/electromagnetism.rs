use std::f64::consts::PI;
use std::slice;

use rapier3d::prelude::Vector;

use crate::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, set_error,
};
use crate::ffi::{
    Bool, ElectromagneticField, FaradayInductionReport, FdtdYeeReport, LorentzForceReport,
    MagneticFluxReport, MaxwellPointReport, Vec3, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

use crate::math::{KahanSum, finite_non_negative, finite_positive};

const EPSILON: f64 = 1.0e-12;
const VACUUM_PERMITTIVITY: f64 = 8.854_187_812_8e-12;
const VACUUM_PERMEABILITY: f64 = 1.256_637_062_12e-6;
const MAX_FIELD_CELLS: u32 = 2_000_000;

fn field_valid(field: ElectromagneticField) -> bool {
    vec3_finite(field.electric) && vec3_finite(field.magnetic)
}

#[unsafe(no_mangle)]
pub extern "C" fn em_lorentz_force(
    charge: f64,
    velocity: Vec3,
    field: ElectromagneticField,
    mass: f64,
    out_report: *mut LorentzForceReport,
) -> Bool {
    if !charge.is_finite()
        || !vec3_finite(velocity)
        || !field_valid(field)
        || !finite_non_negative(mass)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid Lorentz force parameters");
        return Bool::FALSE;
    }
    let v = vec3_to_rapier(velocity);
    let electric = vec3_to_rapier(field.electric);
    let magnetic = vec3_to_rapier(field.magnetic);
    let force = (electric + v.cross(magnetic)) * charge;
    let acceleration = if mass > EPSILON {
        force / mass
    } else {
        Vector::ZERO
    };
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "Lorentz force output is null");
        return Bool::FALSE;
    };
    *out_report = LorentzForceReport {
        electric_force: vec3_from_rapier(electric * charge),
        magnetic_force: vec3_from_rapier(v.cross(magnetic) * charge),
        total_force: vec3_from_rapier(force),
        acceleration: vec3_from_rapier(acceleration),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn em_magnetic_flux(
    magnetic_field: Vec3,
    area_normal: Vec3,
    area: f64,
    out_report: *mut MagneticFluxReport,
) -> Bool {
    if !vec3_finite(magnetic_field) || !vec3_finite(area_normal) || !finite_non_negative(area) {
        set_error(ERR_INVALID_ARGUMENT, "invalid magnetic flux parameters");
        return Bool::FALSE;
    }
    let b = vec3_to_rapier(magnetic_field);
    let n = vec3_to_rapier(area_normal);
    let normal_len = n.length();
    if normal_len <= EPSILON {
        set_error(ERR_INVALID_ARGUMENT, "magnetic flux normal is zero");
        return Bool::FALSE;
    }
    let unit_normal = n / normal_len;
    let normal_component = b.dot(unit_normal);
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "magnetic flux output is null");
        return Bool::FALSE;
    };
    *out_report = MagneticFluxReport {
        flux: normal_component * area,
        normal_component,
        area,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn em_faraday_induction(
    previous_flux: f64,
    current_flux: f64,
    dt: f64,
    turns: f64,
    resistance: f64,
    out_report: *mut FaradayInductionReport,
) -> Bool {
    if !previous_flux.is_finite()
        || !current_flux.is_finite()
        || !finite_positive(dt)
        || !finite_non_negative(turns)
        || !finite_non_negative(resistance)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid Faraday induction parameters");
        return Bool::FALSE;
    }
    let flux_rate = (current_flux - previous_flux) / dt;
    let induced_emf = -turns * flux_rate;
    let induced_current = if resistance > EPSILON {
        induced_emf / resistance
    } else {
        0.0
    };
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "Faraday induction output is null");
        return Bool::FALSE;
    };
    *out_report = FaradayInductionReport {
        flux_rate,
        induced_emf,
        induced_current,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn em_maxwell_point_update(
    field: ElectromagneticField,
    curl_electric: Vec3,
    curl_magnetic: Vec3,
    current_density: Vec3,
    charge_density: f64,
    divergence_electric: f64,
    divergence_magnetic: f64,
    permittivity: f64,
    permeability: f64,
    dt: f64,
    out_report: *mut MaxwellPointReport,
) -> Bool {
    if !field_valid(field)
        || !vec3_finite(curl_electric)
        || !vec3_finite(curl_magnetic)
        || !vec3_finite(current_density)
        || !charge_density.is_finite()
        || !divergence_electric.is_finite()
        || !divergence_magnetic.is_finite()
        || !finite_positive(permittivity)
        || !finite_positive(permeability)
        || !finite_non_negative(dt)
    {
        set_error(
            ERR_INVALID_ARGUMENT,
            "invalid Maxwell point update parameters",
        );
        return Bool::FALSE;
    }

    let e = vec3_to_rapier(field.electric);
    let b = vec3_to_rapier(field.magnetic);
    let curl_e = vec3_to_rapier(curl_electric);
    let curl_b = vec3_to_rapier(curl_magnetic);
    let j = vec3_to_rapier(current_density);
    let electric_derivative = curl_b / (permittivity * permeability) - j / permittivity;
    let magnetic_derivative = -curl_e;
    let next_electric = e + electric_derivative * dt;
    let next_magnetic = b + magnetic_derivative * dt;

    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "Maxwell point output is null");
        return Bool::FALSE;
    };
    *out_report = MaxwellPointReport {
        next_field: ElectromagneticField {
            electric: vec3_from_rapier(next_electric),
            magnetic: vec3_from_rapier(next_magnetic),
        },
        electric_derivative: vec3_from_rapier(electric_derivative),
        magnetic_derivative: vec3_from_rapier(magnetic_derivative),
        gauss_electric_residual: divergence_electric - charge_density / permittivity,
        gauss_magnetic_residual: divergence_magnetic,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn em_fdtd_yee_update(
    electric_fields: *const Vec3,
    magnetic_fields: *const Vec3,
    curl_electric: *const Vec3,
    curl_magnetic: *const Vec3,
    cell_count: u32,
    permittivity: f64,
    permeability: f64,
    conductivity: f64,
    dt: f64,
    out_electric_fields: *mut Vec3,
    out_magnetic_fields: *mut Vec3,
    capacity: u32,
    out_report: *mut FdtdYeeReport,
) -> Bool {
    if cell_count == 0 || cell_count > MAX_FIELD_CELLS || capacity < cell_count {
        set_error(ERR_CAPACITY, "invalid FDTD Yee grid capacity");
        return Bool::FALSE;
    }
    if electric_fields.is_null()
        || magnetic_fields.is_null()
        || curl_electric.is_null()
        || curl_magnetic.is_null()
        || out_electric_fields.is_null()
        || out_magnetic_fields.is_null()
    {
        set_error(ERR_NULL_POINTER, "FDTD Yee grid pointers are null");
        return Bool::FALSE;
    }
    if !finite_positive(permittivity)
        || !finite_positive(permeability)
        || !finite_non_negative(conductivity)
        || !finite_non_negative(dt)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid FDTD Yee grid parameters");
        return Bool::FALSE;
    }

    let electric_fields = unsafe { slice::from_raw_parts(electric_fields, cell_count as usize) };
    let magnetic_fields = unsafe { slice::from_raw_parts(magnetic_fields, cell_count as usize) };
    let curl_electric = unsafe { slice::from_raw_parts(curl_electric, cell_count as usize) };
    let curl_magnetic = unsafe { slice::from_raw_parts(curl_magnetic, cell_count as usize) };
    let out_electric = unsafe { slice::from_raw_parts_mut(out_electric_fields, capacity as usize) };
    let out_magnetic = unsafe { slice::from_raw_parts_mut(out_magnetic_fields, capacity as usize) };

    let mut max_electric_delta = 0.0;
    let mut max_magnetic_delta = 0.0;
    let mut total_energy_acc = KahanSum::default();
    for index in 0..cell_count as usize {
        if !vec3_finite(electric_fields[index])
            || !vec3_finite(magnetic_fields[index])
            || !vec3_finite(curl_electric[index])
            || !vec3_finite(curl_magnetic[index])
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid FDTD Yee grid cell");
            return Bool::FALSE;
        }
        let e = vec3_to_rapier(electric_fields[index]);
        let b = vec3_to_rapier(magnetic_fields[index]);
        let e_delta = (vec3_to_rapier(curl_magnetic[index]) / (permittivity * permeability)
            - e * (conductivity / permittivity))
            * dt;
        let b_delta = -vec3_to_rapier(curl_electric[index]) * dt;
        let next_e = e + e_delta;
        let next_b = b + b_delta;
        out_electric[index] = vec3_from_rapier(next_e);
        out_magnetic[index] = vec3_from_rapier(next_b);
        max_electric_delta = f64::max(max_electric_delta, e_delta.length());
        max_magnetic_delta = f64::max(max_magnetic_delta, b_delta.length());
        total_energy_acc.add(
            0.5 * permittivity * next_e.length_squared()
                + 0.5 * next_b.length_squared() / permeability,
        );
    }

    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = FdtdYeeReport {
            cell_count,
            max_electric_delta,
            max_magnetic_delta,
            total_energy_density: total_energy_acc.value(),
            courant_number: dt / (permittivity * permeability).sqrt(),
        };
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn em_vacuum_permittivity() -> f64 {
    VACUUM_PERMITTIVITY
}

#[unsafe(no_mangle)]
pub extern "C" fn em_vacuum_permeability() -> f64 {
    VACUUM_PERMEABILITY
}



// ---------------------------------------------------------------------------
// Biot-Savart law
// ---------------------------------------------------------------------------

/// Biot-Savart law: dB = (mu0/4pi) * I * dl x r_hat / r^2
/// Returns the magnetic field contribution at `point` from a current element.
pub fn biot_savart_element(current: f64, dl: Vec3, position: Vec3, point: Vec3) -> Option<Vec3> {
    let mu0 = 1.25663706212e-6;
    if !vec3_finite(dl) || !vec3_finite(position) || !vec3_finite(point) || !current.is_finite() { return None; }
    let r_vec = vec3_to_rapier(point) - vec3_to_rapier(position);
    let r = r_vec.length();
    if r < 1.0e-12 { return None; }
    let r_hat = r_vec / r;
    let cross = vec3_to_rapier(dl).cross(r_hat);
    let factor = mu0 * current / (4.0 * PI * r * r);
    Some(vec3_from_rapier(cross * factor))
}

/// Biot-Savart law for a finite straight wire segment.
/// Returns B at `point` from wire from `p1` to `p2` carrying current.
pub fn biot_savart_wire_segment(current: f64, p1: Vec3, p2: Vec3, point: Vec3) -> Option<Vec3> {
    let mu0 = 1.25663706212e-6;
    if !finite_6(&[current, p1.x, p1.y, p1.z, p2.x, p2.y]) || !vec3_finite(point) { return None; }
    let a = vec3_to_rapier(p1); let b = vec3_to_rapier(p2); let p = vec3_to_rapier(point);
    let l = b - a; let l_len = l.length();
    if l_len < 1.0e-12 { return None; }
    let r1 = p - a; let r2 = p - b;
    let r1_len = r1.length(); let r2_len = r2.length();
    if r1_len < 1.0e-12 || r2_len < 1.0e-12 { return None; }
    let l_hat = l / l_len;
    let sin_theta1 = l_hat.cross(r1 / r1_len).length();
    let sin_theta2 = l_hat.cross(r2 / r2_len).length();
    let direction = l_hat.cross(r1 / r1_len).try_normalize()?;
    let factor = mu0 * current / (4.0 * PI);
    let term = (1.0 / r1_len + 1.0 / r2_len) * (sin_theta1 + sin_theta2) / 2.0;
    Some(vec3_from_rapier(direction * factor * term))
}

// ---------------------------------------------------------------------------
// Poynting vector
// ---------------------------------------------------------------------------

/// Poynting vector: S = E x H (W/m^2) where H = B / mu0
pub fn poynting_vector(e: Vec3, b: Vec3) -> Option<Vec3> {
    let mu0 = 1.25663706212e-6;
    if !vec3_finite(e) || !vec3_finite(b) { return None; }
    let e_v = vec3_to_rapier(e); let b_v = vec3_to_rapier(b);
    let s = e_v.cross(b_v) / mu0;
    Some(vec3_from_rapier(s))
}

/// Poynting vector magnitude for plane wave: |S| = |E|^2 / (mu0 * c)
pub fn poynting_magnitude_plane_wave(e_field_magnitude: f64) -> Option<f64> {
    let c = 299_792_458.0;
    let mu0 = 1.25663706212e-6;
    if !e_field_magnitude.is_finite() || e_field_magnitude < 0.0 { return None; }
    Some(e_field_magnitude * e_field_magnitude / (mu0 * c))
}

// ---------------------------------------------------------------------------
// EM wave propagation
// ---------------------------------------------------------------------------

/// Phase velocity in medium: v = c / n
pub fn phase_velocity(refractive_index: f64) -> Option<f64> {
    let c = 299_792_458.0;
    if !refractive_index.is_finite() || refractive_index <= 0.0 { return None; }
    Some(c / refractive_index)
}

/// Wavelength: lambda = c / (n * f)
pub fn wavelength_in_medium(frequency: f64, refractive_index: f64) -> Option<f64> {
    let c = 299_792_458.0;
    if !frequency.is_finite() || frequency <= 0.0 || !refractive_index.is_finite() || refractive_index <= 0.0 { return None; }
    Some(c / (refractive_index * frequency))
}

/// Intrinsic impedance of medium: eta = sqrt(mu / epsilon)
pub fn intrinsic_impedance(permeability: f64, permittivity: f64) -> Option<f64> {
    if !permeability.is_finite() || permeability <= 0.0 || !permittivity.is_finite() || permittivity <= 0.0 { return None; }
    Some((permeability / permittivity).sqrt())
}

/// Skin depth: delta = 1 / sqrt(pi * f * mu * sigma)
pub fn skin_depth(frequency: f64, permeability: f64, conductivity: f64) -> Option<f64> {
    if !frequency.is_finite() || frequency <= 0.0 || !permeability.is_finite() || permeability <= 0.0 || !conductivity.is_finite() || conductivity <= 0.0 { return None; }
    Some(1.0 / (PI * frequency * permeability * conductivity).sqrt())
}

/// EM wave vacuum wavelength: lambda = c / f
pub fn vacuum_wavelength(frequency: f64) -> Option<f64> {
    let c = 299_792_458.0;
    if !frequency.is_finite() || frequency <= 0.0 { return None; }
    Some(c / frequency)
}

/// EM wave frequency: f = c / lambda
pub fn wave_frequency(wavelength: f64) -> Option<f64> {
    let c = 299_792_458.0;
    if !wavelength.is_finite() || wavelength <= 0.0 { return None; }
    Some(c / wavelength)
}

fn finite_6(v: &[f64; 6]) -> bool {
    v.iter().all(|x| x.is_finite())
}