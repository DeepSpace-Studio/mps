use std::slice;

use std::f64::consts::PI;
use rapier3d::prelude::Vector;

use crate::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, set_error,
};
use crate::ffi::{
    Bool, NBodyForceReport, NBodyParticle, NBodySolverParams, OrbitalResonanceReport,
    RelativisticOrbitReport, RocheLimitReport, Vec3, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};
use crate::math::mul_add;

use crate::math::{finite_non_negative, finite_positive};

const MAX_NBODY_PARTICLES: u32 = 100_000;
const SPEED_OF_LIGHT: f64 = 299_792_458.0;
const EPSILON: f64 = 1.0e-12;

#[derive(Clone, Copy)]
struct Bounds {
    center: Vector,
    half_size: f64,
}

#[derive(Clone)]
struct QuadNode {
    bounds: Bounds,
    mass: f64,
    center_of_mass: Vector,
    particle: Option<usize>,
    children: [Option<usize>; 4],
}

fn params_valid(params: NBodySolverParams) -> bool {
    finite_positive(params.gravitational_constant)
        && finite_non_negative(params.softening)
        && finite_positive(params.opening_angle)
}

fn particle_valid(particle: NBodyParticle) -> bool {
    vec3_finite(particle.position)
        && vec3_finite(particle.velocity)
        && finite_non_negative(particle.mass)
}

fn child_index(bounds: Bounds, position: Vector) -> usize {
    usize::from(position.x >= bounds.center.x) + 2 * usize::from(position.y >= bounds.center.y)
}

fn child_bounds(bounds: Bounds, index: usize) -> Bounds {
    let quarter = bounds.half_size * 0.5;
    Bounds {
        center: Vector::new(
            bounds.center.x + if index & 1 == 0 { -quarter } else { quarter },
            bounds.center.y + if index & 2 == 0 { -quarter } else { quarter },
            0.0,
        ),
        half_size: quarter,
    }
}

fn update_mass(node: &mut QuadNode, particle: NBodyParticle) {
    if particle.mass <= 0.0 {
        return;
    }
    let old_mass = node.mass;
    node.mass += particle.mass;
    node.center_of_mass = (node.center_of_mass * old_mass
        + vec3_to_rapier(particle.position) * particle.mass)
        / node.mass;
}

fn insert_particle(
    nodes: &mut Vec<QuadNode>,
    node_index: usize,
    particle_index: usize,
    particles: &[NBodyParticle],
) {
    update_mass(&mut nodes[node_index], particles[particle_index]);
    if nodes[node_index].bounds.half_size <= 1.0e-9 {
        nodes[node_index].particle = Some(particle_index);
        return;
    }
    if nodes[node_index].particle.is_none()
        && nodes[node_index].children.iter().all(Option::is_none)
    {
        nodes[node_index].particle = Some(particle_index);
        return;
    }
    if let Some(existing) = nodes[node_index].particle.take() {
        let child = child_index(
            nodes[node_index].bounds,
            vec3_to_rapier(particles[existing].position),
        );
        let child_node = nodes.len();
        nodes.push(QuadNode {
            bounds: child_bounds(nodes[node_index].bounds, child),
            mass: 0.0,
            center_of_mass: Vector::ZERO,
            particle: None,
            children: [None; 4],
        });
        nodes[node_index].children[child] = Some(child_node);
        insert_particle(nodes, child_node, existing, particles);
    }
    let child = child_index(
        nodes[node_index].bounds,
        vec3_to_rapier(particles[particle_index].position),
    );
    let child_node = if let Some(child_node) = nodes[node_index].children[child] {
        child_node
    } else {
        let child_node = nodes.len();
        nodes.push(QuadNode {
            bounds: child_bounds(nodes[node_index].bounds, child),
            mass: 0.0,
            center_of_mass: Vector::ZERO,
            particle: None,
            children: [None; 4],
        });
        nodes[node_index].children[child] = Some(child_node);
        child_node
    };
    insert_particle(nodes, child_node, particle_index, particles);
}

fn acceleration_from_mass(
    position: Vector,
    center: Vector,
    mass: f64,
    params: NBodySolverParams,
) -> Vector {
    if mass <= 0.0 {
        return Vector::ZERO;
    }
    let offset = center - position;
    // Use mul_add for softened distance: r² + ε² with single rounding
    let r2 = mul_add(params.softening, params.softening, offset.length_squared());
    if r2 <= EPSILON {
        return Vector::ZERO;
    }
    // r2 * sqrt(r2) = r³; compute as r2.sqrt() * r2 to avoid overflow
    let r3 = r2.sqrt() * r2;
    offset * (params.gravitational_constant * mass / r3)
}

