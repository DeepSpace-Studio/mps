use std::slice;

use crate::error::{
    ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, clear_error, set_error,
};
use crate::ffi::{
    Bool, FemHeatDiffusionReport, FemHeatEdge, FemHeatNode, HeatConductionReport,
    MaterialProperties, PhaseChangeReport, ThermalRadiationReport, ThermalStressReport,
    ThermoelasticReport,
};

use crate::math::{KahanSum, finite_non_negative, finite_positive};

const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
const MAX_FEM_NODES: u32 = 1_000_000;
const MAX_FEM_EDGES: u32 = 2_000_000;

fn material_valid(material: MaterialProperties) -> bool {
    finite_non_negative(material.density)
        && finite_non_negative(material.friction)
        && finite_non_negative(material.restitution)
        && finite_positive(material.youngs_modulus)
        && material.poisson_ratio.is_finite()
        && material.poisson_ratio > -1.0
        && material.poisson_ratio < 0.5
        && material.thermal_expansion.is_finite()
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_fourier_conduction(
    hot_temperature: f64,
    cold_temperature: f64,
    conductivity: f64,
    area: f64,
    thickness: f64,
    out_report: *mut HeatConductionReport,
) -> Bool {
    if !hot_temperature.is_finite()
        || !cold_temperature.is_finite()
        || !finite_non_negative(conductivity)
        || !finite_non_negative(area)
        || !finite_positive(thickness)
    {
        set_error(
            ERR_INVALID_ARGUMENT,
            "invalid Fourier conduction parameters",
        );
        return Bool::FALSE;
    }
    let temperature_delta = hot_temperature - cold_temperature;
    let temperature_gradient = temperature_delta / thickness;
    let heat_flux = conductivity * temperature_gradient;
    let heat_rate = heat_flux * area;
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "heat conduction output is null");
        return Bool::FALSE;
    };
    *out_report = HeatConductionReport {
        temperature_delta,
        temperature_gradient,
        heat_flux,
        heat_rate,
        thermal_resistance: if conductivity > 0.0 && area > 0.0 {
            thickness / (conductivity * area)
        } else {
            f64::INFINITY
        },
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_phase_change(
    temperature: f64,
    phase_temperature: f64,
    mass: f64,
    specific_heat: f64,
    latent_heat: f64,
    heat_input: f64,
    out_report: *mut PhaseChangeReport,
) -> Bool {
    if !temperature.is_finite()
        || !phase_temperature.is_finite()
        || !finite_positive(mass)
        || !finite_positive(specific_heat)
        || !finite_non_negative(latent_heat)
        || !heat_input.is_finite()
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid phase-change parameters");
        return Bool::FALSE;
    }

    let mut sensible_heat = heat_input;
    let mut latent_heat_used = 0.0;
    let mut final_temperature = temperature;
    let mut phase_fraction_delta = 0.0;
    let phase_energy = mass * latent_heat;

    if heat_input > 0.0 && temperature < phase_temperature {
        let heat_to_phase = (phase_temperature - temperature) * mass * specific_heat;
        let used = heat_input.min(heat_to_phase);
        final_temperature += used / (mass * specific_heat);
        sensible_heat = used;
        let remaining = heat_input - used;
        if remaining > 0.0 && phase_energy > 0.0 {
            latent_heat_used = remaining.min(phase_energy);
            phase_fraction_delta = latent_heat_used / phase_energy;
            final_temperature = phase_temperature;
        }
    } else if heat_input > 0.0
        && (temperature - phase_temperature).abs() <= f64::EPSILON
        && phase_energy > 0.0
    {
        latent_heat_used = heat_input.min(phase_energy);
        phase_fraction_delta = latent_heat_used / phase_energy;
        sensible_heat = heat_input - latent_heat_used;
        final_temperature = phase_temperature + sensible_heat / (mass * specific_heat);
    } else {
        final_temperature += heat_input / (mass * specific_heat);
    }

    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "phase-change output is null");
        return Bool::FALSE;
    };
    *out_report = PhaseChangeReport {
        final_temperature,
        sensible_heat,
        latent_heat_used,
        phase_fraction_delta,
        phase_changed: Bool::from(phase_fraction_delta > 0.0),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_phase_condition(
    temperature: f64,
    solidus_temperature: f64,
    liquidus_temperature: f64,
    out_report: *mut PhaseChangeReport,
) -> Bool {
    if !temperature.is_finite()
        || !solidus_temperature.is_finite()
        || !liquidus_temperature.is_finite()
        || solidus_temperature > liquidus_temperature
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid phase condition temperatures");
        return Bool::FALSE;
    }
    let fraction = if temperature <= solidus_temperature {
        0.0
    } else if temperature >= liquidus_temperature {
        1.0
    } else {
        (temperature - solidus_temperature) / (liquidus_temperature - solidus_temperature)
    };
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "phase condition output is null");
        return Bool::FALSE;
    };
    *out_report = PhaseChangeReport {
        final_temperature: temperature,
        sensible_heat: 0.0,
        latent_heat_used: 0.0,
        phase_fraction_delta: fraction,
        phase_changed: Bool::from(fraction > 0.0),
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_stefan_boltzmann_radiation(
    temperature: f64,
    ambient_temperature: f64,
    emissivity: f64,
    area: f64,
    out_report: *mut ThermalRadiationReport,
) -> Bool {
    if !finite_non_negative(temperature)
        || !finite_non_negative(ambient_temperature)
        || !emissivity.is_finite()
        || !(0.0..=1.0).contains(&emissivity)
        || !finite_non_negative(area)
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid thermal radiation parameters");
        return Bool::FALSE;
    }
    let emitted_power = emissivity * STEFAN_BOLTZMANN * area * temperature.powi(4);
    let absorbed_power = emissivity * STEFAN_BOLTZMANN * area * ambient_temperature.powi(4);
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "thermal radiation output is null");
        return Bool::FALSE;
    };
    *out_report = ThermalRadiationReport {
        emitted_power,
        absorbed_power,
        net_power: emitted_power - absorbed_power,
        radiative_coefficient: emissivity * STEFAN_BOLTZMANN * area,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_fem_diffusion_step(
    nodes: *const FemHeatNode,
    node_count: u32,
    edges: *const FemHeatEdge,
    edge_count: u32,
    dt: f64,
    out_temperatures: *mut f64,
    capacity: u32,
    out_report: *mut FemHeatDiffusionReport,
) -> Bool {
    if node_count == 0
        || node_count > MAX_FEM_NODES
        || edge_count > MAX_FEM_EDGES
        || capacity < node_count
    {
        set_error(ERR_CAPACITY, "invalid FEM heat diffusion capacity");
        return Bool::FALSE;
    }
    if nodes.is_null() || out_temperatures.is_null() || (edge_count > 0 && edges.is_null()) {
        set_error(ERR_NULL_POINTER, "FEM heat diffusion pointers are null");
        return Bool::FALSE;
    }
    if !finite_non_negative(dt) {
        set_error(ERR_INVALID_ARGUMENT, "invalid FEM heat diffusion timestep");
        return Bool::FALSE;
    }

    let nodes = unsafe { slice::from_raw_parts(nodes, node_count as usize) };
    let edges = unsafe { slice::from_raw_parts(edges, edge_count as usize) };
    let out_temperatures =
        unsafe { slice::from_raw_parts_mut(out_temperatures, capacity as usize) };
    // Use out_temperatures as temporary scratch before writing final values:
    // first pass accumulates heat_rates into out_temperatures directly,
    // second pass converts to temperature deltas in-place.
    let heat_rates = &mut out_temperatures[..node_count as usize];
    heat_rates.fill(0.0);

    for (index, node) in nodes.iter().enumerate() {
        if !node.temperature.is_finite()
            || !finite_positive(node.heat_capacity)
            || !node.heat_source.is_finite()
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid FEM heat node");
            return Bool::FALSE;
        }
        heat_rates[index] += node.heat_source;
    }

    for edge in edges {
        if edge.node_a >= node_count
            || edge.node_b >= node_count
            || !finite_non_negative(edge.conductance)
        {
            set_error(ERR_INVALID_ARGUMENT, "invalid FEM heat edge");
            return Bool::FALSE;
        }
        let a = edge.node_a as usize;
        let b = edge.node_b as usize;
        let heat_rate = edge.conductance * (nodes[b].temperature - nodes[a].temperature);
        heat_rates[a] += heat_rate;
        heat_rates[b] -= heat_rate;
    }

    let mut max_temperature_delta = 0.0;
    let mut total_heat_rate_acc = KahanSum::default();
    for (index, node) in nodes.iter().enumerate() {
        let delta = heat_rates[index] * dt / node.heat_capacity;
        heat_rates[index] = node.temperature + delta;
        max_temperature_delta = f64::max(max_temperature_delta, delta.abs());
        total_heat_rate_acc.add(heat_rates[index]);
    }

    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = FemHeatDiffusionReport {
            node_count,
            edge_count,
            total_heat_rate: total_heat_rate_acc.value(),
            max_temperature_delta,
        };
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_stress_from_expansion(
    material: MaterialProperties,
    strain: f64,
    delta_temperature: f64,
    out_report: *mut ThermalStressReport,
) -> Bool {
    if !material_valid(material) || !strain.is_finite() || !delta_temperature.is_finite() {
        set_error(ERR_INVALID_ARGUMENT, "invalid thermal stress parameters");
        return Bool::FALSE;
    }
    let thermal_strain = material.thermal_expansion * delta_temperature;
    let mechanical_strain = strain - thermal_strain;
    let stress = material.youngs_modulus * mechanical_strain;
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "thermal stress output is null");
        return Bool::FALSE;
    };
    *out_report = ThermalStressReport {
        free_thermal_strain: thermal_strain,
        mechanical_strain,
        stress,
        deformation: thermal_strain,
        elastic_energy_density: 0.5 * stress * mechanical_strain,
    };
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn thermal_thermoelastic_stress_strain(
    material: MaterialProperties,
    strain_x: f64,
    strain_y: f64,
    strain_z: f64,
    delta_temperature: f64,
    out_report: *mut ThermoelasticReport,
) -> Bool {
    if !material_valid(material)
        || !strain_x.is_finite()
        || !strain_y.is_finite()
        || !strain_z.is_finite()
        || !delta_temperature.is_finite()
    {
        set_error(ERR_INVALID_ARGUMENT, "invalid thermoelastic parameters");
        return Bool::FALSE;
    }
    let thermal_strain = material.thermal_expansion * delta_temperature;
    let ex = strain_x - thermal_strain;
    let ey = strain_y - thermal_strain;
    let ez = strain_z - thermal_strain;
    let lambda = material.youngs_modulus * material.poisson_ratio
        / ((1.0 + material.poisson_ratio) * (1.0 - 2.0 * material.poisson_ratio));
    let shear = material.youngs_modulus / (2.0 * (1.0 + material.poisson_ratio));
    let trace = ex + ey + ez;
    let stress_x = lambda * trace + 2.0 * shear * ex;
    let stress_y = lambda * trace + 2.0 * shear * ey;
    let stress_z = lambda * trace + 2.0 * shear * ez;
    let Some(out_report) = (unsafe { out_report.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "thermoelastic output is null");
        return Bool::FALSE;
    };
    *out_report = ThermoelasticReport {
        thermal_strain,
        mechanical_strain_x: ex,
        mechanical_strain_y: ey,
        mechanical_strain_z: ez,
        stress_x,
        stress_y,
        stress_z,
        bulk_modulus: material.youngs_modulus / (3.0 * (1.0 - 2.0 * material.poisson_ratio)),
        shear_modulus: shear,
    };
    clear_error();
    Bool::TRUE
}



