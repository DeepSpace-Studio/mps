use topcoat::view::component;

/// A metric card showing a large number and label
#[component]
pub async fn MetricCard(value: &str, label: &str) -> topcoat::Result {
    topcoat::view::view! {
        <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px 28px; text-align:center; min-width:120px;">
            <strong style="display:block;font-size:28px;color:#4a9eff;font-weight:300;">(value)</strong>
            <span style="font-size:12px;color:#888;text-transform:uppercase;letter-spacing:1px;">(label)</span>
        </div>
    }
}
