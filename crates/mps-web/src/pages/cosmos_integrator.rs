use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — Integrators (Verlet / high-order / Kahan-compensated).
pub fn CosmosIntegrator() -> Element {
    rsx! {
        section { id: "sec-cosmos-integrator", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("ci-tag") } }
                    h1 { class: "page-title", { t!("ci-title") } }
                    p { class: "page-desc", { t!("ci-desc") } }
                }
                div { class: "page-index", "08·3" }
            }

            div { class: "section-card",
                h2 { { t!("ci-overview-title") } }
                p { class: "p-lead", { t!("ci-overview-lead") } }
                p { class: "p-muted", { t!("ci-overview-body") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("ci-fn-title") } }
                p { class: "p-muted", { t!("ci-fn-desc") } }
                ul { class: "ul-plain",
                    li { { t!("ci-fn-1") } }
                    li { { t!("ci-fn-2") } }
                    li { { t!("ci-fn-3") } }
                    li { { t!("ci-fn-4") } }
                    li { { t!("ci-fn-5") } }
                }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("ci-toggle-title") } }
                p { class: "p-muted", { t!("ci-toggle-desc") } }
            }

            // ── FFI ↔ JNI ──────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("ci-map-title") } }
                p { class: "p-muted", { t!("ci-map-note") } }
                div { class: "code-block",
                    pre { code { { t!("ci-map-body") } } }
                }
            }
        }
    }
}
