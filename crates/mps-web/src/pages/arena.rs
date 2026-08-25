use dioxus::prelude::*;
use dioxus_i18n::t;

/// Shared Memory Arena — DirectByteBuffer bridge between Java and Rust physics state.
pub fn Arena() -> Element {
    rsx! {
        section { id: "sec-arena", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("arena-tag") } }
                h1 { class: "page-title", { t!("arena-title") } }
                p { class: "page-desc", { t!("arena-desc") } }
            }
            div { class: "page-index", "07" }
        }

        // ── Why an arena ──────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("arena-why-title") } }
            p { class: "p-lead", { t!("arena-why-lead") } }
            p { class: "p-muted", { t!("arena-why-body") } }
        }

        // ── Memory layout ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arena-layout-title") } }
            p { class: "p-muted", { t!("arena-layout-desc") } }
            div { class: "code-block",
                pre { code {
                    "┌──────────────── Arena Header ────────────────┐\n│ magic • version • body_count • slots_capacity │\n├──────────────────┬─────────────────────────────┤\n│ BodySlot[N]      │ position • rotation         │\n│  (SoA fields)    │ linear_vel • angular_vel    │\n│                  │ force_accum • torque_accum  │\n├──────────────────┼─────────────────────────────┤\n│ Holes ring       │ 复用被删除槽位的索引         │\n│ RingBuffer<u32>  │ O(1) alloc / free            │\n└──────────────────┴─────────────────────────────┘"
                } }
            }
        }

        // ── Sub-modules ───────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arena-mods-title") } }
            ul { class: "ul-plain",
                li { { t!("arena-mod-header") } }
                li { { t!("arena-mod-layout") } }
                li { { t!("arena-mod-ring") } }
                li { { t!("arena-mod-holes") } }
            }
        }

        // ── Per-frame protocol ────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arena-flow-title") } }
            ol { class: "ol-plain",
                li { { t!("arena-flow-step-1") } }
                li { { t!("arena-flow-step-2") } }
                li { { t!("arena-flow-step-3") } }
            }
            div { class: "callout-note",
                p { { t!("arena-flow-note") } }
            }
        }

        // ── Java side ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("arena-java-title") } }
            p { class: "p-muted", { t!("arena-java-desc") } }
            div { class: "code-block",
                pre { code {
                    "// Java 21 — DirectByteBuffer + JNI\nByteBuffer arena = arena_alloc(N * SLOT_BYTES);\narena_write_positions(arena, positions);\nworld_step(world, dt);\narena_read_positions(arena, positions);"
                } }
            }
        }

        }
    }
}
