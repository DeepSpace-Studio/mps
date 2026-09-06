//! Natural-disaster simulation — typhoon (tropical cyclone), tornado and
//! hailstorm sources attached to a [`WorldHandle`].
//!
//! Pure field math lives in `mps_formula::disasters` (no Rapier state); this
//! module only holds the per-world source list, samples the combined wind
//! field, and turns it into rigid-body forces / hail impact impulses via the
//! `disaster_*` C ABI. Typical frame:
//!
//! ```text
//! disaster_advance(world, dt)            // storms translate, clocks tick
//! disaster_apply_forces(world, …)        // wind drag + hail impacts
//! world_step(world, dt)
//! ```
//!
//! All entry points return a `u8` status code from `error.rs`
//! (`ERR_OK = 0` on success) and never panic across the FFI boundary.

use mps_formula::disasters::{
    AIR_DENSITY_SEA_LEVEL, CycloneParams, ICE_DENSITY, TornadoParams, boundary_layer_height_factor,
    cyclone_tangential_speed, cyclone_wind_at, cyclostrophic_pressure_drop, hail_impulse,
    hail_mass, hail_terminal_velocity, tornado_wind_at, wind_drag_force,
};

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INTERNAL, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
    clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, MAX_OUTPUT_CAPACITY, Vec3, WorldHandle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

/// Per-disaster snapshot used while iterating bodies (the disaster registry
/// and the Rapier body set cannot be borrowed simultaneously).
enum FieldSample {
    Cyclone {
        params: CycloneParams,
    },
    Tornado {
        params: TornadoParams,
    },
    Hail {
        center: Vec3,
        radius: f64,
        intensity: f64,
        hail_radius: f64,
        rng_state: u64,
    },
}

/// A disaster source registered on a world.
pub enum Disaster {
    /// Tropical cyclone (typhoon / hurricane — same model, regional names).
    Cyclone { params: CycloneParams },
    /// Tornado on a vertical axis.
    Tornado { params: TornadoParams },
    /// Hailstorm cell: hailstones rain down inside a horizontal disc.
    Hail {
        /// Cell centre (ground point); impacts fall inside `radius`.
        center: Vec3,
        /// Cell radius (m).
        radius: f64,
        /// Impact intensity (expected impacts per m² of reference area per
        /// second).
        intensity: f64,
        /// Hailstone radius (m).
        hail_radius: f64,
        /// Deterministic RNG state for impact counts and scatter.
        rng_state: u64,
    },
}

impl Disaster {
    /// Advance the storm by `dt`: translate the centre/base with the storm
    /// motion. Hail cells do not translate; their state only feeds the RNG.
    fn advance(&mut self, dt: f64) {
        match self {
            Disaster::Cyclone { params } => {
                params.center.x += params.translation.x * dt;
                params.center.y += params.translation.y * dt;
                params.center.z += params.translation.z * dt;
            }
            Disaster::Tornado { params } => {
                params.base.x += params.translation.x * dt;
                params.base.y += params.translation.y * dt;
                params.base.z += params.translation.z * dt;
            }
            Disaster::Hail { rng_state, .. } => {
                *rng_state = rng_state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            }
        }
    }

    /// Snapshot for the body-iteration pass.
    fn sample(&self) -> FieldSample {
        match self {
            Disaster::Cyclone { params } => FieldSample::Cyclone { params: *params },
            Disaster::Tornado { params } => FieldSample::Tornado { params: *params },
            Disaster::Hail {
                center,
                radius,
                intensity,
                hail_radius,
                rng_state,
            } => FieldSample::Hail {
                center: *center,
                radius: *radius,
                intensity: *intensity,
                hail_radius: *hail_radius,
                rng_state: *rng_state,
            },
        }
    }

