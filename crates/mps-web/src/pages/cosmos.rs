use topcoat::router::page;
use topcoat::view::view;

/// Cosmos world page — 太空刚体演算 (mps-cosmos)
///
/// 独立于 `mps-core` 的 C ABI / 共享 arena / 力律登记表，是一个面向轨道
/// 演算的独立物理 world：自行持有 `RigidBodySet` / `PhysicsPipeline`，只
/// 复用 `mps-formula` 的纯计算函数。本页讲清它解决了 `mps-core` 在长弧
/// 轨道积分上的相位误差问题，以及 Verlet / n-body / 环境扰动 / 诊断的
/// 设计取舍。
#[page("/cosmos")]
pub async fn cosmos() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ COSMOS — 太空刚体演算"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"太空刚体演算 (mps-cosmos)"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"基于 " <code>"rapier3d-f64"</code> " 的独立太空物理世界，自带天体引力源 / n-body 互引力 / 环境扰动 / velocity-Verlet 轨道积分，长弧精度较 rapier 力注入路径提升一个量级。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#0f1a2e;border-left:4px solid #4a9eff;padding:14px 18px;border-radius:4px;margin:20px 0;">
                <p><strong>"为什么独立 crate："</strong> <code>"mps-core"</code> " 的 " <code>"add_force"</code> " 路径走 rapier 的 semi-implicit Euler，1s 步长一圈 LEO 即漂数百公里相位误差。" <code>"mps-cosmos"</code> " 把轨道推进从 rapier 力律里抽出来，对天体引力 + n-body 用二阶辛 velocity-Verlet 直接写回 " <code>"translation"</code> " / " <code>"linvel"</code> "，rapier 只负责碰撞 / 约束 / 姿态，无需共享 arena 或力律登记表。"</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"CosmosWorld 核心组成"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"rapier3d-f64 后端"</strong> " — " <code>"RigidBodySet"</code> " / " <code>"ColliderSet"</code> " / " <code>"PhysicsPipeline"</code> " / " <code>"CCDSolver"</code> "，自行持有，不经 C ABI"</li>
                    <li><strong style="color:#ddd;">"CelestialSource[]"</strong> " — 一组天体引力源（自适应模型分支：<2R 椭球 / <10R 球谐 / <100R J2-J6 / >100R 点质量）"</li>
                    <li><strong style="color:#ddd;">"NBodySource[]"</strong> " — 一组参与 n-body 互引力的动态质点源（软化平方防奇点）"</li>
                    <li><strong style="color:#ddd;">"PerturbationConfig (per-body)"</strong> " — 大气阻力 + 太阳光压配置"</li>
                    <li><strong style="color:#ddd;">"central_body + sun_position"</strong> " — 环境扰动的参考天体与太阳方向"</li>
                    <li><strong style="color:#ddd;">"OrbitIntegration"</strong> " — " <code>"RapierForce"</code> " / " <code>"Verlet"</code> " 两条积公路径可切"</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"轨道积分模式"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"模式"</th><th>"精度"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"RapierForce"</code></td><td>"semi-implicit Euler"</td><td>"默认。合力用 " <code>"add_force"</code> " 喂给 rapier。dt>10s 自动拆 ≤10s 子步，每子步重注入力"</td></tr>
                            <tr><td><code>"Verlet"</code></td><td>"2 阶辛 velocity-Verlet"</td><td>"天体引力+n-body 显式积分直接写 translation/linvel，rapier 只跑碰撞/姿态。" <code>"verlet_substeps"</code> " 控内部子步，长弧相位误差 O(dt²)"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">"阻力 / 光压等耗散力一并进 Verlet 的加速度函数（在一步内变化缓慢，用当前位置 / 速度评估足够），保持单条积分路径，避免半步力再喂回 rapier。"</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"环境扰动力"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"大气阻力"</strong> " — " <code>"atmospheric_drag_force"</code> "：Cd·A·½ρ·v_rel²，按天体自转角速度算大气运动速度"</li>
                    <li><strong style="color:#ddd;">"大气密度采样"</strong> " — " <code>"atmosphere_density_at"</code> "：天体表面密度 + 标高指数模型，无大气返回 0"</li>
                    <li><strong style="color:#ddd;">"太阳光压"</strong> " — " <code>"solar_pressure_force"</code> "：Cr·P·A·ŝ，按 1/AU² 平方反比衰减"</li>
                    <li><strong style="color:#ddd;">"per-body 配置"</strong> " — " <code>"set_perturbation"</code> " 可逐体开启 / 关闭阻力与光压及截面积、系数"</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"step 诊断 (StepResult)"</h2>
                <p style="color:#aaa;line-height:1.7;">"每次 " <code>"step(dt)"</code> " 返回 " <code>"StepResult"</code> "，调用方可据此判断 "为什么没推进" 而非靠静默 return 猜："</p>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"变体"</th><th>"含义"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"Stepped(f64)"</code></td><td>"正常推进了 dt 秒"</td></tr>
                            <tr><td><code>"Substepped { substeps, sub_dt }"</code></td><td>"dt>10s 被拆成 ≤10s 子步完成（RapierForce 路径）"</td></tr>
                            <tr><td><code>"Skipped(NonFinite)"</code></td><td>"dt 为 NaN / Inf"</td></tr>
                            <tr><td><code>"Skipped(NonPositive)"</code></td><td>"dt ≤ 0"</td></tr>
                            <tr><td><code>"Skipped(TooLarge)"</code></td><td>"dt 超过 30s 硬上限，防止误把 "一帧" 当 "一小时" 喂进来"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">"批量推进用 " <code>"step_n(dt, n)"</code> "，把 dt 合法性校验前置一次性完成。"</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"Rust 用法"</h2>
                <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig, world::OrbitIntegration};
