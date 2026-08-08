use topcoat::router::page;
use topcoat::view::view;

/// FFM page
#[page("/ffm")]
pub async fn ffm() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">"/ FFM"</div>
                    <h1 class="page-title">"Java 25 Foreign Function & Memory API"</h1>
                    <p class="page-desc">{ <span data-lang="zh">"test25 用 "</span><span data-lang="en">"test25 uses "</span> }<code>"RigidBodyFfm"</code>{ <span data-lang="zh">" 直接 downcall mps-core 的 C ABI —— 不经 JNI 符号命名，按 rigid_body.h 原名调用。mps-ffm 仅提供 ABI 版本元数据（abi_version / abi_supports_ffm / abi_supports_jni），让双方协商兼容。"</span><span data-lang="en">" to directly downcall mps-core C ABI — no JNI symbol naming, calls rigid_body.h by original name. mps-ffm provides only ABI version metadata (abi_version / abi_supports_ffm / abi_supports_jni) for compatibility negotiation."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "ABI 元数据（mps-ffm）" }</span><span data-lang="en">{ "ABI Metadata (mps-ffm)" }</span></h2>
                <pre><code class="language-rust">
"pub const ABI_VERSION: u32 = 1;

pub extern "C" fn abi_version() -> u32 { ABI_VERSION }
pub extern "C" fn abi_supports_ffm() -> Bool { Bool::TRUE }
pub extern "C" fn abi_supports_jni() -> Bool { Bool::TRUE }"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"RigidBodyFfm 启动时先调 abi_version() 比较 ABI_VERSION 不匹配 → 抛 IllegalStateException 退场。这是 FFM 路径的版本门——evens reflow 出 rigid_body.h 改动时必须 bump ABI_VERSION。"</span><span data-lang="en">"RigidBodyFfm calls abi_version() at startup; ABI_VERSION mismatch → throws IllegalStateException. This is the FFM path's version gate — any rigid_body.h change must bump ABI_VERSION."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "downcall 模型" }</span><span data-lang="en">{ "Downcall Model" }</span></h2>
                <p class="p-lead">{ "test25 为每个 rigid_body.h 函数建一个 MethodHandle —— Linker.nativeLinker().downcallHandle(lookup(\"world_step\"), FunctionDescriptor.ofVoid(ADDRESS, JAVA_DOUBLE), ...). 结构 exactly 对应 #[repr(C)] 的 Rust struct 用 MemoryLayout.structLayout 把字段名 + 对齐 padding 钉到字节，Java 侧布局与 Rust 侧布局 1:1。" }</p>
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
                <p class="p-note">{ <span data-lang="zh">"执行："</span><span data-lang="en">"Execute: "</span> }<code>"worldStep.invoke(worldSeg, 1.0/60.0)"</code>{ <span data-lang="zh">"。无 JNIEnv、无 Java 包名拼接 —— 直接按 C 原名 -------- rigid_body.h 函数名与 test25 downcall 字符串须字节一致；header 重建后 test25 调任何新函数需补 downcall + MethodHandle.invoke 调用点。"</span><span data-lang="en">". No JNIEnv, no Java package name concatenation — calls C symbols by original name. rigid_body.h function names must match test25 downcall strings byte-for-byte; after header rebuild, test25 calls to any new function need new downcall + MethodHandle.invoke call site."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"SharedPhysicsArena 直映射"</span><span data-lang="en">"SharedPhysicsArena Direct Mapping"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"FFM 路径让 arena 零拷贝变得自然——"</span><span data-lang="en">"FFM path makes arena zero-copy natural — "</span> }<a href="./arena" class="link">"shared arena"</a>{ <span data-lang="zh">" 返回的 u64 基址被 "</span><span data-lang="en">" returned u64 base address is "</span> }<code>"MemorySegment.ofAddress(addr).reinterpret(size)"</code>{ <span data-lang="zh">" 重映射为整段可读写段。Body slots / Command ring 全部按 header 里读出的 region offset 直接 sli‌ce 访问。test25 这里是 FFM 相对 JNI 的最大优势 —— JNI 路径要 NewDirectByteBuffer 一层包装。"</span><span data-lang="en">" remapped into a full read-write segment. Body slots / Command ring accessed directly via region offsets read from header. This is FFM's biggest advantage over JNI — JNI needs a NewDirectByteBuffer wrapper."</span> }</p>
                <pre><code class="language-java">
