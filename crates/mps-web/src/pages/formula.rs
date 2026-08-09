use dioxus::prelude::*;
use dioxus_i18n::t;

/// Formula Modules — 33 pure-Rust domain modules mapped to their category headings.
pub fn Formula() -> Element {
    rsx! {
        div { class: "page-head",
            div {
                div { class: "page-tag", { t!("form-tag") } }
                h1 { class: "page-title", { t!("form-title") } }
                p { class: "page-desc", { t!("form-desc") } }
            }
            div { class: "page-index", "04" }
        }

        div { class: "callout-note",
            p { { t!("form-intro-pure") } }
        }

        // ── Spaceflight (rasid 88 fn / 9 files) ────────────────────────────
        div { class: "section-card",
            h2 { { t!("form-cat-spaceflight") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-kepler") } }
                li { { t!("form-mod-dynamics") } }
                li { { t!("form-mod-perturbation") } }
                li { { t!("form-mod-propulsion") } }
                li { { t!("form-mod-rotation") } }
                li { { t!("form-mod-thermal") } }
                li { { t!("form-mod-debris") } }
                li { { t!("form-mod-gnss") } }
                li { { t!("form-mod-trajectory") } }
            }
        }

        // ── Astrophysics & stellar physics ────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-astrophysics") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-astrophysics") } }
                li { { t!("form-mod-stellar") } }
                li { { t!("form-mod-galactic") } }
                li { { t!("form-mod-cosmology") } }
                li { { t!("form-mod-helio") } }
                li { { t!("form-mod-high-energy") } }
                li { { t!("form-mod-celestial") } }
                li { { t!("form-mod-planetary") } }
            }
        }

        // ── Mechanics ──────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-mechanics") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-mechanics") } }
                li { { t!("form-mod-material") } }
                li { { t!("form-mod-biomech") } }
                li { { t!("form-mod-control") } }
                li { { t!("form-mod-chaos") } }
                li { { t!("form-mod-topology") } }
                li { { t!("form-mod-softbody") } }
            }
        }

        // ── Relativity ─────────────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-relativity") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-relativity") } }
                li { { t!("form-mod-transmission") } }
            }
        }

        // ── Quantum & electromagnetism ────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-quantum") } }
            p { class: "p-muted", { t!("form-mod-quantum") } }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-electromagnetism") } }
            p { class: "p-muted", { t!("form-mod-em") } }
        }

        // ── Nuclear, thermodynamics & continuum ───────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-nuclear") } }
            p { class: "p-muted", { t!("form-mod-nuclear") } }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-cat-fluid") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-fluid") } }
                li { { t!("form-mod-plasma") } }
                li { { t!("form-mod-superfluid") } }
                li { { t!("form-mod-continuum") } }
            }
        }
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-mod-physchem-title") } }
            ul { class: "ul-plain",
                li { { t!("form-mod-physchem") } }
                li { { t!("form-mod-thermo") } }
                li { { t!("form-mod-molecular") } }
                li { { t!("form-mod-wave-optics") } }
                li { { t!("form-mod-acoustics") } }
                li { { t!("form-mod-aero") } }
            }
        }

        // ── Supporting modules ─────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-support-title") } }
            p { class: "p-muted", { t!("form-support-intro") } }
            ul { class: "ul-plain",
                li { "math.rs — finite-many / vec3 / clamp01 共享原语" }
                li { "integrators.rs — Leapfrog / Yoshida 4 / Forest–Ruth 8 / Kahan / 1PN+2PN" }
                li { "gravitational_models.rs — Legendre / 球谐 / Carlson RF·RD / 椭球 / J2 张量" }
                li { "celestial_data.rs — JPL DE441 10 天体精密参数" }
            }
        }

        // ── Calling from Java ──────────────────────────────────────────────
        div { class: "section-divider",
            h2 { class: "section-heading", { t!("form-call-title") } }
            p { class: "p-muted", { t!("form-call-desc") } }
            div { class: "code-block",
                pre { code {
                    "// 全部公式函数经 C ABI 暴露，无 WorldHandle 依赖\n// 例：双椭球引力加速度\nVec3 a = mps_formula_ellipsoid_gravity(pos, body);\n// 例：Yoshida 4 阶辛积分器推进\nmps_formula_yoshida4_step(&mut pos, &mut vel, gm, dt);"
                } }
            }
        }
    }
}
