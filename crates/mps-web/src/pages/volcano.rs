use dioxus::prelude::*;
use dioxus_i18n::t;

/// Volcano — eruptive plume flow, lava-bomb ejecta, heating and progressive
/// melting of rigid bodies.
pub fn Volcano() -> Element {
    rsx! {
        section { id: "sec-volcano", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("vol-tag") } }
                    h1 { class: "page-title", { t!("vol-title") } }
                    p { class: "page-desc", { t!("vol-desc") } }
                }
                div { class: "page-index", "09·B" }
            }

            // ── Models ──────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("vol-model-title") } }
                p { class: "p-lead", { t!("vol-model-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("vol-model-plume-title") } }
                        p { { t!("vol-model-plume-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("vol-model-bomb-title") } }
                        p { { t!("vol-model-bomb-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("vol-model-melt-title") } }
                        p { { t!("vol-model-melt-desc") } }
                    }
                }
            }

            // ── Melt pipeline ───────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("vol-melt-title") } }
                p { class: "p-lead", { t!("vol-melt-lead") } }
                div { class: "code-block",
                    pre { code {
                        r#"dT/dt = heatExchangeRate · (T_exposure − T_body)     // 牛顿热交换
loss  = meltRate · (T − meltPoint) / meltPoint        // 超热线性质量损失
mass  = originalMass · massScale                      // set_additional_mass
massScale ≤ 5%  →  set_enabled(false)，句柄写入 out_melted"#
                    } }
                }
                p { class: "p-note", { t!("vol-melt-note") } }
            }

            // ── C ABI ───────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("vol-api-title") } }
                div { class: "code-block",
                    code {
                        "volcano_add(world, vent, plume_radius, plume_height, max_updraft, lava_temperature, ejecta_rate, out_id) -> u8\n"
                        "volcano_remove(world, id) / volcano_clear(world) -> u8\n"
                        "volcano_advance(world, dt) -> u8\n"
                        "volcano_sample_flow(world, point, out_flow) -> u8\n"
                        "volcano_sample_temperature(world, point, out_temp) -> u8\n"
                        "volcano_body_temperature(world, body, out_temp) -> u8\n"
                        "volcano_set_body_temperature(world, body, temperature) -> u8\n"
                        "volcano_apply(world, dt, melt_point, melt_rate, heat_exchange_rate, wake, out_bombs, out_melted, capacity, out_melted_count) -> u8\n"
                    }
                }
                p { class: "p-note", { t!("vol-api-note") } }
            }

            // ── Java example ────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("vol-java-title") } }
                p { class: "p-lead", { t!("vol-java-lead") } }
                div { class: "code-block",
                    pre { code {
                        r#"// Java —— RapierNative。火山每帧两步：advance → apply → worldStep
long world = RapierNative.worldCreate(0, -9.81, 0);
long body  = ...; // 木制小车（熔点低的可燃/可熔物体）

// 1) 注册一座火山：柱底半径 200 m、柱高 2000 m、核心上升气流 60 m/s、
//    熔岩温度 0 → 玄武岩默认 1473.15 K（~1200 °C），每秒 3 发火山弹
long volcano = RapierNative.volcanoAdd(
        world,
        0, 0, 0,           // vent x/y/z（喷口，柱轴垂直）
        200.0, 2000.0,     // plumeRadius (m), plumeHeight (m)
        60.0,              // maxUpdraft (m/s)
        0.0,               // lavaTemperature (K, 0 = 默认)
        3.0,               // ejectaRate（火山弹/秒，0 = 关闭弹射）
        pVolcanoId);

// 2) 可视化：采样烟柱流场与暴露温度
RapierNative.volcanoSampleFlow(world, 0, 500, 0, pFlow);        // out: 3×f64
RapierNative.volcanoSampleTemperature(world, 0, 500, 0, pTemp); // out: f64 (K)

// 3) 脚本化预热（可选）：把某物体直接置为炽热状态
RapierNative.volcanoSetBodyTemperature(world, body, 900.0);

// 4) 每帧：加热 + 熔化。木车熔点 600 K，熔化速率 1.0，交换率 0.8 (1/s)
RapierNative.volcanoAdvance(world, dt);
RapierNative.volcanoApply(world,
        dt,
        600.0,   // meltPoint (K)
        1.0,     // meltRate（2 倍超热时每秒损失的比例）
        0.8,     // heatExchangeRate (1/s)
        1,       // wakeUp
        pBombs,          // out: 本次火山弹数
        pMelted, 4,      // out: 熔化句柄槽（打包句柄）+ 容量
        pMeltedCount);   // out: 本次熔化数（"本次调用"语义）
RapierNative.worldStep(world, dt);

// 5) 读取刚体温度做 UI 血条 / 冒烟特效
RapierNative.volcanoBodyTemperature(world, body, pTemp);

// 6) 喷发结束
RapierNative.volcanoRemove(world, volcano);"#
                    } }
                }
                p { class: "p-note", { t!("vol-java-note") } }
            }

            // ── Stale-property notes ────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("vol-det-title") } }
                p { class: "p-lead", { t!("vol-det-lead") } }
                div { class: "callout-note",
                    p { { t!("vol-det-body") } }
                }
            }
        }
    }
}
