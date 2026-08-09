use dioxus::prelude::*;
use dioxus_i18n::t;

/// Box3D 批量碰撞体 — 批量插入 + 合并 + 物理感预设。
pub fn Batch() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("batch-tag") } }
                h1 { class: "page-title", { t!("batch-title") } }
                p { class: "page-desc", { t!("batch-desc") } }
            }
            div { class: "page-index", "13" }
        }

        // ── 管线概览 ──────────────────────────────────────────────────────
        div { class: "section-card",
            h2 { { t!("batch-pipeline-title") } }
            p { class: "p-lead", { t!("batch-pipeline-lead") } }
            div { class: "step-row",
                div { class: "step-circle", "1" }
                div { class: "step-body",
                    h3 { { t!("batch-step-1-title") } }
                    p { { t!("batch-step-1-desc") } }
                }
            }
            div { class: "step-row",
                div { class: "step-circle", "2" }
                div { class: "step-body",
                    h3 { { t!("batch-step-2-title") } }
                    p { { t!("batch-step-2-desc") } }
                }
            }
            div { class: "step-row",
                div { class: "step-circle", "3" }
                div { class: "step-body",
                    h3 { { t!("batch-step-3-title") } }
                    p { { t!("batch-step-3-desc") } }
                }
            }
            div { class: "step-row",
                div { class: "step-circle", "4" }
                div { class: "step-body",
                    h3 { { t!("batch-step-4-title") } }
                    p { { t!("batch-step-4-desc") } }
                }
            }
        }

        // ── ColliderRequest 字段 ──────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-request-title") } }
            p { class: "p-muted", { t!("batch-request-lead") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("batch-col-field") } }
                        th { { t!("batch-col-type") } }
                        th { { t!("batch-col-desc") } }
                    } }
                    tbody {
                        tr {
                            td { "shape" }
                            td { "ShapeDesc" }
                            td { { t!("batch-field-shape") } }
                        }
                        tr {
                            td { "translation" }
                            td { "Vec3" }
                            td { { t!("batch-field-translation") } }
                        }
                        tr {
                            td { "rotation" }
                            td { "Quat" }
                            td { { t!("batch-field-rotation") } }
                        }
                        tr {
                            td { "friction" }
                            td { "f64" }
                            td { { t!("batch-field-friction") } }
                        }
                        tr {
                            td { "restitution" }
                            td { "f64" }
                            td { { t!("batch-field-restitution") } }
                        }
                        tr {
                            td { "density" }
                            td { "f64" }
                            td { { t!("batch-field-density") } }
                        }
                        tr {
                            td { "collision_groups" }
                            td { "InteractionGroupsDesc" }
                            td { { t!("batch-field-collision-groups") } }
                        }
                        tr {
                            td { "solver_groups" }
                            td { "InteractionGroupsDesc" }
                            td { { t!("batch-field-solver-groups") } }
                        }
                        tr {
                            td { "body_parent" }
                            td { "RigidBodyHandleRaw" }
                            td { { t!("batch-field-body-parent") } }
                        }
                        tr {
                            td { "is_sensor" }
                            td { "Bool" }
                            td { { t!("batch-field-is-sensor") } }
                        }
                        tr {
                            td { "erosion_margin" }
                            td { "f64" }
                            td { { t!("batch-field-erosion-margin") } }
                        }
                    }
                }
            }
        }

        // ── Box3D 预设 ────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-preset-title") } }
            p { class: "p-lead", { t!("batch-preset-lead") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("batch-preset-default-title") } }
                    p { { t!("batch-preset-default-desc") } }
                    div { class: "code-block",
                        pre { code {
                            "friction=0.6  restitution=0.2\ndensity=1.0  erosion=0.01\ndamping=0.05  ccd=1  solver=4"
                        } }
                    }
                }
                div { class: "feature-card",
                    h3 { { t!("batch-preset-sticky-title") } }
                    p { { t!("batch-preset-sticky-desc") } }
                    div { class: "code-block",
                        pre { code {
                            "friction=0.9  restitution=0.0\ndensity=1.0  erosion=0.01\ndamping=0.05  ccd=1  solver=4"
                        } }
                    }
                }
                div { class: "feature-card",
                    h3 { { t!("batch-preset-bouncy-title") } }
                    p { { t!("batch-preset-bouncy-desc") } }
                    div { class: "code-block",
                        pre { code {
                            "friction=0.3  restitution=0.8\ndensity=0.8  erosion=0.005\ndamping=0.02  ccd=2  solver=8"
                        } }
                    }
                }
            }
        }

        // ── 合并策略 ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-merge-title") } }
            p { class: "p-lead", { t!("batch-merge-lead") } }
            div { class: "table-wrap",
                table {
                    thead { tr {
                        th { { t!("batch-col-scenario") } }
                        th { { t!("batch-col-result") } }
                    } }
                    tbody {
                        tr {
                            td { { t!("batch-merge-same-material") } }
                            td { { t!("batch-merge-compound") } }
                        }
                        tr {
                            td { { t!("batch-merge-diff-material") } }
                            td { { t!("batch-merge-separate") } }
                        }
                        tr {
                            td { { t!("batch-merge-dynamic-parent") } }
                            td { { t!("batch-merge-attach") } }
                        }
                        tr {
                            td { { t!("batch-merge-sensor") } }
                            td { { t!("batch-merge-sensor-result") } }
                        }
                    }
                }
            }
        }

        // ── 侵蚀 (Erosion) ────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-erosion-title") } }
            p { class: "p-lead", { t!("batch-erosion-lead") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { "Cuboid → RoundCuboid" }
                    p { { t!("batch-erosion-cuboid") } }
                }
                div { class: "feature-card",
                    h3 { "Cylinder → RoundCylinder" }
                    p { { t!("batch-erosion-cylinder") } }
                }
                div { class: "feature-card",
                    h3 { "Cone → RoundCone" }
                    p { { t!("batch-erosion-cone") } }
                }
            }
            p { class: "p-note", { t!("batch-erosion-note") } }
        }

        // ── FFI 入口 ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-ffi-title") } }
            p { class: "p-muted", { t!("batch-ffi-lead") } }
            div { class: "code-block",
                pre { code {
                    "// 批量插入碰撞体\nuint32_t world_batch_add_colliders(\n    WorldHandle* world,\n    const ColliderRequest* requests,\n    uint32_t count,\n    Box3DPreset preset,\n    ColliderHandleRaw* out_handles,\n    uint32_t out_capacity);\n\n// 合并静态形状为单个 compound\nuint32_t world_merge_static_shapes(\n    WorldHandle* world,\n    const ColliderRequest* requests,\n    uint32_t count,\n    Box3DPreset preset,\n    ColliderHandleRaw* out_handles,\n    uint32_t out_capacity);\n\n// 预设构造器\nBox3DPreset box3d_preset_default(void);\nBox3DPreset box3d_preset_sticky(void);\nBox3DPreset box3d_preset_bouncy(void);"
                } }
            }
        }

        // ── 限制 ──────────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-limits-title") } }
            ul { class: "ul-plain",
                li { { t!("batch-limit-max-requests") } }
                li { { t!("batch-limit-max-compound") } }
                li { { t!("batch-limit-erosion-zero") } }
            }
        }

        // ── 使用示例 ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("batch-example-title") } }
            p { class: "p-muted", { t!("batch-example-lead") } }
            div { class: "code-block",
                pre { code {
                    "// Rust 端：构造 3 个球体请求\nlet requests = [\n    ColliderRequest {{\n        shape: ShapeDesc {{ shape_type: 0, a: 0.5, ..Default::default() }},\n        translation: Vec3 {{ x: 0.0, y: 0.0, z: 0.0 }},\n        ..Default::default()\n    }},\n    // ... 两个更多\n];\nlet preset = Box3DPreset::box3d_default();\nlet handles = world.inner.batch_add_colliders(&requests, &preset);\n// 同材质 → 合并为 1 个 compound（3 个球）"
                } }
            }
        }
    }
}
