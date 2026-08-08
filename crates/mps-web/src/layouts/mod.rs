use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::langid;

use crate::Route;
use crate::metrics::VERSION;
use crate::i18n::{t, langs};

/// Global CSS for the entire site — dark theme, responsive grid, no inline styles.
const CSS: &str = include_str!("site.css");

/// Root layout — wraps all pages with HTML skeleton, header, nav, footer.
/// Language switcher uses dioxus-i18n's I18n context (set_language).
#[component]
pub fn Layout(children: Element) -> Element {
    let mut i18n = i18n();

    let mut switch_lang = move |lang: unic_langid::LanguageIdentifier| {
        i18n.set_language(lang);
    };

    rsx! {
        style { {CSS} }

        header { class: "mps-header",
            a { class: "mps-brand", href: "/",
                span { class: "mps-brand-badge", "MPS" }
                span { class: "mps-brand-ver", "PHYSICS / {VERSION}" }
            }
            nav { class: "mps-nav",
                Link { to: Route::Home {}, class: "nav-link", { t!("nav-home") } }
                Link { to: Route::Quickstart {}, class: "nav-link", { t!("nav-quickstart") } }
                Link { to: Route::Architecture {}, class: "nav-link", { t!("nav-architecture") } }
                Link { to: Route::Gravity {}, class: "nav-link", { t!("nav-gravity") } }
                Link { to: Route::Integrators {}, class: "nav-link", { t!("nav-integrators") } }
                Link { to: Route::Formula {}, class: "nav-link", { t!("nav-formula") } }
                Link { to: Route::Voxel {}, class: "nav-link", { t!("nav-voxel") } }
                Link { to: Route::Events {}, class: "nav-link", { t!("nav-events") } }
                Link { to: Route::Arena {}, class: "nav-link", { t!("nav-arena") } }
                Link { to: Route::Cosmos {}, class: "nav-link", { t!("nav-cosmos") } }
                Link { to: Route::Jni {}, class: "nav-link", { t!("nav-jni") } }
                Link { to: Route::Ffm {}, class: "nav-link", { t!("nav-ffm") } }
                Link { to: Route::Api {}, class: "nav-link", { t!("nav-api") } }
            }
            div {
                select {
                    class: "lang-select",
                    value: "{i18n.language()}",
                    onchange: move |e: Event<FormData>| {
                        if let Some(lang) = langs::parse(&e.value()) {
                            switch_lang(lang);
                        }
                    },
                    option { value: "zh-CN", "中文" }
                    option { value: "en", "English" }
                }
            }
        }

        main { class: "mps-main",
            { children }
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
