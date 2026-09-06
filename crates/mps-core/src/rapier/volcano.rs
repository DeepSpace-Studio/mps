//! Volcanic-eruption simulation — eruptive plume flow, heating and
//! progressive melting of rigid bodies, attached to a [`WorldHandle`].
//!
//! Pure field math lives in `mps_formula::volcano` (no Rapier state); this
//! module holds the per-world volcano list plus per-body thermal / melt
//! state, and turns the field into forces and mass loss via the `volcano_*`
//! C ABI. Typical frame:
//!
//! ```text
//! volcano_advance(world, dt)         // clocks tick
//! volcano_apply(world, dt, …)        // plume drag + ejecta + heat/melt
//! world_step(world, dt)
//! ```
//!
//! Melting model: each body tracked by the module carries a temperature and
//! a remaining-mass scale. While the plume exposure exceeds the body's melt
//! point it loses mass proportionally to the superheat; at ≤ 5 % remaining
//! mass the body is fully melted — it is disabled in place and its handle is
//! reported to the caller. All entry points return a `u8` status code from
//! `error.rs` (`ERR_OK = 0` on success) and never panic across the FFI
//! boundary.

use mps_formula::disasters::{AIR_DENSITY_SEA_LEVEL, wind_drag_force};
use mps_formula::volcano::{
    AMBIENT_TEMPERATURE, BOMB_RADIUS, BOMB_SPEED, PlumeParams, ROCK_DENSITY, body_heating_rate,
    lava_bomb_impulse, lava_bomb_mass, melt_mass_loss_rate, plume_flow_at, plume_temperature_at,
};

use crate::rapier::error::{
    ERR_CAPACITY, ERR_INTERNAL, ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, ERR_OK,
    clear_error, ffi_guard, set_error,
};
use crate::rapier::ffi::{
    Bool, MAX_OUTPUT_CAPACITY, RigidBodyHandleRaw, Vec3, WorldHandle, vec3_finite, vec3_to_rapier,
};

/// A volcanic-eruption source registered on a world.
pub struct Volcano {
    /// Plume geometry / strength parameters.
    pub params: PlumeParams,
    /// Ejecta rate (lava bombs per second reaching the near-vent zone).
    pub ejecta_rate: f64,
    /// Deterministic RNG state for bomb counts and scatter.
    rng_state: u64,
}

impl Volcano {
    /// Advance by `dt`: tick the ejecta RNG stream.
    fn advance(&mut self, dt: f64) {
        self.rng_state = self
            .rng_state
            .wrapping_add(0x9E37_79B9_7F4A_7C15 ^ (dt.to_bits()));
    }
}

/// Per-body thermal / melt state, created lazily on first contact.
pub struct VolcanoBody {
    /// Body temperature (K). Starts at [`AMBIENT_TEMPERATURE`].
    pub temperature: f64,
    /// Total mass (kg) at first contact — the 100 % reference.
    pub original_total_mass: f64,
    /// Mass contributed by attached colliders (kg); the melt only rescales
    /// the additional-mass slot so the total lands on `scale · original`.
    pub collider_part_mass: f64,
    /// Remaining mass fraction in `0…1`.
    pub mass_scale: f64,
    /// Whether the body has fully melted (disabled in place).
    pub melted: bool,
}

/// Mass contributed by attached colliders = total mass − additional mass.
fn collider_part_of(body: &rapier3d::dynamics::RigidBody) -> f64 {
    let total = body.mass();
    let additional = body
        .mass_properties()
        .additional_local_mprops
        .as_ref()
        .map(|props| match **props {
            rapier3d::dynamics::RigidBodyAdditionalMassProps::Mass(m) => m,
            rapier3d::dynamics::RigidBodyAdditionalMassProps::MassProps(ref mprops) => {
                mprops.mass()
            }
        })
        .unwrap_or(0.0);
    (total - additional).max(0.0)
}

/// `splitmix64` step — deterministic RNG stream for ejecta counts and
/// scatter (same construction as the disaster module).
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

// ---------------------------------------------------------------------------
// C ABI — registration & lifecycle
// ---------------------------------------------------------------------------

