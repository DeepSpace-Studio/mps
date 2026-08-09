use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::TEST_COUNT;

/// Quickstart page — 5-step setup guide
pub fn Quickstart() -> Element {
    let steps = [
        ("1", "quickstart-step1-title", "quickstart-step1-desc"),
        ("2", "quickstart-step2-title", "quickstart-step2-desc"),
        ("3", "quickstart-step3-title", "quickstart-step3-desc"),
        ("4", "quickstart-step4-title", "quickstart-step4-desc"),
        ("5", "quickstart-step5-title", "quickstart-step5-desc"),
    ];

    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("quickstart-tag") } }
                h1 { class: "page-title", { t!("quickstart-title") } }
                p { class: "page-desc", { t!("quickstart-desc") } }
            }
            div { class: "page-index", "01" }
        }

        div { class: "section-card",
            h2 { { t!("quickstart-title") } }

            { steps.iter().map(|(num, title_key, desc_key)| rsx! {
                div { class: "step-row",
                    div { class: "step-circle", { *num } }
                    h3 { class: "step-title", { t!(title_key) } }
                }
                div { class: "step-body",
                    p { class: "p-lead", { t!(desc_key, tests: TEST_COUNT) } }
                }
            })}
        }
    }
}
