use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — Flight dynamics, trim, stability & perturbation.
pub fn CosmosFlight() -> Element {
    rsx! {
        section { id: "sec-cosmos-flight", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("cf-tag") } }
                    h1 { class: "page-title", { t!("cf-title") } }
                    p { class: "page-desc", { t!("cf-desc") } }
                }
                div { class: "page-index", "08·5" }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cf-dyn-title") } }
                p { class: "p-lead", { t!("cf-dyn-lead") } }
                p { class: "p-muted", { t!("cf-dyn-desc") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cf-trim-title") } }
                p { class: "p-muted", { t!("cf-trim-desc") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cf-stab-title") } }
                p { class: "p-muted", { t!("cf-stab-desc") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cf-pert-title") } }
                p { class: "p-muted", { t!("cf-pert-desc") } }
            }

            // ── FFI ↔ JNI ──────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cf-map-title") } }
                p { class: "p-muted", { t!("cf-map-note") } }
                div { class: "code-block",
                    pre { code { { t!("cf-map-body") } } }
                }
            }
        }
    }
}
