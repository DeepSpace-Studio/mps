use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::metrics::{
    CELESTIAL_COUNT, CORE_FFI_COUNT, FORMULA_MODULE_COUNT, GRAVITY_MODEL_COUNT,
    INTEGRATOR_COUNT, JNI_METHOD_COUNT, TEST_COUNT,
};
use crate::Route;

/// Home page — MPS Physics System overview
pub fn Home() -> Element {
    rsx! {
        div { class: "hero",
            div { class: "hero-tag", { t!("home-hero-tag") } }
            h1 { class: "hero-title", { t!("home-hero-title") } }
            p { class: "hero-desc",
                { t!("home-hero-desc",
                    rapier: "Rapier3D-f64",
                    ffi: CORE_FFI_COUNT,
                    jni: JNI_METHOD_COUNT,
                    tests: TEST_COUNT,
                    gravity: GRAVITY_MODEL_COUNT,
                    integrators: INTEGRATOR_COUNT,
                    modules: FORMULA_MODULE_COUNT,
                    bodies: CELESTIAL_COUNT
                )}
            }
            div { class: "hero-actions",
                Link { to: Route::Quickstart {}, class: "btn-primary",
                    { t!("home-cta-quickstart") }
                }
                Link { to: Route::Api {}, class: "btn-outline",
                    { t!("home-cta-api") }
                }
            }
        }

        div { class: "metric-grid",
            div { class: "metric-card",
                strong { class: "num", { TEST_COUNT } }
                span { class: "label", { t!("home-stat-tests") } }
            }
            div { class: "metric-card",
                strong { class: "num", "300+" }
                span { class: "label", { t!("home-stat-formula-fns") } }
            }
            div { class: "metric-card",
                strong { class: "num", { FORMULA_MODULE_COUNT } }
                span { class: "label", { t!("home-stat-formula-modules") } }
            }
            div { class: "metric-card",
                strong { class: "num", { CELESTIAL_COUNT } }
                span { class: "label", { t!("home-stat-celestial") } }
            }
        }

        div { class: "text-center section-divider",
            div { class: "hero-tag", "/ MODULE DIRECTORY" }
            h2 { class: "section-heading-lg", { t!("home-section-directory") } }

            div { class: "module-grid",
                Link { to: Route::Architecture {}, class: "module-card",
                    span { class: "idx", "01" }
                    strong { class: "title", { t!("home-mod-core-title") } }
                    small { class: "desc", { t!("home-mod-core-desc") } }
                    em { class: "arrow", "↗" }
                }
                Link { to: Route::Cosmos {}, class: "module-card",
                    span { class: "idx", "06" }
                    strong { class: "title", { t!("home-mod-cosmos-title") } }
                    small { class: "desc", { t!("home-mod-cosmos-desc") } }
                    em { class: "arrow", "↗" }
                }
                Link { to: Route::Gravity {}, class: "module-card",
                    span { class: "idx", "02" }
                    strong { class: "title", { t!("home-mod-physics-title") } }
                    small { class: "desc", { t!("home-mod-physics-desc") } }
                    em { class: "arrow", "↗" }
                }
                Link { to: Route::Formula {}, class: "module-card",
                    span { class: "idx", "03" }
                    strong { class: "title", { t!("home-mod-formula-title") } }
                    small { class: "desc", { t!("home-mod-formula-desc") } }
                    em { class: "arrow", "↗" }
                }
                Link { to: Route::Arena {}, class: "module-card",
                    span { class: "idx", "04" }
                    strong { class: "title", { t!("home-mod-integration-title") } }
                    small { class: "desc", { t!("home-mod-integration-desc") } }
                    em { class: "arrow", "↗" }
                }
                Link { to: Route::Api {}, class: "module-card",
                    span { class: "idx", "05" }
                    strong { class: "title", { t!("home-mod-reference-title") } }
                    small { class: "desc", { t!("home-mod-reference-desc") } }
                    em { class: "arrow", "↗" }
                }
            }
        }

        div { class: "section-divider",
            h2 { class: "section-heading",
                { t!("home-section-formula-modules", count: FORMULA_MODULE_COUNT) }
            }
            div { class: "mini-stat-grid",
                div { class: "stat-card", span { class: "num", "88" }, span { class: "label", "Spaceflight" } }
                div { class: "stat-card", span { class: "num", "23" }, span { class: "label", "Nuclear" } }
                div { class: "stat-card", span { class: "num", "26" }, span { class: "label", "Mechanics" } }
                div { class: "stat-card", span { class: "num", "19" }, span { class: "label", "Astrophysics" } }
                div { class: "stat-card", span { class: "num", "23" }, span { class: "label", "Relativity" } }
                div { class: "stat-card", span { class: "num", "20" }, span { class: "label", "Quantum" } }
                div { class: "stat-card", span { class: "num", "16" }, span { class: "label", "Electromagnetism" } }
                div { class: "stat-card", span { class: "num", "18" }, span { class: "label", "Fluid Dynamics" } }
            }
        }

        div { class: "callout",
            p { { t!("home-callout", crate: "mps-formula") } }
        }

        div { class: "section-divider",
            h2 { class: "section-heading", { t!("home-section-key-features") } }
            div { class: "feature-grid",
                div { class: "feature-card",
                    h3 { { t!("home-feat-gravity-title") } }
                    p { { t!("home-feat-gravity-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("home-feat-integrators-title") } }
                    p { { t!("home-feat-integrators-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("home-feat-celestial-title") } }
                    p { { t!("home-feat-celestial-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("home-feat-terrain-title") } }
                    p { { t!("home-feat-terrain-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("home-feat-registry-title") } }
                    p { { t!("home-feat-registry-desc") } }
                }
                div { class: "feature-card",
                    h3 { { t!("home-feat-jni-title", count: JNI_METHOD_COUNT) } }
                    p { { t!("home-feat-jni-desc", count: JNI_METHOD_COUNT) } }
                }
            }
        }

        div { class: "section-divider",
            h2 { class: "section-heading", { t!("home-section-architecture") } }
            pre { code {
"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI ({CORE_FFI_COUNT} functions)
       ├─ mps-formula  — 33 pure formula modules (300+ functions)
       ├─ mps-core     — physics engine + Rapier wrapper (World, bodies, colliders, queries, events)
       ├─ mps-cosmos   — cosmos rigid body (separate world, Verlet orbit integration)
       ├─ mps-jni      — JNI bindings ({JNI_METHOD_COUNT} methods, incl. cosmos batch)
       ├─ mps-ffm      — FFM metadata
       └─ mps-test     — integration tests (incl. cosmos 19)"
            }}
        }
    }
}
