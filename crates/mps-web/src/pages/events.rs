use topcoat::router::page;
use topcoat::view::view;

/// Events page
#[page("/events")]
pub async fn events() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ EVENTS"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"事件系统"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"碰撞事件和接触力事件 — 锁自由设计，支持批量读取。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"事件类型"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"CollisionEvent"</strong> " — 碰撞开始/结束事件"</li>
                    <li><strong style="color:#ddd;">"ContactForceEvent"</strong> " — 接触力事件"</li>
                    <li><strong style="color:#ddd;">"IntersectionEvent"</strong> " — 传感器交集事件"</li>
                </ul>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"API"</h2>
                <pre><code class="language-rust">
"// 读取碰撞事件
let mut events = Vec::new();
world_get_collision_events(world, &mut events);

// 读取接触力事件
let mut contacts = Vec::new();
world_get_contact_force_events(world, &mut contacts);

// 清除事件队列
world_clear_events(world);"
                </code></pre>
            </div>
        </div>
    }
}