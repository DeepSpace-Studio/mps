use dioxus::prelude::*;
use dioxus_i18n::prelude::*;

use crate::Route;
use crate::i18n::t;
use crate::metrics::VERSION;

/// Literal client path for each route. Used to render plain `<a href>` nav
/// links instead of Dioxus `Link`. This site is served **SSR-only** (no WASM
/// client bundle — see build.rs / no `dx` CLI), so `Link` would attach a
/// client-side click interceptor that never hydrates and silently eats
/// navigation. Plain anchors always perform a native full-page load, which the
/// server renders correctly for every route.
fn route_path(r: &Route) -> &'static str {
    match r {
        Route::Home {} => "/",
        Route::Quickstart {} => "/quickstart",
        Route::Architecture {} => "/architecture",
        Route::Gravity {} => "/gravity",
        Route::Integrators {} => "/integrators",
        Route::Formula {} => "/formula",
        Route::Voxel {} => "/voxel",
        Route::Events {} => "/events",
        Route::Arena {} => "/arena",
        Route::Batch {} => "/batch",
        Route::Cosmos {} => "/cosmos",
        Route::Jni {} => "/jni",
        Route::Ffm {} => "/ffm",
        Route::Api {} => "/api",
        Route::NotFound {} => "/404",
    }
}

/// Global CSS for the entire site — deep-space sci-fi theme, responsive sidebar,
/// no inline styles. Restyled from the old "orbital galaxy" layout (which was
/// fragile — z-index click bugs + a non-clickable rotating nav). The new layout
/// is a fixed left "constellation" sidebar (pure `<a href>`, SSR-safe, no client
/// hydration needed) + a scrolling main panel over an animated starfield.
const CSS: &str = include_str!("site.css");

/// Grouped navigation model. Each link is a real `<a href>` (Dioxus `Link`
/// renders to `<a>` under fullstack SSR) so navigation works with zero JS.
struct NavItem {
    route: Route,
    label_key: &'static str,
}

struct NavGroup {
    title: &'static str,
    items: &'static [NavItem],
}

const NAV: &[NavGroup] = &[
    NavGroup {
        title: "导航 / NAVIGATION",
        items: &[
            NavItem {
                route: Route::Home {},
                label_key: "nav-home",
            },
            NavItem {
                route: Route::Quickstart {},
                label_key: "nav-quickstart",
            },
            NavItem {
                route: Route::Architecture {},
                label_key: "nav-architecture",
            },
        ],
    },
    NavGroup {
        title: "物理引擎 / PHYSICS",
        items: &[
            NavItem {
                route: Route::Gravity {},
                label_key: "nav-gravity",
            },
            NavItem {
                route: Route::Integrators {},
                label_key: "nav-integrators",
            },
            NavItem {
                route: Route::Formula {},
                label_key: "nav-formula",
            },
            NavItem {
                route: Route::Voxel {},
                label_key: "nav-voxel",
            },
            NavItem {
                route: Route::Events {},
                label_key: "nav-events",
            },
            NavItem {
                route: Route::Arena {},
                label_key: "nav-arena",
            },
        ],
    },
    NavGroup {
        title: "宇宙 / COSMOS",
        items: &[NavItem {
            route: Route::Cosmos {},
            label_key: "nav-cosmos",
        }],
    },
    NavGroup {
        title: "绑定 / BINDINGS",
        items: &[
            NavItem {
                route: Route::Jni {},
                label_key: "nav-jni",
            },
            NavItem {
                route: Route::Ffm {},
                label_key: "nav-ffm",
            },
            NavItem {
                route: Route::Api {},
                label_key: "nav-api",
            },
        ],
    },
];

#[component]
pub fn Layout() -> Element {
    let i18n = i18n();
    let current: Route = use_route();

    let zh_active = if i18n.language() == "zh-CN" {
        " is-active"
    } else {
        ""
    };
    let en_active = if i18n.language() == "en" {
        " is-active"
    } else {
        ""
    };

    rsx! {
        style { {CSS} }

        // Deep-space animated starfield backdrop (fixed, behind everything).
        div { class: "starfield-bg" }

        // ── Sidebar: brand + constellation nav + language toggle ──────────
        aside { class: "mps-sidebar",
            div { class: "mps-brand",
                a { class: "mps-brand-badge", href: "/", "MPS" }
                div { class: "mps-brand-meta",
                    span { class: "mps-brand-name", "RIGID BODY" }
                    span { class: "mps-brand-ver", "v{VERSION}" }
                }
            }

            nav { class: "mps-nav",
                for group in NAV {
                    div { class: "mps-nav-group",
                        div { class: "mps-nav-group-title", { group.title } }
                        for item in group.items {
                            a {
                                href: route_path(&item.route),
                                class: if current == item.route { "mps-nav-link is-active" } else { "mps-nav-link" },
                                { t!(item.label_key) }
                            }
                        }
                    }
                }
            }

            div { class: "mps-sidebar-foot",
                div { class: "mps-lang-group",
                    a { class: "mps-lang{zh_active}", href: "/?lang=zh-CN", "中" }
                    a { class: "mps-lang{en_active}", href: "/?lang=en", "EN" }
                }
                a {
                    class: "mps-repo",
                    href: "https://github.com/Polari-Stars-MC/rigid-body",
                    "GitHub ↗"
                }
            }
        }

        // ── Mobile top bar (only visible < breakpoint; pure CSS toggle) ────
        header { class: "mps-topbar",
            a { class: "mps-brand-badge", href: "/", "MPS" }
            span { class: "mps-topbar-title", "RIGID BODY" }
            label { class: "mps-burger", r#for: "mps-nav-toggle", "☰" }
        }
        input { class: "mps-nav-toggle", id: "mps-nav-toggle", r#type: "checkbox" }

        // ── Main content ───────────────────────────────────────────────────
        main { class: "mps-main",
            Outlet::<Route> {}
        }

        footer { class: "mps-footer",
            p {
                { t!("footer-text", version: VERSION) }
                " — "
                a { href: "https://github.com/Polari-Stars-MC/rigid-body", class: "link", "GitHub" }
            }
        }
    }
}
