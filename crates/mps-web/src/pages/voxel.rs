use dioxus::prelude::*;

/// Voxel System — migration stub (content TBD)
pub fn Voxel() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", "// MPS" }
                h1 { class: "page-title", "Voxel System" }
                p { class: "page-desc", "Content migration in progress." }
            }
            div { class: "page-index", "--" }
        }
        div { class: "section-card",
            h2 { "Voxel System" }
            p { class: "p-lead", "This page is pending migration from Topcoat to Dioxus." }
        }
    }
}
