use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::CELESTIAL_COUNT;

/// Cosmos Rigid Body — separate physics domain for orbital-scale simulation.
/// Overview page; detailed capabilities live on the six sub-pages linked below.
pub fn Cosmos() -> Element {
    rsx! {
        section { id: "sec-cosmos", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("cosmos-tag") } }
                    h1 { class: "page-title", { t!("cosmos-title") } }
                    p { class: "page-desc", { t!("cosmos-desc") } }
                }
                div { class: "page-index", "08" }
            }

            // ── What Cosmos is ────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("cosmos-what-title") } }
                p { class: "p-lead", { t!("cosmos-what-lead") } }
                p { class: "p-muted", { t!("cosmos-what-body") } }
            }

            // ── Feature landing cards (6 sub-pages) ──────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cosmos-land-title") } }
                p { class: "p-lead", { t!("cosmos-land-lead") } }
                div { class: "feature-grid",
                    a { class: "feature-card", href: "#sec-cosmos-world",
                        h3 { { t!("cosmos-class-world-title") } }
                        p { { t!("cosmos-class-world-desc") } }
                    }
                    a { class: "feature-card", href: "#sec-cosmos-gravity",
                        h3 { { t!("cosmos-class-gravity-title") } }
                        p { { t!("cosmos-class-gravity-desc") } }
                    }
                    a { class: "feature-card", href: "#sec-cosmos-integrator",
                        h3 { { t!("cosmos-class-integrator-title") } }
                        p { { t!("cosmos-class-integrator-desc") } }
                    }
                    a { class: "feature-card", href: "#sec-cosmos-orbit",
                        h3 { { t!("cosmos-class-orbit-title") } }
                        p { { t!("cosmos-class-orbit-desc") } }
                    }
                    a { class: "feature-card", href: "#sec-cosmos-flight",
                        h3 { { t!("cosmos-class-flight-title") } }
                        p { { t!("cosmos-class-flight-desc") } }
                    }
                    a { class: "feature-card feature-card-accent", href: "#sec-cosmos-arena",
                        h3 { { t!("cosmos-class-arena-title") } }
                        p { { t!("cosmos-class-arena-desc") } }
                    }
                }
            }

            // ── Sub-modules (mps-formula files cosmos reuses) ────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cosmos-mods-title") } }
                div { class: "feature-grid",
                    a { class: "feature-card", href: "#form-mod-kepler",
                        h3 { { t!("cosmos-mod-kepler-title") } }
                        p { { t!("cosmos-mod-kepler-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-dynamics",
                        h3 { { t!("cosmos-mod-dynamics-title") } }
                        p { { t!("cosmos-mod-dynamics-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-perturbation",
                        h3 { { t!("cosmos-mod-perturbation-title") } }
                        p { { t!("cosmos-mod-perturbation-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-propulsion",
                        h3 { { t!("cosmos-mod-propulsion-title") } }
                        p { { t!("cosmos-mod-propulsion-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-rotation",
                        h3 { { t!("cosmos-mod-rotation-title") } }
                        p { { t!("cosmos-mod-rotation-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-thermal",
                        h3 { { t!("cosmos-mod-thermal-title") } }
                        p { { t!("cosmos-mod-thermal-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-debris",
                        h3 { { t!("cosmos-mod-debris-title") } }
                        p { { t!("cosmos-mod-debris-desc") } }
                    }
                    a { class: "feature-card", href: "#form-mod-gnss",
                        h3 { { t!("cosmos-mod-gnss-title") } }
                        p { { t!("cosmos-mod-gnss-desc") } }
                    }
                }
            }

            // ── n-body pairwise (signature trait) ───────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("cosmos-nbody-title") } }
                p { class: "p-lead", { t!("cosmos-nbody-lead") } }
                p { class: "p-note", { t!("cosmos-nbody-note") } }
            }

            // ── Celestial catalogue ─────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading",
                    { t!("cosmos-bodies-title", count: CELESTIAL_COUNT) }
                }
                p { class: "p-muted", { t!("cosmos-bodies-desc") } }
            }
        }
    }
}