fn bh_acceleration(
    nodes: &[QuadNode],
    node_index: usize,
    particle_index: usize,
    particles: &[NBodyParticle],
    params: NBodySolverParams,
    approximate_count: &mut u32,
    direct_count: &mut u32,
) -> Vector {
    let node = &nodes[node_index];
    if node.mass <= 0.0 {
        return Vector::ZERO;
    }
    if node.particle == Some(particle_index) && node.children.iter().all(Option::is_none) {
        return Vector::ZERO;
    }
    let position = vec3_to_rapier(particles[particle_index].position);
    let distance = (node.center_of_mass - position).length();
    let width = node.bounds.half_size * 2.0;
    if node.children.iter().all(Option::is_none)
        || width / distance.max(EPSILON) < params.opening_angle
    {
        *approximate_count += 1;
        return acceleration_from_mass(position, node.center_of_mass, node.mass, params);
    }
    let mut acceleration = Vector::ZERO;
    for child in node.children.into_iter().flatten() {
        if nodes[child].children.iter().all(Option::is_none) {
            *direct_count += 1;
        }
        acceleration += bh_acceleration(
            nodes,
            child,
            particle_index,
            particles,
            params,
            approximate_count,
            direct_count,
        );
    }
    acceleration
}

fn root_bounds(particles: &[NBodyParticle]) -> Bounds {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for particle in particles {
        min_x = f64::min(min_x, particle.position.x);
        max_x = f64::max(max_x, particle.position.x);
        min_y = f64::min(min_y, particle.position.y);
        max_y = f64::max(max_y, particle.position.y);
    }
    let center = Vector::new(0.5 * (min_x + max_x), 0.5 * (min_y + max_y), 0.0);
    let half_size = (0.5 * f64::max(max_x - min_x, max_y - min_y)).max(1.0);
    Bounds { center, half_size }
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_nbody_direct_accelerations(
    particles: *const NBodyParticle,
    particle_count: u32,
    params: NBodySolverParams,
    out_accelerations: *mut Vec3,
    capacity: u32,
    out_report: *mut NBodyForceReport,
) -> Bool {
    if particle_count == 0 || particle_count > MAX_NBODY_PARTICLES || capacity < particle_count {
        set_error(ERR_CAPACITY, "invalid N-body direct capacity");
        return Bool::FALSE;
    }
    if particles.is_null() || out_accelerations.is_null() {
        set_error(ERR_NULL_POINTER, "N-body direct pointers are null");
        return Bool::FALSE;
    }
    if !params_valid(params) {
        set_error(ERR_INVALID_ARGUMENT, "invalid N-body direct parameters");
        return Bool::FALSE;
    }
    let particles = unsafe { slice::from_raw_parts(particles, particle_count as usize) };
    if particles.iter().any(|particle| !particle_valid(*particle)) {
        set_error(ERR_INVALID_ARGUMENT, "invalid N-body particle");
        return Bool::FALSE;
    }
    let out = unsafe { slice::from_raw_parts_mut(out_accelerations, capacity as usize) };
    let mut report = NBodyForceReport {
        body_count: particle_count,
        ..NBodyForceReport::default()
    };
    for i in 0..particles.len() {
        let mut acceleration = Vector::ZERO;
        for j in 0..particles.len() {
            if i == j {
                continue;
            }
            acceleration += acceleration_from_mass(
                vec3_to_rapier(particles[i].position),
                vec3_to_rapier(particles[j].position),
                particles[j].mass,
                params,
            );
            report.direct_pair_count += 1;
        }
        report.max_acceleration = f64::max(report.max_acceleration, acceleration.length());
        out[i] = vec3_from_rapier(acceleration);
    }
    for i in 0..particles.len() {
        for j in i + 1..particles.len() {
            let distance = (vec3_to_rapier(particles[j].position)
                - vec3_to_rapier(particles[i].position))
            .length()
            .max(params.softening);
            report.potential_energy -=
                params.gravitational_constant * particles[i].mass * particles[j].mass / distance;
        }
    }
    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_nbody_barnes_hut_accelerations(
    particles: *const NBodyParticle,
    particle_count: u32,
    params: NBodySolverParams,
    out_accelerations: *mut Vec3,
    capacity: u32,
    out_report: *mut NBodyForceReport,
) -> Bool {
    if particle_count == 0 || particle_count > MAX_NBODY_PARTICLES || capacity < particle_count {
        set_error(ERR_CAPACITY, "invalid Barnes-Hut capacity");
        return Bool::FALSE;
    }
    if particles.is_null() || out_accelerations.is_null() {
        set_error(ERR_NULL_POINTER, "Barnes-Hut pointers are null");
        return Bool::FALSE;
    }
    if !params_valid(params) {
        set_error(ERR_INVALID_ARGUMENT, "invalid Barnes-Hut parameters");
        return Bool::FALSE;
    }
    let particles = unsafe { slice::from_raw_parts(particles, particle_count as usize) };
    if particles.iter().any(|particle| !particle_valid(*particle)) {
        set_error(ERR_INVALID_ARGUMENT, "invalid Barnes-Hut particle");
        return Bool::FALSE;
    }
    let mut nodes = vec![QuadNode {
        bounds: root_bounds(particles),
        mass: 0.0,
        center_of_mass: Vector::ZERO,
        particle: None,
        children: [None; 4],
    }];
    for index in 0..particles.len() {
        insert_particle(&mut nodes, 0, index, particles);
    }
    let out = unsafe { slice::from_raw_parts_mut(out_accelerations, capacity as usize) };
    let mut report = NBodyForceReport {
        body_count: particle_count,
        ..NBodyForceReport::default()
    };
    for (i, out_item) in out.iter_mut().enumerate().take(particles.len()) {
        let mut approximate = 0;
        let mut direct = 0;
        let acceleration = bh_acceleration(
            &nodes,
            0,
            i,
            particles,
            params,
            &mut approximate,
            &mut direct,
        );
        report.approximate_node_count += approximate;
        report.direct_pair_count += direct;
        report.max_acceleration = f64::max(report.max_acceleration, acceleration.length());
        *out_item = vec3_from_rapier(acceleration);
    }
    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_fmm_monopole_acceleration(
    position: Vec3,
    cluster_center: Vec3,
    cluster_mass: f64,
    params: NBodySolverParams,
    out_acceleration: *mut Vec3,
) -> Bool {
    if !vec3_finite(position)
        || !vec3_finite(cluster_center)
        || !finite_non_negative(cluster_mass)
        || !params_valid(params)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid FMM monopole parameters");
        return Bool::FALSE;
    }
    let Some(out_acceleration) = (unsafe { out_acceleration.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "FMM monopole output is null");
        return Bool::FALSE;
    };
    *out_acceleration = vec3_from_rapier(acceleration_from_mass(
        vec3_to_rapier(position),
        vec3_to_rapier(cluster_center),
        cluster_mass,
        params,
    ));
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_relativistic_orbit_correction(
    position: Vec3,
    velocity: Vec3,
    central_mass: f64,
    gravitational_constant: f64,
    out_report: *mut RelativisticOrbitReport,
) -> Bool {
    if !vec3_finite(position)
        || !vec3_finite(velocity)
        || !finite_positive(central_mass)
        || !finite_positive(gravitational_constant)
    {
        set_error(
            ERR_INVALID_ARGUMENT,
            "invalid relativistic correction parameters",
        );
        return Bool::FALSE;
    }
    let r = vec3_to_rapier(position);
    let v = vec3_to_rapier(velocity);
    let radius = r.length();
    if radius <= EPSILON {
        set_error(
            ERR_INVALID_ARGUMENT,
            "relativistic correction radius is zero",
        );
        return Bool::FALSE;
    }
    let mu = gravitational_constant * central_mass;
    let h = r.cross(v).length();
    let radial_velocity = r.dot(v) / radius;
    let c2 = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
    // Compute r² and r³ safely; prefer r² * r over powi(3)
    let r2 = radius * radius;
    let r3 = r2 * radius;
    let correction = r * (mu / (c2 * r3)) * (4.0 * mu / radius - v.length_squared())
        + v * (4.0 * mu * radial_velocity / (c2 * r2));
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "relativistic correction output is null");
        return Bool::FALSE;
    };
    *out_report = RelativisticOrbitReport {
        schwarzschild_radius: 2.0 * mu / (SPEED_OF_LIGHT * SPEED_OF_LIGHT),
        periapsis_precession_per_orbit: if h > EPSILON {
            6.0 * std::f64::consts::PI * mu * mu / (SPEED_OF_LIGHT * SPEED_OF_LIGHT * h * h)
        } else {
            0.0
        },
        correction_acceleration: vec3_from_rapier(correction),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_roche_limit(
    primary_radius: f64,
    primary_density: f64,
    secondary_density: f64,
    orbital_distance: f64,
    out_report: *mut RocheLimitReport,
) -> Bool {
    if !finite_positive(primary_radius)
        || !finite_positive(primary_density)
        || !finite_positive(secondary_density)
        || !finite_non_negative(orbital_distance)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid Roche limit parameters");
        return Bool::FALSE;
    }
    let ratio = (primary_density / secondary_density).cbrt();
    let fluid = 2.44 * primary_radius * ratio;
    let rigid = 1.26 * primary_radius * ratio;
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "Roche limit output is null");
        return Bool::FALSE;
    };
    *out_report = RocheLimitReport {
        fluid_roche_limit: fluid,
        rigid_roche_limit: rigid,
        inside_fluid_limit: Bool::from(orbital_distance > 0.0 && orbital_distance < fluid),
        inside_rigid_limit: Bool::from(orbital_distance > 0.0 && orbital_distance < rigid),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_orbital_resonance_detect(
    inner_period: f64,
    outer_period: f64,
    max_denominator: u32,
    tolerance: f64,
    out_report: *mut OrbitalResonanceReport,
) -> Bool {
    if !finite_positive(inner_period)
        || !finite_positive(outer_period)
        || max_denominator == 0
        || max_denominator > 128
        || !finite_non_negative(tolerance)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid orbital resonance parameters");
        return Bool::FALSE;
    }
    let actual = outer_period / inner_period;
    let mut best_num = 1;
    let mut best_den = 1;
    let mut best_error = f64::INFINITY;
    for den in 1..=max_denominator {
        let num = (actual * den as f64).round().max(1.0) as u32;
        let target = num as f64 / den as f64;
        let error = ((actual - target) / target).abs();
        if error < best_error {
            best_error = error;
            best_num = num;
            best_den = den;
        }
    }
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "orbital resonance output is null");
        return Bool::FALSE;
    };
    let target = best_num as f64 / best_den as f64;
    *out_report = OrbitalResonanceReport {
        ratio_numerator: best_num,
        ratio_denominator: best_den,
        actual_ratio: actual,
        target_ratio: target,
        relative_error: best_error,
        resonant: Bool::from(best_error <= tolerance),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn astro_barnes_hut_should_open(
    node_width: f64,
    distance: f64,
    opening_angle: f64,
) -> Bool {
    if !finite_positive(node_width) || !finite_positive(distance) || !finite_positive(opening_angle)
    {
        return Bool::FALSE;
    }
    Bool::from(node_width / distance >= opening_angle)
}



// ---------------------------------------------------------------------------
// Stellar structure
// ---------------------------------------------------------------------------

/// Lane-Emden equation (polytropic): dimensionless central density for polytropic index n.
/// Returns the dimensionless radius xi_1 where theta becomes zero (surface).
pub fn lane_emden_first_zero(polytropic_index: f64) -> Option<f64> {
    if !polytropic_index.is_finite() || polytropic_index < 0.0 || polytropic_index > 5.0 { return None; }
    if (polytropic_index - 0.0).abs() < 1e-6 { return Some(2.449_489_742_783_178_f64); }
    if (polytropic_index - 1.0).abs() < 1e-6 { return Some(3.141_592_653_589_793_f64); }
    if (polytropic_index - 1.5).abs() < 1e-6 { return Some(3.653_753_735_236_717_f64); }
    if (polytropic_index - 3.0).abs() < 1e-6 { return Some(6.896_848_624_348_534_f64); }
    // n=4: xi_1 ~ 14.97, n=4.5: xi_1 ~ 31.84, n=5: infinite
    None
}

/// Mass-luminosity relation for main sequence stars: L ~ M^alpha
/// alpha ~ 3.5 for solar-type stars
pub fn mass_luminosity_relation(mass_solar: f64, exponent: f64) -> Option<f64> {
    if !mass_solar.is_finite() || mass_solar <= 0.0 || !exponent.is_finite() || exponent <= 0.0 { return None; }
    Some(mass_solar.powf(exponent))
}

/// Eddington luminosity: L_Edd = 4*pi*G*M*c / kappa (W)
pub fn eddington_luminosity(mass: f64, opacity: f64) -> Option<f64> {
    let g = 6.67430e-11;
    let c = 299_792_458.0;
    if !mass.is_finite() || mass <= 0.0 || !opacity.is_finite() || opacity <= 0.0 { return None; }
    Some(4.0 * PI * g * mass * c / opacity)
}

/// Eddington luminosity in solar units (L/L_sun).
pub fn eddington_luminosity_solar(mass_solar: f64, opacity: f64) -> Option<f64> {
    let l_sun = 3.828e26;
    let m = mass_solar * 1.98847e30;
    let l = eddington_luminosity(m, opacity)?;
    Some(l / l_sun)
}

// ---------------------------------------------------------------------------
// Hubble's law and cosmology
// ---------------------------------------------------------------------------

/// Hubble's law: v = H0 * d
pub fn hubble_velocity(hubble_constant: f64, distance: f64) -> Option<f64> {
    if !hubble_constant.is_finite() || hubble_constant <= 0.0 || !distance.is_finite() || distance < 0.0 { return None; }
    Some(hubble_constant * distance)
}

/// Hubble distance: d = v / H0
pub fn hubble_distance(velocity: f64, hubble_constant: f64) -> Option<f64> {
    if !velocity.is_finite() || velocity < 0.0 || !hubble_constant.is_finite() || hubble_constant <= 0.0 { return None; }
    Some(velocity / hubble_constant)
}

// ---------------------------------------------------------------------------
// NFW dark matter profile
// ---------------------------------------------------------------------------

/// NFW dark matter density profile: rho(r) = rho_0 / (r/r_s * (1 + r/r_s)^2)
pub fn nfw_density(radius: f64, scale_radius: f64, characteristic_density: f64) -> Option<f64> {
    if !finite_4(radius, scale_radius, characteristic_density, 0.0) || scale_radius <= 0.0 || characteristic_density < 0.0 { return None; }
    let x = radius / scale_radius;
    if x <= 0.0 { return Some(characteristic_density); }
    Some(characteristic_density / (x * (1.0 + x) * (1.0 + x)))
}

/// NFW enclosed mass within radius r: M(r) = 4*pi*rho_0*r_s^3 * (ln(1+r/r_s) - r/(r_s+r))
pub fn nfw_enclosed_mass(radius: f64, scale_radius: f64, characteristic_density: f64) -> Option<f64> {
    if !finite_4(radius, scale_radius, characteristic_density, 0.0) || scale_radius <= 0.0 || characteristic_density < 0.0 { return None; }
    let x = radius / scale_radius;
    let term = (1.0 + x).ln() - x / (1.0 + x);
    Some(4.0 * PI * characteristic_density * scale_radius.powi(3) * term)
}

// ---------------------------------------------------------------------------
// Blackbody radiation
// ---------------------------------------------------------------------------

/// Planck blackbody spectral radiance: B_lambda(T) = 2hc^2/lambda^5 * 1/(exp(hc/(lambda*kb*T))-1)
pub fn blackbody_spectral_radiance(wavelength: f64, temperature: f64) -> Option<f64> {
    let h = 6.62607015e-34;
    let c = 299_792_458.0;
    let kb = 1.380649e-23;
    if !wavelength.is_finite() || wavelength <= 0.0 || !temperature.is_finite() || temperature <= 0.0 { return None; }
    let exponent = h * c / (wavelength * kb * temperature);
    if exponent > 700.0 { return Some(0.0); }
    Some(2.0 * h * c * c / wavelength.powi(5) / (exponent.exp() - 1.0))
}

/// Wien's displacement law: lambda_max * T = b, where b = 2.898e-3 m·K
pub fn wien_displacement(temperature: f64) -> Option<f64> {
    if !temperature.is_finite() || temperature <= 0.0 { return None; }
    Some(2.897771955e-3 / temperature)
}

// ---------------------------------------------------------------------------
// Jeans criterion
// ---------------------------------------------------------------------------

/// Jeans mass: M_J = (5*kb*T/(G*mu*mH))^(3/2) * (3/(4*pi*rho))^(1/2)
pub fn jeans_mass(temperature: f64, density: f64, mean_molecular_weight: f64) -> Option<f64> {
    let g = 6.67430e-11;
    let kb = 1.380649e-23;
    let mh = 1.6735575e-27;
    if !finite_4(temperature, density, mean_molecular_weight, 0.0) || temperature < 0.0 || density <= 0.0 || mean_molecular_weight <= 0.0 { return None; }
    let cs2 = kb * temperature / (mean_molecular_weight * mh);
    Some((5.0 * cs2 / (2.0 * g)).powf(1.5) * (3.0 / (4.0 * PI * density)).sqrt())
}

/// Jeans length: lambda_J = cs * sqrt(pi / (G * rho))
pub fn jeans_length(temperature: f64, density: f64, mean_molecular_weight: f64) -> Option<f64> {
    let g = 6.67430e-11;
    let kb = 1.380649e-23;
    let mh = 1.6735575e-27;
    if !finite_4(temperature, density, mean_molecular_weight, 0.0) || temperature < 0.0 || density <= 0.0 || mean_molecular_weight <= 0.0 { return None; }
    let cs = (kb * temperature / (mean_molecular_weight * mh)).sqrt();
    Some(cs * (PI / (g * density)).sqrt())
}

fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
    a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
}