/// Register a volcanic-eruption source: a vertical eruptive plume rising
/// from `vent`, plus optional lava-bomb ejecta (`ejecta_rate` bombs per
/// second; 0 disables ejecta). `lava_temperature` may be 0 to use the
/// basaltic default (1473.15 K). Returns `ERR_OK` and writes the stable
/// source id to `out_id` (when non-null).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_id`, when non-null,
/// must be valid for a single `u32` write.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_add(
    world: *mut WorldHandle,
    vent: Vec3,
    plume_radius: f64,
    plume_height: f64,
    max_updraft: f64,
    lava_temperature: f64,
    ejecta_rate: f64,
    out_id: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if world.inner.volcanoes.map.len() >= MAX_OUTPUT_CAPACITY as usize {
            set_error(ERR_CAPACITY, "too many active volcanoes");
            return ERR_CAPACITY as u8;
        }
        if !vec3_finite(vent) {
            set_error(ERR_INVALID_ARGUMENT, "vent must be finite");
            return ERR_INVALID_ARGUMENT as u8;
        }
        for (value, name) in [
            (plume_radius, "plume_radius"),
            (plume_height, "plume_height"),
        ] {
            if !value.is_finite() || value <= 0.0 {
                set_error(ERR_INVALID_ARGUMENT, name);
                return ERR_INVALID_ARGUMENT as u8;
            }
        }
        // max_updraft may be 0: a degenerate plume with no wind but full
        // thermal exposure (lava lake).
        if !max_updraft.is_finite() || max_updraft < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "max_updraft");
            return ERR_INVALID_ARGUMENT as u8;
        }
        if !lava_temperature.is_finite() || !ejecta_rate.is_finite() || ejecta_rate < 0.0 {
            set_error(ERR_INVALID_ARGUMENT, "invalid lava_temperature/ejecta_rate");
            return ERR_INVALID_ARGUMENT as u8;
        }

        let volcano = Volcano {
            params: PlumeParams {
                vent,
                plume_radius,
                plume_height,
                max_updraft,
                lava_temperature: if lava_temperature == 0.0 {
                    mps_formula::volcano::LAVA_TEMPERATURE
                } else {
                    lava_temperature
                },
            },
            ejecta_rate,
            rng_state: (world.inner.volcanoes.next_id.wrapping_mul(0x9E37_79B9)) as u64,
        };
        let id = world.inner.volcanoes.insert(volcano);
        if let Some(out) = unsafe { out_id.as_mut() } {
            *out = id;
        }
        clear_error();
        ERR_OK as u8
    })
}

/// Remove a volcano source by id (returns `ERR_NOT_FOUND` for a stale id).
/// Per-body thermal state is kept so a re-registered volcano resumes where
/// the field left off; use [`volcano_clear`] to reset it.
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_remove(world: *mut WorldHandle, id: u32) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if world.inner.volcanoes.map.remove(&id).is_some() {
            clear_error();
            ERR_OK as u8
        } else {
            set_error(ERR_NOT_FOUND, "volcano id not found");
            ERR_NOT_FOUND as u8
        }
    })
}

/// Remove every volcano source and reset all per-body thermal / melt state
/// (bodies melted earlier stay disabled).
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_clear(world: *mut WorldHandle) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        world.inner.volcanoes.map.clear();
        world.inner.volcano_bodies.clear();
        clear_error();
        ERR_OK as u8
    })
}

/// Advance every volcano by `dt` seconds (clocks / ejecta RNG). Call once
/// per frame before [`volcano_apply`].
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_advance(world: *mut WorldHandle, dt: f64) -> u8 {
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
        for volcano in world.inner.volcanoes.map.values_mut() {
            volcano.advance(dt);
        }
        clear_error();
        ERR_OK as u8
    })
}

// ---------------------------------------------------------------------------
// C ABI — sampling
// ---------------------------------------------------------------------------

