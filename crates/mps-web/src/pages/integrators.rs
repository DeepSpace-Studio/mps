use dioxus::prelude::*;

/// Symplectic Integrators — migration stub (content TBD)
pub fn Integrators() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", "// MPS" }
                h1 { class: "page-title", "Symplectic Integrators" }
                p { class: "page-desc", "Content migration in progress." }
            }
            div { class: "page-index", "--" }
        }
        div { class: "section-card",
            h2 { "Symplectic Integrators" }
            p { class: "p-lead", "This page is pending migration from Topcoat to Dioxus." }
        }
    }
}
