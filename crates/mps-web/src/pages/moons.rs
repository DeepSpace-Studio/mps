//! 太阳系主行星规则卫星数据表（来自 `mps_formula::celestial_data::MOONS`）。
//!
//! 单页 SSR 内的一节，锚点 `#sec-moons`。数据在编译期从 `mps-formula`
//! 注入，避免写死、与物理库单一来源保持一致。

use crate::metrics::MOON_COUNT;
use dioxus::prelude::*;
use dioxus_i18n::t;
use mps_formula::celestial_data::MOONS;

/// 把米格式化为千米（带千分位近似）。
fn km(meters: f64) -> String {
    format!("{:.1}", meters / 1_000.0)
}

/// 把秒格式化为天。
fn days(seconds: f64) -> String {
    format!("{:.2}", seconds / 86_400.0)
}

/// 把 GM (m³/s²) 格式化为 10⁹ 单位。
fn gm_g(m: f64) -> String {
    format!("{:.3}", m / 1e9)
}

pub fn Moons() -> Element {
    rsx! {
        section { id: "sec-moons", class: "doc-section",
            div { class: "page-header",
                h1 { class: "page-title", { t!("moons-title") } }
                p { class: "page-desc", { t!("moons-desc", count: MOON_COUNT) } }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("moons-catalog-title") } }
                p { class: "p-lead", { t!("moons-catalog-lead") } }
                p { class: "p-muted", { t!("moons-source-note") } }

                table { class: "data-table",
                    thead {
                        tr {
                            th { { t!("moons-col-planet") } }
                            th { { t!("moons-col-name") } }
                            th { { t!("moons-col-gm") } }
                            th { { t!("moons-col-radius") } }
                            th { { t!("moons-col-sma") } }
                            th { { t!("moons-col-period") } }
                        }
                    }
                    tbody {
                        for moon in MOONS.iter() {
                            tr {
                                td { { moon.parent_planet } }
                                td { { moon.name } }
                                td { { gm_g(moon.gm) } }
                                td { { km(moon.radius) } }
                                td { { km(moon.semi_major_axis) } }
                                td { { days(moon.orbital_period) } }
                            }
                        }
                    }
                }
            }

            div { class: "section-divider",
                h2 { class: "section-heading", { t!("moons-ffi-title") } }
                p { class: "p-lead", { t!("moons-ffi-lead") } }
                pre { code { { t!("moons-ffi-body") } } }
            }
        }
    }
}