use mps_cosmos::gravity::CelestialSource;
use mps_cosmos::bodies::satellite_builder;
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};

let earth = get_celestial_body(CelestialBodyId::Earth);
let mut world = CosmosWorld::new(CosmosWorldConfig {
    dt: 1.0,
    orbit_integration: OrbitIntegration::Verlet,
    verlet_substeps: 4,
    central_body: Some(earth),
    n_body_softening_sq: 1e3,
    ..Default::default()
});
world.add_celestial(CelestialSource::new(earth, 8));

let sat = world.insert_body_as_gravity_source(
    satellite_builder(1000.0,
        Vector::new(7e6, 0.0, 0.0),
        Vector::new(0.0, 7800.0, 0.0), 1.0),
    1000.0,
);

// 跑一圈 LEO，Verlet 闭合误差 < 0.1% r
for _ in 0..5400 { let _ = world.step(1.0); }"
                </code></pre>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"JNI 导出 (Java 端演练)"</h2>
                <p style="color:#aaa;line-height:1.7;">"通过 " <code>"mps-jni"</code> " 包一层 C ABI 暴露给 Java。句柄约定：" <code>"long world"</code> " = " <code>"*mut CosmosWorld"</code> "，" <code>"long body"</code> " = " <code>"RigidBodyHandle"</code> "（高 32 位 index + 低 32 位 generation）。"</p>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"JNI 函数"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"cosmosWorldCreate"</code></td><td>"建 world（dt、求解器、orbit_integration、verlet_substeps、softening）"</td></tr>
                            <tr><td><code>"cosmosWorldDestroy"</code></td><td>"销毁"</td></tr>
                            <tr><td><code>"cosmosWorldSetCentralBody"</code></td><td>"设环境扰动参考中心天体（0..9）"</td></tr>
                            <tr><td><code>"cosmosWorldSetSunPosition"</code></td><td>"设太阳位置（光压方向）"</td></tr>
                            <tr><td><code>"cosmosWorldAddCelestial"</code></td><td>"注册天体引力源（含球谐最高阶）"</td></tr>
                            <tr><td><code>"cosmosWorldAddNBody"</code></td><td>"把已插入刚体登记为 n-body 源"</td></tr>
                            <tr><td><code>"cosmosSatelliteBuilder"</code></td><td>"动态刚体 builder（质量+初位姿+半径估惯量）"</td></tr>
                            <tr><td><code>"cosmosFixedBodyBuilder"</code></td><td>"固定刚体 builder（做 n-body 中心本体）"</td></tr>
                            <tr><td><code>"cosmosWorldInsertBody(AsGravitySource)"</code></td><td>"插入 builder，后一步到位挂 n-body"</td></tr>
                            <tr><td><code>"cosmosWorldSetPerturbation"</code></td><td>"逐体设大气阻力 + 光压（Cd/A/Cr/area/开关）"</td></tr>
                            <tr><td><code>"cosmosWorldStep / StepN"</code></td><td>"推进，返回 int 编码的 StepResult（>0=n_ms, -1=Substepped, -2/-3/-4=Skipped）"</td></tr>
                            <tr><td><code>"cosmosBodyTranslationOut / LinvelOut"</code></td><td>"读刚体位置 / 速度到 native 缓冲"</td></tr>
                            <tr><td><code>"cosmosBodyMass"</code></td><td>"读刚体质量"</td></tr>
                            <tr><td><code>"cosmosWorldDynamicBodyCount"</code></td><td>"动态刚体数"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"测试覆盖"</h2>
                <p style="color:#aaa;line-height:1.7;">{ "位于 " }<code>"mps-test/src/cosmos/"</code>{ "：bodies / gravity / integrator / orbit / perturbation / world 共 19 项单元测试，含 Verlet 一圈闭合 < 0.1% r + 无能量漂、Verlet 面积守恒容差、softening 默认值 等精度回归。" }</p>
                <pre><code class="language-bash">
"cargo test -p mps-test --lib cosmos"
                </code></pre>
            </div>
        </div>
    }
}
