use dioxus::prelude::*;
use dioxus_i18n::t;

/// Java FFM Bindings — Foreign Function & Memory API (JEP 454) metadata
/// surface for Java 25+ callers via the `mpd-ffm` crate.
pub fn Ffm() -> Element {
    
    rsx! {
        section { id: "sec-ffm", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("ffm-tag") } }
                h1 { class: "page-title", { t!("ffm-title") } }
                p { class: "page-desc", { t!("ffm-desc") } }
            }
            div { class: "page-index", "12" }
        }

        // ── What FFM is here ─────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("ffm-what-title") } }
            p { class: "p-lead", { t!("ffm-what-lead") } }
            p { class: "p-muted", { t!("ffm-what-body") } }
        }

        // ── ABI surface ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-surface-title") } }
            p { class: "p-lead", { t!("ffm-surface-lead") } }
            div { class: "code-block",
                pre { code {
                    r#"// mps-ffm/src/lib.rs — 整个 crate 就是 ABI 探测 + 版本协商
#[unsafe(no_mangle)]
pub extern "C" fn abi_version() -> u32 {{ ABI_VERSION }}

#[unsafe(no_mangle)]
pub extern "C" fn abi_supports_ffm() -> Bool {{ Bool::TRUE }}

#[unsafe(no_mangle)]
pub extern "C" fn abi_supports_jni() -> Bool {{ Bool::TRUE }}"#
                } }
            }
            p { class: "p-note", { t!("ffm-surface-note") } }
        }

        // ── JNI vs FFM comparison ────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-vs-title") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("ffm-col-feature") } }
                        th { "JNI" }
                        th { "FFM" }
                    } }
                    tbody {
                        tr { td { { t!("ffm-row-min-java") } } td { "Java 8+" } td { "Java 22+ (JEP 454)" } }
                        tr { td { { t!("ffm-row-binding") } } td { { t!("ffm-row-jni-bind") } } td { { t!("ffm-row-ffm-bind") } } }
                        tr { td { { t!("ffm-row-overhead") } } td { { t!("ffm-row-jni-over") } } td { { t!("ffm-row-ffm-over") } } }
                        tr { td { { t!("ffm-row-memory") } } td { { t!("ffm-row-jni-mem") } } td { { t!("ffm-row-ffm-mem") } } }
                        tr { td { { t!("ffm-row-panic") } } td { { t!("ffm-row-jni-panic") } } td { { t!("ffm-row-ffm-panic") } } }
                    }
                }
            }
        }

        // ── Linker layout ───────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-layout-title") } }
            p { class: "p-muted", { t!("ffm-layout-desc") } }
            div { class: "code-block",
                pre { code {
                    "// Linker.downcallHandle(\"world_create\", FunctionDescriptor.of(\n//     JAVA_LONG,                            // 返回 WorldHandle*\n//     C_DOUBLE, C_INT, C_INT                // dt, iters, ccd\n// ))\nMethodHandle h = LINKER.downcallHandle(\n    lookup.find(\"world_create\"),\n    FunctionDescriptor.of(JAVA_LONG, C_DOUBLE, C_INT, C_INT)\n);\nlong world = (long) h.invokeExact(dt, iters, ccd);"
                } }
            }
        }

        // ── C ABI input ─────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-header-title") } }
            p { class: "p-lead", { t!("ffm-header-lead") } }
            ul { class: "ul-plain",
                li { { t!("ffm-header-cbindgen") } }
                li { { t!("ffm-header-structs") } }
                li { { t!("ffm-header-load") } }
            }
            p { class: "p-note", { t!("ffm-header-note") } }
        }

        // ── Allocation strategy ─────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-alloc-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("ffm-alloc-segment-title") } }
                    p { { t!("ffm-alloc-segment-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("ffm-alloc-arena-title") } }
                    p { { t!("ffm-alloc-arena-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("ffm-alloc-shared-title") } }
                    p { { t!("ffm-alloc-shared-desc") } }
                }
            }
        }

        // ── Status ──────────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("ffm-status-title") } }
            div { class: "callout",
                p { { t!("ffm-status-body") } }
            }
        }
    
        }
    }
}
