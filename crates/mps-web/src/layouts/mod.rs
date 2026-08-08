/// Root layout wraps all pages with HTML skeleton, header, and footer.
#[topcoat::router::layout("/")]
pub async fn root_layout(slot: topcoat::router::Slot<'_>) -> topcoat::Result {
    use crate::metrics::VERSION;

    let css = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: #0a0a1a; color: #e0e0e0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; min-height: 100vh; display: flex; flex-direction: column; line-height: 1.7; }
.mps-main { flex: 1; max-width: 1100px; margin: 0 auto; padding: 30px 20px; width: 100%; }
pre { background: #0d0d2b; border: 1px solid #333; border-radius: 6px; padding: 16px; overflow-x: auto; font-family: "Consolas", "Monaco", monospace; font-size: 13px; line-height: 1.5; }
code { font-family: "Consolas", "Monaco", monospace; font-size: 13px; }
:not(pre) > code { background: #1a1a3e; padding: 2px 6px; border-radius: 3px; color: #4a9eff; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
table th { background: #1a1a3e; color: #bbb; font-weight: 600; text-align: left; padding: 10px 12px; border-bottom: 2px solid #444; white-space: nowrap; }
table td { padding: 8px 12px; border-bottom: 1px solid #333; color: #aaa; }
table tr:hover td { background: #1a1a3e; }
.callout { background: #0f1a2e; border-left: 4px solid #4a9eff; padding: 14px 18px; border-radius: 4px; margin: 20px 0; font-size: 14px; color: #bbb; }
.callout strong { color: #fff; }
.callout .hi { color: #4a9eff; font-family: monospace; }
[data-lang]:not(html) { display: none; }
[data-lang="zh"] { display: block; }
html[data-lang="en"] [data-lang="en"] { display: block; }
html[data-lang="en"] [data-lang="zh"] { display: none; }
@media (max-width: 768px) { .mps-main { padding: 20px 12px; } }

/* ---- Reusable component classes (replaces 567 inline styles) ---- */

/* Page header block: page title row with breadcrumb tag + big index number */
.page-head { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 30px; padding-bottom: 20px; border-bottom: 1px solid #333; }
.page-tag { font-size: 12px; color: #4a9eff; letter-spacing: 3px; text-transform: uppercase; font-family: monospace; margin-bottom: 8px; }
.page-title { font-size: 28px; font-weight: 300; color: #fff; margin: 0 0 10px; }
.page-desc { font-size: 14px; color: #999; line-height: 1.7; margin: 0; }
.page-index { font-size: 48px; font-weight: 700; color: #333; font-family: monospace; line-height: 1; }

/* Section card: the main content container used on every page */
.section-card { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 24px; margin-bottom: 20px; }
.section-card h2 { color: #fff; font-size: 20px; font-weight: 400; margin: 0 0 16px; padding-bottom: 10px; border-bottom: 1px solid #333; }

/* Table wrapper for horizontal scroll */
.table-wrap { overflow-x: auto; }

/* Paragraph styles */
.p-lead { color: #aaa; line-height: 1.7; }
.p-note { color: #aaa; line-height: 1.7; margin-top: 8px; }
.p-note-top14 { color: #aaa; line-height: 1.7; margin-top: 14px; }
.p-muted { color: #777; line-height: 1.7; font-size: 13px; }

/* Lists */
.ul-plain { color: #999; line-height: 2; padding-left: 20px; }
.ol-plain { color: #999; line-height: 2; padding-left: 20px; }

/* Metric cards grid */
.metric-grid { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; margin: 40px 0; }
.metric-card { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 20px 28px; text-align: center; min-width: 120px; }
.metric-card .num { display: block; font-size: 28px; color: #4a9eff; font-weight: 300; }
.metric-card .label { font-size: 12px; color: #888; text-transform: uppercase; letter-spacing: 1px; }
.stat-card { background: #16213e; border: 1px solid #333; border-radius: 6px; padding: 16px; text-align: center; }
.stat-card .num { display: block; font-size: 22px; color: #4a9eff; font-weight: 600; }
.stat-card .label { font-size: 11px; color: #888; text-transform: uppercase; letter-spacing: 1px; }

/* Module card grid */
.module-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; }
.module-card { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 20px; text-decoration: none; color: #ccc; display: flex; flex-direction: column; gap: 8px; transition: border-color 0.2s; }
.module-card:hover { border-color: #4a9eff; }
.module-card .idx { font-family: monospace; font-size: 12px; color: #4a9eff; }
.module-card .title { font-size: 16px; color: #fff; }
.module-card .desc { font-size: 13px; color: #888; line-height: 1.5; }
.module-card .arrow { font-style: normal; font-size: 18px; color: #4a9eff; text-align: right; margin-top: auto; }

/* Feature card grid */
.feature-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 16px; }
.feature-card { background: #16213e; border: 1px solid #333; border-radius: 8px; padding: 20px; }
.feature-card h3 { font-size: 16px; color: #fff; margin: 0 0 8px; }
.feature-card p { font-size: 14px; color: #999; line-height: 1.6; margin: 0; }

/* Hero section (home page) */
.hero { text-align: center; padding: 60px 20px 40px; }
.hero-tag { font-size: 12px; color: #4a9eff; letter-spacing: 3px; text-transform: uppercase; margin-bottom: 12px; font-family: monospace; }
.hero-title { font-size: 36px; font-weight: 300; color: #fff; margin: 0 0 16px; }
.hero-desc { font-size: 16px; color: #aaa; max-width: 720px; margin: 0 auto 30px; line-height: 1.7; }
.hero-actions { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
.btn-primary { background: #4a9eff; color: #fff; padding: 12px 24px; border-radius: 6px; text-decoration: none; font-weight: 500; }
.btn-outline { border: 1px solid #4a9eff; color: #4a9eff; padding: 12px 24px; border-radius: 6px; text-decoration: none; font-weight: 500; }

/* Step circle (quickstart numbered steps) */
.step-row { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.step-circle { background: #4a9eff; color: #1a1a2e; width: 32px; height: 32px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 14px; flex-shrink: 0; }
.step-title { margin: 0; font-size: 18px; color: #fff; font-weight: 500; }
.step-body { padding-left: 44px; margin-bottom: 24px; }

/* Inline keyword highlights */
.kw { color: #4a9eff; }
.kw-mono { color: #4a9eff; font-family: monospace; }
.text-white { color: #fff; }
.text-muted { color: #999; }
.text-faint { color: #666; }
.text-hl { color: #e0e0e0; }
.link { color: #4a9eff; text-decoration: none; }

/* Nav bar */
.mps-nav { display: flex; gap: 2px; flex-wrap: wrap; }
.mps-nav a { color: #bbb; padding: 8px 14px; font-size: 13px; border-radius: 4px; text-decoration: none; }
.mps-nav a:hover { background: #333; }
.mps-nav a.active { background: #4a9eff; color: #fff; }
.mps-header { background: #1a1a2e; border-bottom: 1px solid #333; padding: 10px 20px; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; }
.mps-brand { display: flex; align-items: center; gap: 12px; color: #fff; text-decoration: none; font-size: 18px; font-weight: 600; }
.mps-brand-badge { background: #4a9eff; color: #1a1a2e; padding: 4px 10px; border-radius: 4px; font-weight: 700; letter-spacing: 1px; }
.mps-brand-ver { color: #999; font-weight: 400; font-size: 14px; }
.mps-footer { text-align: center; padding: 24px 20px; border-top: 1px solid #333; color: #666; font-size: 12px; margin-top: 40px; }
.lang-select { background: #333; color: #ddd; border: 1px solid #555; padding: 4px 8px; border-radius: 4px; font-size: 12px; }

/* Formula mini-cards (formula.rs 28-module grid) */
.formula-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 8px; }
.formula-card { background: #0d0d2b; border: 1px solid #333; border-radius: 4px; padding: 10px 14px; }
.formula-card code { color: #4a9eff; font-size: 13px; }
.formula-card .formula-label { color: #999; font-size: 12px; margin-top: 2px; }

/* Code blocks inside sections */
.code-block { background: #0d0d2b; border: 1px solid #333; border-radius: 6px; padding: 12px 16px; }
.code-block pre { background: #0d0d2b; border: none; border-radius: 0; padding: 0; }

/* Variant callouts */
.callout-warn { background: #0f1a2e; border-left: 4px solid #f04a6a; padding: 14px 18px; border-radius: 4px; margin: 20px 0; font-size: 14px; color: #bbb; }
.callout-note { background: #0f1a2e; border-left: 4px solid #f0a04a; padding: 14px 18px; border-radius: 4px; margin: 20px 0; font-size: 14px; color: #bbb; }
.callout-warn strong, .callout-note strong { color: #fff; }

/* Generic layout helpers */
.section-divider { margin: 40px 0; }
.section-heading { font-size: 20px; font-weight: 300; color: #fff; margin: 0 0 16px; }
.section-heading-lg { font-size: 24px; font-weight: 300; color: #fff; margin: 0 0 24px; }
.text-center { text-align: center; }
.mini-stat-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; }

"#;

    let js = r#"
(function(){
    // Restore language preference
    var lang = localStorage.getItem('mps-lang') || 'zh';
    document.documentElement.lang = lang === 'en' ? 'en' : 'zh-CN';
    document.documentElement.dataset.lang = lang;
    document.querySelectorAll('[data-lang]').forEach(function(el){
        el.hidden = el.dataset.lang !== lang;
    });
    document.querySelectorAll('select[data-role="lang"]').forEach(function(sel){
        sel.value = lang;
    });
    if(window.hljs) document.querySelectorAll('pre code').forEach(function(x){ hljs.highlightElement(x); });

    // Generic base-URL rewrite: any href starting with "./" gets the
    // localStorage 'mps-base-url' prepended (for GitHub Pages sub-paths).
    var baseUrl = localStorage.getItem('mps-base-url');
    if (baseUrl) {
        if (!baseUrl.endsWith('/')) baseUrl += '/';
        document.querySelectorAll('a[href^="./"]').forEach(function(el) {
            el.href = baseUrl + el.getAttribute('href').replace(/^\.\//, '');
        });
    }
})();
"#;

    // Base path for deployments under a sub-path (e.g. GitHub Pages project
    // sites). Set MPS_BASE_PATH="/rigid-body/" when exporting so relative
    // links like ./quickstart resolve under the sub-path instead of the
    // domain root. Empty/unset means no <base> tag (local dev at /).
    let base_path = std::env::var("MPS_BASE_PATH").unwrap_or_default();
    let base_path = base_path.trim().trim_matches('/');
    let base_tag = if base_path.is_empty() {
        String::new()
    } else {
        format!("<base href=\"/{base_path}/\">")
    };

    let version_str = VERSION;

    topcoat::view::view! {
        <!DOCTYPE html>
        <html lang="zh-CN" data-lang="zh">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                (topcoat::view::Unescaped::new_unchecked(base_tag))
                <title>"MPS Motion Physics System"</title>
                <meta name="description" content="MPS (Meters Per Second) — high-precision Rust physics engine built on Rapier3D-f64. C FFI, Java JNI, Java FFM bindings. 5 gravity models, 3 symplectic integrators, 28 formula modules, shared-memory zero-copy Arena.">
                <meta name="keywords" content="Rust,physics,Rapier3D,JNI,FFM,simulation,spaceflight,orbital mechanics,rigid body">
                <meta name="author" content="Polaris Stars MC">
                <meta name="robots" content="index, follow">
                <meta property="og:type" content="website">
                <meta property="og:title" content="MPS Motion Physics System">
                <meta property="og:description" content="High-precision Rust physics engine — Rapier3D-f64, C FFI + JNI + FFM, 28 formula modules, 10 celestial bodies.">
                <meta property="og:url" content="https://polari-stars-mc.github.io/rigid-body/">
                <meta name="twitter:card" content="summary">
                <meta name="twitter:title" content="MPS Motion Physics System">
                <meta name="twitter:description" content="High-precision Rust physics engine — Rapier3D-f64, C FFI + JNI + FFM.">
                <link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='20' fill='%234a9eff'/><text x='50' y='68' font-size='52' font-weight='bold' text-anchor='middle' fill='%231a1a2e'>M</text></svg>">
                <link rel="canonical" href="https://polari-stars-mc.github.io/rigid-body/">
                <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css">
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/rust.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/java.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/bash.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/c.min.js"></script>
                <style>(topcoat::view::Unescaped::new_unchecked(css))</style>
            </head>
            <body>
                <header class="mps-header">
                    <a href="./" class="mps-brand">
                        <span class="mps-brand-badge">"MPS"</span>
                        <span class="mps-brand-ver">"PHYSICS / "{ (version_str) }</span>
                    </a>
                    <nav class="mps-nav">
                        <a href="./"><span data-lang="zh">"首页"</span><span data-lang="en">"Home"</span></a>
                        <a href="./quickstart"><span data-lang="zh">"快速入门"</span><span data-lang="en">"Quickstart"</span></a>
                        <a href="./architecture"><span data-lang="zh">"架构"</span><span data-lang="en">"Architecture"</span></a>
                        <a href="./gravity"><span data-lang="zh">"引力模型"</span><span data-lang="en">"Gravity"</span></a>
                        <a href="./integrators"><span data-lang="zh">"积分器"</span><span data-lang="en">"Integrators"</span></a>
                        <a href="./formula"><span data-lang="zh">"公式模块"</span><span data-lang="en">"Formula"</span></a>
                        <a href="./voxel"><span data-lang="zh">"体素"</span><span data-lang="en">"Voxel"</span></a>
                        <a href="./events"><span data-lang="zh">"事件"</span><span data-lang="en">"Events"</span></a>
                        <a href="./arena"><span data-lang="zh">"Arena"</span><span data-lang="en">"Arena"</span></a>
                        <a href="./cosmos"><span data-lang="zh">"太空"</span><span data-lang="en">"Cosmos"</span></a>
                        <a href="./jni"><span data-lang="zh">"JNI"</span><span data-lang="en">"JNI"</span></a>
                        <a href="./ffm"><span data-lang="zh">"FFM"</span><span data-lang="en">"FFM"</span></a>
                        <a href="./api"><span data-lang="zh">"API"</span><span data-lang="en">"API"</span></a>
                    </nav>
                    <div>
                        <select data-role="lang" class="lang-select" onchange="var lang=this.value;document.documentElement.lang=lang==='zh'?'zh-CN':'en';document.documentElement.dataset.lang=lang;localStorage.setItem('mps-lang',lang);document.querySelectorAll('[data-lang]').forEach(function(el){el.hidden=el.dataset.lang!==lang;});">
                            <option value="zh">"中文"</option>
                            <option value="en">"English"</option>
                        </select>
                    </div>
                </header>
                <main class="mps-main">
                    (slot.await?)
                </main>
                <footer class="mps-footer">
                    <p>"MPS Motion Physics System v"{ (version_str) }" — "
                        <a href="https://github.com/Polari-Stars-MC/rigid-body" class="link">"GitHub"</a>
                    </p>
                </footer>
                <script>(topcoat::view::Unescaped::new_unchecked(js))</script>
            </body>
        </html>
    }
}
