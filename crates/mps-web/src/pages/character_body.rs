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
                        "character_body_set_shape(world, id, shape) -> Bool\n"
                        "character_body_set_up(world, id, up) -> Bool\n"
                        "character_body_set_offset_absolute(world, id, offset) -> Bool\n"
                        "character_body_set_autostep(world, id, enabled, max_height, min_width, include_dynamic) -> Bool\n"
                        "character_body_set_snap_to_ground(world, id, enabled, distance) -> Bool\n"
                        "character_body_set_slope_angles(world, id, max_climb, min_slide) -> Bool\n"
                        "character_body_set_slide(world, id, enabled) -> Bool\n"
                        "character_body_is_grounded(world, id) -> Bool\n"
                        "character_body_is_on_ground(world, id) -> Bool\n"
                        "character_body_is_sliding_down_slope(world, id) -> Bool\n"
                        "character_body_move_with_terrain(world, id, desired, dt) -> EffectiveCharacterMovement\n"
                        "character_body_collision_count(world, id) -> u32\n"
                        "character_body_get_collision(world, id, index) -> CharacterCollision\n"
                        "character_body_solve_impulses(world, id, dt, mass) -> Bool\n"
                        "character_body_get_translation(world, id, out) -> Bool\n"
                        "character_body_destroy(world, id) -> Bool\n"
                    }
                }
            }

            // ── Minecraft-style tuning ───────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("char-mc-title") } }
                p { class: "p-lead", { t!("char-mc-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("char-mc-01-title") } }
                        p { { t!("char-mc-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-mc-02-title") } }
                        p { { t!("char-mc-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-mc-03-title") } }
                        p { { t!("char-mc-03-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-mc-04-title") } }
                        p { { t!("char-mc-04-desc") } }
                    }
                }
                p { class: "p-muted", { t!("char-mc-note") } }
            }

            // ── Collision readback & terrain gravity ──────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("char-col-title") } }
                p { class: "p-lead", { t!("char-col-lead") } }
                p { class: "p-muted", { t!("char-col-body") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("char-col-01-title") } }
                        p { { t!("char-col-01-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-col-02-title") } }
                        p { { t!("char-col-02-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("char-col-03-title") } }
                        p { { t!("char-col-03-desc") } }
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
                    div { class: "feature-card",
                        h3 { { t!("char-cap-04-title") } }
                        p { { t!("char-cap-04-desc") } }
                    }
                }
            }
        }
    }
}
