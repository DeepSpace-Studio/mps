use topcoat::router::page;
use topcoat::view::view;

/// Cosmos world page — 太空刚体演算 (mps-cosmos)
#[page("/cosmos")]
pub async fn cosmos() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag"><span data-lang="zh">"/ COSMOS — 太空刚体演算"</span><span data-lang="en">"/ COSMOS — Cosmos Rigid Body"</span></div>
                    <h1 class="page-title"><span data-lang="zh">{ "太空刚体演算 (mps-cosmos)" }</span><span data-lang="en">{ "Cosmos Rigid Body (mps-cosmos)" }</span></h1>
                    <p class="page-desc">{ <span data-lang="zh">"基于 rapier3d-f64 的独立太空物理世界，自带天体引力源 / n-body 互引力 / 环境扰动 / 辛轨道积分器（Verlet / Yoshida4 / ForestRuth8 / +Kahan），长弧精度较 rapier 力注入路径提升一个量级。"</span><span data-lang="en">"Standalone cosmos physics world built on rapier3d-f64, with celestial gravity sources / n-body / perturbations / symplectic orbit integrators (Verlet / Yoshida4 / ForestRuth8 / +Kahan). Long-arc precision one order better than the rapier force-injection path."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">"为什么独立 crate："</span><span data-lang="en">"Why a standalone crate: "</span> }<code>"mps-core"</code>{ <span data-lang="zh">" 的 add_force 路径走 rapier 的 semi-implicit Euler，1s 步长一圈 LEO 即漂数百公里相位误差。"</span><span data-lang="en">"'s add_force path uses rapier's semi-implicit Euler; 1s step around LEO accumulates hundreds of km phase error."</span> }<code>"mps-cosmos"</code>{ <span data-lang="zh">" 把轨道推进从 rapier 力律里抽出来，对天体引力 + n-body 用辛积分器直接写回 translation / linvel，rapier 只负责碰撞 / 约束 / 姿态，无需共享 arena 或力律登记表。"</span><span data-lang="en">" extracts orbit propagation from rapier force laws — symplectic integrator writes back translation / linvel for celestial + n-body; rapier handles only collision / constraints / pose. No shared arena or force registry needed."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "CosmosWorld 核心组成" }</span><span data-lang="en">{ "CosmosWorld Core Components" }</span></h2>
                <ul class="ul-plain">
                    <li><strong><span data-lang="zh">"rapier3d-f64 后端"</span><span data-lang="en">"rapier3d-f64 backend"</span></strong> { <span data-lang="zh">" — RigidBodySet / ColliderSet / PhysicsPipeline / CCDSolver，自行持有，不经 C ABI"</span><span data-lang="en">" — RigidBodySet / ColliderSet / PhysicsPipeline / CCDSolver, self-owned, no C ABI"</span> }</li>
                    <li><strong><span data-lang="zh">"CelestialSource[]"</span><span data-lang="en">"CelestialSource[]"</span></strong> { <span data-lang="zh">" — 一组天体引力源（自适应模型分支：<2R 椭球 / <10R 球谐 / <100R J2-J6 / >100R 点质量）"</span><span data-lang="en">" — celestial gravity sources (auto model branch: <2R ellipsoidal / <10R spherical harmonic / <100R J2-J6 / >100R point mass)"</span> }</li>
                    <li><strong><span data-lang="zh">"NBodySource[]"</span><span data-lang="en">"NBodySource[]"</span></strong> { <span data-lang="zh">" — 一组参与 n-body 互引力的动态质点源（gm = G·mass，软化平方防奇点）"</span><span data-lang="en">" — dynamic mass sources for n-body mutual gravity (gm = G·mass, softening squared avoids singularities)"</span> }</li>
                    <li><strong><span data-lang="zh">"PerturbationConfig (per-body)"</span><span data-lang="en">"PerturbationConfig (per-body)"</span></strong> { <span data-lang="zh">" — 大气阻力 + 太阳光压配置（Cd/A/Cr/area/开关）"</span><span data-lang="en">" — atmospheric drag + solar pressure config (Cd/A/Cr/area/enabled)"</span> }</li>
                    <li><strong><span data-lang="zh">"central_body + sun_position"</span><span data-lang="en">"central_body + sun_position"</span></strong> { <span data-lang="zh">" — 环境扰动的参考天体与太阳方向"</span><span data-lang="en">" — reference body and sun direction for perturbations"</span> }</li>
                    <li><strong><span data-lang="zh">"OrbitIntegration"</span><span data-lang="en">"OrbitIntegration"</span></strong> { <span data-lang="zh">" — 6 种积子模式可切（见下表）"</span><span data-lang="en">" — 6 integrator modes (see table below)"</span> }</li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"轨道积分模式 (OrbitIntegration)"</span><span data-lang="en">"Orbit Integration Modes (OrbitIntegration)"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"模式"</span><span data-lang="en">"Mode"</span></th><th>{ <span data-lang="zh">"阶 / 能量误差"</span><span data-lang="en">"Order / Energy Error"</span> }</th><th><span data-lang="zh">"说明"</span><span data-lang="en">"Notes"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"RapierForce"</code></td><td>{ <span data-lang="zh">"1 阶 ~1e-5"</span><span data-lang="en">"Order 1 ~1e-5"</span> }</td><td>{ <span data-lang="zh">"合力用 add_force 喂给 rapier 的 semi-implicit Euler。仅作兼容/对照路径。dt>10s 自动拆 ≤10s 子步，每子步重注入力"</span><span data-lang="en">"Total force fed to rapier's semi-implicit Euler via add_force. Compatibility/reference path only. dt>10s auto-split into ≤10s substeps, re-inject force each substep"</span> }</td></tr>
                            <tr><td><code>"Verlet"</code></td><td>{ <span data-lang="zh">"2 阶 ~1e-10"</span><span data-lang="en">"Order 2 ~1e-10"</span> }</td><td>{ <span data-lang="zh">"velocity-Verlet 显式积分直接写 translation/linvel，rapier 只跑碰撞/姿态。长弧相位误差随 dt² 收敛"</span><span data-lang="en">"velocity-Verlet explicit integration directly writes translation/linvel; rapier handles only collision/pose. Long-arc phase error converges as dt²"</span> }</td></tr>
                            <tr><td><code>"Yoshida4"</code></td><td>{ <span data-lang="zh">"4 阶 ~1e-14（默认）"</span><span data-lang="en">"Order 4 ~1e-14 (default)"</span> }</td><td>{ <span data-lang="zh">"3 级复合 leapfrog。每步精度比 Verlet 升两个量级，每步多 2 次加速度评估"</span><span data-lang="en">"3-stage composite leapfrog. Two orders better per step than Verlet; 2 extra acceleration evals per step"</span> }</td></tr>
                            <tr><td><code>"ForestRuth8"</code></td><td>{ <span data-lang="zh">"8 阶 ~1e-16"</span><span data-lang="en">"Order 8 ~1e-16"</span> }</td><td>{ <span data-lang="zh">"15 级 McLachlan 系数复合，逼近 f64 极限。算力约 Verlet 的 15 倍"</span><span data-lang="en">"15-stage McLachlan-coefficient composite, near f64 limit. ~15× Verlet cost"</span> }</td></tr>
                            <tr><td><code>"Yoshida4Kahan"</code></td><td>{ <span data-lang="zh">"4 阶 + Kahan"</span><span data-lang="en">"Order 4 + Kahan"</span> }</td><td>{ <span data-lang="zh">"Yoshida4 叠加 Kahan 补偿累加位置/速度增量，长弧闭合精度再升 1-3 量级"</span><span data-lang="en">"Yoshida4 + Kahan compensated accumulation of position/velocity increments; long-arc closure precision improves 1-3 orders"</span> }</td></tr>
                            <tr><td><code>"ForestRuth8Kahan"</code></td><td>{ <span data-lang="zh">"8 阶 + Kahan"</span><span data-lang="en">"Order 8 + Kahan"</span> }</td><td>{ <span data-lang="zh">"ForestRuth8 + Kahan 补偿，用于数千-数万步超长弧高精导航"</span><span data-lang="en">"ForestRuth8 + Kahan compensation, for thousands-tens of thousands step ultra-long-arc high-precision navigation"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <div class="callout">
                    <p>{ <span data-lang="zh">"⚠️ 载入式陷阱：显式积子路径"</span><span data-lang="en">"⚠️ Footgun: explicit integrator path "</span> }<strong><span data-lang="zh">"不调用 pipeline.step"</span><span data-lang="en">" does NOT call pipeline.step"</span></strong>{ <span data-lang="zh">"。原因是 rapier 的 advance_to_final_positions 会在 step 末尾用 solver 内部 next_position 覆盖 body.position，把显式积子写回的 translation 抹掉。本路径手写最小推进（积子写回 + sync_colliders_after_verlet 重挂 collider + 姿态/阻尼单独积分）。若未来要在该路径下处理对接约束，应插一次 velocity-only 求解，切勿直接加 pipeline.step。"</span><span data-lang="en">". Because rapier's advance_to_final_positions overwrites body.position with solver's internal next_position at step end, clobbering symplectic integrator's writeback. This path uses hand-rolled minimal advance (integrator writeback + sync_colliders_after_verlet re-attach collider + integrate pose/damping separately). If you add docking constraints to this path in the future, insert a velocity-only solve; do NOT just add pipeline.step."</span> }</p>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"step 诊断 (StepResult)"</span><span data-lang="en">"step Diagnostics (StepResult)"</span> }</h2>
                <p class="p-lead">{ "每次 step(dt) 返回 StepResult，调用方可据此判断 \"为什么没推进\" 而非靠静默 return 猜：" }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"变体"</span><span data-lang="en">"Variant"</span></th><th><span data-lang="zh">"含义"</span><span data-lang="en">"Meaning"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"Stepped(f64)"</code></td><td>{ <span data-lang="zh">"正常推进了 dt 秒"</span><span data-lang="en">"Advanced dt seconds normally"</span> }</td></tr>
                            <tr><td><code>"Substepped { substeps, sub_dt }"</code></td><td>{ <span data-lang="zh">"dt>10s 被拆成 ≤10s 子步完成（RapierForce 路径）"</span><span data-lang="en">"dt>10s split into ≤10s substeps (RapierForce path)"</span> }</td></tr>
                            <tr><td><code>"Skipped(NonFinite)"</code></td><td>{ <span data-lang="zh">"dt 为 NaN / Inf"</span><span data-lang="en">"dt is NaN / Inf"</span> }</td></tr>
                            <tr><td><code>"Skipped(NonPositive)"</code></td><td>"dt ≤ 0"</td></tr>
                            <tr><td><code>"Skipped(TooLarge)"</code></td><td>{ "dt 超过 30s 硬上限，防止误把 \"一帧\" 当 \"一小时\" 喂进来" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"批量推进用 step_n(dt, n)，把 dt 合法性校验前置一次性完成，返回 Result<(), StepSkipReason>。"</span><span data-lang="en">"Batch advance with step_n(dt, n); dt validation batched once, returns Result<(), StepSkipReason>."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"环境扰动力"</span><span data-lang="en">"Perturbation Forces"</span> }</h2>
                <ul class="ul-plain">
                    <li><strong><span data-lang="zh">"大气阻力"</span><span data-lang="en">"Atmospheric drag"</span></strong> { <span data-lang="zh">" — atmospheric_drag_force：Cd·A·½ρ·v_rel²，按天体自转角速度算大气运动速度"</span><span data-lang="en">" — atmospheric_drag_force: Cd·A·½ρ·v_rel², atmosphere motion computed from celestial body spin rate"</span> }</li>
                    <li><strong><span data-lang="zh">"大气密度采样"</span><span data-lang="en">"Atmosphere density sampling"</span></strong> { <span data-lang="zh">" — atmosphere_density_at：天体表面密度 + 标高指数模型，无大气返回 0"</span><span data-lang="en">" — atmosphere_density_at: surface density + scale-height exponential model, returns 0 for bodies with no atmosphere"</span> }</li>
                    <li><strong><span data-lang="zh">"太阳光压"</span><span data-lang="en">"Solar radiation pressure"</span></strong> { <span data-lang="zh">" — solar_pressure_force：Cr·P·A·ŝ，按 1/AU² 平方反比衰减"</span><span data-lang="en">" — solar_pressure_force: Cr·P·A·ŝ, inverse-square decay at 1/AU²"</span> }</li>
                    <li><strong><span data-lang="zh">"per-body 配置"</span><span data-lang="en">"Per-body config"</span></strong> { <span data-lang="zh">" — set_perturbation 可逐体开启/关闭阻力与光压及截面积、系数"</span><span data-lang="en">" — set_perturbation enables/disables drag and solar pressure per body, with area and coefficients"</span> }</li>
                    <li><strong><span data-lang="zh">"相对论修正"</span><span data-lang="en">"Relativistic correction"</span></strong> { <span data-lang="zh">" — RelativisticCorrection::None/OnePN/TwoPN/Full，仅叠在中心天体引力上（n-body 不修正）"</span><span data-lang="en">" — RelativisticCorrection::None/OnePN/TwoPN/Full, applied only to central body gravity (n-body not corrected)"</span> }</li>
                    <li><strong><span data-lang="zh">"n-body 软化"</span><span data-lang="en">"n-body softening"</span></strong> { <span data-lang="zh">" — n_body_softening_sq 默认 1e3 m²（约 31.6m 软化长度），近距离数值限幅；另 dist_sq<1 硬截断"</span><span data-lang="en">" — n_body_softening_sq default 1e3 m² (~31.6 m softening length), near-distance numerical limiter; dist_sq<1 hard truncation"</span> }</li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"Rust 用法"</span><span data-lang="en">"Rust Usage"</span> }</h2>
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

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"JNI 导出 (Java 端演练)"</span><span data-lang="en">"JNI Exports (Java-side drills)"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"通过 mps-jni 包一层 C ABI 暴露给 Java。句柄约定：long world = *mut CosmosWorld，long body = RigidBodyHandle（高 32 位 index + 低 32 位 generation）。step 返回 int 编码的 StepResult（>0=推进毫秒数，-1=Substepped，-2=NonFinite，-3=NonPositive，-4=TooLarge）。"</span><span data-lang="en">"Wrapped through mps-jni to expose C ABI to Java. Handle convention: long world = *mut CosmosWorld; long body = RigidBodyHandle (high 32 bits = index, low 32 bits = generation). step returns int-encoded StepResult (>0 = advanced ms, -1 = Substepped, -2 = NonFinite, -3 = NonPositive, -4 = TooLarge)."</span> }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"JNI 函数"</span><span data-lang="en">"JNI Function"</span></th><th><span data-lang="zh">"说明"</span><span data-lang="en">"Notes"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"cosmosWorldCreate"</code></td><td>{ <span data-lang="zh">"建 world（dt、求解器、orbit_integration、verlet_substeps、softening）"</span><span data-lang="en">"Create world (dt, solver, orbit_integration, verlet_substeps, softening)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldDestroy"</code></td><td><span data-lang="zh">"销毁"</span><span data-lang="en">"Destroy"</span></td></tr>
                            <tr><td><code>"cosmosWorldSetCentralBody"</code></td><td>{ <span data-lang="zh">"设环境扰动参考中心天体（0..9）"</span><span data-lang="en">"Set central body for environment perturbation reference (0..9)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldSetSunPosition"</code></td><td>{ <span data-lang="zh">"设太阳位置（光压方向）"</span><span data-lang="en">"Set sun position (solar pressure direction)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldAddCelestial"</code></td><td>{ <span data-lang="zh">"注册天体引力源（含球谐最高阶）"</span><span data-lang="en">"Register celestial gravity source (with max spherical-harmonic degree)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldAddNBody"</code></td><td>{ <span data-lang="zh">"把已插入刚体登记为 n-body 源"</span><span data-lang="en">"Register an inserted rigid body as n-body source"</span> }</td></tr>
                            <tr><td><code>"cosmosSatelliteBuilder"</code></td><td>{ <span data-lang="zh">"动态刚体 builder（质量+初位姿+半径估惯量）"</span><span data-lang="en">"Dynamic rigid body builder (mass + initial pose + radius-estimated inertia)"</span> }</td></tr>
                            <tr><td><code>"cosmosFixedBodyBuilder"</code></td><td>{ <span data-lang="zh">"固定刚体 builder（做 n-body 中心本体）"</span><span data-lang="en">"Fixed rigid body builder (used as n-body central body)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldInsertBody(AsGravitySource)"</code></td><td>{ <span data-lang="zh">"插入 builder，后一步到位挂 n-body"</span><span data-lang="en">"Insert builder; AsGravitySource variant attaches as n-body in one step"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldSetPerturbation"</code></td><td>{ <span data-lang="zh">"逐体设大气阻力+光压（Cd/A/Cr/area/开关）"</span><span data-lang="en">"Per-body set drag + solar pressure (Cd/A/Cr/area/enabled)"</span> }</td></tr>
                            <tr><td><code>"cosmosWorldStep / StepN"</code></td><td>{ <span data-lang="zh">"推进，返回 int 编码 StepResult"</span><span data-lang="en">"Advance, returns int-encoded StepResult"</span> }</td></tr>
                            <tr><td><code>"cosmosBodyTranslationOut / LinvelOut"</code></td><td>{ <span data-lang="zh">"读刚体位置/速度到 native 缓冲"</span><span data-lang="en">"Read body position/velocity into native buffer"</span> }</td></tr>
                            <tr><td><code>"cosmosBodyMass"</code></td><td><span data-lang="zh">"读刚体质量"</span><span data-lang="en">"Read body mass"</span></td></tr>
                            <tr><td><code>"cosmosWorldDynamicBodyCount"</code></td><td>{ <span data-lang="zh">"动态刚体数"</span><span data-lang="en">"Dynamic body count"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">"测试覆盖"</span><span data-lang="en">"Test Coverage"</span></h2>
                <p class="p-lead">{ <span data-lang="zh">"位于 mps-test/src/cosmos/：bodies / gravity / integrator / orbit / perturbation / world 共 19 项单元测试，含 Verlet 一圈闭合 < 0.1% r + 无能量漂、Verlet 面积守恒容差、softening 默认值等精度回归。"</span><span data-lang="en">"Lives in mps-test/src/cosmos/: bodies / gravity / integrator / orbit / perturbation / world — 19 unit tests, including Verlet one-orbit closure < 0.1% r + zero energy drift, Verlet area conservation tolerance, default softening regression."</span> }</p>
                <pre><code class="language-bash">
"cargo test -p mps-test --lib cosmos"
                </code></pre>
            </div>
        </div>
    }
}
