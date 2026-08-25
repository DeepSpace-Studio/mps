use dioxus::prelude::*;
use dioxus_i18n::t;

/// 404 page — route not found.
pub fn NotFound() -> Element {
    rsx! {
        div { class: "hero",
            h1 { class: "hero-title", { t!("not-found-title") } }
            p { class: "hero-desc",
                { t!("not-found-desc") }
            }
            a { href: "/", class: "btn-primary",
                { t!("not-found-back") }
            }
        }
    }
}
