use dioxus::prelude::*;
use dioxus_i18n::t;

/// Formula Modules — 33 pure-Rust domain modules mapped to their category headings.
pub fn Formula() -> Element {
    rsx! {
        section { id: "sec-formula", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("form-tag") } }
                h1 { class: "page-title", { t!("form-title") } }
                p { class: "page-desc", { t!("form-desc") } }
            }
            div { class: "page-index", "04" }
        }

        div { class: "callout-note",
            p { { t!("form-intro-pure") } }
        }

        // ── Spaceflight (rasid 88 fn / 9 files) ────────────────────────────
        div { class: "section-card",
            h2 { { t!("formula-cat-spaceflight") } }
            ul { class: "ul-plain",
                li { id: "form-mod-kepler",
                    span { class: "mod-name", { t!("form-mod-kepler") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "kepler_period" } }
                    li { code { "hohmann_transfer" } }
                    li { code { "lambert_universal_variable" } }
                    li { code { "state_to_elements" } }
                    li { code { "elements_to_state" } }
                    li { code { "tsiolkovsky_delta_v" } }
                    li { code { "bi_elliptic_transfer_delta_v" } }
                    li { code { "plane_change_delta_v" } }
                    }
                }
                li { id: "form-mod-dynamics",
                    span { class: "mod-name", { t!("form-mod-dynamics") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "variational_two_body" } }
                    li { code { "mass_properties_two_body" } }
                    li { code { "manipulator_dynamics_diag" } }
                    li { code { "flexible_mode_derivative" } }
                    li { code { "slosh_pendulum_derivative" } }
                    li { code { "docking_glideslope_command" } }
                    li { code { "artificial_potential_guidance" } }
                    li { code { "cw_derivative" } }
                    }
                }
                li { id: "form-mod-perturbation",
                    span { class: "mod-name", { t!("form-mod-perturbation") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "atmospheric_drag_acceleration" } }
                    li { code { "solar_radiation_pressure_acceleration" } }
                    li { code { "gauss_variational_equations" } }
                    li { code { "atmospheric_density_scale_height" } }
                    li { code { "solar_activity_density_correction" } }
                    li { code { "igrf_tilted_dipole" } }
                    }
                }
                li { id: "form-mod-propulsion",
                    span { class: "mod-name", { t!("form-mod-propulsion") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "hall_thruster_performance" } }
                    li { code { "solar_panel_power" } }
                    li { code { "spe_oxygen_rate" } }
                    li { code { "sabatier_methane_rate" } }
                    li { code { "battery_equivalent_circuit" } }
                    li { code { "co" } }
                    }
                }
                li { id: "form-mod-rotation",
                    span { class: "mod-name", { t!("form-mod-rotation") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "quaternion_derivative" } }
                    li { code { "rigid_body_euler_derivative" } }
                    li { code { "gravity_gradient_torque" } }
                    li { code { "cmg_robust_pseudoinverse_diag" } }
                    li { code { "triad_attitude" } }
                    li { code { "least_squares_attitude_two_vector" } }
                    li { code { "ekf_predict_scalar" } }
                    }
                }
                li { id: "form-mod-thermal",
                    span { class: "mod-name", { t!("form-mod-thermal") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "sutton_graves_heat_rate" } }
                    li { code { "thermal_balance" } }
                    li { code { "heat_pipe_thermal_resistance" } }
                    li { code { "radiator_power" } }
                    li { code { "reentry_peak_g_load" } }
                    li { code { "single_phase_loop_heat_transfer" } }
                    }
                }
                li { id: "form-mod-debris",
                    span { class: "mod-name", { t!("form-mod-debris") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "debris_collision_probability" } }
                    li { code { "whipple_critical_projectile_diameter" } }
                    li { code { "eclipse_duration_circular" } }
                    li { code { "lagrange_collinear_gamma" } }
                    li { code { "lense_thirring_precession_rate" } }
                    li { code { "atomic_oxygen_erosion" } }
                    }
                }
                li { id: "form-mod-gnss",
                    span { class: "mod-name", { t!("form-mod-gnss") } }
                    ul { class: "ul-plain mod-fns",
                    li { code { "gnss_pseudorange" } }
                    li { code { "gnss_double_difference_carrier_phase" } }
                    li { code { "radar_range_rate" } }
                    li { code { "friis_link" } }
                    li { code { "friis_wavelength_from_frequency" } }
                    }
                }
                li { { t!("form-mod-trajectory") } }
            }
        }

        // ── Astrophysics & stellar physics ────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-astrophysics") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-astrophysics") } }
                li { { t!("form-mod-stellar") } }
                li { { t!("form-mod-galactic") } }
                li { { t!("form-mod-cosmology") } }
                li { { t!("form-mod-helio") } }
                li { { t!("form-mod-high-energy") } }
                li { { t!("form-mod-celestial") } }
                li { { t!("form-mod-planetary") } }
            }
        }

        // ── Mechanics ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-mechanics") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-mechanics") } }
                li { { t!("form-mod-material") } }
                li { { t!("form-mod-biomech") } }
                li { { t!("form-mod-control") } }
                li { { t!("form-mod-chaos") } }
                li { { t!("form-mod-topology") } }
                li { { t!("form-mod-softbody") } }
            }
        }

        // ── Relativity ─────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-relativity") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-relativity") } }
                li { { t!("form-mod-transmission") } }
            }
        }

        // ── Quantum & electromagnetism ────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-quantum") } }
            p { class: "p-muted", { t!("form-mod-quantum") } }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-electromagnetism") } }
            p { class: "p-muted", { t!("form-mod-em") } }
        }

        // ── Nuclear, thermodynamics & continuum ───────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-nuclear") } }
            p { class: "p-muted", { t!("form-mod-nuclear") } }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("formula-cat-fluid") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-fluid") } }
                li { { t!("form-mod-plasma") } }
                li { { t!("form-mod-superfluidity") } }
                li { { t!("form-mod-continuum") } }
            }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-mod-physchem-title") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-physchem") } }
                li { { t!("form-mod-thermo") } }
                li { { t!("form-mod-molecular") } }
                li { { t!("form-mod-wave-optics") } }
                li { { t!("form-mod-acoustics") } }
                li { { t!("form-mod-aero") } }
            }
        }

        // ── Supporting modules ─────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-support-title") } }
            p { class: "p-muted", { t!("form-support-intro") } }
            ul { class: "ul-plain",
                li { "math.rs — finite-many / vec3 / clamp01 共享原语" }
                li { "integrators.rs — Leapfrog / Yoshida 4 / Forest–Ruth 8 / Kahan / 1PN+2PN" }
                li { "gravitational_models.rs — Legendre / 球谐 / Carlson RF·RD / 椭球 / J2 张量" }
                li { "celestial_data.rs — JPL DE441 10 天体精密参数" }
            }
        }

        // ── Calling from Java ──────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-call-title") } }
            p { class: "p-muted", { t!("form-call-desc") } }
            div { class: "code-block",
                pre { code {
                    "// 全部公式函数经 C ABI 暴露，无 WorldHandle 依赖\n// 例：双椭球引力加速度\nVec3 a = mps_formula_ellipsoid_gravity(pos, body);\n// 例：Yoshida 4 阶辛积分器推进\nmps_formula_yoshida4_step(&mut pos, &mut vel, gm, dt);"
                } }
            }
        }

        }
    }
}
