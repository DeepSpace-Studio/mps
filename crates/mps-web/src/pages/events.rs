use topcoat::router::page;
use topcoat::view::view;

/// Events page
#[page("/events")]
pub async fn events() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">"/ EVENTS"</div>
                    <h1 class="page-title"><span data-lang="zh">"碰撞 / 接触力事件系统"</span><span data-lang="en">"Collision / Contact Force Event System"</span></h1>
                    <p class="page-desc">{ <span data-lang="zh">"三类事件通道：legacy Vec 队列、SPSC 环形缓冲、类型化回调。可在 world_step 产出期与 Java 排走期并发执行（环形单生产者 / 单消费者）。运行时通过 step_active + init_guard 兜底，违规返回 ERR_UNSUPPORTED 而非 UB。"</span><span data-lang="en">"Three event channels: legacy Vec queue, SPSC ring buffer, typed callbacks. Can run concurrently between world_step production and Java draining (ring is SPSC). Runtime guarded by step_active + init_guard, violations return ERR_UNSUPPORTED rather than UB."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "三种事件通道" }</span><span data-lang="en">{ "Three Event Channels" }</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"通道"</span><span data-lang="en">"Channel"</span></th><th><span data-lang="zh">"线程契约"</span><span data-lang="en">"Thread Contract"</span></th><th><span data-lang="zh">"典型用途"</span><span data-lang="en">"Typical Use"</span></th></tr></thead>
                        <tbody>
                            <tr><td><strong><span data-lang="zh">"Legacy Vec"</span><span data-lang="en">"Legacy Vec"</span></strong><br/><code>"collision_events / contact_force_events"</code></td><td>{ <span data-lang="zh">"Mutex 保护，任意并发安全"</span><span data-lang="en">"Mutex-guarded, safe under any concurrency"</span> }</td><td>{ <span data-lang="zh">"兼容老 API：world_get_collision_event(index) 逐条读 / world_get_collision_events 批量读"</span><span data-lang="en">"Compat with old API: world_get_collision_event(index) per-item read / world_get_collision_events batch read"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"SPSC 环形"</span><span data-lang="en">"SPSC Ring"</span></strong><br/><code>"EventRing<T> @ producer_cache"</code></td><td>{ <span data-lang="zh">"单生产者（物理线程）/ 单消费者（Java 排走），Release/Acquire 原子游标"</span><span data-lang="en">"Single producer (physics thread) / single consumer (Java drain), Release/Acquire atomic cursors"</span> }</td><td>{ <span data-lang="zh">"热路径：单 drain 调用拉满一帧"</span><span data-lang="en">"Hot path: one drain call covers a full frame"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"Callback 槽"</span><span data-lang="en">"Callback Slot"</span></strong><br/><code>"CallbackSlot / CollisionEventCallback"</code></td><td>{ <span data-lang="zh">"Mutex 内函数指针 / world_step 内分发"</span><span data-lang="en">"Function pointer under Mutex / dispatched within world_step"</span> }</td><td>{ <span data-lang="zh">"脚本绑定：把 C fn 地址作为 usize 注入"</span><span data-lang="en">"Script bindings: inject C fn address as usize"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"EventDispatchMode 控制分发"</span><span data-lang="en">"EventDispatchMode Controls Dispatch"</span> }</h2>
                <pre><code class="language-c">
"pub enum EventDispatchMode {
    Poll     = 0,  // 仅入环缓冲，Java 轮询读（默认）
    Callback = 1,  // world_step 内调注册回调，不入环
    Both     = 2,  // 同时入环 + 调回调
}"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"通过 "</span><span data-lang="en">"Via "</span> }<code>"world_set_event_dispatch_mode(world, mode)"</code>{ <span data-lang="zh">" 切换。该 API 与 register_* 均为 "</span><span data-lang="en">" to switch. This API and register_* are both "</span> }</p>
                <p class="p-lead">{ <span data-lang="zh">"init-time-only：不能和 world_step 并发。运行时通过 step_active 原子守护，违规返回 "</span><span data-lang="en">"init-time-only: cannot run concurrently with world_step. Runtime guarded by step_active atomic, violations return "</span> }<code>"ERR_UNSUPPORTED"</code>{ <span data-lang="zh">" 而非 UB。"</span><span data-lang="en">" rather than UB."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"SPSC 环形：一期 init + 一帧一 drain"</span><span data-lang="en">"SPSC Ring: Alloc Once in Init, Drain Once Per Frame"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"一次 init 分配，永不重分配（跨步进无 realloc）。每帧仅一次 FFI 调用排空。capacity 上限 "</span><span data-lang="en">"One init allocation, never reallocated (no realloc across steps). Only one FFI drain call per frame. Capacity cap "</span> }<code>"MAX_EVENT_RECORDS = 16384"</code>{ <span data-lang="zh">"。Buffer 总大小 = capacity × EVENT_SLOT_STRIDE(=64) 字节。"</span><span data-lang="en">". Buffer total = capacity × EVENT_SLOT_STRIDE(=64) bytes."</span> }</p>
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

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"事件记录"</span><span data-lang="en">"Event Records"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"结构"</span><span data-lang="en">"Struct"</span></th><th><span data-lang="zh">"字段"</span><span data-lang="en">"Field"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"CollisionEventRecord"</code></td><td>"started · collider1 · collider2 · sensor · removed"</td></tr>
                            <tr><td><code>"ContactForceEventRecord"</code></td><td>"collider1 · collider2 · total_force · total_force_magnitude · max_force_direction · max_force_magnitude"</td></tr>
                            <tr><td><code>"EventRingBufferStats"</code></td><td>"capacity · len · dropped · wrapped"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ "ColliderHandle = u64 packed（高位 world id / 低位]);

world_collision_event_ring_stats / world_contact_force_event_ring_stats 取统计——可观测 dropped/wrapped 判断环是否太小。world_reset_event_ring 清环并重置计数。world_collision_event_ring_len 不加锁只读当前条数。" }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"回调注册（init-time-only）"</span><span data-lang="en">"Callback Registration (init-time-only)"</span> }</h2>
                <pre><code class="language-c">
"// C 侧函数签名（典型）：
// void on_collision(void* ctx, CollisionEventRecord ev);
EventCallbackHandle h =
    world_register_collision_callback(world, on_collision, ctx);
EventCallbackHandle h2 =
    world_register_contact_force_callback(world, on_force, ctx2);
world_set_event_dispatch_mode(world, 1 /*Callback*/ 或 2 /*Both*/);"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"回调在物理线程、world_step 期内同步分发——回调内禁止再调 world_*。返回 EventCallbackHandle 用于后续注销（同样 init-time-only）。地址传 usize 的 frozen-ABI 要求决定此路径；典型绑定方为脚本桥（JNI/FFM 把回调地址传进来）。"</span><span data-lang="en">"Callbacks dispatched synchronously on the physics thread during world_step — callbacks must not call back into world_*. Returns EventCallbackHandle for later unregister (also init-time-only). The usize-address freeze-ABI requirement shapes this path; typical binders are script bridges (JNI/FFM injects callback address)."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" 选型：热路径轮询首选 SPSC 环（init 一次、每帧一 drain）；脚本桥用 Callback；需要兼容旧逐条读 API 留 Poll 走 Legacy Vec。环形逻辑在 mps-test/rapier/events.rs 经环绕-drop-并发drain 时单测覆盖；looom 可对游标协议做模型检查。"</span><span data-lang="en">"Selection: SPSC ring for hot-path polling (init once, drain once per frame); Callback for script bridges; Poll via Legacy Vec for old per-item read API compat. Ring logic tested in mps-test/rapier/events.rs (wrap-drop-concurrent drain); looom can model-check the cursor protocol."</span> }</p>
            </div>

            <div class="callout callout-warn">
                <p>{ <span data-lang="zh">" ⚠ 并发契约：SPSC 仅单生产单消费。多线程同时 drain 同一环会破坏游标环不变量——但 Rust 侧无运行期检测（panic-free 路径），责任在调用方。建议每个 world 一条专责事件线程。"</span><span data-lang="en">" ⚠ Concurrency contract: SPSC strictly single-producer single-consumer. Concurrent drains on the same ring break cursor invariants — but Rust has no runtime check (panic-free path), responsibility on caller. Recommend a dedicated event thread per world."</span> }</p>
            </div>
        </div>
    }
}
