use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — Orbit & Diagnostics (elements, Hill radius, Kozai, snapshots).
pub fn CosmosOrbit() -> Element {
    rsx! {
        section { id: "sec-cosmos-orbit", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("co-tag") } }
                    h1 { class: "page-title", { t!("co-title") } }
                    p { class: "page-desc", { t!("co-desc") } }
                }
                div { class: "page-index", "08·4" }
            }

            div { class: "section-card",
                h2 { { t!("co-overview-title") } }
                p { class: "p-lead", { t!("co-overview-lead") } }
                p { class: "p-muted", { t!("co-overview-body") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("co-fn-title") } }
                p { class: "p-muted", { t!("co-fn-desc") } }
                ul { class: "ul-plain",
                    li { { t!("co-fn-1") } }
                    li { { t!("co-fn-2") } }
                    li { { t!("co-fn-3") } }
                    li { { t!("co-fn-4") } }
                    li { { t!("co-fn-5") } }
                }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("co-snap-title") } }
                p { class: "p-muted", { t!("co-snap-desc") } }
            }
        }
    }
}
