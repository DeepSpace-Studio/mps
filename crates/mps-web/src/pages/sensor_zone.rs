use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::*;

/// Sensor Trigger Zone — the fourth body type: a sensor collider polled for
/// overlaps (no physical response, just events).
pub fn SensorZone() -> Element {
    rsx! {
        section { id: "sec-sensor-zone", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("sensor-tag") } }
                    h1 { class: "page-title", { t!("sensor-title") } }
                    p { class: "page-desc", { t!("sensor-desc") } }
                }
                div { class: "page-index", "08" }
            }

            // ── Overview ────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("sensor-overview-title") } }
                p { class: "p-lead", { t!("sensor-overview-lead") } }
                p { class: "p-muted", { t!("sensor-overview-body") } }
            }

            // ── C ABI ───────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("sensor-api-title") } }
                p { class: "p-muted", { t!("sensor-api-desc") } }
                div { class: "code-block",
                    code {
                        "sensor_zone_create(world, shape, translation) -> u32\n"
                        "sensor_zone_poll(world, id) -> Bool\n"
                        "sensor_zone_contact_count(world, id) -> u32\n"
                        "sensor_zone_get_contacts(world, id, out, max_count) -> u32\n"
                        "sensor_zone_get_translation(world, id, out) -> Bool\n"
                        "sensor_zone_set_translation(world, id, translation) -> Bool\n"
                        "sensor_zone_set_enabled(world, id, enabled) -> Bool\n"
                        "sensor_zone_is_triggered(world, id) -> Bool\n"
                        "sensor_zone_destroy(world, id) -> Bool\n"
                    }
                }
            }

            // ── Capabilities ────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("sensor-cap-title") } }
                p { class: "p-lead", { t!("sensor-cap-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("sensor-cap-01-title") } }
                        p { { t!("sensor-cap-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("sensor-cap-02-title") } }
                        p { { t!("sensor-cap-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("sensor-cap-03-title") } }
                        p { { t!("sensor-cap-03-desc") } }
                    }
                }
            }
        }
    }
}
