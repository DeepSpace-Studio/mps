use topcoat::router::page;
use topcoat::view::view;

/// Arena page
#[page("/arena")]
pub async fn arena() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ ARENA"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">{ "共享内存 Arena（零拷贝状态镜像）" }</span><span data-lang="en">{ "Shared-Memory Arena (Zero-Copy State Mirror)" }</span></h1>
                    <p class="page-desc">{ <span data-lang="zh">"world_create_shared_arena 在 native 申请一段连续内存，把物理 world 的关键状态（刚体姿态/速度、命令环、事件环、力律聚合、积分参数）以固定 stride 排布。Java 经 DirectByteBuffer / MemorySegment 直接读写——每帧不再跨 FFI 逐字段 memcpy。"</span><span data-lang="en">"world_create_shared_arena allocates a contiguous native memory region, mirroring the world's key state (body poses/velocities, command ring, event ring, force aggregate, integration params) at fixed stride. Java reads/writes directly via DirectByteBuffer / MemorySegment — no per-field FFI memcpy each frame."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "内存布局（ARENA_VERSION=2）" }</span><span data-lang="en">{ "Memory Layout (ARENA_VERSION=2)" }</span></h2>
                <pre class="code-block">
                    <code>
"+0     Header (128 B)
       magic 'MPS_AREN' (0x4D50535F4152454E)  [+0]
       version = 2                          [+8]
       max_bodies/colliders/events/commands [+16/20/24/28]
       strides: BODY=96 COLLIDER=80 CMD=32 EVENT=64  [+48/52/56/60]
       region offsets [@64..]  —— Java 读这些偏移，不重算
