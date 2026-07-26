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
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">"引力模型、天体参数与辛积分器"</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">"内置 10 个太阳系天体精密参数，支持 5 种引力模型自动选择，以及 3 种辛积分器 + 后牛顿修正。"</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"内置天体参数"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead>
                            <tr><th>"ID"</th><th>"天体"</th><th>"GM (m³/s²)"</th><th>"赤道半径 (km)"</th><th>"J2"</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>"0"</td><td>"Sun"</td><td>"1.327×10²⁰"</td><td>"695,700"</td><td>"2.22×10⁻⁷"</td></tr>
                            <tr><td>"1"</td><td>"Mercury"</td><td>"2.203×10¹³"</td><td>"2,440"</td><td>"6.0×10⁻⁵"</td></tr>
                            <tr><td>"2"</td><td>"Venus"</td><td>"3.249×10¹⁴"</td><td>"6,052"</td><td>"4.46×10⁻⁶"</td></tr>
                            <tr><td>"3"</td><td>"Earth"</td><td>"3.986×10¹⁴"</td><td>"6,378"</td><td>"1.083×10⁻³"</td></tr>
                            <tr><td>"4"</td><td>"Moon"</td><td>"4.903×10¹²"</td><td>"1,737"</td><td>"2.033×10⁻⁴"</td></tr>
                            <tr><td>"5"</td><td>"Mars"</td><td>"4.283×10¹³"</td><td>"3,396"</td><td>"1.960×10⁻³"</td></tr>
                            <tr><td>"6"</td><td>"Jupiter"</td><td>"1.267×10¹⁷"</td><td>"71,492"</td><td>"1.474×10⁻²"</td></tr>
                            <tr><td>"7"</td><td>"Saturn"</td><td>"3.793×10¹⁶"</td><td>"60,268"</td><td>"1.629×10⁻²"</td></tr>
                            <tr><td>"8"</td><td>"Uranus"</td><td>"5.794×10¹⁵"</td><td>"25,559"</td><td>"3.343×10⁻³"</td></tr>
                            <tr><td>"9"</td><td>"Neptune"</td><td>"6.835×10¹⁵"</td><td>"24,764"</td><td>"3.408×10⁻³"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"引力模型"</h2>
                <p style="color:#aaa; line-height:1.7;">"MPS 支持 5 种引力模型，自动根据轨道高度和精度需求选择最优模型："</p>
                <ul style="color:#999; line-height:2; padding-left:20px;">
                    <li><strong style="color:#ddd;">"球谐展开"</strong> " — EGM2008 8×8 阶，地球最高精度"</li>
                    <li><strong style="color:#ddd;">"椭球引力"</strong> " — 考虑天体扁率的简化模型"</li>
                    <li><strong style="color:#ddd;">"J2-J6 带谐"</strong> " — 带谐项摄动修正"</li>
                    <li><strong style="color:#ddd;">"四极张量"</strong> " — 完整引力梯度张量"</li>
                    <li><strong style="color:#ddd;">"多面体引力"</strong> " — Werner-Scheeres 方法，适合不规则天体"</li>
                </ul>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"API"</h2>
                <pre><code class="language-rust">
"// 注册天体引力到 ForceRegistry
world_register_celestial_gravity(world, CelestialBody::Earth,
    GravityModel::SphericalHarmonics { degree: 8, order: 8 });

// 设置引力参数
world_set_gravity(world, &Vec3 { x: 0.0, y: -9.81, z: 0.0 });

// 设置积分参数
world_set_integration_parameters(world, 1.0/60.0, 4, 1);"
                </code></pre>
            </div>
        </div>
    }
}
