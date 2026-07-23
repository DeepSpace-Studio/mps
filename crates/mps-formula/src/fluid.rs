//! Fluid dynamics — AABB buoyancy/drag, SPH, Navier-Stokes, and Bernoulli formulas.
//!
//! Pure computation only — no access to `WorldHandle`, `RigidBody`, or Rapier state.

use crate::ffi::{
    BernoulliReport, FluidForceReport, FluidVolume, NavierStokesReport,
    SphForceReport, SphParticle, Vec3,
    vec3_finite, vec3_to_rapier, vec3_from_rapier,
};
use crate::math::{KahanSum, KahanVec3, finite_non_negative, finite_positive, mul_add};
use rapier3d::prelude::Vector;

const EPSILON: f64 = 1.0e-12;
const PI: f64 = std::f64::consts::PI;

fn fluid_valid(fluid: &FluidVolume) -> bool {
    vec3_finite(fluid.center)
        && vec3_finite(fluid.half_extents)
        && vec3_finite(fluid.flow_velocity)
        && vec3_finite(fluid.gravity)
        && fluid.half_extents.x >= 0.0
        && fluid.half_extents.y >= 0.0
        && fluid.half_extents.z >= 0.0
        && fluid.density.is_finite()
        && fluid.density >= 0.0
        && fluid.linear_drag.is_finite()
        && fluid.linear_drag >= 0.0
        && fluid.quadratic_drag.is_finite()
        && fluid.quadratic_drag >= 0.0
        && fluid.angular_drag.is_finite()
        && fluid.angular_drag >= 0.0
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn submerged_fraction_aabb(body_center: Vec3, body_half_extents: Vec3, fluid: &FluidVolume) -> f64 {
    let body_min_y = body_center.y - body_half_extents.y;
    let body_max_y = body_center.y + body_half_extents.y;
    let fluid_min_y = fluid.center.y - fluid.half_extents.y;
    let fluid_max_y = fluid.center.y + fluid.half_extents.y;
    let body_height = body_max_y - body_min_y;
    if body_height <= 0.0 {
        return 0.0;
    }
    let overlap = body_max_y.min(fluid_max_y) - body_min_y.max(fluid_min_y);
    clamp01(overlap / body_height)
}

/// Compute AABB-based fluid buoyancy and drag forces.
pub fn compute_fluid_forces(
    fluid: FluidVolume,
    body_center: Vec3,
    body_half_extents: Vec3,
    body_volume: f64,
    body_linvel: Vec3,
    body_angvel: Vec3,
) -> Option<FluidForceReport> {
    if !fluid_valid(&fluid)
        || !vec3_finite(body_center)
        || !vec3_finite(body_half_extents)
        || !vec3_finite(body_linvel)
        || !vec3_finite(body_angvel)
        || !finite_positive(body_volume)
        || body_half_extents.x < 0.0
        || body_half_extents.y < 0.0
        || body_half_extents.z < 0.0
    {
        return None;
    }

    let submerged_fraction = submerged_fraction_aabb(body_center, body_half_extents, &fluid);
    if submerged_fraction <= 0.0 || fluid.density <= 0.0 {
        return Some(FluidForceReport::default());
    }

    let displaced_volume = body_volume * submerged_fraction;
    let gravity = vec3_to_rapier(fluid.gravity);
    let relative_velocity = vec3_to_rapier(fluid.flow_velocity) - vec3_to_rapier(body_linvel);
    let speed = relative_velocity.length_squared().sqrt();
    let drag_force = if speed > 1.0e-12 {
        let drag_scale =
            submerged_fraction * (fluid.linear_drag * speed + fluid.quadratic_drag * speed * speed);
        relative_velocity / speed * drag_scale
    } else {
        rapier3d::prelude::Vector::ZERO
    };
    let buoyancy_force = -gravity * (fluid.density * displaced_volume);
    let angular_damping_torque =
        -vec3_to_rapier(body_angvel) * (fluid.angular_drag * submerged_fraction);
    let total_force = buoyancy_force + drag_force;

    Some(FluidForceReport {
        buoyancy_force: vec3_from_rapier(buoyancy_force),
        drag_force: vec3_from_rapier(drag_force),
        angular_damping_torque: vec3_from_rapier(angular_damping_torque),
        total_force: vec3_from_rapier(total_force),
        total_torque: vec3_from_rapier(angular_damping_torque),
        submerged_fraction,
        displaced_volume,
    })
}

/// Simplified Navier-Stokes step.
pub fn navier_stokes_simplified_step(
    velocity: Vec3,
    advection: Vec3,
    pressure_gradient: Vec3,
    laplacian_velocity: Vec3,
    external_acceleration: Vec3,
    density: f64,
    kinematic_viscosity: f64,
    dt: f64,
) -> Option<NavierStokesReport> {
    if !vec3_finite(velocity)
        || !vec3_finite(advection)
        || !vec3_finite(pressure_gradient)
        || !vec3_finite(laplacian_velocity)
        || !vec3_finite(external_acceleration)
        || !finite_positive(density)
        || !finite_non_negative(kinematic_viscosity)
        || !finite_non_negative(dt)
    {
        return None;
    }

    let adv = vec3_to_rapier(advection);
    let pressure_acceleration = -vec3_to_rapier(pressure_gradient) / density;
    let viscosity_acceleration = vec3_to_rapier(laplacian_velocity) * kinematic_viscosity;
    let external = vec3_to_rapier(external_acceleration);
    let total = -adv + pressure_acceleration + viscosity_acceleration + external;
    let next_velocity = vec3_to_rapier(velocity) + total * dt;

    Some(NavierStokesReport {
        advection,
        pressure_acceleration: vec3_from_rapier(pressure_acceleration),
        viscosity_acceleration: vec3_from_rapier(viscosity_acceleration),
        external_acceleration,
        total_acceleration: vec3_from_rapier(total),
        next_velocity: vec3_from_rapier(next_velocity),
    })
}

// ---------------------------------------------------------------------------
// SPH kernels
// ---------------------------------------------------------------------------

/// SPH Poly6 kernel.
pub fn sph_poly6_kernel(distance: f64, smoothing_radius: f64) -> f64 {
    if !finite_non_negative(distance) || !finite_positive(smoothing_radius) {
        return f64::NAN;
    }
    if distance >= smoothing_radius {
        return 0.0;
    }
    let h2 = smoothing_radius * smoothing_radius;
    let r2 = distance * distance;
    let diff = -mul_add(r2, 1.0_f64, -h2);
    if diff <= 0.0 {
        return 0.0;
    }
    315.0 / (64.0 * PI * smoothing_radius.powi(9)) * diff.powi(3)
}

/// SPH Spiky gradient.
pub fn sph_spiky_gradient(offset: Vec3, smoothing_radius: f64) -> Option<Vec3> {
    if !vec3_finite(offset) || !finite_positive(smoothing_radius) {
        return None;
    }
    let r = vec3_to_rapier(offset);
    let distance = r.length();
    let gradient = if distance <= EPSILON || distance >= smoothing_radius {
        rapier3d::prelude::Vector::ZERO
    } else {
        let diff = mul_add(-1.0_f64, distance, smoothing_radius);
        -r / distance * (45.0 / (PI * smoothing_radius.powi(6)) * diff * diff)
    };
    Some(vec3_from_rapier(gradient))
}

/// SPH viscosity Laplacian.
pub fn sph_viscosity_laplacian(distance: f64, smoothing_radius: f64) -> f64 {
    if !finite_non_negative(distance) || !finite_positive(smoothing_radius) {
        return f64::NAN;
    }
    if distance >= smoothing_radius {
        return 0.0;
    }
    45.0 / (PI * smoothing_radius.powi(6)) * (smoothing_radius - distance)
}

/// Estimate density at a position using SPH particles.
pub fn sph_estimate_density(
    position: Vec3,
    particles: &[SphParticle],
    smoothing_radius: f64,
) -> Option<f64> {
    if !vec3_finite(position) || !finite_positive(smoothing_radius) {
        return None;
    }
    let p = vec3_to_rapier(position);
    let mut density = KahanSum::default();
    for particle in particles {
        if !vec3_finite(particle.position) || !particle.mass.is_finite() || particle.mass < 0.0 {
            return None;
        }
        density.add(
            particle.mass
                * sph_poly6_kernel(
                    (p - vec3_to_rapier(particle.position)).length(),
                    smoothing_radius,
                ),
        );
    }
    Some(density.value())
}

/// Estimate SPH forces on a particle from its neighbors.
pub fn sph_estimate_forces(
    particle: SphParticle,
    particles: &[SphParticle],
    smoothing_radius: f64,
    gas_constant: f64,
    rest_density: f64,
    viscosity: f64,
    surface_tension: f64,
) -> Option<SphForceReport> {
    if !vec3_finite(particle.position)
        || !vec3_finite(particle.velocity)
        || !finite_positive(particle.mass)
        || !finite_positive(smoothing_radius)
        || !gas_constant.is_finite()
        || !finite_positive(rest_density)
        || !finite_non_negative(viscosity)
        || !finite_non_negative(surface_tension)
    {
        return None;
    }
    let p = vec3_to_rapier(particle.position);
    let v = vec3_to_rapier(particle.velocity);
    let density = if particle.density > EPSILON {
        particle.density
    } else {
        let mut density = KahanSum::default();
        for neighbor in particles {
            density.add(
                neighbor.mass
                    * sph_poly6_kernel(
                        (p - vec3_to_rapier(neighbor.position)).length(),
                        smoothing_radius,
                    ),
            );
        }
        density.value().max(rest_density)
    };
    let pressure = if particle.pressure.is_finite() {
        particle.pressure
    } else {
        gas_constant * (density - rest_density)
    };
    let mut pressure_force = KahanVec3::default();
    let mut viscosity_force = KahanVec3::default();
    let mut color_gradient = KahanVec3::default();

    for neighbor in particles {
        if !vec3_finite(neighbor.position)
            || !vec3_finite(neighbor.velocity)
            || !finite_positive(neighbor.mass)
            || neighbor.density < 0.0
        {
            return None;
        }
        let offset = p - vec3_to_rapier(neighbor.position);
        let distance = offset.length();
        if distance <= EPSILON || distance >= smoothing_radius {
            continue;
        }
        let neighbor_density = neighbor.density.max(rest_density);
        let neighbor_pressure = if neighbor.pressure.is_finite() {
            neighbor.pressure
        } else {
            gas_constant * (neighbor_density - rest_density)
        };
        let diff = mul_add(-1.0_f64, distance, smoothing_radius);
        let gradient = -offset / distance * (45.0 / (PI * smoothing_radius.powi(6)) * diff * diff);
        pressure_force.add(vec3_from_rapier(
            -neighbor.mass * ((pressure + neighbor_pressure) / (2.0 * neighbor_density)) * gradient,
        ));
        viscosity_force.add(vec3_from_rapier(
            viscosity * neighbor.mass * (vec3_to_rapier(neighbor.velocity) - v)
                / neighbor_density
                * sph_viscosity_laplacian(distance, smoothing_radius),
        ));
        color_gradient.add(vec3_from_rapier(neighbor.mass / neighbor_density * gradient));
    }

    let color_gradient_vec = vec3_to_rapier(color_gradient.value());
    let surface_tension_force = if color_gradient_vec.length() > EPSILON {
        -color_gradient_vec / color_gradient_vec.length()
            * surface_tension
            * color_gradient_vec.length()
    } else {
        rapier3d::prelude::Vector::ZERO
    };
    let total_force = vec3_to_rapier(pressure_force.value())
        + vec3_to_rapier(viscosity_force.value())
        + surface_tension_force;

    Some(SphForceReport {
        density,
        pressure,
        pressure_force: pressure_force.value(),
        viscosity_force: viscosity_force.value(),
        surface_tension_force: vec3_from_rapier(surface_tension_force),
        total_force: vec3_from_rapier(total_force),
    })
}

// ---------------------------------------------------------------------------
// Bernoulli
// ---------------------------------------------------------------------------

/// Bernoulli static pressure.
pub fn bernoulli_pressure(
    total_pressure: f64,
    density: f64,
    velocity: f64,
    gravity: f64,
    elevation: f64,
) -> f64 {
    if !total_pressure.is_finite()
        || !finite_positive(density)
        || !finite_non_negative(velocity)
        || !gravity.is_finite()
        || !elevation.is_finite()
    {
        return f64::NAN;
    }
    total_pressure - 0.5 * density * velocity * velocity - density * gravity * elevation
}

/// Bernoulli report.
pub fn bernoulli_report(
    pressure: f64,
    density: f64,
    velocity: f64,
    gravity: f64,
    elevation: f64,
) -> Option<BernoulliReport> {
    if !pressure.is_finite()
        || !finite_positive(density)
        || !finite_non_negative(velocity)
        || !gravity.is_finite()
        || !elevation.is_finite()
    {
        return None;
    }
    let dynamic_pressure = 0.5 * density * velocity * velocity;
    let total_pressure = pressure + dynamic_pressure + density * gravity * elevation;
    Some(BernoulliReport {
        pressure,
        velocity,
        elevation,
        total_head: total_pressure / (density * gravity),
        dynamic_pressure,
    })
}
// ---------------------------------------------------------------------------
// Reynolds number
// ---------------------------------------------------------------------------

/// Reynolds number: Re = rho * v * L / mu
pub fn re_n(density: f64, velocity: f64, char_length: f64, viscosity: f64) -> Option<f64> {
    if !density.is_finite() || density < 0.0 || !velocity.is_finite() || velocity < 0.0 || !char_length.is_finite() || char_length <= 0.0 || !viscosity.is_finite() || viscosity <= 0.0 { return None; }
    Some(density * velocity * char_length / viscosity)
}

/// Flow regime based on Reynolds number: 0=laminar, 1=transition, 2=turbulent
pub fn flow_regime(reynolds: f64) -> u8 {
    if reynolds < 2000.0 { 0 }
    else if reynolds < 4000.0 { 1 }
    else { 2 }
}

/// Friction factor for pipe flow (Darcy-Weisbach).
/// Laminar: f = 64/Re. Turbulent: Colebrook equation (iterative).
pub fn darcy_friction_factor(reynolds: f64, relative_roughness: f64) -> Option<f64> {
    if !reynolds.is_finite() || reynolds <= 0.0 || !relative_roughness.is_finite() || relative_roughness < 0.0 { return None; }
    if reynolds < 2000.0 {
        return Some(64.0 / reynolds);
    }
    // Colebrook-White: 1/sqrt(f) = -2*log10(eps/(3.7*D) + 2.51/(Re*sqrt(f)))
    let eps = relative_roughness;
    let mut f: f64 = 0.02; // initial guess
    for _ in 0..30 {
        let f_sqrt = f.sqrt();
        let rhs = -2.0 * (eps / 3.7 + 2.51 / (reynolds * f_sqrt)).log10();
        let f_new = 1.0 / (rhs * rhs);
        if (f_new - f).abs() < 1.0e-10 { f = f_new; break; }
        f = f_new;
    }
    Some(f)
}

// ---------------------------------------------------------------------------
// k-epsilon turbulence model
// ---------------------------------------------------------------------------

/// Standard k-epsilon turbulence model: production of TKE.
/// P_k = nut * S^2 where S = sqrt(2 * S_ij * S_ij)
pub fn k_epsilon_production(eddy_viscosity: f64, strain_rate_magnitude: f64) -> Option<f64> {
    if !eddy_viscosity.is_finite() || eddy_viscosity < 0.0 || !strain_rate_magnitude.is_finite() || strain_rate_magnitude < 0.0 { return None; }
    Some(eddy_viscosity * strain_rate_magnitude * strain_rate_magnitude)
}

/// Eddy viscosity from k-epsilon: nut = C_mu * k^2 / epsilon
pub fn k_epsilon_eddy_viscosity(tke: f64, dissipation: f64, c_mu: f64) -> Option<f64> {
    if !tke.is_finite() || tke < 0.0 || !dissipation.is_finite() || dissipation <= 0.0 || !c_mu.is_finite() || c_mu <= 0.0 { return None; }
    Some(c_mu * tke * tke / dissipation)
}

/// k-epsilon: source term for k transport equation.
/// dk/dt = P_k - epsilon + diffusion
pub fn k_equation_source(production: f64, dissipation: f64) -> Option<f64> {
    if !production.is_finite() || !dissipation.is_finite() || dissipation < 0.0 { return None; }
    Some(production - dissipation)
}

/// k-epsilon: source term for epsilon transport equation.
/// depsilon/dt = C_eps1 * P_k * epsilon/k - C_eps2 * epsilon^2/k + diffusion
pub fn epsilon_equation_source(production: f64, tke: f64, dissipation: f64, c_eps1: f64, c_eps2: f64) -> Option<f64> {
    if !finite_5(production, tke, dissipation, c_eps1, c_eps2) || tke <= 0.0 || dissipation < 0.0 || c_eps1 <= 0.0 || c_eps2 <= 0.0 { return None; }
    Some(c_eps1 * production * dissipation / tke - c_eps2 * dissipation * dissipation / tke)
}

/// Standard k-epsilon model constants.
pub fn k_epsilon_constants() -> (f64, f64, f64, f64, f64) {
    (0.09, 1.44, 1.92, 1.0, 1.3) // C_mu, C_eps1, C_eps2, sigma_k, sigma_eps
}

/// Characteristic turbulent length scale: L = C_mu^0.75 * k^1.5 / epsilon
pub fn turbulent_length_scale(tke: f64, dissipation: f64) -> Option<f64> {
    if !tke.is_finite() || tke < 0.0 || !dissipation.is_finite() || dissipation <= 0.0 { return None; }
    let cmu: f64 = 0.09;
    Some(cmu.powf(0.75) * tke.powf(1.5) / dissipation)
}

/// Turbulent Reynolds number: Re_t = k^2 / (nu * epsilon)
pub fn turbulent_reynolds(tke: f64, dissipation: f64, kinematic_viscosity: f64) -> Option<f64> {
    if !tke.is_finite() || tke < 0.0 || !dissipation.is_finite() || dissipation <= 0.0 || !kinematic_viscosity.is_finite() || kinematic_viscosity <= 0.0 { return None; }
    Some(tke * tke / (kinematic_viscosity * dissipation))
}

fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
    a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
}

