use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::{CORE_FFI_COUNT, FFI_COLLIDER, FFI_QUERY, FFI_RIGID_BODY, FFI_WORLD};

/// API Reference — the C ABI surface of `mps-core` as exported in
/// `crates/mps-core/include/rigid_body.h` by cbindgen.
pub fn Api() -> Element {
    
    rsx! {
        section { id: "sec-api", class: "doc-section",

        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("api-tag") } }
                h1 { class: "page-title", { t!("api-title") } }
                p { class: "page-desc", { t!("api-desc", total: CORE_FFI_COUNT) } }
            }
            div { class: "page-index", "14" }
        }

        // ── Header surface ──────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("api-header-title") } }
            p { class: "p-lead", { t!("api-header-lead", total: CORE_FFI_COUNT) } }
            p { class: "p-muted", { t!("api-header-body") } }
            div { class: "code-block",
                pre { code {
                    "// crates/mps-core/include/rigid_body.h\n// Generated with cbindgen:0.29.4 — do not edit by hand.\n#include <stdbool.h>\n#include <stdint.h>\n#include <stdlib.h>\n\ntypedef struct WorldHandle WorldHandle;\nuint32_t world_create(double dt, uint32_t iters, uint32_t ccd);"
                } }
            }
        }

        // ── Function prefix breakdown ───────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-prefix-title") } }
            p { class: "p-lead", { t!("api-prefix-lead") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("api-col-prefix") } }
                        th { { t!("api-col-count") } }
                        th { { t!("api-col-domain") } }
                    } }
                    tbody {
                        tr { td { "world_*" } td { { FFI_WORLD } } td { { t!("api-row-world") } } }
                        tr { td { "rigid_body_*" } td { { FFI_RIGID_BODY } } td { { t!("api-row-rigid") } } }
                        tr { td { "collider_*" } td { { FFI_COLLIDER } } td { { t!("api-row-collider") } } }
                        tr { td { "query_*" } td { { FFI_QUERY } } td { { t!("api-row-query") } } }
                    }
                }
            }
            p { class: "p-note", { t!("api-prefix-note") } }
        }

        // ── Common handle types ────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-handles-title") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("api-col-type") } }
                        th { { t!("api-col-scope") } }
                    } }
                    tbody {
                        tr { td { "WorldHandle" } td { { t!("api-handle-world") } } }
                        tr { td { "RigidBodyHandleRaw" } td { { t!("api-handle-rigid") } } }
                        tr { td { "ColliderHandleRaw" } td { { t!("api-handle-collider") } } }
                        tr { td { "RigidBodyBuilderHandle" } td { { t!("api-handle-rb-build") } } }
                        tr { td { "ColliderBuilderHandle" } td { { t!("api-handle-col-build") } } }
                        tr { td { "JointBuilderHandle" } td { { t!("api-handle-joint") } } }
                        tr { td { "RTreeHandle" } td { { t!("api-handle-rtree") } } }
                        tr { td { "CRbTreeHandle" } td { { t!("api-handle-crbtree") } } }
                        tr { td { "CharacterControllerHandle" } td { { t!("api-handle-cc") } } }
                    }
                }
            }
        }

        // ── Flat record types ──────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-records-title") } }
            p { class: "p-lead", { t!("api-records-lead") } }
            ul { class: "ul-plain",
                li { { t!("api-record-vec3") } }
                li { { t!("api-record-quat") } }
                li { { t!("api-record-aabb") } }
                li { { t!("api-record-shape") } }
                li { { t!("api-record-event") } }
                li { { t!("api-record-filter") } }
            }
        }

        // ── Error reporting ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-error-title") } }
            p { class: "p-lead", { t!("api-error-lead") } }
            div { class: "code-block",
                pre { code {
                    "// 错误码 + 线程局部消息两件套\n#define ERR_OK              0\n#define ERR_NULL_POINTER    1\n#define ERR_INVALID_ARGUMENT 2\n#define ERR_INTERNAL        3\n\nuint32_t last_error_code(void);\nconst char *last_error_message(void);\nvoid last_error_clear(void);"
                } }
            }
            p { class: "p-note", { t!("api-error-note") } }
        }

        // ── World lifecycle ─────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-lifecycle-title") } }
            div { class: "code-block",
                pre { code {
                    r#"// 1. 创建 world
WorldHandle *w = world_create(dt, iters, ccd);

// 2. 构造刚体 + 碰撞体
RigidBodyBuilderHandle rb = rigid_body_builder_create(...);
RigidBodyHandleRaw h   = world_add_rigid_body(w, rb);

// 3. 步进
world_step(w);

// 4. 查询 + 读写状态
Vec3 p; rigid_body_translation_out(w, h, &p);

// 5. 销毁
world_destroy(w);"#
                } }
            }
        }

        // ── ABI stability ──────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("api-stability-title") } }
            ul { class: "ul-plain",
                li { { t!("api-stability-cbindgen") } }
                li { { t!("api-stability-repr") } }
                li { { t!("api-stability-version") } }
                li { { t!("api-stability-redline") } }
            }
        }
    
        }
    }
}
