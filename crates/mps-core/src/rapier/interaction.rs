//! Body-body interactions: Newtonian gravity, Coulomb friction, and air drag.
//!
//! This module bridges the existing formula layer (aerodynamics, trajectory,
//! spaceflight) to Rapier rigid bodies.  Callers configure laws once via the
//! `world_set_*_law` FFI, and `apply_body_interactions` runs inside `world_step`
//! to inject computed forces into the physics pipeline.
//!
//! ## Architecture
//!
//! ```text
//! world_step()
//!   ├── apply_body_interactions()
//!   │     ├── pairwise_gravity()      ← Newton's law between every body pair
//!   │     ├── pairwise_coulomb_friction() ← tangential friction from contacts
//!   │     └── per_body_air_drag()     ← aerodynamic drag per dynamic body
//!   └── pipeline.step()              ← Rapier solver
//! ```
//!
//! Each sub-system produces an `InteractionReport` with per-frame statistics
//! (body count, total force, peak values).  Reports are exposed via the existing
//! `CustomPhysicsReport` and can be queried through `world_get_custom_physics_report`.

use rapier3d::prelude::Vector;

use crate::rapier::ffi::{AirDragLaw, CustomPhysicsReport, vec3_from_rapier, vec3_to_rapier};
use crate::rapier::math::KahanVec3;

// ---------------------------------------------------------------------------
// Pairwise Newtonian gravity
// ---------------------------------------------------------------------------

/// Gravitational constant (N·m²/kg²).
pub const G: f64 = 6.67430e-11;

/// Minimum distance to avoid division-by-zero singularity.
const MIN_GRAVITY_DISTANCE: f64 = 0.01;

/// Apply Newtonian gravitational attraction between all body pairs.
///
/// Force on body i from body j:  Fᵢ = G · mᵢ · mⱼ / r² · r̂
///
/// Uses the O(n²) direct method; for large N (> 1000) prefer the Barnes-Hut
/// implementation in `astrophysics.rs`.
///
/// Bodies without explicit mass (e.g. no colliders, or mass set via
/// additional properties) are included; we use the body's reported mass
/// via `body.mass()`.
pub fn pairwise_gravity(
    world: &mut crate::rapier::world::PhysicsWorld,
    report: &mut CustomPhysicsReport,
) {
    // Collect (handle, mass, position) for all bodies with mass
    let bodies: Vec<(_, f64, Vector)> = world
        .bodies
        .iter()
        .filter(|(_, b)| b.is_dynamic())
        .map(|(h, b)| {
            let mass = b.mass();
            (h, if mass > 0.0 { mass } else { 0.0 }, b.translation())
        })
        .filter(|(_, m, _)| *m > 0.0)
        .collect();

    if bodies.len() < 2 {
        return;
    }

    let mut total_force = KahanVec3::default();
    let mut gravity_body_count = 0u32;

    // O(n²) pairwise — for large N, use Barnes-Hut from astrophysics.rs
    for i in 0..bodies.len() {
        let (hi, mi, pi) = (bodies[i].0, bodies[i].1, bodies[i].2);
        let mut net_force = Vector::ZERO;

        for (j, &(_, mj, pj)) in bodies.iter().enumerate() {
            if i == j {
                continue;
            }
            let offset = pj - pi;
            let dist_sq = offset.length_squared();
            let dist = dist_sq.sqrt().max(MIN_GRAVITY_DISTANCE);
            // F = G * mᵢ * mⱼ / r² * r̂  =  G * mᵢ * mⱼ / r³ * r
            let force_mag = G * mi * mj / (dist_sq * dist);
            net_force += offset * force_mag;
        }

        if net_force != Vector::ZERO
            && let Some(body) = world.bodies.get_mut(hi)
        {
            body.add_force(net_force, true);
            total_force.add(vec3_from_rapier(net_force));
            gravity_body_count += 1;
        }
    }

    report.body_count += bodies.len() as u32;
    report.total_external_force = total_force.value();
    report.external_force_body_count = gravity_body_count;
}

// ---------------------------------------------------------------------------
// Coulomb friction — tangential friction force events
// ---------------------------------------------------------------------------

/// Coulomb friction model parameters for body-body contacts.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoulombFrictionParams {
    pub static_coefficient: f64,
    pub dynamic_coefficient: f64,
    pub velocity_threshold: f64,
    pub enabled: bool,
}

