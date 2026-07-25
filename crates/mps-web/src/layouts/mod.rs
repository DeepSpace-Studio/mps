/// Root layout wraps all pages with HTML skeleton, header, and footer
#[topcoat::router::layout("/")]
pub async fn root_layout(slot: topcoat::router::Slot<'_>) -> topcoat::Result {
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
[data-lang] { display: none; }
[data-lang="zh"] { display: block; }
html[data-lang="en"] [data-lang="en"] { display: block; }
html[data-lang="en"] [data-lang="zh"] { display: none; }
@media (max-width: 768px) { .mps-main { padding: 20px 12px; } }
"#;

    let js = r#"
(function(){
    var lang = localStorage.getItem('mps-lang') || 'zh';
    document.documentElement.lang = lang === 'en' ? 'en' : 'zh-CN';
    document.documentElement.dataset.lang = lang;
    document.querySelectorAll('[data-lang]').forEach(function(el){
        el.hidden = el.dataset.lang !== lang;
    });
    document.querySelectorAll('select').forEach(function(sel){
        sel.value = lang;
    });
    if(window.hljs) document.querySelectorAll('pre code').forEach(function(x){ hljs.highlightElement(x); });
})();
"#;

    topcoat::view::view! {
        <!DOCTYPE html>
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>"MPS Motion Physics System"</title>
                <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css">
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/rust.min.js"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/java.min.js"></script>
                <style>(topcoat::view::Unescaped::new_unchecked(css))</style>
            </head>
            <body>
                <header style="background:#1a1a2e; border-bottom:1px solid #333; padding:10px 20px; display:flex; align-items:center; justify-content:space-between; flex-wrap:wrap;">
                    <a href="/" style="display:flex; align-items:center; gap:12px; color:#fff; text-decoration:none; font-size:18px; font-weight:600;">
                        <span style="background:#4a9eff; color:#1a1a2e; padding:4px 10px; border-radius:4px; font-weight:700; letter-spacing:1px;">"MPS"</span>
                        <span style="color:#999; font-weight:400; font-size:14px;">"PHYSICS / 0.1.4"</span>
                    </a>
                    <nav style="display:flex; gap:2px; flex-wrap:wrap;">
                        <a href="/" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"首页"</span><span data-lang="en">"HOME"</span></a>
                        <a href="/quickstart" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"快速入门"</span><span data-lang="en">"QUICKSTART"</span></a>
                        <a href="/architecture" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"架构"</span><span data-lang="en">"ARCHITECTURE"</span></a>
                        <a href="/gravity" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"引力模型"</span><span data-lang="en">"GRAVITY"</span></a>
                        <a href="/formula" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"公式模块"</span><span data-lang="en">"FORMULA"</span></a>
                        <a href="/api" style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;"><span data-lang="zh">"API"</span><span data-lang="en">"API"</span></a>
                    </nav>
                    <div>
                        <select onchange="var lang=this.value;document.documentElement.lang=lang==='zh'?'zh-CN':'en';document.documentElement.dataset.lang=lang;localStorage.setItem('mps-lang',lang);document.querySelectorAll('[data-lang]').forEach(function(el){el.hidden=el.dataset.lang!==lang;});" style="background:#333; color:#ddd; border:1px solid #555; padding:4px 8px; border-radius:4px; font-size:12px;">
                            <option value="zh">"中文"</option>
                            <option value="en">"English"</option>
                        </select>
                    </div>
                </header>
                <main class="mps-main">
                    (slot.await?)
                </main>
                <footer style="text-align:center; padding:24px 20px; border-top:1px solid #333; color:#666; font-size:12px; margin-top:40px;">
                    <p>"MPS Motion Physics System v0.1.4 — " <a href="https://github.com/Polari-Stars-MC/rigid-body" style="color:#4a9eff; text-decoration:none;">"GitHub"</a></p>
                </footer>
                <script>(topcoat::view::Unescaped::new_unchecked(js))</script>
            </body>
        </html>
    }
}