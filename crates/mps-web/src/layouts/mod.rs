use dioxus::prelude::*;
use dioxus_i18n::prelude::*;

use crate::Route;
use crate::i18n::{langs, t};
use crate::metrics::VERSION;

/// Global CSS for the entire site — dark theme, responsive grid, no inline styles.
const CSS: &str = include_str!("site.css");

/// Root layout — wraps all pages with HTML skeleton, header, footer.
///
/// Orbital nav architecture (pure CSS, SSR-works):
///   - Home (`/`) renders the galaxy: star (the home index) at center + 2
///     rotating orbits holding 7 primary + 7 secondary pages as planets.
///     The home page body (hero / metrics / directory) renders *underneath*
///     the starfield.
///   - Every other page renders as a bottom sheet modal (`.page-sheet.open`)
///     that slides up from below with a "× 返回星图" close anchor pointing
///     back at `/`. Pure `<a href>` navigation — no client hydration needed.
///
/// `onClick`/`onchange` handlers were removed because the SSR build ships
/// no Dioxus client JS bundle, so those events never bind in the browser.
/// All interactivity is anchor navigation + CSS animations (@keyframes).
/// `use_route::<Route>()` is SSR-safe (Dioxus resolves it during render).
#[component]
pub fn Layout() -> Element {
    let i18n = i18n();
    let route: Route = use_route();

    // Home → galaxy page; everything else → bottom sheet modal.
    let is_home = matches!(route, Route::Home {});

    // Active-language class bits for the two language links — precomputed
    // because rsx! format-arg parsing does not accept inline `if … { "…" }`
    // inside a class string literal.
    let zh_active = if i18n.language() == "zh-CN" { " is-active" } else { "" };
    let en_active = if i18n.language() == "en" { " is-active" } else { "" };

    rsx! {
        style { {CSS} }

        // ── Minimal top bar: brand + language links + back-to-galaxy ──────
        // Language switching is anchor-based (?lang=en) so it works under
        // SSR without any JS. The page reloads with the new locale.
        header { class: "orbital-header",
            a { class: "orbital-brand", href: "/",
                span { class: "orbital-brand-badge", "MPS" }
                span { class: "orbital-brand-ver", "PHYSICS / {VERSION}" }
            }
            div { class: "orbital-lang-group",
                a { class: "orbital-lang{zh_active}", href: "/?lang=zh-CN", "中" }
                a { class: "orbital-lang{en_active}", href: "/?lang=en", "EN" }
            }
            if !is_home {
                a { class: "orbital-back", href: "/", "⌂ 返回星图" }
            }
        }

        // ── Home: galaxy (rendered by home.rs via Outlet) ────────────────
        // ── Other: bottom sheet modal wrapping the page content ───────────
        if is_home {
            main { class: "mps-main mps-main-home",
                Outlet::<Route> {}
            }
        } else {
            div { class: "page-sheet open",
                a { class: "modal-close", href: "/", "× 返回星图" }
                main { class: "mps-main mps-main-sheet",
                    Outlet::<Route> {}
                }
            }
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
