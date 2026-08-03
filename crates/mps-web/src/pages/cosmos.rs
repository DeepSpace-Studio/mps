use topcoat::router::page;
use topcoat::view::view;

/// Cosmos world page — 太空刚体演算 (mps-cosmos)
#[page("/cosmos")]
pub async fn cosmos() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ COSMOS — 太空刚体演算"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">{ "太空刚体演算 (mps-cosmos)" }</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "基于 rapier3d-f64 的独立太空物理世界，自带天体引力源 / n-body 互引力 / 环境扰动 / 辛轨道积分器（Verlet / Yoshida4 / ForestRuth8 / +Kahan），长弧精度较 rapier 力注入路径提升一个量级。" }</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#0f1a2e;border-left:4px solid #4a9eff;padding:14px 18px;border-radius:4px;margin:20px 0;">
                <p>{ "为什么独立 crate：" }<code>"mps-core"</code>{ " 的 add_force 路径走 rapier 的 semi-implicit Euler，1s 步长一圈 LEO 即漂数百公里相位误差。" }<code>"mps-cosmos"</code>{ " 把轨道推进从 rapier 力律里抽出来，对天体引力 + n-body 用辛积分器直接写回 translation / linvel，rapier 只负责碰撞 / 约束 / 姿态，无需共享 arena 或力律登记表。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "CosmosWorld 核心组成" }</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"rapier3d-f64 后端"</strong> { " — RigidBodySet / ColliderSet / PhysicsPipeline / CCDSolver，自行持有，不经 C ABI" }</li>
                    <li><strong style="color:#ddd;">"CelestialSource[]"</strong> { " — 一组天体引力源（自适应模型分支：<2R 椭球 / <10R 球谐 / <100R J2-J6 / >100R 点质量）" }</li>
                    <li><strong style="color:#ddd;">"NBodySource[]"</strong> { " — 一组参与 n-body 互引力的动态质点源（gm = G·mass，软化平方防奇点）" }</li>
                    <li><strong style="color:#ddd;">"PerturbationConfig (per-body)"</strong> { " — 大气阻力 + 太阳光压配置（Cd/A/Cr/area/开关）" }</li>
                    <li><strong style="color:#ddd;">"central_body + sun_position"</strong> { " — 环境扰动的参考天体与太阳方向" }</li>
                    <li><strong style="color:#ddd;">"OrbitIntegration"</strong> { " — 6 种积子模式可切（见下表）" }</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "轨道积分模式 (OrbitIntegration)" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"模式"</th><th>{ "阶 / 能量误差" }</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"RapierForce"</code></td><td>{ "1 阶 ~1e-5" }</td><td>{ "合力用 add_force 喂给 rapier 的 semi-implicit Euler。仅作兼容/对照路径。dt>10s 自动拆 ≤10s 子步，每子步重注入力" }</td></tr>
                            <tr><td><code>"Verlet"</code></td><td>{ "2 阶 ~1e-10" }</td><td>{ "velocity-Verlet 显式积分直接写 translation/linvel，rapier 只跑碰撞/姿态。长弧相位误差随 dt² 收敛" }</td></tr>
                            <tr><td><code>"Yoshida4"</code></td><td>{ "4 阶 ~1e-14（默认）" }</td><td>{ "3 级复合 leapfrog。每步精度比 Verlet 升两个量级，每步多 2 次加速度评估" }</td></tr>
                            <tr><td><code>"ForestRuth8"</code></td><td>{ "8 阶 ~1e-16" }</td><td>{ "15 级 McLachlan 系数复合，逼近 f64 极限。算力约 Verlet 的 15 倍" }</td></tr>
                            <tr><td><code>"Yoshida4Kahan"</code></td><td>{ "4 阶 + Kahan" }</td><td>{ "Yoshida4 叠加 Kahan 补偿累加位置/速度增量，长弧闭合精度再升 1-3 量级" }</td></tr>
                            <tr><td><code>"ForestRuth8Kahan"</code></td><td>{ "8 阶 + Kahan" }</td><td>{ "ForestRuth8 + Kahan 补偿，用于数千-数万步超长弧高精导航" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <div class="callout" style="background:#0f1a2e;border-left:4px solid #4a9eff;padding:14px 18px;border-radius:4px;margin-top:14px;">
                    <p>{ "⚠️ 载入式陷阱：显式积子路径" }<strong>"不调用 pipeline.step"</strong>{ "。原因是 rapier 的 advance_to_final_positions 会在 step 末尾用 solver 内部 next_position 覆盖 body.position，把显式积子写回的 translation 抹掉。本路径手写最小推进（积子写回 + sync_colliders_after_verlet 重挂 collider + 姿态/阻尼单独积分）。若未来要在该路径下处理对接约束，应插一次 velocity-only 求解，切勿直接加 pipeline.step。" }</p>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "step 诊断 (StepResult)" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "每次 step(dt) 返回 StepResult，调用方可据此判断 \"为什么没推进\" 而非靠静默 return 猜：" }</p>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"变体"</th><th>"含义"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"Stepped(f64)"</code></td><td>{ "正常推进了 dt 秒" }</td></tr>
                            <tr><td><code>"Substepped { substeps, sub_dt }"</code></td><td>{ "dt>10s 被拆成 ≤10s 子步完成（RapierForce 路径）" }</td></tr>
                            <tr><td><code>"Skipped(NonFinite)"</code></td><td>{ "dt 为 NaN / Inf" }</td></tr>
                            <tr><td><code>"Skipped(NonPositive)"</code></td><td>"dt ≤ 0"</td></tr>
                            <tr><td><code>"Skipped(TooLarge)"</code></td><td>{ "dt 超过 30s 硬上限，防止误把 \"一帧\" 当 \"一小时\" 喂进来" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "批量推进用 step_n(dt, n)，把 dt 合法性校验前置一次性完成，返回 Result<(), StepSkipReason>。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "环境扰动力" }</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"大气阻力"</strong> { " — atmospheric_drag_force：Cd·A·½ρ·v_rel²，按天体自转角速度算大气运动速度" }</li>
                    <li><strong style="color:#ddd;">"大气密度采样"</strong> { " — atmosphere_density_at：天体表面密度 + 标高指数模型，无大气返回 0" }</li>
                    <li><strong style="color:#ddd;">"太阳光压"</strong> { " — solar_pressure_force：Cr·P·A·ŝ，按 1/AU² 平方反比衰减" }</li>
                    <li><strong style="color:#ddd;">"per-body 配置"</strong> { " — set_perturbation 可逐体开启/关闭阻力与光压及截面积、系数" }</li>
                    <li><strong style="color:#ddd;">"相对论修正"</strong> { " — RelativisticCorrection::None/OnePN/TwoPN/Full，仅叠在中心天体引力上（n-body 不修正）" }</li>
                    <li><strong style="color:#ddd;">"n-body 软化"</strong> { " — n_body_softening_sq 默认 1e3 m²（约 31.6m 软化长度），近距离数值限幅；另 dist_sq<1 硬截断" }</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "Rust 用法" }</h2>
                <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig, world::OrbitIntegration};