// ---------------------------------------------------------------------------
// Compressible flow
// ---------------------------------------------------------------------------

/// Isentropic pressure ratio: P/P₀ = (1 + (γ-1)/2 · M²)^(-γ/(γ-1))
pub fn isentropic_pressure_ratio(mach: f64, gamma: f64) -> Option<f64> {
    if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some((1.0 + (gamma - 1.0) / 2.0 * mach * mach).powf(-gamma / (gamma - 1.0)))
}

/// Isentropic density ratio: ρ/ρ₀ = (1 + (γ-1)/2 · M²)^(-1/(γ-1))
pub fn isentropic_density_ratio(mach: f64, gamma: f64) -> Option<f64> {
    if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some((1.0 + (gamma - 1.0) / 2.0 * mach * mach).powf(-1.0 / (gamma - 1.0)))
}

/// Isentropic temperature ratio: T/T₀ = 1/(1 + (γ-1)/2 · M²)
pub fn isentropic_temperature_ratio(mach: f64, gamma: f64) -> Option<f64> {
    if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some(1.0 / (1.0 + (gamma - 1.0) / 2.0 * mach * mach))
}

/// Area-Mach relation for isentropic flow: A/A* = (1/M) · ((2/(γ+1))·(1+(γ-1)·M²/2))^((γ+1)/(2(γ-1)))
pub fn area_mach_ratio(mach: f64, gamma: f64) -> Option<f64> {
    if !mach.is_finite() || mach < 0.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    let term = (1.0 + (gamma - 1.0) / 2.0 * mach * mach) * 2.0 / (gamma + 1.0);
    Some(1.0 / mach * term.powf((gamma + 1.0) / (2.0 * (gamma - 1.0))))
}