    /// Cyclostrophic pressure deficit (Pa) contributed at `point`.
    fn pressure_drop_at(&self, point: Vec3, air_density: f64) -> f64 {
        match self {
            Disaster::Cyclone { params } => {
                let dx = point.x - params.center.x;
                let dz = point.z - params.center.z;
                let r = (dx * dx + dz * dz).sqrt();
                let v = cyclone_tangential_speed(params.max_wind, params.radius_max_wind, r)
                    * boundary_layer_height_factor(
                        (point.y - params.center.y).max(0.0),
                        params.reference_height,
                        params.height_exponent,
                    );
                cyclostrophic_pressure_drop(v, air_density)
            }
            Disaster::Tornado { params } => {
                let dx = point.x - params.base.x;
                let dz = point.z - params.base.z;
                let r = (dx * dx + dz * dz).sqrt();
                let rc = params.core_radius.max(1.0e-9);
                let height = (point.y - params.base.y).max(0.0);
                if height > 2.0 * params.top_height {
                    return 0.0;
                }
                let v = params.max_wind * if r <= rc { r / rc } else { rc / r.max(1.0e-9) };
                cyclostrophic_pressure_drop(v, air_density)
            }
            Disaster::Hail { .. } => 0.0,
        }
    }
}

/// `splitmix64` step — deterministic, allocation-free RNG stream for hail
/// impact counts and lateral scatter.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn u01(x: u64) -> f64 {
    (x >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Apply quadratic wind drag to one body from the relative wind
/// `wind − linvel`; flips `received` when a nonzero force was added.
fn apply_wind_drag(
    body: &mut rapier3d::dynamics::RigidBody,
    wind: Vec3,
    linvel: Vec3,
    air_density: f64,
    drag_coefficient: f64,
    reference_area: f64,
    wake: bool,
    received: &mut bool,
) {
    let relative = Vec3 {
        x: wind.x - linvel.x,
        y: wind.y - linvel.y,
        z: wind.z - linvel.z,
    };
    let force = wind_drag_force(relative, air_density, drag_coefficient, reference_area);
    if force.x != 0.0 || force.y != 0.0 || force.z != 0.0 {
        body.add_force(vec3_to_rapier(force), wake);
        *received = true;
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn finite_or_invalid(value: f64, message: &str) -> Result<(), u8> {
    if value.is_finite() {
        Ok(())
    } else {
        set_error(ERR_INVALID_ARGUMENT, message);
        Err(ERR_INVALID_ARGUMENT as u8)
    }
}

fn positive_or_invalid(value: f64, message: &str) -> Result<(), u8> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        set_error(ERR_INVALID_ARGUMENT, message);
        Err(ERR_INVALID_ARGUMENT as u8)
    }
}

fn spin_or_invalid(spin: f64) -> Result<f64, u8> {
    if spin.is_finite() && spin != 0.0 {
        Ok(spin.signum())
    } else {
        set_error(ERR_INVALID_ARGUMENT, "spin must be a nonzero finite value");
        Err(ERR_INVALID_ARGUMENT as u8)
    }
}

// ---------------------------------------------------------------------------
// C ABI — registration & lifecycle
// ---------------------------------------------------------------------------

/// Shared registration path: validate the world pointer, derive a
/// deterministic RNG seed from the registry counter, insert the disaster and
/// report its stable id.
fn run_add(
    world: *mut WorldHandle,
    out_id: *mut u32,
    build: impl FnOnce(u64) -> Result<Disaster, u8>,
) -> u8 {
    let Some(world) = (unsafe { world.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "world is null");
        return ERR_NULL_POINTER as u8;
    };
    if world.inner.disasters.map.len() >= MAX_OUTPUT_CAPACITY as usize {
        set_error(ERR_CAPACITY, "too many active disasters");
        return ERR_CAPACITY as u8;
    }
    let seed = (world.inner.disasters.next_id.wrapping_mul(0x9E37_79B9)) as u64;
    match build(seed) {
        Ok(disaster) => {
            let id = world.inner.disasters.insert(disaster);
            if let Some(out) = unsafe { out_id.as_mut() } {
                *out = id;
            }
            clear_error();
            ERR_OK as u8
        }
        Err(code) => code,
    }
}

/// Register a tropical cyclone (typhoon / hurricane — the same model covers
/// both regional names; use `spin = -1` for the southern hemisphere).
///
/// `inflow_fraction` / `height_exponent` / `reference_height` may be 0 to use
/// the model defaults (0.15 / 0.11 / 10 m). Returns `ERR_OK` and writes the
/// stable source id to `out_id` (when non-null).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_id`, when non-null, must
/// be valid for a single `u32` write.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_add_typhoon(
    world: *mut WorldHandle,
    center: Vec3,
    max_wind: f64,
    radius_max_wind: f64,
    translation: Vec3,
    spin: f64,
    inflow_fraction: f64,
    height_exponent: f64,
    reference_height: f64,
    out_id: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        run_add(world, out_id, |_seed| {
            positive_or_invalid(max_wind, "max_wind must be positive")?;
            positive_or_invalid(radius_max_wind, "radius_max_wind must be positive")?;
            let spin = spin_or_invalid(spin)?;
            if !vec3_finite(center) || !vec3_finite(translation) {
                set_error(ERR_INVALID_ARGUMENT, "center/translation must be finite");
                return Err(ERR_INVALID_ARGUMENT as u8);
            }
            let inflow = if inflow_fraction == 0.0 {
                0.15
            } else if (0.0..=0.5).contains(&inflow_fraction) {
                inflow_fraction
            } else {
                set_error(ERR_INVALID_ARGUMENT, "inflow_fraction must be in 0..=0.5");
                return Err(ERR_INVALID_ARGUMENT as u8);
            };
            let exponent = if height_exponent == 0.0 {
                0.11
            } else if (0.0..=0.5).contains(&height_exponent) {
                height_exponent
            } else {
                set_error(ERR_INVALID_ARGUMENT, "height_exponent must be in 0..=0.5");
                return Err(ERR_INVALID_ARGUMENT as u8);
            };
            let reference = if reference_height == 0.0 {
                10.0
            } else if reference_height.is_finite() && reference_height > 0.0 {
                reference_height
            } else {
                set_error(ERR_INVALID_ARGUMENT, "reference_height must be positive");
                return Err(ERR_INVALID_ARGUMENT as u8);
            };
            Ok(Disaster::Cyclone {
                params: CycloneParams {
                    center,
                    translation,
                    max_wind,
                    radius_max_wind,
                    spin,
                    inflow_fraction: inflow,
                    height_exponent: exponent,
                    reference_height: reference,
                },
            })
        })
    })
}

