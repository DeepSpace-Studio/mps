use topcoat::router::page;
use topcoat::view::view;

use crate::metrics::CORE_FFI_COUNT;

/// Architecture overview
#[page("/architecture")]
pub async fn architecture() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ CORE MODULE"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">"架构概述"</span><span data-lang="en">"Architecture"</span></h1>
                    <p class="page-desc"><span data-lang="zh">{ "MPS 的分层架构 —— 一棵 Rapier3D-f64 物理核心，上层挂 C ABI / JNI / FFM / 纯公式 / 太空演算多个侧枝。" }</span><span data-lang="en">{ "MPS layered architecture — a Rapier3D-f64 physics core with C ABI / JNI / FFM / pure formula / cosmos side branches." }</span></p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "分层架构" }</span><span data-lang="en">{ "Layered Architecture" }</span></h2>
                <pre class="code-block">
                    <code class="language-text">
"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI ({ (CORE_FFI_COUNT) } 函数, mps-core)        ┌─ mps-cosmos Rust pub API
       ├─ mps-formula  — 28 纯公式模块 (300+ 函数) │   (CosmosWorld, 不经 C ABI)
       ├─ mps-core     — 物理引擎 + Rapier 封装     │
       ├─ mps-cosmos   — 太空刚体演算 (独立 world) ──┘
       ├─ mps-jni      — JNI 绑定 (~290 方法, 含 cosmos*)
       ├─ mps-ffm      — FFM 元数据 (Java 25)
       └─ mps-test     — 342 集成测试 (含 cosmos 19)"
                    </code>
                </pre>
                <p class="p-note-top14">{ <span data-lang="zh">"唯一基础依赖是 "</span><span data-lang="en">"Only base dependency "</span> }<code>"rapier3d-f64"</code>{ <span data-lang="zh">"（f64 后端，地面/通用）+ "</span><span data-lang="en">" (f64 backend, ground/general) + "</span> }<code>"rapier3d-f64"</code>{ <span data-lang="zh">" 的另一次实例化（mps-cosmos 自持）。mps-formula 是 "</span><span data-lang="en">"'s second instantiation (mps-cosmos self-owned). mps-formula is "</span> }<strong><span data-lang="zh">"零依赖"</span><span data-lang="en">"zero-dependency"</span></strong>{ <span data-lang="zh">" 纯函数层，不依赖 rapier，可独立编译运行。"</span><span data-lang="en">" pure-function layer, not dependent on rapier, independently compilable."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "逐 crate 角色" }</span><span data-lang="en">{ "Per-crate Roles" }</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"Crate"</span><span data-lang="en">"Crate"</span></th><th><span data-lang="zh">"角色"</span><span data-lang="en">"Role"</span></th><th><span data-lang="zh">"导出形态"</span><span data-lang="en">"Export Form"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"mps-formula"</code></td><td>{ <span data-lang="zh">"28 纯公式模块 (航天/天体/核/相对论/...) + 公共印象误差/数学 + last-error 线程槽"</span><span data-lang="en">"28 pure-formula modules (spaceflight/celestial/nuclear/relativity/...) + common error/math + last-error thread slot"</span> }</td><td>{ <span data-lang="zh">"pub fn + 静态数据; panic-free"</span><span data-lang="en">"pub fn + static data; panic-free"</span> }</td></tr>
                            <tr><td><code>"mps-core"</code></td><td>{ <span data-lang="zh">"Rapier 封装 + C ABI（"</span><span data-lang="en">"Rapier wrapper + C ABI ("</span> }{ (CORE_FFI_COUNT) }{ <span data-lang="zh">" fns）+ ForceRegistry + SharedArena"</span><span data-lang="en">" fns) + ForceRegistry + SharedArena"</span> }</td><td>{ "extern \"C\" + rigid_body.h" }</td></tr>
                            <tr><td><code>"mps-cosmos"</code></td><td>{ <span data-lang="zh">"独立太空 world (Phase/碰撞/姿态 + 辛积子轨道/n-body/扰动)"</span><span data-lang="en">"Standalone cosmos world (Phase/collision/pose + symplectic orbit/n-body/perturbations)"</span> }</td><td>{ <span data-lang="zh">"pub Rust API only"</span><span data-lang="en">"pub Rust API only"</span> }</td></tr>
                            <tr><td><code>"mps-jni"</code></td><td>{ <span data-lang="zh">"JNI 绑定 (Java 21), panic-guard via catch_unwind, jni! 宏生成符号"</span><span data-lang="en">"JNI bindings (Java 21), panic-guard via catch_unwind, jni! macro generated symbols"</span> }</td><td>{ <span data-lang="zh">".dll/.so + .class"</span><span data-lang="en">".dll/.so + .class"</span> }</td></tr>
                            <tr><td><code>"mps-ffm"</code></td><td>{ <span data-lang="zh">"FFM 元数据（rigid_body.h 的 Java 25 描述）"</span><span data-lang="en">"FFM metadata (Java 25 description of rigid_body.h)"</span> }</td><td>{ <span data-lang="zh">"Linker downcall 元数据"</span><span data-lang="en">"Linker downcall metadata"</span> }</td></tr>
                            <tr><td><code>"mps-test"</code></td><td>{ "342 集成测试，直接调 extern \"C\"" }</td><td>{ <span data-lang="zh">"#[test]，workspace 独占"</span><span data-lang="en">"#[test], workspace exclusive"</span> }</td></tr>
                            <tr><td><code>"mps-web"</code></td><td>{ <span data-lang="zh">"本文档站 (Topcoat，server-side 渲染)"</span><span data-lang="en">"Documentation site (Topcoat, server-side rendered)"</span> }</td><td>{ <span data-lang="zh">"静态 HTML → GitHub Pages"</span><span data-lang="en">"Static HTML → GitHub Pages"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"PhysicsWorld 核心组件 (mps-core)"</span><span data-lang="en">"PhysicsWorld Core Components (mps-core)"</span> }</h2>
                <p class="p-lead">
                    { <span data-lang="zh">"PhysicsWorld (mps-core) 包含以下核心组件 - 通用/地面场景使用; "</span><span data-lang="en">"PhysicsWorld (mps-core) contains these core components — for general/ground scenarios; "</span> }<strong>"CosmosWorld"</strong>{ <span data-lang="zh">" (mps-cosmos) 为太空轨道演算独立持有同款 rapier 后端, 详见 "</span><span data-lang="en">" (mps-cosmos) holds the same rapier backend independently for cosmos scenarios, see "</span> }<a href="./cosmos" class="link"><span data-lang="zh">"太空演算"</span><span data-lang="en">"Cosmos page"</span></a>{ <span data-lang="zh">"。"</span><span data-lang="en">"."</span> }
                </p>
                <ul class="ul-plain">
                    <li><strong>"PhysicsPipeline"</strong> { <span data-lang="zh">" - Rapier 物理管线 (碰撞检测 + 约束求解)"</span><span data-lang="en">" — Rapier physics pipeline (collision detection + constraint solving)"</span> }</li>
                    <li><strong>"RigidBodySet / ColliderSet"</strong> { <span data-lang="zh">" - 刚体/碰撞体集合 (RigidBodyHandle = u64 packed)"</span><span data-lang="en">" — RigidBody/Collider sets (RigidBodyHandle = u64 packed)"</span> }</li>
                    <li><strong>"ImpulseJointSet / MultibodyJointSet"</strong> { <span data-lang="zh">" - 脉冲关节 / 多体关节"</span><span data-lang="en">" — Impulse joints / multibody joints"</span> }</li>
                    <li><strong>"IslandManager"</strong> { <span data-lang="zh">" - 睡眠/唤醒岛管理"</span><span data-lang="en">" — Sleep/wake island management"</span> }</li>
                    <li><strong>"BroadPhaseBvh / NarrowPhase"</strong> { <span data-lang="zh">" - 宽相位 BVH + 窄相位"</span><span data-lang="en">" — Broad-phase BVH + narrow-phase"</span> }</li>
                    <li><strong>"CCDSolver"</strong> { <span data-lang="zh">" - 连续碰撞检测求解器"</span><span data-lang="en">" — Continuous collision detection solver"</span> }</li>
                    <li><strong>"IntegrationParameters"</strong> { <span data-lang="zh">" - dt / solver iterations / CCD substeps"</span><span data-lang="en">" — dt / solver iterations / CCD substeps"</span> }</li>
                    <li><strong>"ForceRegistry"</strong> { <span data-lang="zh">" - 类型化力注册表 (CoulombFriction / AirDrag / External / NewtonGravity laws)"</span><span data-lang="en">" — Typed force registry (CoulombFriction / AirDrag / External / NewtonGravity laws)"</span> }</li>
                    <li><strong>"FrameWorkBuffers"</strong> { <span data-lang="zh">" - 预分配工作缓冲区（每帧复用，零分配热路径）"</span><span data-lang="en">" — Pre-allocated work buffers (reused per frame, zero-alloc hot path)"</span> }</li>
                    <li><strong>"SharedArena (optional)"</strong> { <span data-lang="zh">" - world_create_shared_arena 申请的 native 缓冲，Java DirectByteBuffer / FFM MemorySegment 直接读"</span><span data-lang="en">" — native buffer from world_create_shared_arena, read directly by Java DirectByteBuffer / FFM MemorySegment"</span> }</li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"两个 World 的分工"</span><span data-lang="en">"Two Worlds Split"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th>" "</th><th>"mps-core PhysicsWorld"</th><th>"mps-cosmos CosmosWorld"</th></tr></thead>
                        <tbody>
                            <tr><td><strong><span data-lang="zh">"场景"</span><span data-lang="en">"Scenario"</span></strong></td><td>{ <span data-lang="zh">"地面 / 通用刚体"</span><span data-lang="en">"Ground / general rigid body"</span> }</td><td>{ <span data-lang="zh">"太空轨道演练"</span><span data-lang="en">"Cosmos orbital drills"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"重力"</span><span data-lang="en">"Gravity"</span></strong></td><td>{ <span data-lang="zh">"全局重力锚 + ForceRegistry"</span><span data-lang="en">"Global gravity anchor + ForceRegistry"</span> }</td><td>{ <span data-lang="zh">"天体源 + n-body 互引力"</span><span data-lang="en">"Celestial sources + n-body mutual gravity"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"轨道积分"</span><span data-lang="en">"Orbit Integration"</span></strong></td><td>{ <span data-lang="zh">"rapier semi-implicit Euler"</span><span data-lang="en">"rapier semi-implicit Euler"</span> }</td><td>{ <span data-lang="zh">"可选 辛积子 (Verlet/Yoshida4/ForestRuth8 + Kahan)"</span><span data-lang="en">"Optional symplectic integrator (Verlet/Yoshida4/ForestRuth8 + Kahan)"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"环境扰动"</span><span data-lang="en">"Environmental Perturbations"</span></strong></td><td>"-"</td><td>{ <span data-lang="zh">"大气阻力 + 太阳光压"</span><span data-lang="en">"Atmospheric drag + solar pressure"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"C ABI / Arena"</span><span data-lang="en">"C ABI / Arena"</span></strong></td><td>{ <span data-lang="zh">"共享 arena + 力律登记表"</span><span data-lang="en">"Shared arena + force registry"</span> }</td><td>{ <span data-lang="zh">"独立持有, 不经 C ABI (Rust pub API)"</span><span data-lang="en">"Self-owned, no C ABI (Rust pub API)"</span> }</td></tr>
                            <tr><td><strong><span data-lang="zh">"Java 路径"</span><span data-lang="en">"Java Path"</span></strong></td><td><code>"RigidBodyNative / RigidBodyFfm"</code></td><td><code>"cosmos*"</code>{ <span data-lang="zh">" 语义（JNI）"</span><span data-lang="en">" semantics (JNI)"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"IntegrationParameters 默认值"</span><span data-lang="en">"IntegrationParameters Defaults"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead>
                            <tr><th><span data-lang="zh">"参数"</span><span data-lang="en">"Parameter"</span></th><th><span data-lang="zh">"默认值"</span><span data-lang="en">"Default"</span></th><th><span data-lang="zh">"说明"</span><span data-lang="en">"Notes"</span></th></tr>
                        </thead>
                        <tbody>
                            <tr><td><code>"dt"</code></td><td>"1/60"</td><td>{ <span data-lang="zh">"时间步长 秒"</span><span data-lang="en">"Time step (seconds)"</span> }</td></tr>
                            <tr><td><code>"num_solver_iterations"</code></td><td>"4"</td><td>{ <span data-lang="zh">"每个子步的约束求解器迭代次数（越大越准越慢）"</span><span data-lang="en">"Constraint solver iterations per substep (higher = more accurate but slower)"</span> }</td></tr>
                            <tr><td><code>"max_ccd_substeps"</code></td><td>"1"</td><td>{ <span data-lang="zh">"快速移动体的 CCD 求解子步数（mps-cosmos 默认 4）"</span><span data-lang="en">"CCD solving substeps for fast-moving bodies (mps-cosmos default 4)"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"通过 "</span><span data-lang="en">"Via "</span> }<code>"world_set_integration_parameters(world, dt, solver_iter, ccd_sub)"</code>{ <span data-lang="zh">" 改。失败会写 last-error 并返回 "</span><span data-lang="en">" to change. Failure writes last-error and returns "</span> }<code>"Bool::FALSE"</code>{ <span data-lang="zh">"（如 dt ≤ 0）。"</span><span data-lang="en">" (e.g. dt ≤ 0)."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" 构建：workspace 根 "</span><span data-lang="en">" Build: workspace root "</span> }<code>"Cargo.toml"</code>{ " 设 " }<code>"panic = \"abort\""</code>{ <span data-lang="zh">"。所有 FFI 入口都由 ffi_guard (mps-core) 或 catch_unwind (mps-jni) 兜底 —— panic 永不 unwind 穿过 FFI 边界。对 Java 调用者而言，native panic 表现为返回 ERR_INTERNAL 的 abiLastErrorMessage()。"</span><span data-lang="en">". All FFI entries are guarded by ffi_guard (mps-core) or catch_unwind (mps-jni) — panics never unwind across the FFI boundary. To Java callers, a native panic surfaces as ERR_INTERNAL via abiLastErrorMessage()."</span> }</p>
            </div>
        </div>
    }
}
