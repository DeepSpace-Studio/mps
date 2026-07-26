use topcoat::router::page;
use topcoat::view::view;

/// Voxel colliders page
#[page("/voxel")]
pub async fn voxel() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ VOXEL"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"Voxel 碰撞体"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"体素碰撞体支持从原始网格、AABB 和 OBB 构建，支持多种构建模式。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"构建模式"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"Auto"</strong> " — 根据体素数量和刚体类型自动选择"</li>
                    <li><strong style="color:#ddd;">"Cuboids"</strong> " — 每个固体体素一个立方体"</li>
                    <li><strong style="color:#ddd;">"GreedyCuboids"</strong> " — 合并相邻固体体素"</li>
                    <li><strong style="color:#ddd;">"SurfaceMesh"</strong> " — 生成外表面三角网格"</li>
                </ul>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"API"</h2>
                <pre><code class="language-rust">
"// 从原始网格创建
let collider = collider_builder_create_voxel_aabb(
    &grid, 128, 128, 128,
    VoxelColliderOptions::GreedyCuboids
);

// 从 OBB 创建
let collider = collider_builder_create_voxel_obb(
    &grid, &obb, options
);"
                </code></pre>
            </div>
        </div>
    }
}