/// Sample the combined plume flow velocity (m/s) of every registered volcano
/// at `point`.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_flow` must be valid for
/// a single `Vec3` write.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_sample_flow(
    world: *mut WorldHandle,
    point: Vec3,
    out_flow: *mut Vec3,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if out_flow.is_null() {
            set_error(ERR_NULL_POINTER, "out_flow is null");
            return ERR_NULL_POINTER as u8;
        }
        if !vec3_finite(point) {
            set_error(ERR_INVALID_ARGUMENT, "point must be finite");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let mut flow = rapier3d::prelude::Vector::new(0.0, 0.0, 0.0);
        for volcano in world.inner.volcanoes.map.values() {
            let w = plume_flow_at(&volcano.params, point);
            flow += rapier3d::prelude::Vector::new(w.x, w.y, w.z);
        }
        unsafe {
            *out_flow = Vec3 {
                x: flow.x,
                y: flow.y,
                z: flow.z,
            }
        };
        clear_error();
        ERR_OK as u8
    })
}

/// Sample the exposure temperature (K) of every registered volcano at
/// `point` — the highest in-plume air temperature (never below ambient).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_temp` must be valid for
/// a single `f64` write.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_sample_temperature(
    world: *mut WorldHandle,
    point: Vec3,
    out_temp: *mut f64,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if out_temp.is_null() {
            set_error(ERR_NULL_POINTER, "out_temp is null");
            return ERR_NULL_POINTER as u8;
        }
        if !vec3_finite(point) {
            set_error(ERR_INVALID_ARGUMENT, "point must be finite");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let mut exposure = AMBIENT_TEMPERATURE;
        for volcano in world.inner.volcanoes.map.values() {
            let t = plume_temperature_at(&volcano.params, point, AMBIENT_TEMPERATURE);
            if t > exposure {
                exposure = t;
            }
        }
        unsafe { *out_temp = exposure };
        clear_error();
        ERR_OK as u8
    })
}

// ---------------------------------------------------------------------------
// C ABI — per-body thermal state
// ---------------------------------------------------------------------------