/// Normal shock wave: downstream Mach number: M₂² = ((γ-1)M₁² + 2) / (2γ·M₁² - (γ-1))
pub fn normal_shock_downstream_mach(upstream_mach: f64, gamma: f64) -> Option<f64> {
    if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some(((gamma - 1.0) * upstream_mach * upstream_mach + 2.0) / (2.0 * gamma * upstream_mach * upstream_mach - (gamma - 1.0)))
}

/// Normal shock pressure ratio: P₂/P₁ = 1 + 2γ/(γ+1) · (M₁² - 1)
pub fn normal_shock_pressure_ratio(upstream_mach: f64, gamma: f64) -> Option<f64> {
    if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some(1.0 + 2.0 * gamma / (gamma + 1.0) * (upstream_mach * upstream_mach - 1.0))
}

/// Normal shock density ratio: ρ₂/ρ₁ = (γ+1)·M₁² / ((γ-1)·M₁² + 2)
pub fn normal_shock_density_ratio(upstream_mach: f64, gamma: f64) -> Option<f64> {
    if !upstream_mach.is_finite() || upstream_mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    Some((gamma + 1.0) * upstream_mach * upstream_mach / ((gamma - 1.0) * upstream_mach * upstream_mach + 2.0))
}

/// Prandtl-Meyer expansion angle: ν(M) = ((γ+1)/(γ-1))^(1/2) · atan(((γ-1)/(γ+1)·(M²-1))^(1/2)) - atan((M²-1)^(1/2))
pub fn prandtl_meyer_angle(mach: f64, gamma: f64) -> Option<f64> {
    if !mach.is_finite() || mach < 1.0 || !gamma.is_finite() || gamma <= 0.0 { return None; }
    let sqrt = ((gamma - 1.0) / (gamma + 1.0) * (mach * mach - 1.0)).sqrt();
    Some(((gamma + 1.0) / (gamma - 1.0)).sqrt() * sqrt.atan() - (mach * mach - 1.0).sqrt().atan())
}

