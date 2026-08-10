use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::Route;
use crate::metrics::{
    CELESTIAL_COUNT, CORE_FFI_COUNT, FORMULA_MODULE_COUNT, GRAVITY_MODEL_COUNT, INTEGRATOR_COUNT,
    JNI_METHOD_COUNT, TEST_COUNT,
};

/// Home page — MPS Physics System overview.
///
/// Layout: a galaxy (= the home page index = the "sun") + 13 orbiting planets
/// representing every other page. Pure CSS orbital animation drives the rings;
/// every planet is a `<Link>` rendering as `<a href>` so navigation works
/// under pure SSR with no client hydration bundle. The original hero / metric
/// / directory / features content is preserved below the starfield.
///
/// DOM structure for each planet (four nesting layers — each layer has a
/// single transform-related job so CSS animations never fight static
/// transforms; see site.css §Galaxy comments):
///   `.orbit-spin`  → rotates the whole ring (CW/CCW)
///   `.planet-wrap` → counter-rotates so the label stays upright
///   `.planet-pos`  → holds the radial `translateX(R)` offset via
///                    `:nth-of-type` static transform. MUST be a separate
///                    element — `.planet-wrap`'s animation overrides any
///                    static `transform` on that same element, which would
///                    otherwise erase the radial offset and pile every
///                    planet on the galaxy center.
///   `.planet`      → the visible ball, centered on the `.planet-pos` anchor
pub fn Home() -> Element {
    rsx! {
        // ── Galaxy (embedded inside the home page) ───────────────────────
        div { class: "starfield",
            div { class: "galaxy",
                // Inner ring: 7 primary planets.
                div { class: "orbit ring-inner",
                    div { class: "orbit-spin",
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Home {},
                                    class: "planet p-primary active",
                                    { t!("nav-planet-home") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Quickstart {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-quickstart") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Architecture {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-architecture") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Gravity {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-gravity") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Integrators {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-integrators") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Formula {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-formula") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Api {},
                                    class: "planet p-primary",
                                    { t!("nav-planet-api") }
                                }
                            }
                        }
                    }
                }

                // Outer ring: 7 secondary planets, slower counter-spin.
                div { class: "orbit ring-outer",
                    div { class: "orbit-spin",
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Voxel {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-voxel") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Events {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-events") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Arena {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-arena") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Batch {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-batch") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Cosmos {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-cosmos") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Jni {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-jni") }
                                }
                            }
                        }
                        div { class: "planet-wrap",
                            div { class: "planet-pos",
                                Link { to: Route::Ffm {},
                                    class: "planet p-secondary",
                                    { t!("nav-planet-ffm") }
                                }
                            }
                        }
                    }
                }

                // ── The Sun (Home index) — central, glows, click → home ──
                Link { to: Route::Home {}, class: "star", "★",
                    span { class: "star-label", "MPS" }
                }
            }
        }

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
                div { class: "stat-card", span { class: "num", "88" }, span { class: "label", { t!("formula-cat-spaceflight") } } }
                div { class: "stat-card", span { class: "num", "23" }, span { class: "label", { t!("formula-cat-nuclear") } } }
                div { class: "stat-card", span { class: "num", "26" }, span { class: "label", { t!("formula-cat-mechanics") } } }
                div { class: "stat-card", span { class: "num", "19" }, span { class: "label", { t!("formula-cat-astrophysics") } } }
                div { class: "stat-card", span { class: "num", "23" }, span { class: "label", { t!("formula-cat-relativity") } } }
                div { class: "stat-card", span { class: "num", "20" }, span { class: "label", { t!("formula-cat-quantum") } } }
                div { class: "stat-card", span { class: "num", "16" }, span { class: "label", { t!("formula-cat-electromagnetism") } } }
                div { class: "stat-card", span { class: "num", "18" }, span { class: "label", { t!("formula-cat-fluid") } } }
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