use mps_cosmos::gravity::CelestialSource;
use mps_cosmos::bodies::satellite_builder;
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
use rapier3d::prelude::Vector;

let earth = get_celestial_body(CelestialBodyId::Earth);
let mut world = CosmosWorld::new(CosmosWorldConfig {
    dt: 1.0,
    orbit_integration: OrbitIntegration::Yoshida4,   // 默认 4 阶辛
    verlet_substeps: 4,
    central_body: Some(earth),                         // 环境扰动参考
    n_body_softening_sq: 1e3,
    relativistic_correction: mps_cosmos::world::RelativisticCorrection::None,
    ..Default::default()
});
// 球谐 8×8 自动在 <10R 段生效，10-100R 退化为 J2-J6，>100R 点质量+J2
world.add_celestial(CelestialSource::new(earth, 8));

// 月球做第二引力源 + n-body 中心
let moon = get_celestial_body(CelestialBodyId::Moon);
world.add_celestial(CelestialSource::new(moon, 0));
world.set_sun_position(Vector::new(1.5e11, 0.0, 0.0)); // 1 AU

// 卫星：质量 1000kg，7e6 m 半径轨道，7800 m/s 切向速度。半径 1m 用估惯量
let sat = world.insert_body_as_gravity_source(
    satellite_builder(1000.0,
        Vector::new(7e6, 0.0, 0.0),
        Vector::new(0.0, 7800.0, 0.0), 1.0),
    1000.0,  // 同时登记为 n-body 源
);
// 逐体开阻力 + 光压
world.set_perturbation(sat, mps_cosmos::world::PerturbationConfig {
    drag_coefficient: 2.2, area: 5.0, enable_drag: true,
    reflectivity: 1.3, optical_area: 10.0, enable_solar: true,
});