// ---------------------------------------------------------------------------
// Boundary layer
// ---------------------------------------------------------------------------

/// Blasius boundary layer thickness: δ ≈ 5.0 · x / Re_x^(1/2)
pub fn blasius_thickness(x: f64, re_x: f64) -> Option<f64> {
    if !x.is_finite() || x <= 0.0 || !re_x.is_finite() || re_x <= 0.0 { return None; }
    Some(5.0 * x / re_x.sqrt())
}

/// Blasius displacement thickness: δ* ≈ 1.7208 · x / Re_x^(1/2)
pub fn blasius_displacement_thickness(x: f64, re_x: f64) -> Option<f64> {
    if !x.is_finite() || x <= 0.0 || !re_x.is_finite() || re_x <= 0.0 { return None; }
    Some(1.7208 * x / re_x.sqrt())
}

/// Blasius momentum thickness: θ ≈ 0.664 · x / Re_x^(1/2)
pub fn blasius_momentum_thickness(x: f64, re_x: f64) -> Option<f64> {
    if !x.is_finite() || x <= 0.0 || !re_x.is_finite() || re_x <= 0.0 { return None; }
    Some(0.664 * x / re_x.sqrt())
}

/// Flat plate skin friction coefficient (laminar): C_f = 0.664 / Re_x^(1/2)
pub fn laminar_skin_friction(re_x: f64) -> Option<f64> {
    if !re_x.is_finite() || re_x <= 0.0 { return None; }
    Some(0.664 / re_x.sqrt())
}

