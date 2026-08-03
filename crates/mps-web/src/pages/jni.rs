use topcoat::router::page;
use topcoat::view::view;

/// JNI page
#[page("/jni")]
pub async fn jni() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ JNI"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"Java 21 JNI 绑定（fluent builder）"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "mps-jni 编出 cdylib " }<code>"mps_rigid_body"</code>{ "，288 个 " }<code>"jni! / jni_e_c!"</code>{ " 宏导出方法分两块：通用物理走 mps-core 的 " }<code>"RigidBodyNative"</code>{ "（" }<code>"org.polaris2023.mps_rigid_body.RigidBodyNative"</code>{ "），太空刚体走 cosmos* 系列。每方法都由 catch_unwind 兜底——panic 永不进 JVM。" }</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "jni! 宏：每绑定一行" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "几乎每条绑定就是一次宏调用——自动生成 JNI 符号名 " }<code>"Java_org_polaris2023_mps_1rigid_1body_RigidBodyNative_<方法>"</code>{ "、" }<code>"extern \"system\""</code>{ " 调用约定、以及 panic-guard。Java 包名/类名钉死在宏的 concat! 里，重命名须同步改宏。" }</p>
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
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "panic 路径：catch_unwind 抓住 → set_error(ERR_INTERNAL, \"internal panic\") → 返回该返回类型的 @default（0/0.0/()/null）。这是 mps-core " }<code>"ffi_guard"</code>{ " 在 JNI 侧的对应物。配合 workspace " }<code>"panic = \"abort\""</code>{ "，双保险。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "fluent builder（cosmos*）" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "太空路径用 cosmos* 系列——把 " }<a href="./cosmos" style="color:#4a9eff;">"CosmosWorld"</a>{ " 的 Rust pub API 暴露给 Java，传 long 句柄 + 配置数：orbit_integration、verlet_substeps、n_body_softening_sq、max_sh_degree、PerturbationConfig 等。" }</p>
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
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "cosmosWorldStep 把 StepResult 压成单 int —— Java " }<code>"if (r > 0)"</code>{ " 即判成功，细节用 cosmosWorldStepDetailed。这种 \"单 int 装多义\" 是 JNI 减少 JNI 调用代价的常见取舍。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "test21 smoke test" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "Java 21 烟雾测试在仓库根的 " }<code>"test21"</code>{ " Gradle 子项目：加载刚 cargo build 出的 " }<code>"mps_rigid_body.dll/.so"</code>{ "，调一组 native 方法验证符号齐全、传参对齐、panic-guard 生效。CI 每次构建后跑它——新增 jni 方法的最后一步是同步给 test21 的 " }<code>"RigidBodyNative.java"</code>{ " 加匹配的 " }<code>"native"</code>{ " 声明。" }</p>
                <pre><code class="language-bash">
"# 1) Rust 侧构建 JNI cdylib
cargo build --release -p mps-jni

# 2) Java 侧跑烟雾
cd test21
./gradlew.bat check            # Windows
gradlew check                  # *nix"
                </code></pre>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "新增 Java 方法 —— 标准流程" }</h2>
                <ol style="color:#999;line-height:2;padding-left:20px;">
                    <li>{ "确认 C ABI 已在 mps-core 存在（或先加，再 cargo build -p mps-core 拿到重新生成的 rigid_body.h，diff 必提交）" }</li>
                    <li>{ "在 crates/mps-jni/src/lib.rs 加一行 jni! / jni_e_c!，用 m/pm/p/cp/v3/qt/jb/u32_from_jint 这些 marshalling helper 做 jlong↔*mut T、Vec3↔double[3] 等" }</li>
                    <li>{ "test21 的 RigidBodyNative.java 加匹配 native 声明（与 test25 的 FFM upcall，若需 FFM 可见）——FFM 平价缺口见 README \"Current Gaps\"" }</li>
                    <li>{ "CI 必过：fmt --check、clippy -D warnings、test、build --release，再 test21/test25 Gradle，再 generated-header gate" }</li>
                </ol>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " ABI 分类：无需 env 的 static 方法走 " }<code>"jni!"</code>{ "（占大多数）；要返回 jdoubleArray / 接 jbyteArray / NewDirectByteBuffer 这类需 JNIEnv 的走 " }<code>"jni_e_c!"</code>{ " 或手写 extern \"system\"（如 " }<code>"worldGetArenaDirectByteBuffer"</code>{ "、"}<code>"abiLastErrorMessage"</code>{ "）。后者数量少、且仅在需要 env service 时出现。详见 CLAUDE.md \"Binding style\" 节。" }</p>
            </div>
        </div>
    }
}
