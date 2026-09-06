use dioxus::prelude::*;
use dioxus_i18n::t;

/// Natural Disasters — typhoon / tornado / hailstorm field sources driving
/// wind drag, hail impacts and pressure drops on every dynamic rigid body.
pub fn Disasters() -> Element {
    rsx! {
        section { id: "sec-disasters", class: "doc-section",

            div { class: "page-head",
                div {
                    div { class: "page-tag", { t!("dis-tag") } }
                    h1 { class: "page-title", { t!("dis-title") } }
                    p { class: "page-desc", { t!("dis-desc") } }
                }
                div { class: "page-index", "09·A" }
            }

            // ── Models ──────────────────────────────────────────────────
            div { class: "section-card",
                h2 { { t!("dis-model-title") } }
                p { class: "p-lead", { t!("dis-model-lead") } }
                div { class: "feature-grid",
                    div { class: "feature-card",
                        h3 { { t!("dis-model-typhoon-title") } }
                        p { { t!("dis-model-typhoon-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("dis-model-tornado-title") } }
                        p { { t!("dis-model-tornado-desc") } }
                    }
                    div { class: "feature-card",
                        h3 { { t!("dis-model-hail-title") } }
                        p { { t!("dis-model-hail-desc") } }
                    }
                }
                p { class: "p-muted", { t!("dis-model-note") } }
            }

            // ── Frame order ─────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("dis-frame-title") } }
                p { class: "p-lead", { t!("dis-frame-lead") } }
                div { class: "code-block",
                    pre { code {
                        r#"// 每帧三步：推进灾害源 → 施加场力 → 物理 step
disasterAdvance(world, dt);                        // 风暴平移 / 冰雹 RNG
disasterApplyForces(world, air_density, drag, area, dt,
                    wake, out_bodies, out_impacts); // 风阻力 + 冰雹冲击
worldStep(world, dt);"#
                    } }
                }
                p { class: "p-note", { t!("dis-frame-note") } }
            }

            // ── C ABI ───────────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("dis-api-title") } }
                div { class: "code-block",
                    code {
                        "disaster_add_typhoon(world, center, max_wind, radius_max_wind, translation, spin, inflow, exponent, ref_height, out_id) -> u8\n"
                        "disaster_add_tornado(world, base, max_wind, core_radius, top_height, translation, spin, updraft, out_id) -> u8\n"
                        "disaster_add_hail(world, center, radius, intensity, hail_radius, out_id) -> u8\n"
                        "disaster_remove(world, id) / disaster_clear(world) -> u8\n"
                        "disaster_advance(world, dt) -> u8\n"
                        "disaster_sample_wind(world, point, out_wind) -> u8\n"
                        "disaster_sample_pressure_drop(world, point, air_density, out_drop) -> u8\n"
                        "disaster_apply_forces(world, air_density, drag, area, dt, wake, out_bodies, out_impacts) -> u8\n"
                    }
                }
                p { class: "p-note", { t!("dis-api-note") } }
            }

            // ── Java example ────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("dis-java-title") } }
                p { class: "p-lead", { t!("dis-java-lead") } }
                div { class: "code-block",
                    pre { code {
                        r#"// Java —— RapierNative（mps-jni 生成的前缀 org.polaris2023.mps.rapier）
// out 缓冲用 direct ByteBuffer / Unsafe.allocateMemory 分配
long world = RapierNative.worldCreate(0, -9.81, 0);

// 1) 一场西北太平洋台风：40 m/s 最大风速，30 km 最大风半径，+1 逆时针
//    inflow / exponent / refHeight 传 0 使用默认（0.15 / 0.11 / 10 m）
long id = RapierNative.disasterAddTyphoon(
        world,
        0, 0, 0,          // center x/y/z
        40.0, 30_000.0,   // maxWind (m/s), radiusMaxWind (m)
        8.0, 0, 3.0,      // translation：向东北移动
        1.0,              // spin = +1 北半球逆时针（南半球传 -1）
        0, 0, 0,          // inflow / heightExponent / refHeight（0 = 默认）
        pOutId);          // native u32 槽 ← 稳定源 id

// 2) 眼墙外一点采样风场与气压降（可视化 / UI 用）
RapierNative.disasterSampleWind(world, 30_000, 10, 0, pWind);   // out: 3×f64
RapierNative.disasterSamplePressureDrop(world, 30_000, 10, 0,
                                        1.225, pDrop);          // out: f64 (Pa)

// 3) 台风路径上再来一场冰雹：半径 500 m，每 m² 每秒 2 次冲击，雹径 2 cm
long hail = RapierNative.disasterAddHail(
        world, 5_000, 0, 0, 500.0, 2.0, 0.01, pHailId);

// 4) 每帧：推进 + 施加（计数器为"本次调用"语义，跨帧自行累加）
RapierNative.disasterAdvance(world, dt);
RapierNative.disasterApplyForces(world,
        1.225,  // airDensity（0 = ISA 默认）
        1.0,    // dragCoefficient
        2.5,    // referenceArea (m²)
        dt,
        1,      // wakeUp
        pBodies, pImpacts);
RapierNative.worldStep(world, dt);

// 5) 台风过境后移除
RapierNative.disasterRemove(world, id);"#
                    } }
                }
                p { class: "p-note", { t!("dis-java-note") } }
            }

            // ── Determinism ─────────────────────────────────────────────
            div { class: "section-divider",
                h2 { class: "section-heading", { t!("dis-det-title") } }
                p { class: "p-lead", { t!("dis-det-lead") } }
                div { class: "callout-note",
                    p { { t!("dis-det-body") } }
                }
            }
        }
    }
}
