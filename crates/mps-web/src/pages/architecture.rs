use topcoat::router::page;
use topcoat::view::view;

/// Architecture overview
#[page("/architecture")]
pub async fn architecture() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ CORE MODULE"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">"架构概述"</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "MPS 的分层架构 —— 一棵 Rapier3D-f64 物理核心，上层挂 C ABI / JNI / FFM / 纯公式 / 太空演算多个侧枝。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">{ "分层架构" }</h2>
                <pre style="background:#0d0d2b; border:1px solid #333; border-radius:6px; padding:16px; font-size:13px; line-height:1.5;">
                    <code class="language-text">
"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI (~380 函数, mps-core)        ┌─ mps-cosmos Rust pub API
       ├─ mps-formula  — 28 纯公式模块 (300+ 函数) │   (CosmosWorld, 不经 C ABI)
       ├─ mps-core     — 物理引擎 + Rapier 封装     │
       ├─ mps-cosmos   — 太空刚体演算 (独立 world) ──┘
       ├─ mps-jni      — JNI 绑定 (~290 方法, 含 cosmos*)
       ├─ mps-ffm      — FFM 元数据 (Java 25)
       └─ mps-test     — 342 集成测试 (含 cosmos 19)"
                    </code>
                </pre>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "唯一基础依赖是 " }<code>"rapier3d-f64"</code>{ "（f64 后端，地面/通用）+ " }<code>"rapier3d-f64"</code>{ " 的另一次实例化（mps-cosmos 自持）。mps-formula 是 " }<strong>"零依赖"</strong>{ " 纯函数层，不依赖 rapier，可独立编译运行。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">{ "逐 crate 角色" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"Crate"</th><th>"角色"</th><th>"导出形态"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"mps-formula"</code></td><td>{ "28 纯公式模块 (航天/天体/核/相对论/...) + 公共印象误差/数学 + last-error 线程槽" }</td><td>{ "pub fn + 静态数据; panic-free" }</td></tr>
                            <tr><td><code>"mps-core"</code></td><td>{ "Rapier 封装 + C ABI（~380 fns）+ ForceRegistry + SharedArena" }</td><td>{ "extern \"C\" + rigid_body.h" }</td></tr>
                            <tr><td><code>"mps-cosmos"</code></td><td>{ "独立太空 world (Phase/碰撞/姿态 + 辛积子轨道/n-body/扰动)" }</td><td>{ "pub Rust API only" }</td></tr>
                            <tr><td><code>"mps-jni"</code></td><td>{ "JNI 绑定 (Java 21), panic-guard via catch_unwind, jni! 宏生成符号" }</td><td>{ ".dll/.so + .class" }</td></tr>
                            <tr><td><code>"mps-ffm"</code></td><td>{ "FFM 元数据（rigid_body.h 的 Java 25 描述）" }</td><td>{ "Linker downcall 元数据" }</td></tr>
                            <tr><td><code>"mps-test"</code></td><td>{ "342 集成测试，直接调 extern \"C\"" }</td><td>{ "#[test]，workspace 独占" }</td></tr>
                            <tr><td><code>"mps-web"</code></td><td>{ "本文档站 (Topcoat，server-side 渲染)" }</td><td>{ "静态 HTML → GitHub Pages" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">{ "PhysicsWorld 核心组件 (mps-core)" }</h2>
                <p style="color:#aaa; line-height:1.7;">
                    { "PhysicsWorld (mps-core) 包含以下核心组件 - 通用/地面场景使用; " }<strong>"CosmosWorld"</strong>{ " (mps-cosmos) 为太空轨道演算独立持有同款 rapier 后端, 详见 " }<a href="./cosmos" style="color:#4a9eff;">"太空演算"</a>{ "。" }
                </p>
                <ul style="color:#999; line-height:2; padding-left:20px;">
                    <li><strong style="color:#ddd;">"PhysicsPipeline"</strong> { " - Rapier 物理管线 (碰撞检测 + 约束求解)" }</li>
                    <li><strong style="color:#ddd;">"RigidBodySet / ColliderSet"</strong> { " - 刚体/碰撞体集合 (RigidBodyHandle = u64 packed)" }</li>
                    <li><strong style="color:#ddd;">"ImpulseJointSet / MultibodyJointSet"</strong> { " - 脉冲关节 / 多体关节" }</li>
                    <li><strong style="color:#ddd;">"IslandManager"</strong> { " - 睡眠/唤醒岛管理" }</li>
                    <li><strong style="color:#ddd;">"BroadPhaseBvh / NarrowPhase"</strong> { " - 宽相位 BVH + 窄相位" }</li>
                    <li><strong style="color:#ddd;">"CCDSolver"</strong> { " - 连续碰撞检测求解器" }</li>
                    <li><strong style="color:#ddd;">"IntegrationParameters"</strong> { " - dt / solver iterations / CCD substeps" }</li>
                    <li><strong style="color:#ddd;">"ForceRegistry"</strong> { " - 类型化力注册表 (CoulombFriction / AirDrag / External / NewtonGravity laws)" }</li>
                    <li><strong style="color:#ddd;">"FrameWorkBuffers"</strong> { " - 预分配工作缓冲区（每帧复用，零分配热路径）" }</li>
                    <li><strong style="color:#ddd;">"SharedArena (optional)"</strong> { " - world_create_shared_arena 申请的 native 缓冲，Java DirectByteBuffer / FFM MemorySegment 直接读" }</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="font-size:20px;color:#fff;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "两个 World 的分工" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>" "</th><th>"mps-core PhysicsWorld"</th><th>"mps-cosmos CosmosWorld"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"场景"</strong></td><td>{ "地面 / 通用刚体" }</td><td>{ "太空轨道演练" }</td></tr>
                            <tr><td><strong>"重力"</strong></td><td>{ "全局重力锚 + ForceRegistry" }</td><td>{ "天体源 + n-body 互引力" }</td></tr>
                            <tr><td><strong>"轨道积分"</strong></td><td>{ "rapier semi-implicit Euler" }</td><td>{ "可选 辛积子 (Verlet/Yoshida4/ForestRuth8 + Kahan)" }</td></tr>
                            <tr><td><strong>"环境扰动"</strong></td><td>"-"</td><td>{ "大气阻力 + 太阳光压" }</td></tr>
                            <tr><td><strong>"C ABI / Arena"</strong></td><td>{ "共享 arena + 力律登记表" }</td><td>{ "独立持有, 不经 C ABI (Rust pub API)" }</td></tr>
                            <tr><td><strong>"Java 路径"</strong></td><td><code>"RigidBodyNative / RigidBodyFfm"</code></td><td><code>"cosmos*"</code>{ " 语义（JNI）" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">{ "IntegrationParameters 默认值" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead>
                            <tr><th>"参数"</th><th>"默认值"</th><th>"说明"</th></tr>
                        </thead>
                        <tbody>
                            <tr><td><code>"dt"</code></td><td>"1/60"</td><td>{ "时间步长 秒" }</td></tr>
                            <tr><td><code>"num_solver_iterations"</code></td><td>"4"</td><td>{ "每个子步的约束求解器迭代次数（越大越准越慢）" }</td></tr>
                            <tr><td><code>"max_ccd_substeps"</code></td><td>"1"</td><td>{ "快速移动体的 CCD 求解子步数（mps-cosmos 默认 4）" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "通过 " }<code>"world_set_integration_parameters(world, dt, solver_iter, ccd_sub)"</code>{ " 改。失败会写 last-error 并返回 " }<code>"Bool::FALSE"</code>{ "（如 dt ≤ 0）。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 构建：workspace 根 " }<code>"Cargo.toml"</code>{ " 设 " }<code>"panic = \"abort\""</code>{ "。所有 FFI 入口都由 ffi_guard (mps-core) 或 catch_unwind (mps-jni) 兜底 —— panic 永不 unwind 穿过 FFI 边界。对 Java 调用者而言，native panic 表现为返回 ERR_INTERNAL 的 abiLastErrorMessage()。" }</p>
            </div>
        </div>
    }
}
