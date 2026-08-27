use dioxus::prelude::*;
use dioxus_i18n::t;

/// Cosmos — Gravity & n-body mutual gravity.
pub fn CosmosGravity() -> Element {
    rsx! {
        section { id: "sec-cosmos-gravity", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("cg-tag") } }
                    h1 { class: "page-title", { t!("cg-title") } }
                    p { class: "page-desc", { t!("cg-desc") } }
                }
                div { class: "page-index", "08·2" }
            }

            div { class: "section-card",
                h2 { { t!("cg-overview-title") } }
                p { class: "p-lead", { t!("cg-overview-lead") } }
                p { class: "p-muted", { t!("cg-overview-body") } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cg-fn-title") } }
                p { class: "p-muted", { t!("cg-fn-desc") } }
                ul { class: "ul-plain",
                    li { { t!("cg-fn-1") } }
                    li { { t!("cg-fn-2") } }
                    li { { t!("cg-fn-3") } }
                    li { { t!("cg-fn-4") } }
                    li { { t!("cg-fn-5") } }
                }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cg-ffi-title") } }
                p { class: "p-muted", { t!("cg-ffi-desc") } }
            }
        }
    }
}
