use dioxus::prelude::*;

/// Event System — migration stub (content TBD)
pub fn Events() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", "// MPS" }
                h1 { class: "page-title", "Event System" }
                p { class: "page-desc", "Content migration in progress." }
            }
            div { class: "page-index", "--" }
        }
        div { class: "section-card",
            h2 { "Event System" }
            p { class: "p-lead", "This page is pending migration from Topcoat to Dioxus." }
        }
    }
}
