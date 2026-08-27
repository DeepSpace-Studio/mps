use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — Shared-memory Arena & JNI batch bridge.
pub fn CosmosArena() -> Element {
    rsx! {
        section { id: "sec-cosmos-arena", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("ca-tag") } }
                    h1 { class: "page-title", { t!("ca-title") } }
                    p { class: "page-desc", { t!("ca-desc") } }
                }
                div { class: "page-index", "08·6" }
            }

            div { class: "section-card",
                h2 { { t!("ca-overview-title") } }
                p { class: "p-lead", { t!("ca-overview-lead") } }
                p { class: "p-muted", { t!("ca-overview-body") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("ca-ffi-title") } }
                p { class: "p-muted", { t!("ca-ffi-desc") } }
                ul { class: "ul-plain",
                    li { { t!("ca-ffi-1") } }
                    li { { t!("ca-ffi-2") } }
                    li { { t!("ca-ffi-3") } }
                }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("ca-jni-title") } }
                p { class: "p-muted", { t!("ca-jni-desc") } }
            }
        }
    }
}
