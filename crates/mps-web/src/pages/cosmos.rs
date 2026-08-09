use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::CELESTIAL_COUNT;

/// Cosmos Rigid Body — separate physics domain for orbital-scale simulation.
pub fn Cosmos() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("cosmos-tag") } }
                h1 { class: "page-title", { t!("cosmos-title") } }
                p { class: "page-desc", { t!("cosmos-desc") } }
            }
            div { class: "page-index", "08" }
        }

        // ── What Cosmos is ────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("cosmos-what-title") } }
            p { class: "p-lead", { t!("cosmos-what-lead") } }
            p { class: "p-muted", { t!("cosmos-what-body") } }
        }

        // ── Sub-modules ───────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("cosmos-mods-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-kepler-title") } }
                    p { { t!("cosmos-mod-kepler-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-dynamics-title") } }
                    p { { t!("cosmos-mod-dynamics-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-perturbation-title") } }
                    p { { t!("cosmos-mod-perturbation-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-propulsion-title") } }
                    p { { t!("cosmos-mod-propulsion-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-rotation-title") } }
                    p { { t!("cosmos-mod-rotation-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-thermal-title") } }
                    p { { t!("cosmos-mod-thermal-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-debris-title") } }
                    p { { t!("cosmos-mod-debris-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("cosmos-mod-gnss-title") } }
                    p { { t!("cosmos-mod-gnss-desc") } }
                }
            }
        }

        // ── n-body pairwise ───────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("cosmos-nbody-title") } }
            p { class: "p-lead", { t!("cosmos-nbody-lead") } }
            p { class: "p-note", { t!("cosmos-nbody-note") } }
        }

        // ── Celestial catalogue ──────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading",
                { t!("cosmos-bodies-title", count: CELESTIAL_COUNT) }
            }
            p { class: "p-muted", { t!("cosmos-bodies-desc") } }
        }

        // ── JNI integration ───────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("cosmos-jni-title") } }
            p { class: "p-muted", { t!("cosmos-jni-desc") } }
        }
    }
}
