use topcoat::router::page;
use topcoat::view::view;

use crate::metrics::JNI_METHOD_COUNT;

/// JNI page
#[page("/jni")]
pub async fn jni() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">"/ JNI"</div>
                    <h1 class="page-title"><span data-lang="zh">"Java 21 JNI 绑定（fluent builder）"</span><span data-lang="en">"Java 21 JNI Bindings (fluent builder)"</span></h1>
                    <p class="page-desc">{ <span data-lang="zh">"mps-jni 编出 cdylib "</span><span data-lang="en">"mps-jni compiles cdylib "</span> }<code>"mps_rigid_body"</code>{ <span data-lang="zh">"，"</span><span data-lang="en">", "</span> }{ (JNI_METHOD_COUNT) }{ <span data-lang="zh">" 个 "</span><span data-lang="en">" "</span> }<code>"jni! / jni_e_c!"</code>{ <span data-lang="zh">" 宏导出方法分两块：通用物理走 mps-core 的 "</span><span data-lang="en">"-macro exported methods split into two blocks: general physics via mps-core "</span> }<code>"RigidBodyNative"</code>{ <span data-lang="zh">"（"</span><span data-lang="en">" ("</span> }<code>"org.polaris2023.mps_rigid_body.RigidBodyNative"</code>{ <span data-lang="zh">"），太空刚体走 cosmos* 系列。每方法都由 catch_unwind 兜底——panic 永不进 JVM。"</span><span data-lang="en">"), cosmos rigid body via cosmos* series. Every method guarded by catch_unwind — panics never enter JVM."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "jni! 宏：每绑定一行" }</span><span data-lang="en">{ "jni! Macro: One Line Per Binding" }</span></h2>
                <p class="p-lead">{ <span data-lang="zh">"几乎每条绑定就是一次宏调用——自动生成 JNI 符号名 "</span><span data-lang="en">"Almost every binding is one macro call — auto-generates JNI symbol name "</span> }<code>"Java_org_polaris2023_mps_1rigid_1body_RigidBodyNative_<方法>"</code>{ <span data-lang="zh">"、"</span><span data-lang="en">", "</span> }<code>"extern \"system\""</code>{ <span data-lang="zh">" 调用约定、以及 panic-guard。Java 包名/类名钉死在宏的 concat! 里，重命名须同步改宏。"</span><span data-lang="en">" calling convention, and panic-guard. Java package/class names are pinned in the macro's concat!; renaming must sync the macro."</span> }</p>
                <pre><code class="language-rust">
"jni!(int cosmosWorldStep(long world, double dt) {
    let w = unsafe { (world as *mut CosmosWorld).as_mut() };
    let Some(w) = w else { return -2; };
    match w.step(dt) {
        StepResult::Stepped(n)   => ((n * 1000.0).round() as i64).max(1) as jint,
        StepResult::Substepped{..} => -1,
        StepResult::Skipped(StepSkipReason::NonFinite)   => -2,
        StepResult::Skipped(StepSkipReason::NonPositive) => -3,
        StepResult::Skipped(StepSkipReason::TooLarge)    => -4,
    }
});"
                </code></pre>
                <p class="p-note">{ "panic 路径：catch_unwind 抓住 → set_error(ERR_INTERNAL, \"internal panic\") → 返回该返回类型的 @default（0/0.0/()/null）。这是 mps-core " }<code>"ffi_guard"</code>{ <span data-lang="zh">" 在 JNI 侧的对应物。配合 workspace "</span><span data-lang="en">"'s JNI-side counterpart. Together with workspace "</span> }<code>"panic = \"abort\""</code>{ <span data-lang="zh">"，双保险。"</span><span data-lang="en">", double safety."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"fluent builder（cosmos*）"</span><span data-lang="en">"fluent builder (cosmos*)"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"太空路径用 cosmos* 系列——把 "</span><span data-lang="en">"Cosmos path uses cosmos* series — exposes "</span> }<a href="./cosmos" class="link">"CosmosWorld"</a>{ <span data-lang="zh">" 的 Rust pub API 暴露给 Java，传 long 句柄 + 配置数：orbit_integration、verlet_substeps、n_body_softening_sq、max_sh_degree、PerturbationConfig 等。"</span><span data-lang="en">"'s Rust pub API to Java, taking long handle + config: orbit_integration, verlet_substeps, n_body_softening_sq, max_sh_degree, PerturbationConfig, etc."</span> }</p>
                <pre><code class="language-java">
"long world   = RigidBodyNative.cosmosWorldCreate(1.0, 4, 4, 3 /*Yoshida4*/, 1, 1e3);
RigidBodyNative.cosmosWorldSetCentralBody(world, 3 /*Earth*/);
int src     = RigidBodyNative.cosmosWorldAddCelestial(world, 4 /*Moon*/, 0);
long sat    = RigidBodyNative.cosmosWorldInsertBody(world,
                  RigidBodyNative.cosmosSatelliteBuilder(7e6, 7800.0));