impl Default for CoulombFrictionParams {
    fn default() -> Self {
        Self {
            static_coefficient: 0.6,
            dynamic_coefficient: 0.4,
            velocity_threshold: 0.01,
            enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-body air drag — Reynolds-number-aware drag force
// ---------------------------------------------------------------------------

/// Apply air drag to every dynamic body using the aerodynamic drag formula.
///
/// Drag force:  F_drag = -½ · ρ · v² · C_d · A_ref · v̂
///
/// For low Reynolds numbers (creeping flow), uses Stokes drag:
///   F_drag = -3π · μ · L_char · v
///
/// This is the per-body version; for surface-sample drag (wings, etc.)
/// use `aerodynamics.rs::aero_apply_surfaces`.
pub fn per_body_air_drag(
    world: &mut crate::rapier::world::PhysicsWorld,
    law: AirDragLaw,
    report: &mut CustomPhysicsReport,
) {
    if law.enabled.0 == 0 {
        return;
    }

    let fluid_velocity = vec3_to_rapier(law.fluid_velocity);
    let density = law.density;
    let viscosity = law.dynamic_viscosity;
    let char_len = law.characteristic_length;
    let ref_area = law.reference_area;
    let cd = law.drag_coefficient;
    let re_limit = law.reynolds_stokes_limit;

    let mut total_drag = KahanVec3::default();

    for (_, body) in world.bodies.iter_mut() {
        if !body.is_dynamic() {
            continue;
        }

        let relative_velocity = body.linvel() - fluid_velocity;
        let speed = relative_velocity.length();
        if speed <= 1.0e-12 {
            continue;
        }

        let reynolds = density * speed * char_len / viscosity;
        report.max_reynolds_number = report.max_reynolds_number.max(reynolds);

        let drag_magnitude = if reynolds <= re_limit {
            // Stokes regime: F = 3π · μ · L · v
            3.0 * std::f64::consts::PI * viscosity * char_len * speed
        } else {
            // Turbulent regime: F = ½ · ρ · v² · C_d · A
            0.5 * density * speed * speed * cd * ref_area
        };

        let force = -relative_velocity / speed * drag_magnitude;
        body.add_force(force, true);
        total_drag.add(vec3_from_rapier(force));
        report.drag_body_count += 1;
    }

    report.total_drag_force = total_drag.value();
}

// ---------------------------------------------------------------------------
// Unified interaction step — called from world_step
// ---------------------------------------------------------------------------

/// Facade-based wrapper: applies legacy unregistered body interactions through
/// the facade so the frame-log captures them with correct ForceLawType tags.
///
/// This is a temporary shim — once all force sources are registered as ForceLaw
/// impls, this function will be removed.
pub(crate) fn apply_body_interactions_with_facade(
    force_registry: &ForceRegistry,
    custom: &crate::rapier::events::CustomPhysicsState,
    facade: &mut crate::rapier::forces::ForceFacade<'_>,
) {
    use crate::rapier::forces::ForceLawType;

    // 1. Newtonian pairwise gravity
    if let Some(gravity_law) = custom.newton_gravity
        && gravity_law.enabled.0 != 0
    {
        let registered = !force_registry
            .find_by_type(ForceLawType::NewtonianGravity)
            .is_empty();
        if !registered {
            // Use a temporary ForceLaw instance
            let law = NewtonianGravityForceLaw {
                gravitational_constant: gravity_law.gravitational_constant,
                min_distance: gravity_law.min_distance,
                max_distance: gravity_law.max_distance,
                enabled: true,
            };
            law.apply(facade);
        }
    }

    // 2. Coulomb friction from contact data
    if let Some(friction_law) = custom.coulomb_friction
        && friction_law.enabled.0 != 0
    {
        let registered = !force_registry
            .find_by_type(ForceLawType::CoulombFriction)
            .is_empty();
        if !registered {
            apply_coulomb_friction_forces_with_facade(
                facade.narrow_phase,
                CoulombFrictionParams {
                    static_coefficient: friction_law.static_coefficient,
                    dynamic_coefficient: friction_law.dynamic_coefficient,
                    velocity_threshold: friction_law.velocity_threshold,
                    enabled: true,
                },
                facade,
            );
        }
    }

    // 3. Per-body air drag
    if let Some(drag_law) = custom.air_drag
        && drag_law.enabled.0 != 0
    {
        let registered = !force_registry
            .find_by_type(ForceLawType::AirDrag)
            .is_empty();
        if !registered {
            let law = AirDragForceLaw {
                fluid_velocity: vec3_to_rapier(drag_law.fluid_velocity),
                density: drag_law.density,
                dynamic_viscosity: drag_law.dynamic_viscosity,
                characteristic_length: drag_law.characteristic_length,
                reference_area: drag_law.reference_area,
                drag_coefficient: drag_law.drag_coefficient,
                reynolds_stokes_limit: drag_law.reynolds_stokes_limit,
                enabled: true,
            };
            law.apply(facade);
        }
    }
}

/// Coulomb friction via facade — writes typed force records.
fn apply_coulomb_friction_forces_with_facade(
    narrow_phase: &NarrowPhase,
    params: CoulombFrictionParams,
    facade: &mut crate::rapier::forces::ForceFacade<'_>,
) {
    if !params.enabled {
        return;
    }

    let static_mu = params.static_coefficient.max(0.0);
    let dynamic_mu = params.dynamic_coefficient.max(0.0);
    let threshold = params.velocity_threshold.max(0.0);

    let mut friction_work: Vec<(_, _, Vector)> = Vec::new();

    for contact_pair in narrow_phase.contact_pairs() {
        let ch1 = contact_pair.collider1;
        let ch2 = contact_pair.collider2;
        let Some(collider1) = facade.colliders.get(ch1) else {
            continue;
        };
        let Some(collider2) = facade.colliders.get(ch2) else {
            continue;
        };
        let Some(rb_handle1) = collider1.parent() else {
            continue;
        };
        let Some(rb_handle2) = collider2.parent() else {
            continue;
        };
        let Some(body1) = facade.bodies.get(rb_handle1) else {
            continue;
        };
        let Some(body2) = facade.bodies.get(rb_handle2) else {
            continue;
        };
        if !body1.is_dynamic() && !body2.is_dynamic() {
            continue;
        }

        for manifold in &contact_pair.manifolds {
            let normal = manifold.data.normal;
            for contact in &manifold.points {
                let p1_world = body1.position() * contact.local_p1;
                let p2_world = body2.position() * contact.local_p2;
                let point = (p1_world + p2_world) * 0.5;
                let r1 = point - body1.translation();
                let r2 = point - body2.translation();
                let v1 = body1.linvel() + body1.angvel().cross(r1);
                let v2 = body2.linvel() + body2.angvel().cross(r2);
                let rel_vel = v1 - v2;

                let normal_speed = rel_vel.dot(normal);
                let tangential_vel = rel_vel - normal * normal_speed;
                let tangential_speed = tangential_vel.length();

                if tangential_speed < 1.0e-12 {
                    continue;
                }

                let mu = if tangential_speed <= threshold {
                    static_mu
                } else {
                    dynamic_mu
                };

                let normal_force_mag = contact.data.impulse;
                let friction_mag = mu * normal_force_mag;
                let friction_force = -tangential_vel / tangential_speed * friction_mag;

                friction_work.push((rb_handle1, rb_handle2, friction_force));
            }
        }
    }

    use crate::rapier::forces::ForceLawType;
    for (rb1, rb2, force) in &friction_work {
        facade.add_force(*rb1, *force, ForceLawType::CoulombFriction);
        facade.add_force(*rb2, -*force, ForceLawType::CoulombFriction);
    }
}

// ---------------------------------------------------------------------------
// ForceLaw impls — registry-compatible wrappers
// ---------------------------------------------------------------------------

use crate::rapier::forces::{ForceFacade, ForceLaw, ForceLawType, ForceRegistry};
use rapier3d::prelude::{NarrowPhase, RigidBodyHandle};
use smallvec::SmallVec;

/// Newtonian pairwise gravity as a registered force law.
pub(crate) struct NewtonianGravityForceLaw {
    pub gravitational_constant: f64,
    pub min_distance: f64,
    pub max_distance: f64,
    pub enabled: bool,
}

impl ForceLaw for NewtonianGravityForceLaw {
    fn law_type(&self) -> ForceLawType {
        ForceLawType::NewtonianGravity
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn apply(&self, facade: &mut ForceFacade<'_>) {
        let g = self.gravitational_constant;
        let min_dist = self.min_distance;
        let max_dist_sq = if self.max_distance > 0.0 {
            self.max_distance * self.max_distance
        } else {
            f64::MAX
        };
        let source = self.law_type();

        // Collect only dynamic bodies with mass > 0
        let body_data: SmallVec<[(RigidBodyHandle, f64, Vector); 64]> = facade
            .bodies
            .iter()
            .filter(|(_, b)| b.is_dynamic())
            .filter_map(|(h, b)| {
                let mass = b.mass();
                if mass > 0.0 {
                    Some((h, mass, b.translation()))
                } else {
                    None
                }
            })
            .collect();

        if body_data.len() < 2 {
            return;
        }

        // Newton III: compute F_ij = -F_ji, only upper triangle
        // Use SmallVec for force accumulator (stack-allocated for ≤64 bodies)
        let n = body_data.len();
        let mut forces: SmallVec<[(RigidBodyHandle, Vector); 64]> = SmallVec::new();
        for (h, _, _) in &body_data {
            forces.push((*h, Vector::ZERO));
        }

        for i in 0..n {
            let (_hi, mi, pi) = (body_data[i].0, body_data[i].1, body_data[i].2);
            for j in (i + 1)..n {
                let (_hj, mj, pj) = (body_data[j].0, body_data[j].1, body_data[j].2);
                let offset = pj - pi;
                let dist_sq = offset.length_squared();
                if dist_sq > max_dist_sq {
                    continue;
                }
                let dist = dist_sq.sqrt().max(min_dist);
                let force_mag = g * mi * mj / (dist_sq * dist);
                let f_ij = offset * force_mag;
                forces[i].1 += f_ij;
                forces[j].1 -= f_ij;
            }
        }

        // Single pass: apply accumulated forces
        for (handle, force) in &forces {
            if *force != Vector::ZERO {
                facade.add_force(*handle, *force, source);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn ForceLaw> {
        Box::new(Self {
            gravitational_constant: self.gravitational_constant,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            enabled: self.enabled,
        })
    }
}

/// Air drag as a registered force law.
pub(crate) struct AirDragForceLaw {
    pub fluid_velocity: Vector,
    pub density: f64,
    pub dynamic_viscosity: f64,
    pub characteristic_length: f64,
    pub reference_area: f64,
    pub drag_coefficient: f64,
    pub reynolds_stokes_limit: f64,
    pub enabled: bool,
}

impl ForceLaw for AirDragForceLaw {
    fn law_type(&self) -> ForceLawType {
        ForceLawType::AirDrag
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn apply(&self, facade: &mut ForceFacade<'_>) {
        let source = self.law_type();

        // Phase 1: compute forces + max reynolds (immutable body read)
        let mut work: SmallVec<[(RigidBodyHandle, Vector); 64]> = SmallVec::new();
        let mut max_re = 0.0f64;
        for (handle, body) in facade.bodies.iter() {
            if !body.is_dynamic() {
                continue;
            }

            let relative_velocity = body.linvel() - self.fluid_velocity;
            let speed = relative_velocity.length();
            if speed <= 1.0e-12 {
                continue;
            }

            let reynolds =
                self.density * speed * self.characteristic_length / self.dynamic_viscosity;
            max_re = max_re.max(reynolds);

            let drag_magnitude = if reynolds <= self.reynolds_stokes_limit {
                3.0 * std::f64::consts::PI
                    * self.dynamic_viscosity
                    * self.characteristic_length
                    * speed
            } else {
                0.5 * self.density * speed * speed * self.drag_coefficient * self.reference_area
            };

            let force = -relative_velocity / speed * drag_magnitude;
            work.push((handle, force));
        }

        // Phase 2: update facade + apply forces
        facade.update_reynolds(max_re);
        for (handle, force) in work {
            facade.add_force(handle, force, source);
        }
    }

    fn clone_box(&self) -> Box<dyn ForceLaw> {
        Box::new(Self {
            fluid_velocity: self.fluid_velocity,
            density: self.density,
            dynamic_viscosity: self.dynamic_viscosity,
            characteristic_length: self.characteristic_length,
            reference_area: self.reference_area,
            drag_coefficient: self.drag_coefficient,
            reynolds_stokes_limit: self.reynolds_stokes_limit,
            enabled: self.enabled,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