/// Flat plate skin friction coefficient (turbulent, 1/7th power law): C_f = 0.027 / Re_x^(1/7)
pub fn turbulent_skin_friction(re_x: f64) -> Option<f64> {
    if !re_x.is_finite() || re_x <= 0.0 { return None; }
    Some(0.027 / re_x.powf(1.0 / 7.0))
}

// ---------------------------------------------------------------------------
// Potential flow
// ---------------------------------------------------------------------------

/// 2D point source velocity potential: φ = Q/(2π) · ln(r)
pub fn source_potential_2d(strength: f64, r: f64) -> Option<f64> {
    if !strength.is_finite() || !r.is_finite() || r <= 0.0 { return None; }
    Some(strength / (2.0 * std::f64::consts::PI) * r.ln())
}

/// 2D doublet stream function: ψ = -κ · sin(θ) / (2π · r)
pub fn doublet_stream_function_2d(strength: f64, r: f64, theta: f64) -> Option<f64> {
    if !strength.is_finite() || !r.is_finite() || r <= 0.0 || !theta.is_finite() { return None; }
    Some(-strength * theta.sin() / (2.0 * std::f64::consts::PI * r))
}

// ---------------------------------------------------------------------------
// Non-Newtonian fluids
// ---------------------------------------------------------------------------

