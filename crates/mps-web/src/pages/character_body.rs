use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::*;

/// Character Body — kinematic character controller driving a rigid body (Phase 3c).
pub fn CharacterBody() -> Element {
    rsx! {
        section { id: "sec-character-body", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("char-tag") } }
                    h1 { class: "page-title", { t!("char-title") } }
                    p { class: "page-desc", { t!("char-desc") } }
                }
                div { class: "page-index", "07" }
            }

            // ── Overview ────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("char-overview-title") } }
                p { class: "p-lead", { t!("char-overview-lead") } }
                p { class: "p-muted", { t!("char-overview-body") } }
            }

            // ── C ABI ───────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("char-api-title") } }
                p { class: "p-muted", { t!("char-api-desc") } }
                div { class: "code-block",
                    code {
                        "character_body_create(world, shape, translation) -> u32\n"
                        "character_body_move(world, id, desired, dt) -> EffectiveCharacterMovement\n"
                        "character_body_get_translation(world, id, out) -> Bool\n"
                        "character_body_destroy(world, id) -> Bool\n"
                    }
                }
            }

            // ── Capabilities ────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("char-cap-title") } }
                p { class: "p-lead", { t!("char-cap-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("char-cap-01-title") } }
                        p { { t!("char-cap-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-cap-02-title") } }
                        p { { t!("char-cap-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-cap-03-title") } }
                        p { { t!("char-cap-03-desc") } }
                    }
                }
            }
        }
    }
}
