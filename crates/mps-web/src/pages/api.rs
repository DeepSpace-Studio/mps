use topcoat::router::page;
use topcoat::view::view;

/// API reference page
#[page("/api")]
pub async fn api() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ REFERENCE"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"API 参考"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"所有 pub extern C 函数导出，按子系统分组。总计约 480 个函数。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"🌍 世界管理 (World)"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"C 函数"</th><th>"JNI"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td>"world_create"</td><td>"✓"</td><td>"创建物理世界，设置重力向量"</td></tr>
                            <tr><td>"world_destroy"</td><td>"✓"</td><td>"销毁世界及所有资源"</td></tr>
                            <tr><td>"world_step"</td><td>"✓"</td><td>"推进模拟 (包含 ForceRegistry 调度)"</td></tr>
                            <tr><td>"world_set_gravity"</td><td>"✓"</td><td>"设置重力"</td></tr>
                            <tr><td>"world_get_gravity_out"</td><td>"✓"</td><td>"读取重力"</td></tr>
                            <tr><td>"world_set_integration_parameters"</td><td>"✓"</td><td>"设置 dt、求解器迭代次数、CCD 子步"</td></tr>
                            <tr><td>"world_get_rigid_body_set_size"</td><td>"✓"</td><td>"刚体总数"</td></tr>
                            <tr><td>"world_get_collider_set_size"</td><td>"✓"</td><td>"碰撞体总数"</td></tr>
                            <tr><td>"world_body_snapshot"</td><td>"✓"</td><td>"批量快照所有刚体状态"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"🔩 刚体 (Rigid Body)"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"C 函数"</th><th>"JNI"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td>"rigid_body_builder_create"</td><td>"✓"</td><td>"创建刚体构建器"</td></tr>
                            <tr><td>"rigid_body_builder_set_pos"</td><td>"✓"</td><td>"设置位置"</td></tr>
                            <tr><td>"rigid_body_builder_set_rot"</td><td>"✓"</td><td>"设置旋转"</td></tr>
                            <tr><td>"world_insert_rigid_body"</td><td>"✓"</td><td>"插入刚体到世界"</td></tr>
                            <tr><td>"rigid_body_get_position"</td><td>"✓"</td><td>"读取位置"</td></tr>
                            <tr><td>"rigid_body_set_velocity"</td><td>"✓"</td><td>"设置线速度"</td></tr>
                            <tr><td>"rigid_body_apply_force"</td><td>"✓"</td><td>"施加力"</td></tr>
                            <tr><td>"rigid_body_apply_impulse"</td><td>"✓"</td><td>"施加冲量"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"📦 碰撞体 (Collider)"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"C 函数"</th><th>"JNI"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td>"collider_builder_create"</td><td>"✓"</td><td>"创建碰撞体构建器"</td></tr>
                            <tr><td>"collider_builder_set_half_extents"</td><td>"✓"</td><td>"设置半长宽"</td></tr>
                            <tr><td>"collider_set_friction"</td><td>"✓"</td><td>"设置摩擦系数"</td></tr>
                            <tr><td>"collider_set_restitution"</td><td>"✓"</td><td>"设置恢复系数"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
