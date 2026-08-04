//! `spaceflight::perturbation` submodule — orbital perturbations (J2, atmospheric drag, solar radiation pressure, gravity gradient torque, Sagnac, atomic oxygen)
//!
//! Split out of the original 2610-line `spaceflight.rs`. See [`super`]
//! for the shared helpers (`finite`, `write_out`, `invalid_nan`, `cross`, `clamp_unit`)
//! and numeric constants (`EPS`, `SIGMA`, `SPEED_OF_LIGHT`, `PI/TAU`).
//! Every `extern "C" fn space_*` in this file retains its
//! `#[unsafe(no_mangle)]` name, signature, and behaviour — the crate-level
//! `pub use` in `super::mod` keeps ABI paths stable.

use super::*;

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_atmospheric_drag_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    atmosphere_velocity: Vec3,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !mass.is_finite() || mass <= 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid atmospheric drag body mass");
            return Bool::FALSE;
        }
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let velocity = vec3_from_rapier(body.linvel());
        let mut acceleration = Vec3::default();
        if space_atmospheric_drag_acceleration(
            velocity,
            atmosphere_velocity,
            density,
            drag_coefficient,
            area,
            mass,
            &mut acceleration,
        ) == Bool::FALSE
        {
            return Bool::FALSE;
        }
        body.add_force(vec3_to_rapier(acceleration) * mass, wake_up.0 != 0);
        write_optional_out(out_acceleration, acceleration);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_atmospheric_drag_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    atmosphere_velocity: Vec3,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> u8 {
    ffi_guard(0, || {
        space_apply_atmospheric_drag_to_body(
            world,
            body_handle,
            atmosphere_velocity,
            density,
            drag_coefficient,
            area,
            mass,
            wake_up,
            out_acceleration,
        )
        .0
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_torque` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_gravity_gradient_torque_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    inertia_diag: Vec3,
    mu: f64,
    wake_up: Bool,
    out_torque: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let position = vec3_from_rapier(body.translation());
        let mut torque = Vec3::default();
        if space_gravity_gradient_torque(position, inertia_diag, mu, &mut torque) == Bool::FALSE {
            return Bool::FALSE;
        }
        body.add_torque(vec3_to_rapier(torque), wake_up.0 != 0);
        write_optional_out(out_torque, torque);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_torque` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_gravity_gradient_torque_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    inertia_diag: Vec3,
    mu: f64,
    wake_up: Bool,
    out_torque: *mut Vec3,
) -> u8 {
    ffi_guard(0, || {
        space_apply_gravity_gradient_torque_to_body(
            world,
            body_handle,
            inertia_diag,
            mu,
            wake_up,
            out_torque,
        )
        .0
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_j2_force_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    mu: f64,
    equatorial_radius: f64,
    j2: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !mass.is_finite() || mass <= 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid J2 body mass");
            return Bool::FALSE;
        }
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let position = vec3_from_rapier(body.translation());
        let mut acceleration = Vec3::default();
        if space_j2_acceleration(position, mu, equatorial_radius, j2, &mut acceleration)
            == Bool::FALSE
        {
            return Bool::FALSE;
        }
        body.add_force(vec3_to_rapier(acceleration) * mass, wake_up.0 != 0);
        write_optional_out(out_acceleration, acceleration);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_j2_force_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    mu: f64,
    equatorial_radius: f64,
    j2: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> u8 {
    ffi_guard(0, || {
        space_apply_j2_force_to_body(
            world,
            body_handle,
            mu,
            equatorial_radius,
            j2,
            mass,
            wake_up,
            out_acceleration,
        )
        .0
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_solar_radiation_pressure_to_body(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    sun_direction: Vec3,
    solar_flux: f64,
    reflectivity: f64,
    area: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !mass.is_finite() || mass <= 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid solar radiation body mass");
            return Bool::FALSE;
        }
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return Bool::FALSE;
        };
        let Some(body) = world
            .inner
            .bodies
            .get_mut(unpack_rigid_body_handle(body_handle))
        else {
            set_error(ERR_NOT_FOUND, "body was not found");
            return Bool::FALSE;
        };
        let mut acceleration = Vec3::default();
        if space_solar_radiation_pressure_acceleration(
            sun_direction,
            solar_flux,
            reflectivity,
            area,
            mass,
            &mut acceleration,
        ) == Bool::FALSE
        {
            return Bool::FALSE;
        }
        body.add_force(vec3_to_rapier(acceleration) * mass, wake_up.0 != 0);
        write_optional_out(out_acceleration, acceleration);
        clear_error();
        Bool::TRUE
    })
}

/// # Safety
/// `world` must be a valid pointer to a `WorldHandle` created by this library.
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_apply_solar_radiation_pressure_to_body_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    sun_direction: Vec3,
    solar_flux: f64,
    reflectivity: f64,
    area: f64,
    mass: f64,
    wake_up: Bool,
    out_acceleration: *mut Vec3,
) -> u8 {
    ffi_guard(0, || {
        space_apply_solar_radiation_pressure_to_body(
            world,
            body_handle,
            sun_direction,
            solar_flux,
            reflectivity,
            area,
            mass,
            wake_up,
            out_acceleration,
        )
        .0
    })
}

/// Computes atmospheric density using the exponential scale-height model.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_atmospheric_density_scale_height(
    reference_density: f64,
    altitude: f64,
    reference_altitude: f64,
    scale_height: f64,
) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[
            reference_density,
            altitude,
            reference_altitude,
            scale_height,
        ]) || reference_density < 0.0
            || scale_height <= 0.0
        {
            return invalid_nan("invalid atmospheric density scale-height parameters");
        }
        clear_error();
        reference_density * (-(altitude - reference_altitude) / scale_height).exp()
    })
}

