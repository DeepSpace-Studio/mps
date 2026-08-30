use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::*;

/// Ray-Cast Vehicle Controller — the fifth body type: a dynamic chassis body
/// driven by rapier's ray-cast vehicle controller.
pub fn VehicleController() -> Element {
    rsx! {
        section { id: "sec-vehicle-controller", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("veh-tag") } }
                    h1 { class: "page-title", { t!("veh-title") } }
                    p { class: "page-desc", { t!("veh-desc") } }
                }
                div { class: "page-index", "09" }
            }

            // ── Overview ────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("veh-overview-title") } }
                p { class: "p-lead", { t!("veh-overview-lead") } }
                p { class: "p-muted", { t!("veh-overview-body") } }
            }

            // ── C ABI ───────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("veh-api-title") } }
                p { class: "p-muted", { t!("veh-api-desc") } }
                div { class: "code-block",
                    code {
                        "vehicle_controller_create(world, shape, translation) -> u32\n"
                        "vehicle_controller_add_wheel(world, id, chassis_conn, direction, axle, ...) -> u32\n"
                        "vehicle_controller_set_engine_force(world, id, wheel, force) -> Bool\n"
                        "vehicle_controller_set_brake(world, id, wheel, brake) -> Bool\n"
                        "vehicle_controller_set_steering(world, id, wheel, steering) -> Bool\n"
                        "vehicle_controller_update(world, id, dt) -> Bool\n"
                        "vehicle_controller_get_translation(world, id, out) -> Bool\n"
                        "vehicle_controller_get_velocity(world, id, out) -> Bool\n"
                        "vehicle_controller_wheel_on_ground(world, id, wheel) -> Bool\n"
                        "vehicle_controller_wheel_contact_normal(world, id, wheel, out) -> Bool\n"
                        "vehicle_controller_destroy(world, id) -> Bool\n"
                    }
                }
            }

            // ── Capabilities ────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("veh-cap-title") } }
                p { class: "p-lead", { t!("veh-cap-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("veh-cap-01-title") } }
                        p { { t!("veh-cap-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("veh-cap-02-title") } }
                        p { { t!("veh-cap-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("veh-cap-03-title") } }
                        p { { t!("veh-cap-03-desc") } }
                    }
                }
            }
        }
    }
}
