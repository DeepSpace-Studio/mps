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
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"Java 25 Foreign Function & Memory API"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "test25 用 " }<code>"RigidBodyFfm"</code>{ " 直接 downcall mps-core 的 C ABI —— 不经 JNI 符号命名，按 rigid_body.h 原名调用。mps-ffm 仅提供 ABI 版本元数据（abi_version / abi_supports_ffm / abi_supports_jni），让双方协商兼容。" }</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "ABI 元数据（mps-ffm）" }</h2>
                <pre><code class="language-rust">
"pub const ABI_VERSION: u32 = 1;

pub extern "C" fn abi_version() -> u32 { ABI_VERSION }
pub extern "C" fn abi_supports_ffm() -> Bool { Bool::TRUE }
pub extern "C" fn abi_supports_jni() -> Bool { Bool::TRUE }"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "RigidBodyFfm 启动时先调 abi_version() 比较 ABI_VERSION 不匹配 → 抛 IllegalStateException 退场。这是 FFM 路径的版本门——evens reflow 出 rigid_body.h 改动时必须 bump ABI_VERSION。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "downcall 模型" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "test25 为每个 rigid_body.h 函数建一个 MethodHandle —— Linker.nativeLinker().downcallHandle(lookup(\"world_step\"), FunctionDescriptor.ofVoid(ADDRESS, JAVA_DOUBLE), ...). 结构 exactly 对应 #[repr(C)] 的 Rust struct 用 MemoryLayout.structLayout 把字段名 + 对齐 padding 钉到字节，Java 侧布局与 Rust 侧布局 1:1。" }</p>
                <pre><code class="language-java">
"public static final MemoryLayout VEC3 = MemoryLayout.structLayout(
        ValueLayout.JAVA_DOUBLE.withName("x"),
        ValueLayout.JAVA_DOUBLE.withName("y"),
        ValueLayout.JAVA_DOUBLE.withName("z"));

worldStep = downcall("world_step",
    FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_DOUBLE));
worldCreate = downcall("world_create",
    FunctionDescriptor.of(ValueLayout.ADDRESS, VEC3));"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "执行：" }<code>"worldStep.invoke(worldSeg, 1.0/60.0)"</code>{ "。无 JNIEnv、无 Java 包名拼接 —— 直接按 C 原名 -------- rigid_body.h 函数名与 test25 downcall 字符串须字节一致；header 重建后 test25 调任何新函数需补 downcall + MethodHandle.invoke 调用点。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "SharedPhysicsArena 直映射" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "FFM 路径让 arena 零拷贝变得自然——" }<a href="./arena" style="color:#4a9eff;">"shared arena"</a>{ " 返回的 u64 基址被 " }<code>"MemorySegment.ofAddress(addr).reinterpret(size)"</code>{ " 重映射为整段可读写段。Body slots / Command ring 全部按 header 里读出的 region offset 直接 sli‌ce 访问。test25 这里是 FFM 相对 JNI 的最大优势 —— JNI 路径要 NewDirectByteBuffer 一层包装。" }</p>
                <pre><code class="language-java">
"long addr = RigidBodyFfm.worldGetSharedArenaAddress(world);
long size = RigidBodyFfm.worldGetSharedArenaSize(world);
MemorySegment arena = MemorySegment.ofAddress(addr).reinterpret(size);
long cmdRingOff = arena.get(ValueLayout.JAVA_LONG, 96);  // header @ OFF_CMD_RING"
                </code></pre>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "test25 覆盖范围" }</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li>"World / 刚体 / 碰撞体 / CRbTree 基础"</li>
                    <li>{ "Voxel AABB/OBB 构建统计 + 碰撞体创建" }</li>
                    <li>{ "Voxel AABB/OBB 相交查询" }</li>
                    <li>{ "常规查询：ray cast / point projection / AABB/OBB/sphere intersection / shape cast" }</li>
                    <li>{ "刚体运行时突变：pose / 速度 / 力·力矩 / 冲量 / CCD / 睡眠·唤醒" }</li>
                    <li>{ "气动 / 升力面累加助手（参考 " }<a href="./voxel" style="color:#4a9eff;">"voxel"</a>{ "）" }</li>
                    <li>{ "碰撞体运行时突变：pose / sensor / 摩擦 / 恢复 / groups / event bits / hooks / contact-force 阈值" }</li>
                    <li>{ "碰撞 / 接触力事件批量读 + clear" }</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "运行测试" }</h2>
                <pre><code class="language-bash">
"cargo build --release -p mps-core   # 先生成 rigid_body.h + mps_rigid_body.dll
cd test25
./gradlew.bat check"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "test25 需 JDK 25（FFM 为 preview→stable 特性）。CI 先构建 mps-jni（产出 mps_rigid_body cdylib，test25 与 test21 共享同一 native lib），再跑 Gradle。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " JNI vs FFM：JNI 路径每绑定一行 jni! 宏 + Java 侧 native 声明，符号名钉死；FFM 路径每函数一份 MemoryLayout + downcall + 调用点，无符号命名耦合，但布局须与 #[repr(C)] 字节一致。两者共用同一 mps_rigid_body native lib——mps-jni 编它，FFM 仅 Linker 直接调。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #f0a04a; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " FFM 平价缺口：cosmos* 系列、部分新 ABI 入口在 test25 尚未对齐 downcall。新增 mps-core FFI 后要么补 test25 downcall，要么在 README \"Current Gaps\" 记账 —— 后者仅推迟公开契约，不能放弃。" }</p>
            </div>
        </div>
    }
}
