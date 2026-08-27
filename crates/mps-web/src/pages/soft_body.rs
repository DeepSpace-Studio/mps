use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::*;

/// Soft Body — XPBD / MassSpring deformable bodies (Phases 0–25).
pub fn SoftBody() -> Element {
    rsx! {
        section { id: "sec-soft-body", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("soft-tag") } }
                    h1 { class: "page-title", { t!("soft-title") } }
                    p { class: "page-desc", { t!("soft-desc") } }
                }
                div { class: "page-index", "06" }
            }

            // ── Overview ──────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("soft-overview-title") } }
                p { class: "p-lead", { t!("soft-overview-lead") } }
                p { class: "p-muted", { t!("soft-overview-body") } }
            }

            // ── Solver ────────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-solver-title") } }
                p { class: "p-muted", { t!("soft-solver-desc") } }
                ul { class: "ul-plain",
                    li { { t!("soft-solver-li-1") } }
                    li { { t!("soft-solver-li-2") } }
                    li { { t!("soft-solver-li-3") } }
                }
            }

            // ── Data model ─────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-data-title") } }
                p { class: "p-muted", { t!("soft-data-desc") } }
                ul { class: "ul-plain",
                    li { { t!("soft-data-li-1") } }
                    li { { t!("soft-data-li-2") } }
                    li { { t!("soft-data-li-3") } }
                    li { { t!("soft-data-li-4") } }
                }
            }

            // ── Capability matrix (22 deliverables, Phases 0–21) ───────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-cap-title") } }
                p { class: "p-lead", { t!("soft-cap-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-01-title") } }
                        p { { t!("soft-cap-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-02-title") } }
                        p { { t!("soft-cap-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-03-title") } }
                        p { { t!("soft-cap-03-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-04-title") } }
                        p { { t!("soft-cap-04-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-05-title") } }
                        p { { t!("soft-cap-05-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-06-title") } }
                        p { { t!("soft-cap-06-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-07-title") } }
                        p { { t!("soft-cap-07-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-08-title") } }
                        p { { t!("soft-cap-08-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-09-title") } }
                        p { { t!("soft-cap-09-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-10-title") } }
                        p { { t!("soft-cap-10-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-11-title") } }
                        p { { t!("soft-cap-11-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-12-title") } }
                        p { { t!("soft-cap-12-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-13-title") } }
                        p { { t!("soft-cap-13-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-14-title") } }
                        p { { t!("soft-cap-14-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-15-title") } }
                        p { { t!("soft-cap-15-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-16-title") } }
                        p { { t!("soft-cap-16-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-17-title") } }
                        p { { t!("soft-cap-17-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-18-title") } }
                        p { { t!("soft-cap-18-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-19-title") } }
                        p { { t!("soft-cap-19-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-20-title") } }
                        p { { t!("soft-cap-20-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-cap-21-title") } }
                        p { { t!("soft-cap-21-desc") } }
                    }
                    div { class: "feature-card feature-card-accent",
                        h3 { { t!("soft-cap-22-title") } }
                        p { { t!("soft-cap-22-desc") } }
                    }
                }
            }

            // ── FFI safety line (Phases 22–25, zero-fork) ───────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-p25-title") } }
                p { class: "p-lead", { t!("soft-p25-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("soft-p25-1-title") } }
                        p { { t!("soft-p25-1-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-p25-2-title") } }
                        p { { t!("soft-p25-2-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-p25-3-title") } }
                        p { { t!("soft-p25-3-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-p25-4-title") } }
                        p { { t!("soft-p25-4-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("soft-p25-5-title") } }
                        p { { t!("soft-p25-5-desc") } }
                    }
                    div { class: "feature-card feature-card-accent",
                        h3 { { t!("soft-p25-6-title") } }
                        p { { t!("soft-p25-6-desc") } }
                    }
                }
            }

            // ── Phase 25 FFI <-> JNI map ──────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-p25-map-title") } }
                p { class: "p-muted", { t!("soft-p25-map-note") } }
                div { class: "code-block",
                    pre { code { { t!("soft-p25-map-body") } } }
                }
            }

            // ── API surface ────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("soft-api-title") } }
                p { class: "p-muted", { t!("soft-api-desc") } }
                div { class: "stat-grid",
                    div { class: "stat-card",
                        div { class: "stat-num", { FFI_SOFT_BODY } }
                        div { class: "stat-label", { t!("soft-api-stat-ffi") } }
                    }
                    div { class: "stat-card",
                        div { class: "stat-num", { JNI_SOFT_BODY } }
                        div { class: "stat-label", { t!("soft-api-stat-jni") } }
                    }
                    div { class: "stat-card",
                        div { class: "stat-num", { TEST_SOFT_BODY } }
                        div { class: "stat-label", { t!("soft-api-stat-tests") } }
                    }
                }
            }
        }
    }
}
