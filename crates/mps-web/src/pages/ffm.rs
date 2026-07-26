use topcoat::router::page;
use topcoat::view::view;

/// FFM page
#[page("/ffm")]
pub async fn ffm() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ FFM"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"Java FFM 绑定"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"Java 25 Foreign Function & Memory API 绑定，覆盖所有核心功能。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"覆盖范围"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li>"World、刚体、碰撞体、CRbTree"</li>
                    <li>"Voxel AABB/OBB 构建和查询"</li>
                    <li>"射线投射、点投影、AABB/OBB/球体相交"</li>
                    <li>"形状投射 (shape cast)"</li>
                    <li>"刚体运行时突变：位姿、速度、力/力矩、冲量、CCD、睡眠/唤醒"</li>
                    <li>"碰撞体运行时突变：位姿、传感器、摩擦、恢复、事件"</li>
                    <li>"碰撞和接触力事件批量读取"</li>
                </ul>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"运行测试"</h2>
                <pre><code class="language-bash">
"cd test25
./gradlew.bat check"
                </code></pre>
            </div>
        </div>
    }
}
