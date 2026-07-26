use topcoat::view::component;

/// A documentation module section with title and content
#[component]
pub async fn DocModule(title: &str, child: Option<topcoat::view::View>) -> topcoat::Result {
    let content = child.unwrap_or_default();
    topcoat::view::view! {
        <section style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
            <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">(title)</h2>
            (content)
        </section>
    }
}
