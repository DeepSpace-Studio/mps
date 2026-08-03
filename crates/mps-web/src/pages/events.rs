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
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"碰撞 / 接触力事件系统"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "三类事件通道：legacy Vec 队列、SPSC 环形缓冲、类型化回调。可在 world_step 产出期与 Java 排走期并发执行（环形单生产者 / 单消费者）。运行时通过 step_active + init_guard 兜底，违规返回 ERR_UNSUPPORTED 而非 UB。" }</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "三种事件通道" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"通道"</th><th>"线程契约"</th><th>"典型用途"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Legacy Vec"</strong><br/><code>"collision_events / contact_force_events"</code></td><td>{ "Mutex 保护，任意并发安全" }</td><td>{ "兼容老 API：world_get_collision_event(index) 逐条读 / world_get_collision_events 批量读" }</td></tr>
                            <tr><td><strong>"SPSC 环形"</strong><br/><code>"EventRing<T> @ producer_cache"</code></td><td>{ "单生产者（物理线程）/ 单消费者（Java 排走），Release/Acquire 原子游标" }</td><td>{ "热路径：单 drain 调用拉满一帧" }</td></tr>
                            <tr><td><strong>"Callback 槽"</strong><br/><code>"CallbackSlot / CollisionEventCallback"</code></td><td>{ "Mutex 内函数指针 / world_step 内分发" }</td><td>{ "脚本绑定：把 C fn 地址作为 usize 注入" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "EventDispatchMode 控制分发" }</h2>
                <pre><code class="language-c">
"pub enum EventDispatchMode {
    Poll     = 0,  // 仅入环缓冲，Java 轮询读（默认）
    Callback = 1,  // world_step 内调注册回调，不入环
    Both     = 2,  // 同时入环 + 调回调
}"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "通过 " }<code>"world_set_event_dispatch_mode(world, mode)"</code>{ " 切换。该 API 与 register_* 均为 " }</p>
                <p style="color:#aaa;line-height:1.7;">{ "init-time-only：不能和 world_step 并发。运行时通过 step_active 原子守护，违规返回 " }<code>"ERR_UNSUPPORTED"</code>{ " 而非 UB。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "SPSC 环形：一期 init + 一帧一 drain" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "一次 init 分配，永不重分配（跨步进无 realloc）。每帧仅一次 FFI 调用排空。capacity 上限 " }<code>"MAX_EVENT_RECORDS = 16384"</code>{ "。Buffer 总大小 = capacity × EVENT_SLOT_STRIDE(=64) 字节。" }</p>
                <pre><code class="language-c">
"// init 期：分配两环
world_init_collision_event_ring(world, 1024);     // 最多 1024 个 CollisionEventRecord
world_init_contact_force_event_ring(world, 1024);
world_set_event_dispatch_mode(world, 0 /*Poll*/);

// 每帧：step 后单次 drain
CollisionEventRecord evs[1024];
uint32_t n = world_drain_collision_event_ring(world, evs, 1024);
for (uint32_t i = 0; i < n; ++i) {
    // evs[i].started / .collider1 / .collider2 / .sensor / .removed
}"
                </code></pre>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "事件记录" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"结构"</th><th>"字段"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"CollisionEventRecord"</code></td><td>"started · collider1 · collider2 · sensor · removed"</td></tr>
                            <tr><td><code>"ContactForceEventRecord"</code></td><td>"collider1 · collider2 · total_force · total_force_magnitude · max_force_direction · max_force_magnitude"</td></tr>
                            <tr><td><code>"EventRingBufferStats"</code></td><td>"capacity · len · dropped · wrapped"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "ColliderHandle = u64 packed（高位 world id / 低位]);

world_collision_event_ring_stats / world_contact_force_event_ring_stats 取统计——可观测 dropped/wrapped 判断环是否太小。world_reset_event_ring 清环并重置计数。world_collision_event_ring_len 不加锁只读当前条数。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "回调注册（init-time-only）" }</h2>
                <pre><code class="language-c">
"// C 侧函数签名（典型）：
// void on_collision(void* ctx, CollisionEventRecord ev);
EventCallbackHandle h =
    world_register_collision_callback(world, on_collision, ctx);
EventCallbackHandle h2 =
    world_register_contact_force_callback(world, on_force, ctx2);
world_set_event_dispatch_mode(world, 1 /*Callback*/ 或 2 /*Both*/);"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "回调在物理线程、world_step 期内同步分发——回调内禁止再调 world_*。返回 EventCallbackHandle 用于后续注销（同样 init-time-only）。地址传 usize 的 frozen-ABI 要求决定此路径；典型绑定方为脚本桥（JNI/FFM 把回调地址传进来）。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 选型：热路径轮询首选 SPSC 环（init 一次、每帧一 drain）；脚本桥用 Callback；需要兼容旧逐条读 API 留 Poll 走 Legacy Vec。环形逻辑在 mps-test/rapier/events.rs 经环绕-drop-并发drain 时单测覆盖；looom 可对游标协议做模型检查。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #f04a6a; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " ⚠ 并发契约：SPSC 仅单生产单消费。多线程同时 drain 同一环会破坏游标环不变量——但 Rust 侧无运行期检测（panic-free 路径），责任在调用方。建议每个 world 一条专责事件线程。" }</p>
            </div>
        </div>
    }
}