// ---------------------------------------------------------------------------
// Ideal gas law
// ---------------------------------------------------------------------------

/// Ideal gas law: PV = nRT. Returns pressure (Pa).
pub fn ideal_gas_pressure(volume: f64, moles: f64, temperature: f64) -> Option<f64> {
    if !volume.is_finite() || volume <= 0.0 || !moles.is_finite() || moles < 0.0 || !temperature.is_finite() || temperature < 0.0 { return None; }
    Some(moles * 8.314462618 * temperature / volume)
}

/// Returns volume from ideal gas law.
pub fn ideal_gas_volume(pressure: f64, moles: f64, temperature: f64) -> Option<f64> {
    if !pressure.is_finite() || pressure < 0.0 || !moles.is_finite() || moles < 0.0 || !temperature.is_finite() || temperature < 0.0 { return None; }
    Some(moles * 8.314462618 * temperature / pressure)
}

/// Returns temperature from ideal gas law.
pub fn ideal_gas_temperature(pressure: f64, volume: f64, moles: f64) -> Option<f64> {
    if !pressure.is_finite() || pressure < 0.0 || !volume.is_finite() || volume <= 0.0 || !moles.is_finite() || moles <= 0.0 { return None; }
    Some(pressure * volume / (moles * 8.314462618))
}

