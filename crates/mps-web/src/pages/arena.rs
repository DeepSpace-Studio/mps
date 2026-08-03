use topcoat::router::page;
use topcoat::view::view;

/// Arena page
#[page("/arena")]
pub async fn arena() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ ARENA"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">{ "共享内存 Arena（零拷贝状态镜像）" }</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "world_create_shared_arena 在 native 申请一段连续内存，把物理 world 的关键状态（刚体姿态/速度、命令环、事件环、力律聚合、积分参数）以固定 stride 排布。Java 经 DirectByteBuffer / MemorySegment 直接读写——每帧不再跨 FFI 逐字段 memcpy。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "内存布局（ARENA_VERSION=2）" }</h2>
                <pre style="background:#0d0d2b; border:1px solid #333; border-radius:6px; padding:14px; font-size:12px; line-height:1.5;">
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
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "对齐 64 B；零化分配 (alloc_zeroed)。Region 偏移 Java 侧读 header 不重算——跨版本升级仅靠 header 头里的 region offset 向后兼容。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "容量上限" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"容量"</th><th>"硬上限"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"max_bodies"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_colliders"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_events"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><code>"max_commands"</code></td><td>"1,000,000"</td></tr>
                            <tr><td><strong>"total bytes"</strong></td><td>"256 MiB (MAX_ARENA_TOTAL_BYTES)"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "任一上限过或总字节超 " }<code>"world_create_shared_arena"</code>{ " 返回 " }<code>"Bool::FALSE"</code>{ " + 写 " }<code>"ERR_CAPACITY"</code>{ "。每 world 至多一个 arena；重复建返回 " }<code>"ERR_INVALID_ARGUMENT"</code>{ " —— 必须先 " }<code>"world_destroy_shared_arena"</code>{ " 再建下一。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "命令协议（CommandType u32）" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "Java 把命令写入 " }<code>"Command ring"</code>{ "（每槽 32 B：type + body_id + 负载）—— " }<strong>"每步开始时 Rust 侧 apply"</strong>{ "：读 head、解码、更新对应 body / 力注册表，再推进 head。" }</p>
                <div style="overflow-x:auto;margin-top:14px;">
                    <table>
                        <thead><tr><th>"值"</th><th>"命令"</th></tr></thead>
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

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "C ABI 入口" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"函数"</th><th>"用途"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"world_create_shared_arena"</code></td><td>{ "建 arena：传 max_* 容量 + 回写 out_address/out_size（u64）。每 world 至多一个" }</td></tr>
                            <tr><td><code>"world_destroy_shared_arena"</code></td><td>{ "销毁（释放底层内存）。⚠ Java 须先释放映射的 MemorySegment，否则 use-after-free" }</td></tr>
                            <tr><td><code>"world_get_shared_arena_address / _size"</code></td><td>{ "取基址 / 字节数（无则 0），用于 Java 二次映射" }</td></tr>
                            <tr><td><code>"world_reset_shared_arena_events"</code></td><td>{ "排走事件后 Java 调，重置事件环 head/tail" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "Java FFM 落地" }</h2>
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
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "JNI 路径走 " }<code>"DirectByteBuffer"</code>{ " 是同一指针的另一面 (MemorySegment.ofBuffer)。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 跨线程契约：Body/IntegrationParams 区域用 generation-counter 协议；Command/Event 环 SPSC (Rust 物理线程产 event、Java 消费；Java 产 command、Rust 消费)。每步开始 Rust apply command、step 结束 push event——Java 永不直接驱动物理推进，只通过环通信。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #f04a6a; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " ⚠ destroy 前必须 Java 释放 — " }<code>"world_destroy_shared_arena"</code>{ " 直接释放底层内存，仍映射的 MemorySegment 会成为悬空指针。文档将此列为 WARNING，Rust 侧无 GC 协议检测，调用方负责生命周期。" }</p>
            </div>
        </div>
    }
}
