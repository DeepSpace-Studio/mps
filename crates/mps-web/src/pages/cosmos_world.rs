use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — World & Bodies (independent orbital-scale world).
pub fn CosmosWorld() -> Element {
    rsx! {
        section { id: "sec-cosmos-world", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("cw-tag") } }
                    h1 { class: "page-title", { t!("cw-title") } }
                    p { class: "page-desc", { t!("cw-desc") } }
                }
                div { class: "page-index", "08·1" }
            }

            div { class: "section-card",
                h2 { { t!("cw-overview-title") } }
                p { class: "p-lead", { t!("cw-overview-lead") } }
                p { class: "p-muted", { t!("cw-overview-body") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cw-bodies-title") } }
                p { class: "p-muted", { t!("cw-bodies-desc") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cw-batch-title") } }
                p { class: "p-muted", { t!("cw-batch-desc") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cw-ffi-title") } }
                p { class: "p-muted", { t!("cw-ffi-desc") } }
                ul { class: "ul-plain",
                    li { { t!("cw-ffi-1") } }
                    li { { t!("cw-ffi-2") } }
                    li { { t!("cw-ffi-3") } }
                    li { { t!("cw-ffi-4") } }
                    li { { t!("cw-ffi-5") } }
                    li { { t!("cw-ffi-6") } }
                }
            }

            // ── FFI ↔ JNI ──────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cw-map-title") } }
                p { class: "p-muted", { t!("cw-map-note") } }
                div { class: "code-block",
                    pre { code { { t!("cw-map-body") } } }
                }
            }
        }
    }
}