"long addr = RigidBodyFfm.worldGetSharedArenaAddress(world);
long size = RigidBodyFfm.worldGetSharedArenaSize(world);
MemorySegment arena = MemorySegment.ofAddress(addr).reinterpret(size);
long cmdRingOff = arena.get(ValueLayout.JAVA_LONG, 96);  // header @ OFF_CMD_RING"
                </code></pre>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"test25 覆盖范围"</span><span data-lang="en">"test25 Coverage"</span> }</h2>
                <ul class="ul-plain">
                    <li><span data-lang="zh">"World / 刚体 / 碰撞体 / CRbTree 基础"</span><span data-lang="en">"World / rigid body / collider / CRbTree basics"</span></li>
                    <li>{ <span data-lang="zh">"Voxel AABB/OBB 构建统计 + 碰撞体创建"</span><span data-lang="en">"Voxel AABB/OBB build stats + collider creation"</span> }</li>
                    <li>{ <span data-lang="zh">"Voxel AABB/OBB 相交查询"</span><span data-lang="en">"Voxel AABB/OBB intersection query"</span> }</li>
                    <li>{ <span data-lang="zh">"常规查询：ray cast / point projection / AABB/OBB/sphere intersection / shape cast"</span><span data-lang="en">"General queries: ray cast / point projection / AABB/OBB/sphere intersection / shape cast"</span> }</li>
                    <li>{ <span data-lang="zh">"刚体运行时突变：pose / 速度 / 力·力矩 / 冲量 / CCD / 睡眠·唤醒"</span><span data-lang="en">"Rigid body runtime mutation: pose / velocity / force·torque / impulse / CCD / sleep·wake"</span> }</li>
                    <li>{ <span data-lang="zh">"气动 / 升力面累加助手（参考 "</span><span data-lang="en">"Aero / lift-surface accumulate helper (see "</span> }<a href="./voxel" class="link">"voxel"</a>{ <span data-lang="zh">"）"</span><span data-lang="en">")"</span> }</li>
                    <li>{ <span data-lang="zh">"碰撞体运行时突变：pose / sensor / 摩擦 / 恢复 / groups / event bits / hooks / contact-force 阈值"</span><span data-lang="en">"Collider runtime mutation: pose / sensor / friction / restitution / groups / event bits / hooks / contact-force threshold"</span> }</li>
                    <li>{ <span data-lang="zh">"碰撞 / 接触力事件批量读 + clear"</span><span data-lang="en">"Collision / contact force event batch read + clear"</span> }</li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"运行测试"</span><span data-lang="en">"Run Tests"</span> }</h2>
                <pre><code class="language-bash">
"cargo build --release -p mps-core   # 先生成 rigid_body.h + mps_rigid_body.dll
cd test25
./gradlew.bat check"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"test25 需 JDK 25（FFM 为 preview→stable 特性）。CI 先构建 mps-jni（产出 mps_rigid_body cdylib，test25 与 test21 共享同一 native lib），再跑 Gradle。"</span><span data-lang="en">"test25 needs JDK 25 (FFM is preview→stable feature). CI first builds mps-jni (produces mps_rigid_body cdylib shared by test25 and test21), then runs Gradle."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" JNI vs FFM：JNI 路径每绑定一行 jni! 宏 + Java 侧 native 声明，符号名钉死；FFM 路径每函数一份 MemoryLayout + downcall + 调用点，无符号命名耦合，但布局须与 #[repr(C)] 字节一致。两者共用同一 mps_rigid_body native lib——mps-jni 编它，FFM 仅 Linker 直接调。"</span><span data-lang="en">" JNI vs FFM: JNI path adds one jni! macro line + Java-side native declaration per binding, symbol name pinned; FFM path builds one MemoryLayout + downcall + call site per function, no symbol naming coupling, but layout must match #[repr(C)] byte-for-byte. Both share the same mps_rigid_body native lib — mps-jni compiles it; FFM just calls via Linker."</span> }</p>
            </div>

            <div class="callout callout-note">
                <p>{ " FFM 平价缺口：cosmos* 系列、部分新 ABI 入口在 test25 尚未对齐 downcall。新增 mps-core FFI 后要么补 test25 downcall，要么在 README \"Current Gaps\" 记账 —— 后者仅推迟公开契约，不能放弃。" }</p>
            </div>
        </div>
    }
}