/// # Safety
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_atmospheric_drag_acceleration(
    velocity: Vec3,
    atmosphere_velocity: Vec3,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(velocity)
            || !vec3_finite(atmosphere_velocity)
            || !finite(&[density, drag_coefficient, area, mass])
            || density < 0.0
            || drag_coefficient < 0.0
            || area < 0.0
            || mass <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid atmospheric drag parameters");
            return Bool::FALSE;
        }
        let rel = vec3_to_rapier(velocity) - vec3_to_rapier(atmosphere_velocity);
        let speed = rel.length();
        let acc = if speed > EPS {
            -rel * (0.5 * density * speed * drag_coefficient * area / mass)
        } else {
            Vector::ZERO
        };
        write_out(out_acceleration, vec3_from_rapier(acc))
    })
}

/// # Safety
/// `out_erosion` must be null or point to a valid, writable `AtomicOxygenErosion`.
#[unsafe(no_mangle)]
pub extern "C" fn space_atomic_oxygen_erosion(
    fluence: f64,
    erosion_yield: f64,
    area: f64,
    density: f64,
    out_erosion: *mut AtomicOxygenErosion,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !finite(&[fluence, erosion_yield, area, density])
            || fluence < 0.0
            || erosion_yield < 0.0
            || area < 0.0
            || density < 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid atomic oxygen erosion parameters",
            );
            return Bool::FALSE;
        }
        let volume_loss = fluence * erosion_yield * area;
        write_out(
            out_erosion,
            AtomicOxygenErosion {
                volume_loss,
                mass_loss: volume_loss * density,
            },
        )
    })
}

/// # Safety
/// `out_torque` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_gravity_gradient_torque(
    position: Vec3,
    inertia_diag: Vec3,
    mu: f64,
    out_torque: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(position) || !vec3_finite(inertia_diag) || !mu.is_finite() || mu <= 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid gravity-gradient torque parameters",
            );
            return Bool::FALSE;
        }
        let r = vec3_to_rapier(position);
        let rn = r.length();
        if rn <= EPS {
            set_error(ERR_INVALID_ARGUMENT, "gravity-gradient position is zero");
            return Bool::FALSE;
        }
        let n = r / rn;
        let in_vec = Vector::new(
            inertia_diag.x * n.x,
            inertia_diag.y * n.y,
            inertia_diag.z * n.z,
        );
        write_out(
            out_torque,
            vec3_from_rapier(cross(n, in_vec) * (3.0 * mu / (rn * rn.sqrt()))),
        )
    })
}

/// # Safety
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_j2_acceleration(
    position: Vec3,
    mu: f64,
    equatorial_radius: f64,
    j2: f64,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(position)
            || !finite(&[mu, equatorial_radius, j2])
            || mu <= 0.0
            || equatorial_radius <= 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid J2 parameters");
            return Bool::FALSE;
        }
        let r = vec3_to_rapier(position);
        let radius = r.length();
        if radius <= EPS {
            set_error(ERR_INVALID_ARGUMENT, "position magnitude is zero");
            return Bool::FALSE;
        }
        let z2_r2 = (r.z * r.z) / (radius * radius);
        let factor = 1.5 * j2 * mu * equatorial_radius * equatorial_radius / radius.powi(5);
        write_out(
            out_acceleration,
            vec3_from_rapier(Vector::new(
                factor * r.x * (5.0 * z2_r2 - 1.0),
                factor * r.y * (5.0 * z2_r2 - 1.0),
                factor * r.z * (5.0 * z2_r2 - 3.0),
            )),
        )
    })
}

/// Computes the Sagnac phase rate of a ring interferometer.
///
/// # Safety
/// This function takes no pointers and transfers no ownership; it is safe to call with
/// any argument values. Invalid inputs return `f64::NAN` and set `ERR_INVALID_ARGUMENT`.
#[unsafe(no_mangle)]
pub extern "C" fn space_sagnac_phase_rate(area: f64, angular_rate: f64, wavelength: f64) -> f64 {
    ffi_guard(0.0, || {
        if !finite(&[area, angular_rate, wavelength]) || wavelength <= 0.0 {
            return invalid_nan("invalid Sagnac parameters");
        }
        clear_error();
        8.0 * PI * area * angular_rate / (wavelength * SPEED_OF_LIGHT)
    })
}

/// # Safety
/// `out_acceleration` must be null or point to a valid, writable `Vec3`.
#[unsafe(no_mangle)]
pub extern "C" fn space_solar_radiation_pressure_acceleration(
    sun_direction: Vec3,
    solar_flux: f64,
    reflectivity: f64,
    area: f64,
    mass: f64,
    out_acceleration: *mut Vec3,
) -> Bool {
    ffi_guard(Bool::FALSE, || {
        if !vec3_finite(sun_direction)
            || !finite(&[solar_flux, reflectivity, area, mass])
            || solar_flux < 0.0
            || reflectivity < 0.0
            || area < 0.0
            || mass <= 0.0
        {
            set_error(
                ERR_INVALID_ARGUMENT,
                "invalid solar radiation pressure parameters",
            );
            return Bool::FALSE;
        }
        let Some(dir) = vec3_to_rapier(sun_direction).try_normalize() else {
            set_error(ERR_INVALID_ARGUMENT, "sun direction is zero");
            return Bool::FALSE;
        };
        write_out(
            out_acceleration,
            vec3_from_rapier(dir * (solar_flux / SPEED_OF_LIGHT * reflectivity * area / mass)),
        )
    })
}