+128   Body slots       = max_bodies  × 96 B
+?     Collider slots   = max_colliders × 80 B
+?     Body handle map  = max_bodies × 8 B (u64 packed handle → slot idx)
+?     Force report     = 32 × 32 B (ForceLawType 0..31 breakdown)
+?     IntegrationParams= 40 B (dt + solver_iter + ccd_substeps + gravity)
+?     Force summary    = 64 B (max_reynolds + external/drag force + counts)
+?     Command ring     = max_commands × 32 B
+?     Event ring        = max_events × 64 B"
                    </code>
                </pre>
                <p class="p-note-top14">{ <span data-lang="zh">"对齐 64 B；零化分配 (alloc_zeroed)。Region 偏移 Java 侧读 header 不重算——跨版本升级仅靠 header 头里的 region offset 向后兼容。"</span><span data-lang="en">"64 B aligned; alloc_zeroed. Java reads region offsets from header without re-computing — cross-version upgrades backward-compatible via the region offset table in the header."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"容量上限"</span><span data-lang="en">"Capacity Limits"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"容量"</span><span data-lang="en">"Capacity"</span></th><th><span data-lang="zh">"硬上限"</span><span data-lang="en">"Hard Limit"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"max_bodies"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_colliders"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_events"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_commands"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><strong><span data-lang="zh">"total bytes"</span><span data-lang="en">"total bytes"</span></strong></td><td>"256 MiB (MAX_ARENA_TOTAL_BYTES)"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"任一上限过或总字节超 "</span><span data-lang="en">"If any capacity or total bytes exceeds "</span> }<code>"world_create_shared_arena"</code>{ <span data-lang="zh">" 返回 "</span><span data-lang="en">" returns "</span> }<code>"Bool::FALSE"</code>{ <span data-lang="zh">" + 写 "</span><span data-lang="en">" + writes "</span> }<code>"ERR_CAPACITY"</code>{ <span data-lang="zh">"。每 world 至多一个 arena；重复建返回 "</span><span data-lang="en">". At most one arena per world; duplicate create returns "</span> }<code>"ERR_INVALID_ARGUMENT"</code>{ <span data-lang="zh">" —— 必须先 "</span><span data-lang="en">" — must first "</span> }<code>"world_destroy_shared_arena"</code>{ <span data-lang="zh">" 再建下一。"</span><span data-lang="en">" before creating the next."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"命令协议（CommandType u32）"</span><span data-lang="en">"Command Protocol (CommandType u32)"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"Java 把命令写入 "</span><span data-lang="en">"Java writes commands into "</span> }<code>"Command ring"</code>{ <span data-lang="zh">"（每槽 32 B：type + body_id + 负载）—— "</span><span data-lang="en">" (32 B per slot: type + body_id + payload) — "</span> }<strong><span data-lang="zh">"每步开始时 Rust 侧 apply"</span><span data-lang="en">"Rust side applies at step start"</span></strong>{ <span data-lang="zh">"：读 head、解码、更新对应 body / 力注册表，再推进 head。"</span><span data-lang="en">": reads head, decodes, updates the corresponding body / force registry, then advances head."</span> }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"值"</span><span data-lang="en">"Value"</span></th><th><span data-lang="zh">"命令"</span><span data-lang="en">"Command"</span></th></tr></thead>
                        <tbody>
                            <tr><td>"0"</td><td><code>"AddForce"</code></td></tr>
                            <tr><td>"1"</td><td><code>"AddTorque"</code></td></tr>
                            <tr><td>"2"</td><td><code>"SetPose"</code></td></tr>
                            <tr><td>"3"</td><td><code>"SetVelocity"</code></td></tr>
                            <tr><td>"4"</td><td><code>"ApplyImpulse"</code></td></tr>
                            <tr><td>"5"</td><td><code>"ApplyTorqueImpulse"</code></td></tr>
                            <tr><td>"6"</td><td><code>"WakeUp"</code></td></tr>
                            <tr><td>"7"</td><td><code>"Sleep"</code></td></tr>
                            <tr><td>"8"</td><td><code>"SetRotation"</code></td></tr>
                            <tr><td>"9"</td><td><code>"SetGravityScale"</code></td></tr>
                            <tr><td>"10"</td><td><code>"SetLinearDamping"</code></td></tr>
                            <tr><td>"11"</td><td><code>"SetAngularDamping"</code></td></tr>
                            <tr><td>"12"</td><td><code>"AddForceAtPoint"</code></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"C ABI 入口"</span><span data-lang="en">"C ABI Entry Points"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"函数"</span><span data-lang="en">"Function"</span></th><th><span data-lang="zh">"用途"</span><span data-lang="en">"Purpose"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"world_create_shared_arena"</code></td><td>{ <span data-lang="zh">"建 arena：传 max_* 容量 + 回写 out_address/out_size（u64）。每 world 至多一个"</span><span data-lang="en">"Create arena: pass max_* capacities + write out_address/out_size (u64). At most one per world"</span> }</td></tr>
                            <tr><td><code>"world_destroy_shared_arena"</code></td><td>{ <span data-lang="zh">"销毁（释放底层内存）。⚠ Java 须先释放映射的 MemorySegment，否则 use-after-free"</span><span data-lang="en">"Destroy (frees underlying memory). ⚠ Java must first release the mapped MemorySegment, else use-after-free"</span> }</td></tr>
                            <tr><td><code>"world_get_shared_arena_address / _size"</code></td><td>{ <span data-lang="zh">"取基址 / 字节数（无则 0），用于 Java 二次映射"</span><span data-lang="en">"Get base address / byte count (0 if none), for Java remap"</span> }</td></tr>
                            <tr><td><code>"world_reset_shared_arena_events"</code></td><td>{ <span data-lang="zh">"排走事件后 Java 调，重置事件环 head/tail"</span><span data-lang="en">"Called by Java after draining events, resets event ring head/tail"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"Java FFM 落地"</span><span data-lang="en">"Java FFM Usage"</span> }</h2>
                <pre><code class="language-java">
"// Java 25 FFM
Linker linker = Linker.nativeLinker();
MemorySegment arena = MemorySegment.ofAddress(
    world_get_shared_arena_address(world));    // base u64 from C ABI
long size = world_get_shared_arena_size(world);
arena = arena.reinterpret(size);

// 命令环写一跳：偏移 = cmd_ring_offset + head*CMD_SLOT_STRIDE(=32)
long cmdRingOff = arena.get(java_lang_LONG, 96);   // header.region.cmd_ring
MemorySegment cmdSlot = arena.asSlice(cmdRingOff + head*32, 32);
cmdSlot.set(java_lang_INT, 0, 0 /*AddForce*/);
cmdSlot.set(java_lang_LONG, 8, bodyId);
cmdSlot.set(MY_VEC3_LAYOUT, 16, force);
    // head 推进由 Rust 侧 step 应用时做 — Java 只写 payload"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"JNI 路径走 "</span><span data-lang="en">"JNI path uses "</span> }<code>"DirectByteBuffer"</code>{ <span data-lang="zh">" 是同一指针的另一面 (MemorySegment.ofBuffer)。"</span><span data-lang="en">"— same pointer's other face (MemorySegment.ofBuffer)."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" 跨线程契约：Body/IntegrationParams 区域用 generation-counter 协议；Command/Event 环 SPSC (Rust 物理线程产 event、Java 消费；Java 产 command、Rust 消费)。每步开始 Rust apply command、step 结束 push event——Java 永不直接驱动物理推进，只通过环通信。"</span><span data-lang="en">" Cross-thread contract: Body/IntegrationParams regions use generation-counter protocol; Command/Event rings are SPSC (Rust physics thread produces events, Java consumes; Java produces commands, Rust consumes). At step start Rust applies commands; at step end pushes events — Java never drives physics directly, only via rings."</span> }</p>
            </div>

            <div class="callout callout-warn">
                <p>{ <span data-lang="zh">" ⚠ destroy 前必须 Java 释放 — "</span><span data-lang="en">" ⚠ Must release on Java side before destroy — "</span> }<code>"world_destroy_shared_arena"</code>{ <span data-lang="zh">" 直接释放底层内存，仍映射的 MemorySegment 会成为悬空指针。文档将此列为 WARNING，Rust 侧无 GC 协议检测，调用方负责生命周期。"</span><span data-lang="en">" directly frees underlying memory; still-mapped MemorySegment becomes dangling. Documented as WARNING; Rust side has no GC protocol detection, lifetime responsibility is on the caller."</span> }</p>
            </div>
        </div>
    }
}