// ---------------------------------------------------------------------------
// Polytropic process
// ---------------------------------------------------------------------------

/// Polytropic process: P2 = P1 * (V1/V2)^gamma
pub fn polytropic_pressure(p1: f64, v1: f64, v2: f64, gamma: f64) -> Option<f64> {
    if !finite_4(p1, v1, v2, gamma) || p1 < 0.0 || v1 <= 0.0 || v2 <= 0.0 || gamma <= 0.0 { return None; }
    Some(p1 * (v1 / v2).powf(gamma))
}

/// Polytropic work: W = (P2*V2 - P1*V1) / (1 - gamma)
pub fn polytropic_work(p1: f64, v1: f64, p2: f64, v2: f64, gamma: f64) -> Option<f64> {
    if !finite_4(p1, v1, gamma, 0.0) || !finite_4(p2, v2, gamma, 0.0) || p1 < 0.0 || v1 <= 0.0 || p2 < 0.0 || v2 <= 0.0 || gamma <= 0.0 { return None; }
    if (gamma - 1.0).abs() < 1.0e-12 { return None; }
    Some((p2 * v2 - p1 * v1) / (1.0 - gamma))
}

// ---------------------------------------------------------------------------
// Convective heat transfer
// ---------------------------------------------------------------------------

/// Newton's law of cooling: Q = h * A * (T_surface - T_fluid)
pub fn convective_heat_flux(h: f64, area: f64, t_surface: f64, t_fluid: f64) -> Option<f64> {
    if !finite_4(h, area, t_surface, t_fluid) || h < 0.0 || area < 0.0 { return None; }
    Some(h * area * (t_surface - t_fluid))
}

