use topcoat::router::page;
use topcoat::view::view;

/// Gravity models page
#[page("/gravity")]
pub async fn gravity() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ PHYSICS MODULE"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">{ "引力模型、天体参数与辛积分" }</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "内置 10 个太阳系天体精密参数（JPL DE441），提供 5 种引力模型，按轨道高度自动分支选择；mps-cosmos 的 CelestialSource 直接消费这些模型。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "内置天体参数 (JPL DE441)" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead>
                            <tr><th>"ID"</th><th>"天体"</th><th>{ "GM (m³/s²)" }</th><th>{ "赤道半径 (km)" }</th><th>"J2"</th><th>{ "球谐阶" }</th></tr>
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
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "访问：" }<code>"mps_formula::celestial_data::{get_celestial_body, CelestialBodyId::Earth}"</code>{ " 返回 &'static CelestialBody，含 gm/equatorial_radius/j2/球谐系数指针等。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "5 种引力模型 (mps-formula::gravitational_models)" }</h2>
                <p style="color:#aaa; line-height:1.7;">{ "mps-formula 暴露 5 个引力加速函数，按精度/代价权衡：" }</p>
                <div style="overflow-x:auto;margin-top:14px;">
                    <table>
                        <thead><tr><th>"函数"</th><th>{ "模型" }</th><th>"适用"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"spherical_harmonics_acceleration"</code></td><td>{ "球谐展开 normalized Legendre" }</td><td>{ "地球 EGM2008 8×8，高精近场" }</td></tr>
                            <tr><td><code>"ellipsoid_gravity"</code></td><td>{ "椭球引力 (Carlson RF/RD 椭圆积分)" }</td><td>{ "考虑天体扁率，<2R 近场" }</td></tr>
                            <tr><td><code>"zonal_harmonics_acceleration"</code></td><td>{ "J2-J6 带谐" }</td><td>{ "中距离 long-term 摄动" }</td></tr>
                            <tr><td><code>"quadrupole_tensor_acceleration"</code></td><td>{ "完整引力梯度张量 (3×3)" }</td><td>{ "Jordan/Lockheed 重力梯度 GNC" }</td></tr>
                            <tr><td>{ "多面体引力 (Werner-Scheeres)" }</td><td>{ "多面体顶点/面元" }</td><td>{ "不规则小天体（Eros/Itokawa），见 terrain_gravity" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "CelestialSource 的自适应分支选择" }</h2>
                <p style="color:#aaa; line-height:1.7;">{ "mps-cosmos 的 CelestialSource 按轨道高度 r（以天体赤道半径 R 为单位）自动选模型——一条注册语句解决 "选哪个" 问题：" }</p>
                <div style="overflow-x:auto;margin-top:14px;">
                    <table>
                        <thead><tr><th>{ "高度段" }</th><th>"选用的模型"</th><th>"理由"</th></tr></thead>
                        <tbody>
                            <tr><td>{ "<2R（贴近表面）" }</td><td>{ "椭球引力" }</td><td>{ "扁率影响最大，球谐收敛慢" }</td></tr>
                            <tr><td>{ "2R – 10R" }</td><td><code>"spherical_harmonics_acceleration"</code></td><td>{ "8×8 球谐完整刻画非球面项" }</td></tr>
                            <tr><td>{ "10R – 100R" }</td><td><code>"zonal_harmonics_acceleration"</code> { "(J2-J6)" }</td><td>{ "高阶谐项衰减完毕，带谐主导" }</td></tr>
                            <tr><td>{ ">100R" }</td><td>{ "点质量 + J2" }</td><td>{ "点质量足够，J2 保留长期摄动" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "阈值由 " }<code>"CelestialSource::new(body, max_sh_degree)"</code>{ " 的 max_sh_degree 参数共同决定——0 表示只用点质量+J2，8 表示地球那样全阶运行。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "在太空场景的使用" }</h2>
                <p style="color:#aaa; line-height:1.7;">{ "这些引力模型由 " }<a href="./cosmos" style="color:#4a9eff;">"mps-cosmos"</a>{ " 的 CelestialSource 直接消费：注册一组天体源，CosmosWorld::step 会在每个物理子步前对全体动态刚体累加 天体引力 + n-body 互引力 + 环境扰动，再交辛积子或 rapier 积分。配合 per-body PerturbationConfig 可逐体开大气阻力 / 光压。" }</p>
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

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "纯函数直接调用" }</h2>
                <p style="color:#aaa; line-height:1.7;">{ "不进 world 也能单独算——这些函数接收 Vec3 返回加速度 Vec3，无副作用：" }</p>
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