int r       = RigidBodyNative.cosmosWorldStep(world, 1.0);
// r > 0 = 步进子步数×1000;  -1 Substepped;  -2..-4 Skipped(NaN/≤0/)>30s)
double[] p  = RigidBodyNative.cosmosBodyTranslationOut(world, sat, new double[3]);"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"cosmosWorldStep 把 StepResult 压成单 int —— Java "</span><span data-lang="en">"cosmosWorldStep packs StepResult into a single int — Java "</span> }<code>"if (r > 0)"</code>{ " 即判成功，细节用 cosmosWorldStepDetailed。这种 \"单 int 装多义\" 是 JNI 减少 JNI 调用代价的常见取舍。" }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"test21 smoke test"</span><span data-lang="en">"test21 smoke test"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"Java 21 烟雾测试在仓库根的 "</span><span data-lang="en">"Java 21 smoke test lives in repo root "</span> }<code>"test21"</code>{ <span data-lang="zh">" Gradle 子项目：加载刚 cargo build 出的 "</span><span data-lang="en">" Gradle subproject: loads freshly cargo-built "</span> }<code>"mps_rigid_body.dll/.so"</code>{ <span data-lang="zh">"，调一组 native 方法验证符号齐全、传参对齐、panic-guard 生效。CI 每次构建后跑它——新增 jni 方法的最后一步是同步给 test21 的 "</span><span data-lang="en">", calls a set of native methods to verify symbols exist, parameter passing aligns, and panic-guard works. CI runs it after every build — the last step when adding a jni method is to sync test21's "</span> }<code>"RigidBodyNative.java"</code>{ <span data-lang="zh">" 加匹配的 "</span><span data-lang="en">" with matching "</span> }<code>"native"</code>{ <span data-lang="zh">" 声明。"</span><span data-lang="en">" declaration."</span> }</p>
                <pre><code class="language-bash">
"# 1) Rust 侧构建 JNI cdylib
cargo build --release -p mps-jni

# 2) Java 侧跑烟雾
cd test21
./gradlew.bat check            # Windows
gradlew check                  # *nix"
                </code></pre>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"新增 Java 方法 —— 标准流程"</span><span data-lang="en">"Adding Java Methods — Standard Workflow"</span> }</h2>
                <ol class="ul-plain">
                    <li>{ <span data-lang="zh">"确认 C ABI 已在 mps-core 存在（或先加，再 cargo build -p mps-core 拿到重新生成的 rigid_body.h，diff 必提交）"</span><span data-lang="en">"Confirm C ABI exists in mps-core (or add it first, then cargo build -p mps-core to regenerate rigid_body.h, diff must be committed)"</span> }</li>
                    <li>{ <span data-lang="zh">"在 crates/mps-jni/src/lib.rs 加一行 jni! / jni_e_c!，用 m/pm/p/cp/v3/qt/jb/u32_from_jint 这些 marshalling helper 做 jlong↔*mut T、Vec3↔double[3] 等"</span><span data-lang="en">"Add a jni! / jni_e_c! line in crates/mps-jni/src/lib.rs, use marshalling helpers m/pm/p/cp/v3/qt/jb/u32_from_jint for jlong↔*mut T, Vec3↔double[3], etc."</span> }</li>
                    <li>{ "test21 的 RigidBodyNative.java 加匹配 native 声明（与 test25 的 FFM upcall，若需 FFM 可见）——FFM 平价缺口见 README \"Current Gaps\"" }</li>
                    <li>{ <span data-lang="zh">"CI 必过：fmt --check、clippy -D warnings、test、build --release，再 test21/test25 Gradle，再 generated-header gate"</span><span data-lang="en">"CI must pass: fmt --check, clippy -D warnings, test, build --release, then test21/test25 Gradle, then generated-header gate"</span> }</li>
                </ol>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" ABI 分类：无需 env 的 static 方法走 "</span><span data-lang="en">" ABI classification: env-free static methods use "</span> }<code>"jni!"</code>{ <span data-lang="zh">"（占大多数）；要返回 jdoubleArray / 接 jbyteArray / NewDirectByteBuffer 这类需 JNIEnv 的走 "</span><span data-lang="en">" (most); returning jdoubleArray / taking jbyteArray / NewDirectByteBuffer needs JNIEnv and uses "</span> }<code>"jni_e_c!"</code>{ " 或手写 extern \"system\"（如 " }<code>"worldGetArenaDirectByteBuffer"</code>{ <span data-lang="zh">"、"</span><span data-lang="en">", "</span>}<code>"abiLastErrorMessage"</code>{ "）。后者数量少、且仅在需要 env service 时出现。详见 CLAUDE.md \"Binding style\" 节。" }</p>
            </div>
        </div>
    }
}