/// Power-law viscosity: μ_eff = K · γ̇^(n-1)
pub fn power_law_viscosity(consistency: f64, shear_rate: f64, flow_index: f64) -> Option<f64> {
    if !consistency.is_finite() || consistency <= 0.0 || !shear_rate.is_finite() || shear_rate < 0.0 || !flow_index.is_finite() { return None; }
    if shear_rate <= 1.0e-12 && flow_index < 1.0 { return None; }
    Some(consistency * shear_rate.powf(flow_index - 1.0))
}

/// Bingham plastic: τ = τ_y + μ_p · γ̇
pub fn bingham_stress(yield_stress: f64, plastic_viscosity: f64, shear_rate: f64) -> Option<f64> {
    if !yield_stress.is_finite() || yield_stress < 0.0 || !plastic_viscosity.is_finite() || plastic_viscosity < 0.0 || !shear_rate.is_finite() || shear_rate < 0.0 { return None; }
    Some(yield_stress + plastic_viscosity * shear_rate)
}

// ---------------------------------------------------------------------------
// Kelvin-Helmholtz and Rayleigh-Taylor instabilities
// ---------------------------------------------------------------------------

/// KH instability growth rate for two inviscid fluids with velocity shear.
pub fn kelvin_helmholtz_growth_rate(k: f64, rho1: f64, rho2: f64, v1: f64, v2: f64) -> Option<f64> {
    if !finite_5(k, rho1, rho2, v1, v2) || k <= 0.0 || rho1 <= 0.0 || rho2 <= 0.0 { return None; }
    let dv = v1 - v2;
    Some(k * (rho1 * rho2).sqrt() * dv.abs() / (rho1 + rho2))
}