/// Reynolds number: Re = rho * v * L / mu
pub fn reynolds_number(density: f64, velocity: f64, char_length: f64, viscosity: f64) -> Option<f64> {
    if !finite_4(density, velocity, char_length, viscosity) || density < 0.0 || velocity < 0.0 || char_length <= 0.0 || viscosity <= 0.0 { return None; }
    Some(density * velocity * char_length / viscosity)
}

/// Nusselt number (Dittus-Boelter): Nu = 0.023 * Re^0.8 * Pr^n
pub fn dittus_boelter_nusselt(reynolds: f64, prandtl: f64, heating: bool) -> Option<f64> {
    if !reynolds.is_finite() || reynolds < 0.0 || !prandtl.is_finite() || prandtl < 0.0 { return None; }
    if reynolds < 10000.0 { return None; }
    let n = if heating { 0.4 } else { 0.3 };
    Some(0.023 * reynolds.powf(0.8) * prandtl.powf(n))
}

/// Prandtl number: Pr = cp * mu / k
pub fn prandtl_number(cp: f64, viscosity: f64, conductivity: f64) -> Option<f64> {
    if !finite_4(cp, viscosity, conductivity, 0.0) || cp <= 0.0 || viscosity <= 0.0 || conductivity <= 0.0 { return None; }
    Some(cp * viscosity / conductivity)
}

