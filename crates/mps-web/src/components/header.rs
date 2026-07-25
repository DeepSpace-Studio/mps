use topcoat::view::component;

/// Site header with brand and navigation
#[component]
pub async fn MpsHeader(current_path: &str) -> topcoat::Result {
    let nav_items = [
        ("index.html", "首页", "HOME"),
        ("quickstart.html", "快速入门", "QUICKSTART"),
        ("architecture.html", "架构", "ARCHITECTURE"),
        ("gravity.html", "引力模型", "GRAVITY"),
        ("formula.html", "公式模块", "FORMULA"),
        ("api.html", "API", "API"),
    ];

    topcoat::view::view! {
        <header style="background:#1a1a2e; border-bottom:1px solid #333; padding:10px 20px; display:flex; align-items:center; justify-content:space-between; flex-wrap:wrap;">
            <a href="index.html" style="display:flex; align-items:center; gap:12px; color:#fff; text-decoration:none; font-size:18px; font-weight:600;">
                <span style="background:#4a9eff; color:#1a1a2e; padding:4px 10px; border-radius:4px; font-weight:700; letter-spacing:1px;">"MPS"</span>
                <span style="color:#999; font-weight:400; font-size:14px;">"PHYSICS / 0.1.4"</span>
            </a>
            <nav style="display:flex; gap:2px; flex-wrap:wrap;">
                for (href, zh, en) in nav_items {
                    if current_path == href || (current_path == "" && href == "index.html") {
                        <a href=(href) style="background:#4a9eff; color:#fff; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none;">
                            <span data-lang="zh">(zh)</span>
                            <span data-lang="en">(en)</span>
                        </a>
                    } else {
                        <a href=(href) style="color:#bbb; padding:8px 14px; font-size:13px; border-radius:4px; text-decoration:none; hover:background:#333;">
                            <span data-lang="zh">(zh)</span>
                            <span data-lang="en">(en)</span>
                        </a>
                    }
                }
            </nav>
            <div style="margin:0 0 0 10px;">
                <select onchange="var lang=this.value;document.documentElement.lang=lang==='zh'?'zh-CN':'en';document.documentElement.dataset.lang=lang;localStorage.setItem('mps-lang',lang);document.querySelectorAll('[data-lang]').forEach(function(el){el.hidden=el.dataset.lang!==lang;});" style="background:#333; color:#ddd; border:1px solid #555; padding:4px 8px; border-radius:4px; font-size:12px;">
                    <option value="zh">"中文"</option>
                    <option value="en">"English"</option>
                </select>
            </div>
        </header>
    }
}