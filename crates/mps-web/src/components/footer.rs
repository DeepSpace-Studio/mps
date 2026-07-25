use topcoat::view::component;

/// Site footer
#[component]
pub async fn MpsFooter() -> topcoat::Result {
    topcoat::view::view! {
        <footer style="text-align:center; padding:24px 20px; border-top:1px solid #333; color:#666; font-size:12px; margin-top:40px;">
            <p>
                "MPS Motion Physics System v0.1.4 — "
                <a href="https://github.com/Polari-Stars-MC/rigid-body" style="color:#4a9eff; text-decoration:none;">"GitHub"</a>
            </p>
        </footer>
    }
}