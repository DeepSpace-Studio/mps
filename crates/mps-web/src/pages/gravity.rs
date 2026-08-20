use dioxus::prelude::*;
use dioxus_i18n::t;

/// Gravity Models — Newton, spherical harmonics, ellipsoid, polyhedron, Lunar Mascon.
pub fn Gravity() -> Element {
    
    rsx! {
        section { id: "sec-gravity", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("grav-tag") } }
                h1 { class: "page-title", { t!("grav-title") } }
                p { class: "page-desc", { t!("grav-desc") } }
            }
            div { class: "page-index", "02" }
        }

        // ── Model catalogue ──────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("grav-models-title") } }
            p { class: "p-lead", { t!("grav-models-lead") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("grav-col-name") } }
                        th { { t!("grav-col-use") } }
                        th { { t!("grav-col-cost") } }
                    } }
                    tbody {
                        tr { td { "Point-mass / Newton" } td { { t!("grav-row-newton") } } td { "O(1)" } }
                        tr { td { "Spherical harmonics" } td { { t!("grav-row-sh") } } td { "O(N²)" } }
                        tr { td { "Ellipsoid" } td { { t!("grav-row-ellipsoid") } } td { "O(1)" } }
                        tr { td { "Zonal / sectoral J2–J6" } td { { t!("grav-row-zonal") } } td { "O(N)" } }
                        tr { td { "Quadrupole tensor" } td { { t!("grav-row-quad") } } td { "O(1)" } }
                        tr { td { "Polyhedron (Werner–Scheeres)" } td { { t!("grav-row-poly") } } td { "O(F)" } }
                        tr { td { "Lunar Mascon (GRAIL)" } td { { t!("grav-row-mascon") } } td { "O(M)" } }
                    }
                }
            }
        }

        // ── Modelled bodies ──────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("grav-bodies-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("grav-body-earth-title") } }
                    p { { t!("grav-body-earth-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("grav-body-moon-title") } }
                    p { { t!("grav-body-moon-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("grav-body-mars-title") } }
                    p { { t!("grav-body-mars-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("grav-body-sun-title") } }
                    p { { t!("grav-body-sun-desc") } }
                }
            }
        }

        // ── Auto selection ────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("grav-auto-title") } }
            p { class: "p-lead", { t!("grav-auto-lead") } }
            div { class: "callout-note",
                p { { t!("grav-auto-note") } }
            }
        }

        // ── C API surface ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("grav-api-title") } }
            p { class: "p-muted", { t!("grav-api-desc") } }
            div { class: "code-block",
                pre { code {
                    "// C ABI — mps-core\nworld_set_point_mass_gravity(world, gm);\nworld_set_spherical_harmonics(world, &egm2008);\nworld_set_polyhedron_gravity(world, &verts, &faces);\nlunar_mascon_gravity(position);"
                } }
            }
        }
    
        }
    }
}
