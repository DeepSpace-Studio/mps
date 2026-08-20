use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, t};

use crate::metrics::VERSION;

/// Single-page doc theme — calm deep-space, generous whitespace, refined cards.
/// Native in-page anchor scrolling (no router, no client JS).
const CSS: &str = include_str!("site.css");

/// Sticky top table of contents. Every link is a plain `<a href="#sec-...">`
/// — the browser performs the scroll natively, so navigation can never be
/// swallowed by a hydration handler (this site is SSR-only).
#[component]
pub fn Toc() -> Element {
    let i18n = i18n();
    let zh = i18n.language() == "zh-CN";
    rsx! {
        style { {CSS} }
        div { class: "starfield-bg" }
        nav { class: "toc",
            a { class: "toc-brand", href: "#sec-home", "MPS", small { "RIGID BODY v{VERSION}" } }
            a { class: "toc-link", href: "#sec-home", { t!("nav-home") } }
            a { class: "toc-link", href: "#sec-quickstart", { t!("nav-quickstart") } }
            a { class: "toc-link", href: "#sec-architecture", { t!("nav-architecture") } }
            a { class: "toc-link", href: "#sec-gravity", { t!("nav-gravity") } }
            a { class: "toc-link", href: "#sec-integrators", { t!("nav-integrators") } }
            a { class: "toc-link", href: "#sec-formula", { t!("nav-formula") } }
            a { class: "toc-link", href: "#sec-voxel", { t!("nav-voxel") } }
            a { class: "toc-link", href: "#sec-events", { t!("nav-events") } }
            a { class: "toc-link", href: "#sec-arena", { t!("nav-arena") } }
            a { class: "toc-link", href: "#sec-cosmos", { t!("nav-cosmos") } }
            a { class: "toc-link", href: "#sec-jni", { t!("nav-jni") } }
            a { class: "toc-link", href: "#sec-ffm", { t!("nav-ffm") } }
            a { class: "toc-link", href: "#sec-api", { t!("nav-api") } }
            span { class: "toc-lang",
                a { class: if zh { "toc-lang-btn is-active" } else { "toc-lang-btn" }, href: "/?lang=zh-CN", "中" }
                a { class: if !zh { "toc-lang-btn is-active" } else { "toc-lang-btn" }, href: "/?lang=en", "EN" }
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
