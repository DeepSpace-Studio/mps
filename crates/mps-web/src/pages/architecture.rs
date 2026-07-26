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
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">"MPS 物理引擎的模块化架构设计 — Rust 核心 + 多语言绑定。"</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"分层架构"</h2>
                <pre style="background:#0d0d2b; border:1px solid #333; border-radius:6px; padding:16px; font-size:13px; line-height:1.5;">
                    <code class="language-text">
"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI (~480 函数)
       ├─ mps-formula  — 28 纯公式模块 (300+ 函数)
       ├─ mps-core     — 物理引擎 + Rapier 封装
       ├─ mps-jni      — JNI 绑定 (~280 方法)
       ├─ mps-ffm      — FFM 元数据
       └─ mps-test     — 332 集成测试"
                    </code>
                </pre>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"核心组件"</h2>
                <p style="color:#aaa; line-height:1.7;">"PhysicsWorld 包含以下核心组件："</p>
                <ul style="color:#999; line-height:2; padding-left:20px;">
                    <li><strong style="color:#ddd;">"PhysicsPipeline"</strong> " — Rapier 物理管线 (碰撞检测 + 约束求解)"</li>
                    <li><strong style="color:#ddd;">"RigidBodySet"</strong> " — 刚体集合"</li>
                    <li><strong style="color:#ddd;">"ColliderSet"</strong> " — 碰撞体集合"</li>
                    <li><strong style="color:#ddd;">"ImpulseJointSet"</strong> " — 脉冲关节集合"</li>
                    <li><strong style="color:#ddd;">"MultibodyJointSet"</strong> " — 多体关节集合"</li>
                    <li><strong style="color:#ddd;">"IslandManager"</strong> " — 睡眠/唤醒管理"</li>
                    <li><strong style="color:#ddd;">"BroadPhaseBvh"</strong> " — 宽相位 BVH 加速结构"</li>
                    <li><strong style="color:#ddd;">"NarrowPhase"</strong> " — 窄相位碰撞检测"</li>
                    <li><strong style="color:#ddd;">"CCDSolver"</strong> " — 连续碰撞检测求解器"</li>
                    <li><strong style="color:#ddd;">"IntegrationParameters"</strong> " — 积分参数"</li>
                    <li><strong style="color:#ddd;">"ForceRegistry"</strong> " — 类型化力注册表"</li>
                    <li><strong style="color:#ddd;">"FrameWorkBuffers"</strong> " — 预分配工作缓冲区"</li>
                </ul>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"积分参数"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead>
                            <tr><th>"参数"</th><th>"默认值"</th><th>"说明"</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>"dt"</td><td>"1/60"</td><td>"时间步长 (秒)"</td></tr>
                            <tr><td>"num_solver_iterations"</td><td>"4"</td><td>"每个子步的约束求解器迭代次数"</td></tr>
                            <tr><td>"max_ccd_substeps"</td><td>"1"</td><td>"快速移动体的 CCD 子步数"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
