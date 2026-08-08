use dioxus::prelude::*;

/// Java FFM Bindings — migration stub (content TBD)
pub fn Ffm() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", "// MPS" }
                h1 { class: "page-title", "Java FFM Bindings" }
                p { class: "page-desc", "Content migration in progress." }
            }
            div { class: "page-index", "--" }
        }
        div { class: "section-card",
            h2 { "Java FFM Bindings" }
            p { class: "p-lead", "This page is pending migration from Topcoat to Dioxus." }
        }
    }
}