/// Register a tornado on a vertical axis through `base`.
///
/// Returns `ERR_OK` and writes the stable source id to `out_id` (when
/// non-null).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_id`, when non-null, must
/// be valid for a single `u32` write.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_add_tornado(
    world: *mut WorldHandle,
    base: Vec3,
    max_wind: f64,
    core_radius: f64,
    top_height: f64,
    translation: Vec3,
    spin: f64,
    updraft_fraction: f64,
    out_id: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        run_add(world, out_id, |_seed| {
            positive_or_invalid(max_wind, "max_wind must be positive")?;
            positive_or_invalid(core_radius, "core_radius must be positive")?;
            positive_or_invalid(top_height, "top_height must be positive")?;
            let spin = spin_or_invalid(spin)?;
            if !vec3_finite(base) || !vec3_finite(translation) {
                set_error(ERR_INVALID_ARGUMENT, "base/translation must be finite");
                return Err(ERR_INVALID_ARGUMENT as u8);
            }
            Ok(Disaster::Tornado {
                params: TornadoParams {
                    base,
                    translation,
                    max_wind,
                    core_radius,
                    top_height,
                    updraft_fraction: updraft_fraction.clamp(0.0, 1.0),
                    spin,
                },
            })
        })
    })
}

/// Register a hailstorm cell: hailstones rain down inside a horizontal disc
/// of `radius` around `center`. `intensity` is the expected number of
/// impacts per m² of reference area per second; `disaster_apply_forces`
/// converts this into capture impulses on dynamic bodies inside the cell.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_id`, when non-null, must
/// be valid for a single `u32` write.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_add_hail(
    world: *mut WorldHandle,
    center: Vec3,
    radius: f64,
    intensity: f64,
    hail_radius: f64,
    out_id: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        run_add(world, out_id, |seed| {
            positive_or_invalid(radius, "radius must be positive")?;
            positive_or_invalid(hail_radius, "hail_radius must be positive")?;
            finite_or_invalid(intensity, "intensity must be finite")?;
            if intensity < 0.0 {
                set_error(ERR_INVALID_ARGUMENT, "intensity must be non-negative");
                return Err(ERR_INVALID_ARGUMENT as u8);
            }
            if !vec3_finite(center) {
                set_error(ERR_INVALID_ARGUMENT, "center must be finite");
                return Err(ERR_INVALID_ARGUMENT as u8);
            }
            Ok(Disaster::Hail {
                center,
                radius,
                intensity,
                hail_radius,
                rng_state: seed,
            })
        })
    })
}

