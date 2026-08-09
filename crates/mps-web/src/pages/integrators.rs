use dioxus::prelude::*;
use dioxus_i18n::t;

/// Symplectic Integrators — Leapfrog, Yoshida 4, Forest–Ruth 8, Kahan补偿, Post-Newtonian修正.
pub fn Integrators() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("int-tag") } }
                h1 { class: "page-title", { t!("int-title") } }
                p { class: "page-desc", { t!("int-desc") } }
            }
            div { class: "page-index", "03" }
        }

        // ── Why symplectic ────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("int-why-title") } }
            p { class: "p-lead", { t!("int-why-lead") } }
            p { class: "p-muted", { t!("int-why-body") } }
        }

        // ── Integrator catalogue ──────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("int-catalog-title") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("int-col-name") } }
                        th { { t!("int-col-order") } }
                        th { { t!("int-col-notes") } }
                    } }
                    tbody {
                        tr { td { "Leapfrog (KDK)" } td { "2" } td { { t!("int-row-leapfrog") } } }
                        tr { td { "Yoshida 4" } td { "4" } td { { t!("int-row-yoshida4") } } }
                        tr { td { "Forest–Ruth 8" } td { "8" } td { { t!("int-row-forest-ruth") } } }
                    }
                }
            }
        }

        // ── Kahan error-compensated variants ──────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("int-kahan-title") } }
            p { class: "p-lead", { t!("int-kahan-lead") } }
            ul { class: "ul-plain",
                li { { t!("int-kahan-li-1") } }
                li { { t!("int-kahan-li-2") } }
                li { { t!("int-kahan-li-3") } }
            }
            p { class: "p-note", { t!("int-kahan-note") } }
        }

        // ── Post-Newtonian relativistic corrections ───────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("int-pn-title") } }
            p { class: "p-lead", { t!("int-pn-lead") } }
            ul { class: "ul-plain",
                li { { t!("int-pn-li-1pn") } }
                li { { t!("int-pn-li-2pn") } }
                li { { t!("int-pn-li-full") } }
            }
        }

        // ── Adaptive step control ─────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("int-adaptive-title") } }
            p { class: "p-muted", { t!("int-adaptive-desc") } }
            div { class: "code-block",
                pre { code {
                    "let dt = adaptive_step_size(dt, err, tol, order);\nif step_accepted(err, tol) { commit_step(); }"
                } }
            }
        }

        // ── Diagnostics ───────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("int-diag-title") } }
            ul { class: "ul-plain",
                li { { t!("int-diag-energy") } }
                li { { t!("int-diag-am") } }
                li { { t!("int-diag-kepler") } }
            }
        }
    }
}
