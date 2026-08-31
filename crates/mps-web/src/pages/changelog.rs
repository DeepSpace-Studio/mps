use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::{CORE_FFI_COUNT, JNI_METHOD_COUNT, TEST_COUNT};

/// Changelog — capability milestones delivered across the workspace,
/// grounded in real commit history and current surface counts.
pub fn Changelog() -> Element {
    rsx! {
        section { id: "sec-changelog", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("changelog-tag") } }
                h1 { class: "page-title", { t!("changelog-title") } }
                p { class: "page-desc",
                    { t!("changelog-desc", jni: JNI_METHOD_COUNT, ffi: CORE_FFI_COUNT, tests: TEST_COUNT) }
                }
            }
            div { class: "page-index", "15" }
        }

        // ── Capability evolution ──────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("changelog-grid-title") } }
            p { class: "p-lead", { t!("changelog-grid-lead") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("changelog-c1-title") } }
                    p { { t!("changelog-c1-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("changelog-c2-title") } }
                    p { { t!("changelog-c2-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("changelog-c3-title") } }
                    p { { t!("changelog-c3-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("changelog-c4-title") } }
                    p { { t!("changelog-c4-desc") } }
                }
                div { class: "feature-card feature-card-accent",
                    h3 { { t!("changelog-c5-title") } }
                    p { { t!("changelog-c5-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("changelog-c6-title") } }
                    p { { t!("changelog-c6-desc") } }
                }
            }
        }

        // ── Current scale ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("changelog-total-title") } }
            p { class: "p-lead",
                { t!("changelog-total-lead", jni: JNI_METHOD_COUNT, ffi: CORE_FFI_COUNT, tests: TEST_COUNT) }
            }
            div { class: "metric-grid",
                div { class: "metric-card", strong { class: "num", { JNI_METHOD_COUNT } } span { class: "label", { t!("changelog-stat-jni") } } }
                div { class: "metric-card", strong { class: "num", { CORE_FFI_COUNT } } span { class: "label", { t!("changelog-stat-ffi") } } }
                div { class: "metric-card", strong { class: "num", { TEST_COUNT } } span { class: "label", { t!("changelog-stat-tests") } } }
            }
        }

        }
    }
}
