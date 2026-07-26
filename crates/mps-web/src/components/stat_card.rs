use topcoat::view::component;

/// A stat card showing a number and label
#[component]
pub async fn StatCard(num: &str, label: &str) -> topcoat::Result {
    topcoat::view::view! {
        <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
            <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">(num)</span>
            <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">(label)</span>
        </div>
    }
}