/// RT instability growth rate: ω = sqrt(At · g · k)
pub fn rayleigh_taylor_growth_rate(atuood_number: f64, gravity: f64, k: f64) -> Option<f64> {
    if !atuood_number.is_finite() || atuood_number < 0.0 || !gravity.is_finite() || gravity < 0.0 || !k.is_finite() || k <= 0.0 { return None; }
    Some((atuood_number * gravity * k).sqrt())
}

/// Atwood number: At = (ρ₂ - ρ₁)/(ρ₂ + ρ₁)
pub fn atwood_number(density_heavy: f64, density_light: f64) -> Option<f64> {
    if !density_heavy.is_finite() || density_heavy < 0.0 || !density_light.is_finite() || density_light < 0.0 { return None; }
    let sum = density_heavy + density_light;
    if sum <= 0.0 { return None; }
    Some((density_heavy - density_light) / sum)
}

// ---------------------------------------------------------------------------
// Minor losses (pipe flow)
// ---------------------------------------------------------------------------

/// Minor loss pressure drop: ΔP = K · ½ρV²
pub fn minor_loss_pressure_drop(loss_coefficient: f64, density: f64, velocity: f64) -> Option<f64> {
    if !loss_coefficient.is_finite() || loss_coefficient < 0.0 || !density.is_finite() || density < 0.0 || !velocity.is_finite() || velocity < 0.0 { return None; }
    Some(loss_coefficient * 0.5 * density * velocity * velocity)
}

// ---------------------------------------------------------------------------
// Water hammer (Joukowsky)
// ---------------------------------------------------------------------------

/// Joukowsky pressure surge: ΔP = ρ · c · ΔV
pub fn water_hammer_pressure_surge(density: f64, wave_speed: f64, velocity_change: f64) -> Option<f64> {
    if !density.is_finite() || density <= 0.0 || !wave_speed.is_finite() || wave_speed <= 0.0 || !velocity_change.is_finite() { return None; }
    Some(density * wave_speed * velocity_change.abs())
}