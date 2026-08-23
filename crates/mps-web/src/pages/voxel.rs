use dioxus::prelude::*;
use dioxus_i18n::t;

/// Voxel System — dense voxel grid + collider build + terrain gravity bridge.
pub fn Voxel() -> Element {
    rsx! {
        section { id: "sec-voxel", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("vox-tag") } }
                h1 { class: "page-title", { t!("vox-title") } }
                p { class: "page-desc", { t!("vox-desc") } }
            }
            div { class: "page-index", "05" }
        }

        // ── Overview ──────────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("vox-overview-title") } }
            p { class: "p-lead", { t!("vox-overview-lead") } }
            p { class: "p-muted", { t!("vox-overview-body") } }
        }

        // ── VoxelGrid data model ───────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("vox-grid-title") } }
            p { class: "p-muted", { t!("vox-grid-desc") } }
            ul { class: "ul-plain",
                li { { t!("vox-grid-li-1") } }
                li { { t!("vox-grid-li-2") } }
                li { { t!("vox-grid-li-3") } }
            }
        }

        // ── build_voxel_collider pipeline ─────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("vox-build-title") } }
            p { class: "p-lead", { t!("vox-build-lead") } }
            div { class: "code-block",
                pre { code {
                    "// mps-core::rapier::voxel\nlet grid = VoxelGrid::borrow(&cells, &dims, &origin, scale);\nlet collider = build_voxel_collider(&grid, /* density */);\nworld_add_collider(world, collider);"
                } }
            }
            p { class: "p-note", { t!("vox-build-note") } }
        }

        // ── Terrain gravity bridge ─────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("vox-terrain-title") } }
            p { class: "p-muted", { t!("vox-terrain-desc") } }
            ul { class: "ul-plain",
                li { { t!("vox-terrain-li-direct") } }
                li { { t!("vox-terrain-li-fft") } }
                li { { t!("vox-terrain-li-poly") } }
            }
        }

        // ── Use cases ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("vox-cases-title") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("vox-case-lunar-title") } }
                    p { { t!("vox-case-lunar-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("vox-case-terrain-title") } }
                    p { { t!("vox-case-terrain-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("vox-case-proximity-title") } }
                    p { { t!("vox-case-proximity-desc") } }
                }
            }
        }

        }
    }
}
