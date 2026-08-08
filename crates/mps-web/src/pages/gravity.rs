use topcoat::router::page;
use topcoat::view::view;

/// Gravity models page
#[page("/gravity")]
pub async fn gravity() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ PHYSICS MODULE"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">{ <span data-lang="zh">"引力模型、天体参数与辛积分"</span><span data-lang="en">"Gravity Models, Celestial Data & Symplectic Integrators"</span> }</span><span data-lang="en">{ "Gravity Models, Celestial Data & Symplectic Integrators" }</span></h1>
                    <p class="page-desc"><span data-lang="zh">{ <span data-lang="zh">"内置 10 个太阳系天体精密参数（JPL DE441），提供 5 种引力模型，按轨道高度自动分支选择；mps-cosmos 的 CelestialSource 直接消费这些模型。"</span><span data-lang="en">"Built-in precision parameters for 10 solar system bodies (JPL DE441), 5 gravity models with auto-selection by orbital altitude. mps-cosmos CelestialSource consumes them directly."</span> }</span><span data-lang="en">{ "Built-in precision data for 10 solar system bodies (JPL DE441), 5 gravity models with auto-selection by orbital altitude; mps-cosmos CelestialSource consumes them directly." }</span></p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ <span data-lang="zh">"内置天体参数 (JPL DE441)"</span><span data-lang="en">"Built-in Celestial Parameters (JPL DE441)"</span> }</span><span data-lang="en">{ "Built-in Celestial Parameters (JPL DE441)" }</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead>
                            <tr><th>"ID"</th><th><span data-lang="zh">"天体"</span><span data-lang="en">"Body"</span></th><th>{ "GM (m³/s²)" }</th><th>{ <span data-lang="zh">"赤道半径 (km)"</span><span data-lang="en">"Eq. Radius (km)"</span> }</th><th>"J2"</th><th>{ <span data-lang="zh">"球谐阶"</span><span data-lang="en">"SH Degree"</span> }</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>"0"</td><td>"Sun"</td><td>"1.327×10²⁰"</td><td>"695,700"</td><td>"2.22×10⁻⁷"</td><td>"2"</td></tr>
                            <tr><td>"1"</td><td>"Mercury"</td><td>"2.203×10¹³"</td><td>"2,440"</td><td>"6.0×10⁻⁵"</td><td>"2"</td></tr>
                            <tr><td>"2"</td><td>"Venus"</td><td>"3.249×10¹⁴"</td><td>"6,052"</td><td>"4.46×10⁻⁶"</td><td>"2"</td></tr>
                            <tr><td>"3"</td><td>"Earth"</td><td>"3.986×10¹⁴"</td><td>"6,378"</td><td>"1.083×10⁻³"</td><td>"8×8 (EGM2008)"</td></tr>
                            <tr><td>"4"</td><td>"Moon"</td><td>"4.903×10¹²"</td><td>"1,737"</td><td>"2.033×10⁻⁴"</td><td>"LP165 + 12 Mascon (GRAIL)"</td></tr>
                            <tr><td>"5"</td><td>"Mars"</td><td>"4.283×10¹³"</td><td>"3,396"</td><td>"1.960×10⁻³"</td><td>"Mars50c"</td></tr>
                            <tr><td>"6"</td><td>"Jupiter"</td><td>"1.267×10¹⁷"</td><td>"71,492"</td><td>"1.474×10⁻²"</td><td>"4"</td></tr>
                            <tr><td>"7"</td><td>"Saturn"</td><td>"3.793×10¹⁶"</td><td>"60,268"</td><td>"1.629×10⁻²"</td><td>"4"</td></tr>
                            <tr><td>"8"</td><td>"Uranus"</td><td>"5.794×10¹⁵"</td><td>"25,559"</td><td>"3.343×10⁻³"</td><td>"2"</td></tr>
                            <tr><td>"9"</td><td>"Neptune"</td><td>"6.835×10¹⁵"</td><td>"24,764"</td><td>"3.408×10⁻³"</td><td>"2"</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"访问："</span><span data-lang="en">"Access: "</span> }<code>"mps_formula::celestial_data::{get_celestial_body, CelestialBodyId::Earth}"</code>{ <span data-lang="zh">" 返回 &'static CelestialBody，含 gm/equatorial_radius/j2/球谐系数指针等。"</span><span data-lang="en">" returns &'static CelestialBody with gm/equatorial_radius/j2/SH coefficient pointers."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"5 种引力模型 (mps-formula::gravitational_models)"</span><span data-lang="en">"5 Gravity Models (mps-formula::gravitational_models)"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"mps-formula 暴露 5 个引力加速函数，按精度/代价权衡："</span><span data-lang="en">"mps-formula exposes 5 gravity acceleration functions, traded by precision/cost:"</span> }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"函数"</span><span data-lang="en">"Function"</span></th><th>{ <span data-lang="zh">"模型"</span><span data-lang="en">"Model"</span> }</th><th><span data-lang="zh">"适用"</span><span data-lang="en">"Use Case"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"spherical_harmonics_acceleration"</code></td><td>{ <span data-lang="zh">"球谐展开 normalized Legendre"</span><span data-lang="en">"Spherical harmonics, normalized Legendre"</span> }</td><td>{ <span data-lang="zh">"地球 EGM2008 8×8，高精近场"</span><span data-lang="en">"Earth EGM2008 8×8, high-precision near-field"</span> }</td></tr>
                            <tr><td><code>"ellipsoid_gravity"</code></td><td>{ <span data-lang="zh">"椭球引力 (Carlson RF/RD 椭圆积分)"</span><span data-lang="en">"Ellipsoidal gravity (Carlson RF/RD elliptic integrals)"</span> }</td><td>{ <span data-lang="zh">"考虑天体扁率，<2R 近场"</span><span data-lang="en">"Accounts for oblateness, <2R near-field"</span> }</td></tr>
                            <tr><td><code>"zonal_harmonics_acceleration"</code></td><td>{ <span data-lang="zh">"J2-J6 带谐"</span><span data-lang="en">"J2-J6 zonal harmonics"</span> }</td><td>{ <span data-lang="zh">"中距离 long-term 摄动"</span><span data-lang="en">"Mid-range long-term perturbations"</span> }</td></tr>
                            <tr><td><code>"quadrupole_tensor_acceleration"</code></td><td>{ <span data-lang="zh">"完整引力梯度张量 (3×3)"</span><span data-lang="en">"Full gravity gradient tensor (3×3)"</span> }</td><td>{ <span data-lang="zh">"Jordan/Lockheed 重力梯度 GNC"</span><span data-lang="en">"Jordan/Lockheed gravity gradient GNC"</span> }</td></tr>
                            <tr><td>{ <span data-lang="zh">"多面体引力 (Werner-Scheeres)"</span><span data-lang="en">"Polyhedral gravity (Werner-Scheeres)"</span> }</td><td>{ <span data-lang="zh">"多面体顶点/面元"</span><span data-lang="en">"Polyhedron vertices/faces"</span> }</td><td>{ <span data-lang="zh">"不规则小天体（Eros/Itokawa），见 terrain_gravity"</span><span data-lang="en">"Irregular bodies (Eros/Itokawa), see terrain_gravity"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"CelestialSource 的自适应分支选择"</span><span data-lang="en">"CelestialSource Adaptive Branch Selection"</span> }</h2>
                <p class="p-lead">{ "mps-cosmos 的 CelestialSource 按轨道高度 r（以天体赤道半径 R 为单位）自动选模型——一条注册语句解决 "选哪个" 问题：" }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th>{ <span data-lang="zh">"高度段"</span><span data-lang="en">"Altitude Range"</span> }</th><th><span data-lang="zh">"选用的模型"</span><span data-lang="en">"Selected Model"</span></th><th><span data-lang="zh">"理由"</span><span data-lang="en">"Rationale"</span></th></tr></thead>
                        <tbody>
                            <tr><td>{ <span data-lang="zh">"<2R（贴近表面）"</span><span data-lang="en">"<2R (near surface)"</span> }</td><td>{ <span data-lang="zh">"椭球引力"</span><span data-lang="en">"Ellipsoidal gravity"</span> }</td><td>{ <span data-lang="zh">"扁率影响最大，球谐收敛慢"</span><span data-lang="en">"Oblateness dominates, SH convergence slow"</span> }</td></tr>
                            <tr><td>{ <span data-lang="zh">"2R – 10R"</span><span data-lang="en">"2R – 10R"</span> }</td><td><code>"spherical_harmonics_acceleration"</code></td><td>{ <span data-lang="zh">"8×8 球谐完整刻画非球面项"</span><span data-lang="en">"8×8 SH fully captures non-spherical terms"</span> }</td></tr>
                            <tr><td>{ <span data-lang="zh">"10R – 100R"</span><span data-lang="en">"10R – 100R"</span> }</td><td><code>"zonal_harmonics_acceleration"</code> { "(J2-J6)" }</td><td>{ <span data-lang="zh">"高阶谐项衰减完毕，带谐主导"</span><span data-lang="en">"Higher harmonics decayed, zonal dominant"</span> }</td></tr>
                            <tr><td>{ <span data-lang="zh">">100R"</span><span data-lang="en">">100R"</span> }</td><td>{ <span data-lang="zh">"点质量 + J2"</span><span data-lang="en">"Point mass + J2"</span> }</td><td>{ <span data-lang="zh">"点质量足够，J2 保留长期摄动"</span><span data-lang="en">"Point mass sufficient, J2 retains long-term perturbation"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"阈值由 "</span><span data-lang="en">"Threshold set by "</span> }<code>"CelestialSource::new(body, max_sh_degree)"</code>{ <span data-lang="zh">" 的 max_sh_degree 参数共同决定——0 表示只用点质量+J2，8 表示地球那样全阶运行。"</span><span data-lang="en">"'s max_sh_degree parameter — 0 = point mass+J2 only, 8 = full degree like Earth."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ <span data-lang="zh">"在太空场景的使用"</span><span data-lang="en">"Usage in Space Scenarios"</span> }</span><span data-lang="en">{ "Usage in Space Scenarios" }</span></h2>
                <p class="p-lead">{ <span data-lang="zh">"这些引力模型由 "</span><span data-lang="en">"These gravity models are "</span> }<a href="./cosmos" class="link">"mps-cosmos"</a>{ <span data-lang="zh">" 的 CelestialSource 直接消费：注册一组天体源，CosmosWorld::step 会在每个物理子步前对全体动态刚体累加 天体引力 + n-body 互引力 + 环境扰动，再交辛积子或 rapier 积分。配合 per-body PerturbationConfig 可逐体开大气阻力 / 光压。"</span><span data-lang="en">"'s CelestialSource directly consumed: register sources, CosmosWorld::step accumulates celestial + n-body + perturbations for all dynamic bodies per substep, then feeds to integrator or rapier. Per-body PerturbationConfig enables drag / solar pressure individually."</span> }</p>
                <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig};
