use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, t};

use crate::metrics::VERSION;

/// Single-page doc theme — GitBook-style: fixed left sidebar + content column.
/// Rich but tasteful: deep-space panel, grouped nav, refined cards.
/// Native in-page anchor scrolling (no router, no client JS).
const CSS: &str = include_str!("site.css");

/// Fixed left sidebar with grouped navigation. Every link is a plain
/// `<a href="#sec-...">` — the browser performs the scroll natively, so
/// navigation can never be swallowed by a hydration handler (SSR-only site).
#[component]
pub fn Sidebar() -> Element {
    let i18n = i18n();
    let zh = i18n.language() == "zh-CN";

    rsx! {
        style { {CSS} }
        div { class: "starfield-bg" }
        aside { class: "sidebar",
            div { class: "sidebar-brand",
                span { class: "sb-mark", "MPS" }
                span { class: "sb-sub", "RIGID BODY" }
                span { class: "sb-ver", "v{VERSION}" }
            }
            nav { class: "sidebar-nav",
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-overview") } }
                    a { class: "nav-link", href: "#sec-home", { t!("nav-home") } }
                    a { class: "nav-link", href: "#sec-quickstart", { t!("nav-quickstart") } }
                    a { class: "nav-link", href: "#sec-architecture", { t!("nav-architecture") } }
                }
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-core") } }
                    a { class: "nav-link", href: "#sec-gravity", { t!("nav-gravity") } }
                    a { class: "nav-link", href: "#sec-integrators", { t!("nav-integrators") } }
                    a { class: "nav-link", href: "#sec-events", { t!("nav-events") } }
                    a { class: "nav-link", href: "#sec-arena", { t!("nav-arena") } }
                    a { class: "nav-link", href: "#sec-batch", { t!("nav-batch") } }
                    a { class: "nav-link", href: "#sec-voxel", { t!("nav-voxel") } }
                    a { class: "nav-link", href: "#sec-soft-body", { t!("nav-soft-body") } }
                    a { class: "nav-link", href: "#sec-api", { t!("nav-api") } }
                }
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-cosmos") } }
                    a { class: "nav-link", href: "#sec-cosmos", { t!("nav-cosmos") } }
                    a { class: "nav-link", href: "#sec-cosmos-class", { t!("nav-cosmos-class") } }
                }
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-formula") } }
                    a { class: "nav-link", href: "#sec-formula", { t!("nav-formula") } }
                }
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-jni") } }
                    a { class: "nav-link", href: "#sec-jni", { t!("nav-jni") } }
                }
                details { class: "nav-group", open: true,
                    summary { class: "nav-group-title", { t!("nav-group-ffm") } }
                    a { class: "nav-link", href: "#sec-ffm", { t!("nav-ffm") } }
                }
            }
            div { class: "sidebar-foot",
                a { class: if zh { "lang-btn is-active" } else { "lang-btn" }, href: "/?lang=zh-CN", "中" }
                a { class: if !zh { "lang-btn is-active" } else { "lang-btn" }, href: "/?lang=en", "EN" }
            }
        }
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "site-footer",
            p {
                { t!("footer-text", version: VERSION) }
                " — "
                a { href: "https://github.com/Polari-Stars-MC/rigid-body", class: "link", "GitHub" }
            }
        }
    }
}