/// Remove a disaster source by id (returns `ERR_NOT_FOUND` for a stale id).
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_remove(world: *mut WorldHandle, id: u32) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if world.inner.disasters.map.remove(&id).is_some() {
            clear_error();
            ERR_OK as u8
        } else {
            set_error(ERR_NOT_FOUND, "disaster id not found");
            ERR_NOT_FOUND as u8
        }
    })
}

/// Remove every disaster source from the world.
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_clear(world: *mut WorldHandle) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        world.inner.disasters.map.clear();
        clear_error();
        ERR_OK as u8
    })
}

/// Advance every disaster by `dt` seconds: storm centres translate with their
/// storm motion, hail RNG clocks tick. Call once per frame before
/// `disaster_apply_forces`.
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_advance(world: *mut WorldHandle, dt: f64) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if !dt.is_finite() || dt < 0.0 {
            set_error(
                ERR_INVALID_ARGUMENT,
                "dt must be a non-negative finite value",
            );
            return ERR_INVALID_ARGUMENT as u8;
        }
        for disaster in world.inner.disasters.map.values_mut() {
            disaster.advance(dt);
        }
        clear_error();
        ERR_OK as u8
    })
}

// ---------------------------------------------------------------------------
// C ABI — sampling
// ---------------------------------------------------------------------------

/// Sample the combined wind field (m/s) of every registered vortex-type
/// disaster (typhoons + tornadoes; hail contributes none) at `point`.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_wind` must be valid for
/// a single `Vec3` write.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_sample_wind(
    world: *mut WorldHandle,
    point: Vec3,
    out_wind: *mut Vec3,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if out_wind.is_null() {
            set_error(ERR_NULL_POINTER, "out_wind is null");
            return ERR_NULL_POINTER as u8;
        }
        if !vec3_finite(point) {
            set_error(ERR_INVALID_ARGUMENT, "point must be finite");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let mut wind = rapier3d::prelude::Vector::new(0.0, 0.0, 0.0);
        for disaster in world.inner.disasters.map.values() {
            match disaster {
                Disaster::Cyclone { params } => {
                    let w = cyclone_wind_at(params, point);
                    wind += rapier3d::prelude::Vector::new(w.x, w.y, w.z);
                }
                Disaster::Tornado { params } => {
                    let w = tornado_wind_at(params, point);
                    wind += rapier3d::prelude::Vector::new(w.x, w.y, w.z);
                }
                Disaster::Hail { .. } => {}
            }
        }
        unsafe { *out_wind = vec3_from_rapier(wind) };
        clear_error();
        ERR_OK as u8
    })
}

/// Sample the summed cyclostrophic pressure deficit (Pa) of all vortex
/// disasters at `point` (core pressure drop; 0 outside their reach).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_drop` must be valid for
/// a single `f64` write.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_sample_pressure_drop(
    world: *mut WorldHandle,
    point: Vec3,
    air_density: f64,
    out_drop: *mut f64,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if out_drop.is_null() {
            set_error(ERR_NULL_POINTER, "out_drop is null");
            return ERR_NULL_POINTER as u8;
        }
        if !vec3_finite(point) || !air_density.is_finite() || air_density < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid point or air_density");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let mut drop = 0.0;
        for disaster in world.inner.disasters.map.values() {
            drop += disaster.pressure_drop_at(point, air_density);
        }
        unsafe { *out_drop = drop };
        clear_error();
        ERR_OK as u8
    })
}

// ---------------------------------------------------------------------------
// C ABI — force application
// ---------------------------------------------------------------------------