use mps_cosmos::gravity::CelestialSource;
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};

let earth = get_celestial_body(CelestialBodyId::Earth);
let mut world = CosmosWorld::new(CosmosWorldConfig {
    central_body: Some(earth),
    ..Default::default()
});
// 球谐 8×8 自动在 <10R 段生效，10-100R 退化为 J2-J6，>100R 点质量+J2
world.add_celestial(CelestialSource::new(earth, 8));

// 月球作为第二引力源；max_sh=0 表示只走点质量+J2（月球低阶场用 Mascon 更准，
// 但那是 terrain_gravity 路径，CelestialSource 不消费 Mascon）
let moon = get_celestial_body(CelestialBodyId::Moon);
world.add_celestial(CelestialSource::new(moon, 0));"
                </code></pre>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ <span data-lang="zh">"纯函数直接调用"</span><span data-lang="en">"Direct Pure-Function Calls"</span> }</span><span data-lang="en">{ "Direct Pure-Function Calls" }</span></h2>
                <p class="p-lead">{ <span data-lang="zh">"不进 world 也能单独算——这些函数接收 Vec3 返回加速度 Vec3，无副作用："</span><span data-lang="en">"Can compute without world — take Vec3 position, return acceleration Vec3, no side effects:"</span> }</p>
                <pre><code class="language-rust">
"use mps_formula::gravitational_models::{spherical_harmonics_acceleration, zonal_harmonics_acceleration};
use mps_formula::celestial_data::get_celestial_body;
use mps_formula::ffi::Vec3;

let earth = get_celestial_body(mps_formula::celestial_data::CelestialBodyId::Earth);
let pos = Vec3 { x: 6.8e6, y: 0.0, z: 0.0 }; // 422km 轨道
// 8 阶球谐加速度（normalized Legendre + 完全归一化 C/S）
let a_sh = spherical_harmonics_acceleration(pos, earth, 8);
// 或直接要 J2 项
let a_j2 = zonal_harmonics_acceleration(pos, earth.gm, earth.equatorial_radius, earth.j2);"
                </code></pre>
            </div>
        </div>
    }
}