// 跑一圈 LEO，Yoshida4 闭合误差 < 0.1% r
for _ in 0..5400 { let _ = world.step(1.0); }
match world.body_state(sat) {
    Some(s) => println!(\"pos = {:?}, vel = {:?}\", s.position, s.velocity),
    None => println!(\"body removed\"),
}"
                </code></pre>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "JNI 导出 (Java 端演练)" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "通过 mps-jni 包一层 C ABI 暴露给 Java。句柄约定：long world = *mut CosmosWorld，long body = RigidBodyHandle（高 32 位 index + 低 32 位 generation）。step 返回 int 编码的 StepResult（>0=推进毫秒数，-1=Substepped，-2=NonFinite，-3=NonPositive，-4=TooLarge）。" }</p>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"JNI 函数"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"cosmosWorldCreate"</code></td><td>{ "建 world（dt、求解器、orbit_integration、verlet_substeps、softening）" }</td></tr>
                            <tr><td><code>"cosmosWorldDestroy"</code></td><td>"销毁"</td></tr>
                            <tr><td><code>"cosmosWorldSetCentralBody"</code></td><td>{ "设环境扰动参考中心天体（0..9）" }</td></tr>
                            <tr><td><code>"cosmosWorldSetSunPosition"</code></td><td>{ "设太阳位置（光压方向）" }</td></tr>
                            <tr><td><code>"cosmosWorldAddCelestial"</code></td><td>{ "注册天体引力源（含球谐最高阶）" }</td></tr>
                            <tr><td><code>"cosmosWorldAddNBody"</code></td><td>{ "把已插入刚体登记为 n-body 源" }</td></tr>
                            <tr><td><code>"cosmosSatelliteBuilder"</code></td><td>{ "动态刚体 builder（质量+初位姿+半径估惯量）" }</td></tr>
                            <tr><td><code>"cosmosFixedBodyBuilder"</code></td><td>{ "固定刚体 builder（做 n-body 中心本体）" }</td></tr>
                            <tr><td><code>"cosmosWorldInsertBody(AsGravitySource)"</code></td><td>{ "插入 builder，后一步到位挂 n-body" }</td></tr>
                            <tr><td><code>"cosmosWorldSetPerturbation"</code></td><td>{ "逐体设大气阻力+光压（Cd/A/Cr/area/开关）" }</td></tr>
                            <tr><td><code>"cosmosWorldStep / StepN"</code></td><td>{ "推进，返回 int 编码 StepResult" }</td></tr>
                            <tr><td><code>"cosmosBodyTranslationOut / LinvelOut"</code></td><td>{ "读刚体位置/速度到 native 缓冲" }</td></tr>
                            <tr><td><code>"cosmosBodyMass"</code></td><td>"读刚体质量"</td></tr>
                            <tr><td><code>"cosmosWorldDynamicBodyCount"</code></td><td>{ "动态刚体数" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"测试覆盖"</h2>
                <p style="color:#aaa;line-height:1.7;">{ "位于 mps-test/src/cosmos/：bodies / gravity / integrator / orbit / perturbation / world 共 19 项单元测试，含 Verlet 一圈闭合 < 0.1% r + 无能量漂、Verlet 面积守恒容差、softening 默认值等精度回归。" }</p>
                <pre><code class="language-bash">
"cargo test -p mps-test --lib cosmos"
                </code></pre>
            </div>
        </div>
    }
}