/// Query a body's tracked temperature (K). Bodies the module has never seen
/// report `ERR_NOT_FOUND`.
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_temp` must be valid for
/// a single `f64` write.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_body_temperature(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    out_temp: *mut f64,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if out_temp.is_null() {
            set_error(ERR_NULL_POINTER, "out_temp is null");
            return ERR_NULL_POINTER as u8;
        }
        match world.inner.volcano_bodies.get(&body_handle) {
            Some(state) => {
                unsafe { *out_temp = state.temperature };
                clear_error();
                ERR_OK as u8
            }
            None => {
                set_error(ERR_NOT_FOUND, "body has no thermal state yet");
                ERR_NOT_FOUND as u8
            }
        }
    })
}

/// Override a body's tracked temperature (K), creating the thermal state if
/// needed. Useful for scripted pre-heating.
///
/// # Safety
///
/// `world` must be a valid, live world pointer.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_set_body_temperature(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    temperature: f64,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if !temperature.is_finite() {
            set_error(ERR_INVALID_ARGUMENT, "temperature must be finite");
            return ERR_INVALID_ARGUMENT as u8;
        }
        let state = world
            .inner
            .volcano_bodies
            .entry(body_handle)
            .or_insert_with(|| VolcanoBody {
                temperature,
                original_total_mass: 0.0,
                collider_part_mass: 0.0,
                mass_scale: 1.0,
                melted: false,
            });
        state.temperature = temperature;
        clear_error();
        ERR_OK as u8
    })
}

// ---------------------------------------------------------------------------
// C ABI — force / heat / melt application
// ---------------------------------------------------------------------------

/// Apply the combined volcanic field to every dynamic rigid body:
///
/// * **plume drag** — `F = ½ ρ |v_rel| v_rel` (C_d·A folded to 1 m²) from
///   the plume flow sampled at the body origin;
/// * **lava-bomb ejecta** — for bodies in the near-vent zone of a volcano
///   with `ejecta_rate > 0`, a deterministic (splitmix64) number of
///   upward-outward capture impulses delivered as an equivalent frame force;
/// * **heating & melting** — the body temperature moves towards the local
///   exposure temperature at `heat_exchange_rate` (1/s); above `melt_point`
///   (K) the body loses mass at `melt_rate · (T − T_melt) / T_melt` of its
///   remaining mass per second; at ≤ 5 % remaining mass the body is fully
///   melted: it is disabled in place and its raw handle is reported.
///
/// `melt_rate` is a fraction per second at 2× superheat (i.e. the rate
/// formula divides the superheat by the melt point). Melting rescales the
/// body's additional-mass slot so the total mass lands on
/// `scale · original_total_mass`.
///
/// Both counters report **this call only** (a body that melted in an earlier
/// frame is not counted again). Writes up to `melted_capacity` melted body
/// handles into `out_melted` and the delivered bomb count to `out_bomb_count`
/// (both optional).
///
/// # Safety
///
/// `world` must be a valid, live world pointer; `out_melted`, when non-null,
/// must be valid for `melted_capacity` `u64` writes; the counter pointers,
/// when non-null, must be valid for a single `u32` write each.
#[unsafe(no_mangle)]
pub extern "C" fn volcano_apply(
    world: *mut WorldHandle,
    dt: f64,
    melt_point: f64,
    melt_rate: f64,
    heat_exchange_rate: f64,
    wake_up: Bool,
    out_bomb_count: *mut u32,
    out_melted: *mut u64,
    melted_capacity: u32,
    out_melted_count: *mut u32,
) -> u8 {
    ffi_guard(ERR_INTERNAL as u8, || {
        let Some(world) = (unsafe { world.as_mut() }) else {
            set_error(ERR_NULL_POINTER, "world is null");
            return ERR_NULL_POINTER as u8;
        };
        if !dt.is_finite()
            || dt < 0.0
            || !melt_point.is_finite()
            || melt_point <= 0.0
            || !melt_rate.is_finite()
            || melt_rate < 0.0
            || !heat_exchange_rate.is_finite()
            || heat_exchange_rate < 0.0
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid volcano apply parameters");
            return ERR_INVALID_ARGUMENT as u8;
        }

        struct Snapshot {
            params: PlumeParams,
            ejecta_rate: f64,
            rng_state: u64,
        }
        let snapshots: Vec<Snapshot> = world
            .inner
            .volcanoes
            .map
            .values()
            .map(|v| Snapshot {
                params: v.params,
                ejecta_rate: v.ejecta_rate,
                rng_state: v.rng_state,
            })
            .collect();
        let wake = wake_up.0 != 0;
        let mut bomb_count = 0u32;
        let mut melted_count = 0u32;

        // Snapshot the handles (with generation) so the per-body thermal map
        // and the body set can be borrowed separately inside the loop.
        let handles: Vec<rapier3d::prelude::RigidBodyHandle> =
            world.inner.bodies.iter().map(|(h, _)| h).collect();

        for handle in handles {
            let Some(body) = world.inner.bodies.get_mut(handle) else {
                continue;
            };
            let raw = crate::rapier::ffi::pack_rigid_body_handle(handle);
            if !body.is_dynamic() || !body.is_enabled() || (body.is_sleeping() && !wake) {
                continue;
            }
            let pos = vec3_from_flow_origin(body);

            // Lazily create the thermal state (captures the mass reference).
            let state = world
                .inner
                .volcano_bodies
                .entry(raw)
                .or_insert_with(|| VolcanoBody {
                    temperature: AMBIENT_TEMPERATURE,
                    original_total_mass: 0.0,
                    collider_part_mass: 0.0,
                    mass_scale: 1.0,
                    melted: false,
                });
            // `body.mass()` reports 0 until the first step refreshes the mass
            // properties, so capture the reference lazily until it is real.
            if state.original_total_mass <= 0.0 {
                let m = body.mass();
                if m > 0.0 {
                    state.original_total_mass = m;
                    state.collider_part_mass = collider_part_of(body);
                }
            }
            if state.melted {
                continue;
            }

            let mut ejecta_total = rapier3d::prelude::Vector::new(0.0, 0.0, 0.0);
            let mut exposure = AMBIENT_TEMPERATURE;
            let linvel = body.linvel();

            for snap in &snapshots {
                // Plume drag on the body.
                let flow = plume_flow_at(&snap.params, pos);
                let relative = rapier3d::prelude::Vector::new(flow.x, flow.y, flow.z) - linvel;
                if relative != rapier3d::prelude::Vector::new(0.0, 0.0, 0.0) {
                    let drag = wind_drag_force(
                        Vec3 {
                            x: relative.x,
                            y: relative.y,
                            z: relative.z,
                        },
                        AIR_DENSITY_SEA_LEVEL,
                        1.0,
                        1.0,
                    );
                    body.add_force(vec3_to_rapier(drag), wake);
                }

                // Thermal exposure.
                let t = plume_temperature_at(&snap.params, pos, AMBIENT_TEMPERATURE);
                if t > exposure {
                    exposure = t;
                }

                // Lava-bomb ejecta in the near-vent zone.
                if snap.ejecta_rate > 0.0 && dt > 0.0 {
                    let dx = pos.x - snap.params.vent.x;
                    let dz = pos.z - snap.params.vent.z;
                    let r2 = dx * dx + dz * dz;
                    let h = (pos.y - snap.params.vent.y).max(0.0);
                    if r2 <= snap.params.plume_radius * snap.params.plume_radius
                        && h <= snap.params.plume_height * 0.5
                    {
                        let expected = snap.ejecta_rate * dt;
                        let mut rng = snap
                            .rng_state
                            .wrapping_add(raw.wrapping_mul(0x9E37_79B9_7F4A_7C15));
                        let n = expected.trunc() as u32
                            + u32::from(u01(splitmix64(&mut rng)) < expected.fract());
                        if n > 0 {
                            let impulse_mag = lava_bomb_impulse(
                                lava_bomb_mass(BOMB_RADIUS, ROCK_DENSITY),
                                BOMB_SPEED,
                            );
                            // Launch direction: up-dominant, tilted radially
                            // outward from the vent, with deterministic jitter.
                            let r = r2.sqrt();
                            let (rx, rz) = if r > 1.0e-9 {
                                (dx / r, dz / r)
                            } else {
                                (0.0, 0.0)
                            };
                            for _ in 0..n {
                                let jx = (u01(splitmix64(&mut rng)) - 0.5) * 0.5;
                                let jz = (u01(splitmix64(&mut rng)) - 0.5) * 0.5;
                                let up = 2.0 + u01(splitmix64(&mut rng));
                                let dir_x = rx + jx;
                                let dir_y = up;
                                let dir_z = rz + jz;
                                let len = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z)
                                    .sqrt()
                                    .max(1.0e-9);
                                ejecta_total += rapier3d::prelude::Vector::new(
                                    impulse_mag * dir_x / len,
                                    impulse_mag * dir_y / len,
                                    impulse_mag * dir_z / len,
                                );
                                bomb_count += 1;
                            }
                        }
                    }
                }
            }

            // Deliver ejecta momentum as an equivalent frame force (freshly
            // inserted bodies have a stale effective_inv_mass; see the
            // disaster module for the same reasoning).
            if ejecta_total != rapier3d::prelude::Vector::new(0.0, 0.0, 0.0) {
                body.add_force(ejecta_total / dt, wake);
            }

            // Heating towards the exposure temperature, then melting.
            let state = world
                .inner
                .volcano_bodies
                .get_mut(&raw)
                .expect("just inserted");
            state.temperature +=
                body_heating_rate(exposure, state.temperature, heat_exchange_rate) * dt;
            let loss = melt_mass_loss_rate(state.temperature, melt_point, melt_rate);
            if loss > 0.0 && dt > 0.0 && state.original_total_mass > 0.0 {
                state.mass_scale = (state.mass_scale - loss * dt).max(0.0);
                let new_total = state.original_total_mass * state.mass_scale;
                let additional = new_total - state.collider_part_mass;
                body.set_additional_mass(additional, wake);
                if state.mass_scale <= 0.05 {
                    state.melted = true;
                    body.set_enabled(false);
                    if melted_capacity > 0 && melted_count < melted_capacity {
                        unsafe { *out_melted.add(melted_count as usize) = raw };
                    }
                    melted_count += 1;
                }
            }
        }

        if let Some(out) = unsafe { out_bomb_count.as_mut() } {
            *out = bomb_count;
        }
        if let Some(out) = unsafe { out_melted_count.as_mut() } {
            *out = melted_count;
        }
        clear_error();
        ERR_OK as u8
    })
}

/// Body sampling origin — `translation()` because `world_com` is only
/// refreshed on step (see the disaster module for the same pitfall).
fn vec3_from_flow_origin(body: &rapier3d::dynamics::RigidBody) -> Vec3 {
    let t = body.translation();
    Vec3 {
        x: t.x,
        y: t.y,
        z: t.z,
    }
}
