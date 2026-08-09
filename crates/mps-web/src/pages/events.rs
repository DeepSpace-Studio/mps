use dioxus::prelude::*;
use dioxus_i18n::t;

/// Event System — collision + contact-force events, three dispatch modes, C-callback hook.
pub fn Events() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("evt-tag") } }
                h1 { class: "page-title", { t!("evt-title") } }
                p { class: "page-desc", { t!("evt-desc") } }
            }
            div { class: "page-index", "06" }
        }

        // ── Event types ───────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("evt-types-title") } }
            p { class: "p-lead", { t!("evt-types-lead") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("evt-col-type") } }
                        th { { t!("evt-col-fields") } }
                    } }
                    tbody {
                        tr {
                            td { "CollisionEventRecord" }
                            td { { t!("evt-row-collision") } }
                        }
                        tr {
                            td { "ContactForceEventRecord" }
                            td { { t!("evt-row-contact") } }
                        }
                    }
                }
            }
        }

        // ── Dispatch modes ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("evt-modes-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("evt-mode-poll-title") } }
                    p { { t!("evt-mode-poll-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("evt-mode-callback-title") } }
                    p { { t!("evt-mode-callback-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("evt-mode-both-title") } }
                    p { { t!("evt-mode-both-desc") } }
                }
            }
        }

        // ── Ring buffer ───────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("evt-ring-title") } }
            p { class: "p-muted", { t!("evt-ring-desc") } }
            ul { class: "ul-plain",
                li { { t!("evt-ring-li-1") } }
                li { { t!("evt-ring-li-2") } }
                li { { t!("evt-ring-li-3") } }
            }
        }

        // ── ForceLaw dispatch ─────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("evt-forces-title") } }
            p { class: "p-lead", { t!("evt-forces-lead") } }
            ul { class: "ul-plain",
                li { { t!("evt-force-coulomb") } }
                li { { t!("evt-force-airdrag") } }
                li { { t!("evt-force-external") } }
                li { { t!("evt-force-newton") } }
                li { { t!("evt-force-custom") } }
            }
            p { class: "p-note", { t!("evt-forces-note") } }
        }

        // ── C callback ABI ────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("evt-abi-title") } }
            div { class: "code-block",
                pre { code {
                    "// 注册碰撞回调：safe 端 Rust 转 unsafe extern \"C\"\ntypedef void (*CollisionEventFn)\n    (const void* ctx,\n     const CollisionEventRecord* event,\n     void* user);\nworld_set_collision_callback(world, fn, user);"
                } }
            }
        }
    }
}
