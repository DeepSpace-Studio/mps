use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, t};

/// Force Queue Tutorial page — zero-copy shared-memory force application.
/// Live-worked example: Java enqueues forces → native consumes in world_step.
#[component]
pub fn ForceQueue() -> Element {
    rsx! {
        section { id: "sec-force-queue", class: "doc-section",
            h2 { class: "section-title", { t!("force-queue-tag") } }
            h2 { class: "section-title", { t!("force-queue-title") } }
            p { class: "lead", { t!("force-queue-desc") } }

            // Overview
            h3 { class: "subsection-title", { t!("force-queue-overview-title") } }
            p { class: "body-text", { t!("force-queue-overview-lead") } }
            p { class: "body-text", { t!("force-queue-overview-body") } }

            // Memory Layout
            h3 { class: "subsection-title", { t!("force-queue-layout-title") } }
            p { class: "body-text", { t!("force-queue-layout-desc") } }

            pre { class: "code-block", { t!("force-queue-layout-diagram") } }

            div { class: "callout callout-info",
                p { class: "callout-title", { t!("force-queue-layout-note") } }
            }

            // Synchronization Model
            h3 { class: "subsection-title", { t!("force-queue-sync-title") } }
            p { class: "body-text", { t!("force-queue-sync-lead") } }
            ul { class: "bullet-list",
                li { { t!("force-queue-sync-li-1") } }
                li { { t!("force-queue-sync-li-2") } }
                li { { t!("force-queue-sync-li-3") } }
                li { { t!("force-queue-sync-li-4") } }
                li { { t!("force-queue-sync-li-5") } }
                li { { t!("force-queue-sync-li-6") } }
            }

            // Stride Modes
            h3 { class: "subsection-title", { t!("force-queue-stride-title") } }
            p { class: "body-text", { t!("force-queue-stride-desc") } }
            div { class: "table-wrap",
                table { class: "doc-table",
                    thead { tr { th { { t!("force-queue-stride-col-mode") } } th { { t!("force-queue-stride-col-desc") } } th { { t!("force-queue-stride-col-use") } } } }
                    tbody {
                        tr { td { code { "6" } } td { { t!("force-queue-stride-6-desc") } } td { { t!("force-queue-stride-6-use") } } }
                        tr { td { code { "7" } } td { { t!("force-queue-stride-7-desc") } } td { { t!("force-queue-stride-7-use") } } }
                    }
                }
            }

            // FFI Surface
            h3 { class: "subsection-title", { t!("force-queue-ffi-title") } }
            p { class: "body-text", { t!("force-queue-ffi-desc") } }
            pre { class: "code-block", { t!("force-queue-ffi-sample") } }

            // Java Producer Example (JNI)
            h3 { class: "subsection-title", { t!("force-queue-jni-title") } }
            p { class: "body-text", { t!("force-queue-jni-desc") } }
            pre { class: "code-block", { t!("force-queue-jni-sample") } }

            // Java Producer Example (FFM)
            h3 { class: "subsection-title", { t!("force-queue-ffm-title") } }
            p { class: "body-text", { t!("force-queue-ffm-desc") } }
            pre { class: "code-block", { t!("force-queue-ffm-sample") } }

            // Integration Test Reference
            h3 { class: "subsection-title", { t!("force-queue-test-title") } }
            p { class: "body-text", { t!("force-queue-test-desc") } }
            pre { class: "code-block", { t!("force-queue-test-sample") } }

            // Performance Notes
            h3 { class: "subsection-title", { t!("force-queue-perf-title") } }
            ul { class: "bullet-list",
                li { { t!("force-queue-perf-li-1") } }
                li { { t!("force-queue-perf-li-2") } }
                li { { t!("force-queue-perf-li-3") } }
                li { { t!("force-queue-perf-li-4") } }
            }
        }
    }
}
