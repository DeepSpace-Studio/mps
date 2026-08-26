use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::{FORMULA_MODULE_COUNT, JNI_METHOD_COUNT, TEST_COUNT};

/// Architecture Overview — crate graph, layering, and the Java→Rust call path.
pub fn Architecture() -> Element {
    rsx! {
        section { id: "sec-architecture", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("arch-tag") } }
                h1 { class: "page-title", { t!("arch-title") } }
                p { class: "page-desc", { t!("arch-desc") } }
            }
            div { class: "page-index", "01" }
        }

        // ── Crate stack diagram ────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("arch-stack-title") } }
            p { class: "p-lead", { t!("arch-stack-lead") } }
            div { class: "code-block",
                pre { code { { t!("arch-stack-diagram", modules: FORMULA_MODULE_COUNT, methods: JNI_METHOD_COUNT) } } }
            }
        }

        // ── Layer responsibilities ───────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arch-layers-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("arch-layer-formula-title") } }
                    p { { t!("arch-layer-formula-desc",
                            modules: FORMULA_MODULE_COUNT) } }
                }
                div { class: "feature-card",
                    h3 { { t!("arch-layer-core-title") } }
                    p { { t!("arch-layer-core-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("arch-layer-cosmos-title") } }
                    p { { t!("arch-layer-cosmos-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("arch-layer-jni-title") } }
                    p { { t!("arch-layer-jni-desc", methods: JNI_METHOD_COUNT) } }
                }
                div { class: "feature-card",
                    h3 { { t!("arch-layer-test-title") } }
                    p { { t!("arch-layer-test-desc", tests: TEST_COUNT) } }
                }
                div { class: "feature-card",
                    h3 { { t!("arch-layer-ffm-title") } }
                    p { { t!("arch-layer-ffm-desc") } }
                }
            }
        }

        // ── Per-frame data flow ───────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arch-flow-title") } }
            p { class: "p-lead", { t!("arch-flow-lead") } }
            ol { class: "ol-plain",
                li { { t!("arch-flow-step-1") } }
                li { { t!("arch-flow-step-2") } }
                li { { t!("arch-flow-step-3") } }
                li { { t!("arch-flow-step-4") } }
                li { { t!("arch-flow-step-5") } }
            }
        }

        // ── Design tenets ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arch-tenets-title") } }
            div { class: "callout-note",
                p { { t!("arch-tenet-zero-copy") } }
            }
            div { class: "callout-note",
                p { { t!("arch-tenet-formula-pure") } }
            }
            div { class: "callout-note",
                p { { t!("arch-tenet-ffi-stable") } }
            }
        }

        // ── Build pipeline note ───────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arch-build-title") } }
            p { class: "p-muted", { t!("arch-build-cbindgen") } }
            p { class: "p-muted", { t!("arch-build-xtask") } }
        }

        }
    }
}