/// Heat transfer coefficient from Nusselt: h = Nu * k / L
pub fn htc_from_nusselt(nusselt: f64, conductivity: f64, char_length: f64) -> Option<f64> {
    if !finite_4(nusselt, conductivity, char_length, 0.0) || nusselt < 0.0 || conductivity <= 0.0 || char_length <= 0.0 { return None; }
    Some(nusselt * conductivity / char_length)
}

// ---------------------------------------------------------------------------
// Thermodynamic cycle efficiencies
// ---------------------------------------------------------------------------

/// Carnot efficiency: eta = 1 - T_cold / T_hot
pub fn carnot_efficiency(t_hot: f64, t_cold: f64) -> Option<f64> {
    if !finite_4(t_hot, t_cold, 0.0, 0.0) || t_hot <= 0.0 || t_cold < 0.0 || t_cold >= t_hot { return None; }
    Some(1.0 - t_cold / t_hot)
}

/// Otto cycle efficiency: eta = 1 - 1 / r^(gamma-1)
pub fn otto_efficiency(compression_ratio: f64, gamma: f64) -> Option<f64> {
    if !compression_ratio.is_finite() || compression_ratio <= 1.0 || !gamma.is_finite() || gamma <= 1.0 { return None; }
    Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0))
}

/// Diesel cycle efficiency
pub fn diesel_efficiency(compression_ratio: f64, cutoff_ratio: f64, gamma: f64) -> Option<f64> {
    if !finite_4(compression_ratio, cutoff_ratio, gamma, 0.0) || compression_ratio <= 1.0 || cutoff_ratio <= 1.0 || gamma <= 1.0 { return None; }
    let term = (cutoff_ratio.powf(gamma) - 1.0) / (gamma * (cutoff_ratio - 1.0));
    Some(1.0 - 1.0 / compression_ratio.powf(gamma - 1.0) * term)
}

/// Brayton cycle efficiency: eta = 1 - 1 / r_p^((gamma-1)/gamma)
pub fn brayton_efficiency(pressure_ratio: f64, gamma: f64) -> Option<f64> {
    if !pressure_ratio.is_finite() || pressure_ratio <= 1.0 || !gamma.is_finite() || gamma <= 1.0 { return None; }
    Some(1.0 - 1.0 / pressure_ratio.powf((gamma - 1.0) / gamma))
}

// ---------------------------------------------------------------------------
// Clausius-Clapeyron
// ---------------------------------------------------------------------------

/// Clausius-Clapeyron: ln(P2/P1) = -(L/R) * (1/T2 - 1/T1)
pub fn clausius_clapeyron_pressure(p1: f64, t1: f64, t2: f64, latent_heat: f64) -> Option<f64> {
    if !finite_4(p1, t1, t2, latent_heat) || p1 <= 0.0 || t1 <= 0.0 || t2 <= 0.0 || latent_heat < 0.0 { return None; }
    Some(p1 * (-latent_heat / 8.314462618 * (1.0 / t2 - 1.0 / t1)).exp())
}

// ---------------------------------------------------------------------------
// Entropy change
// ---------------------------------------------------------------------------

pub fn entropy_change_constant_volume(moles: f64, cv: f64, t1: f64, t2: f64) -> Option<f64> {
    if !finite_4(moles, cv, t1, t2) || moles < 0.0 || cv <= 0.0 || t1 <= 0.0 || t2 <= 0.0 { return None; }
    Some(moles * cv * (t2 / t1).ln())
}

pub fn entropy_change_constant_pressure(moles: f64, cp: f64, t1: f64, t2: f64) -> Option<f64> {
    if !finite_4(moles, cp, t1, t2) || moles < 0.0 || cp <= 0.0 || t1 <= 0.0 || t2 <= 0.0 { return None; }
    Some(moles * cp * (t2 / t1).ln())
}

fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
    a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
}