/// Apply the combined disaster field to every dynamic rigid body:
///
/// * **wind drag** — for each typhoon/tornado, `F = ½ ρ C_d A |v_rel| v_rel`
///   from the wind sampled at the body centre of mass;
/// * **hail impacts** — for each hail cell containing the body, a
///   deterministic (splitmix64) number of perfectly-inelastic capture
///   impulses `m_hail · v_terminal`, directed mostly downwards with a small
///   lateral scatter; `intensity · reference_area · dt` is the expected
///   impact count. The accumulated impulse is delivered as an equivalent
///   force over the frame so it flows through the normal step integration.
///
/// `air_density` may be 0 to use the ISA sea-level default (1.225 kg/m³).
/// Writes the number of bodies that received wind force and the total number
/// of hail impulses to the optional counters.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; the counter pointers, when
/// non-null, must be valid for a single `u32` write each.
#[unsafe(no_mangle)]
pub extern "C" fn disaster_apply_forces(
    world: *mut WorldHandle,
    air_density: f64,
    drag_coefficient: f64,
    reference_area: f64,
    dt: f64,
    wake_up: Bool,
    out_body_count: *mut u32,
    out_impact_count: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if !air_density.is_finite()
            || !drag_coefficient.is_finite()
            || !reference_area.is_finite()
            || !dt.is_finite()
            || air_density < 0.0
            || drag_coefficient < 0.0
            || reference_area < 0.0
            || dt < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid disaster force parameters");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let air_density = if air_density == 0.0 {
            AIR_DENSITY_SEA_LEVEL
        } else {
            air_density
        };

        let field: Vec<FieldSample> = world
            .inner
            .disasters
            .map
            .values()
            .map(Disaster::sample)
            .collect();
        let mut body_count = 0u32;
        let mut impact_count = 0u32;

        for (handle, body) in world.inner.bodies.iter_mut() {
            if !body.is_dynamic() || (body.is_sleeping() && wake_up.0 == 0) {
                continue;
            }
            // Sample at the body origin: `world_com` is only refreshed on
            // step, so it is stale for freshly inserted bodies.
            let com = vec3_from_rapier(body.translation());
            let linvel = vec3_from_rapier(body.linvel());
            let mut received_wind = false;
            // Hail momentum for this frame, delivered through the force
            // pipeline (`total_impulse / dt`): `apply_impulse` would hit a
            // stale `effective_inv_mass` on freshly inserted bodies, while
            // the step integration sees refreshed mass properties.
            let mut hail_total = rapier3d::prelude::Vector::new(0.0, 0.0, 0.0);

            for sample in &field {
                match sample {
                    FieldSample::Cyclone { params } => apply_wind_drag(
                        body,
                        cyclone_wind_at(params, com),
                        linvel,
                        air_density,
                        drag_coefficient,
                        reference_area,
                        wake_up.0 != 0,
                        &mut received_wind,
                    ),
                    FieldSample::Tornado { params } => apply_wind_drag(
                        body,
                        tornado_wind_at(params, com),
                        linvel,
                        air_density,
                        drag_coefficient,
                        reference_area,
                        wake_up.0 != 0,
                        &mut received_wind,
                    ),
                    FieldSample::Hail {
                        center,
                        radius,
                        intensity,
                        hail_radius,
                        rng_state,
                    } => {
                        let dx = com.x - center.x;
                        let dz = com.z - center.z;
                        if dx * dx + dz * dz > radius * radius {
                            continue;
                        }
                        let expected = intensity * reference_area * dt;
                        if expected <= 0.0 {
                            continue;
                        }
                        let mut rng =
                            rng_state.wrapping_add((handle.into_raw_parts().0 as u64) << 32);
                        let n = expected.trunc() as u32
                            + u32::from(u01(splitmix64(&mut rng)) < expected.fract());
                        let mass = hail_mass(*hail_radius, ICE_DENSITY);
                        let speed =
                            hail_terminal_velocity(*hail_radius, ICE_DENSITY, air_density, 0.47);
                        let impulse_mag = hail_impulse(mass, speed);
                        for _ in 0..n {
                            let jx = (u01(splitmix64(&mut rng)) - 0.5) * 0.3;
                            let jz = (u01(splitmix64(&mut rng)) - 0.5) * 0.3;
                            hail_total += rapier3d::prelude::Vector::new(
                                impulse_mag * jx,
                                -impulse_mag,
                                impulse_mag * jz,
                            );
                            impact_count += 1;
                        }
                    }
                }
            }

            // Deliver the accumulated hail momentum as an equivalent frame
            // force (only possible when dt > 0, which the impact count
            // already guarantees).
            if hail_total != rapier3d::prelude::Vector::new(0.0, 0.0, 0.0) {
                body.add_force(hail_total / dt, wake_up.0 != 0);
            }

            if received_wind {
                body_count += 1;
            }
        }

        if let Some(out) = unsafe { out_body_count.as_mut() } {
            *out = body_count;
        }
        if let Some(out) = unsafe { out_impact_count.as_mut() } {
            *out = impact_count;
        }
        clear_error();
        ERR_OK as u8
    })
}
