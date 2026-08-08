use dioxus::prelude::*;

/// Java JNI Bindings — migration stub (content TBD)
pub fn Jni() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", "// MPS" }
                h1 { class: "page-title", "Java JNI Bindings" }
                p { class: "page-desc", "Content migration in progress." }
            }
            div { class: "page-index", "--" }
        }
        div { class: "section-card",
            h2 { "Java JNI Bindings" }
            p { class: "p-lead", "This page is pending migration from Topcoat to Dioxus." }
        }
    }
}
