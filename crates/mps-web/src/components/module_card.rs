use topcoat::view::component;

/// A module card with index number, title, description, and link
#[component]
pub async fn ModuleCard(
    index: &str,
    title: &str,
    description: &str,
    href: &str,
) -> topcoat::Result {
    topcoat::view::view! {
        <a href=(href) style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px; transition:border-color 0.2s;" onmouseover="this.style.borderColor='#4a9eff'" onmouseout="this.style.borderColor='#333'">
            <span style="font-family:monospace; font-size:12px; color:#4a9eff;">(index)</span>
            <strong style="font-size:16px; color:#fff;">(title)</strong>
            <small style="font-size:13px; color:#888; line-height:1.5;">(description)</small>
            <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
        </a>
    }
}
