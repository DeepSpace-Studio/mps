use topcoat::router::page;
use topcoat::view::view;

/// 404 fallback page.
#[page("/404")]
pub async fn page_not_found() -> topcoat::Result {
    view! {
        <div class="hero">
            <div class="hero-tag">"/ 404"</div>
            <h1 class="hero-title">
                <span data-lang="zh">"页面未找到"</span>
                <span data-lang="en">"Page Not Found"</span>
            </h1>
            <p class="hero-desc">
                <span data-lang="zh">"你访问的页面不存在。可能链接已移动或拼写有误。"</span>
                <span data-lang="en">"The page you are looking for does not exist. It may have been moved or the URL is misspelled."</span>
            </p>
            <div class="hero-actions">
                <a href="./" class="btn-primary">
                    <span data-lang="zh">"返回首页"</span>
                    <span data-lang="en">"Back to Home"</span>
                </a>
                <a href="./api" class="btn-outline">
                    <span data-lang="zh">"API 参考"</span>
                    <span data-lang="en">"API Reference"</span>
                </a>
            </div>
        </div>
    }
}
