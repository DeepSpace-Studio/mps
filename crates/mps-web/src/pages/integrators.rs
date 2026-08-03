use topcoat::router::page;
use topcoat::view::view;

/// Integrators page
#[page("/integrators")]
pub async fn integrators() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ INTEGRATORS"
                    </div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">{ "辛积分器 + Kahan + 后牛顿" }</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "MPS 在 mps-formula 里提供 4 阶/8 阶辛积分器、Kahan 补偿累加、1PN/2PN 后牛顿修正，作为纯函数；mps-cosmos 把这些积子接进独立物理 world。rapier 走 semi-implicit Euler，仅适用地面/短弧。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "积子选型表（mps-formula::integrators）" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"函数"</th><th>{ "阶 / 每步误差" }</th><th>{ "子步评估数" }</th><th>"适用"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"leapfrog_step"</code></td><td>{ "2 阶辛 ~10⁻¹⁰" }</td><td>"1"</td><td>{ "短弧快速估算 / mps-cosmos Verlet" }</td></tr>
                            <tr><td><code>"leapfrog_step_kahan"</code></td><td>{ "2 阶 + Kahan ~10⁻¹³" }</td><td>"1"</td><td>{ "高精度长弧 Verlet" }</td></tr>
                            <tr><td><code>"yoshida4_step"</code></td><td>{ "4 阶辛 ~10⁻¹⁴（默认）" }</td><td>"3"</td><td>{ "主流轨道积分 / mps-cosmos Yoshida4" }</td></tr>
                            <tr><td><code>"yoshida4_step_kahan"</code></td><td>{ "4 阶 + Kahan ~10⁻¹⁷" }</td><td>"3"</td><td>{ "长弧无能量漂" }</td></tr>
                            <tr><td><code>"forest_ruth8_step"</code></td><td>{ "8 阶辛 ~10⁻¹⁶" }</td><td>"15"</td><td>{ "f64 极限精度 / mps-cosmos ForestRuth8" }</td></tr>
                            <tr><td><code>"forest_ruth8_step_kahan"</code></td><td>{ "8 阶 + Kahan ~10⁻¹⁸" }</td><td>"15"</td><td>{ "超长弧高精导航" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "所有积子函数签名一致：" }<code>"(position: Vec3, velocity: Vec3, accel_fn: F, dt: f64) -> (Vec3, Vec3)"</code>{ "，accel_fn 返回当前位置的加速度。纯函数、无世界状态、无 panic——可在任何上下文（含 WebAssembly）独立使用。每个 " }<code>"*_kahan"</code>{ " 版本叠加 " }<code>"KahanSum / KahanVec3"</code>{ "（mps-formula::math）做位置/速度增量的补偿累加。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "Kahan 补偿求和" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "f64 加法在累加数千步后较低位会丢失——长弧轨道积分里这表现为能量缓慢漂移（非物理）。Kahan 补偿用补偿位 \"捡回\" 每次加法的低位误差，把有效精度从 15 位提升到接近 30 位。MPS 在 " }<code>"mps-formula::math"</code>{ " 提供 " }<code>"KahanSum"</code>{ "（标量）与 " }<code>"KahanVec3"</code>{ "（3 维）。" }</p>
                <pre><code class="language-rust">
"use mps_formula::math::{KahanSum, KahanVec3};

let mut sum = KahanSum::new();
for i in 0..100_000 {
    sum.add(1.0e-10); // 极小量累加
}
// naive 求和会漂 1e-5 量级，Kahan 保留至机器精度
assert!((sum.value() - 100_000.0 * 1.0e-10).abs() < 1.0e-15);"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "积子内置 Kahan 版本把位置/速度增量走 KahanVec3 累加。长弧闭合误差比裸版降 1-3 个量级（见 mps-test/cosmos/orbit.rs 的闭合容差回归）。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "后牛顿 (PN) 相对论修正" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "MPS 在 " }<code>"integrators::post_newtonian_*"</code>{ " 暴露标量函数返回中心引力项的 PN 修正加速度，叠加在牛顿 " }<code>"a = -GM·r̂/r²"</code>{ " 之上：" }</p>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"1PN"</strong> { " — post_newtonian_1pn，近日点进动主导项；适合水星。每步 ~10⁻⁸" }</li>
                    <li><strong style="color:#ddd;">"2PN"</strong> { " — post_newtonian_2pn，二阶修正；适合太阳系内高精历表" }</li>
                    <li><strong style="color:#ddd;">"Full"</strong> { " — post_newtonian_full，1PN + 2PN 全修" }</li>
                </ul>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "mps-cosmos 的 " }<code>"RelativisticCorrection"</code>{ " 枚举叠在 " }<code>"total_acceleration"</code>{ " 的中心天体引力项上（n-body 与扰动项不做修正——多体相对论算法复杂、物理意义弱）。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "纯函数用法（无 World）" }</h2>
                <pre><code class="language-rust">
"use mps_formula::integrators::yoshida4_step;
use mps_formula::ffi::Vec3;

let gm = 3.986e14; // Earth GM
let mut pos = Vec3 { x: 7e6, y: 0.0, z: 0.0 };
let mut vel = Vec3 { x: 0.0, y: 7800.0, z: 0.0 };
let dt = 1.0;

for _ in 0..5400 {
    let accel = |p: Vec3| {
        let r2 = p.x * p.x + p.y * p.y + p.z * p.z;
        let r = r2.sqrt();
        Vec3 { x: -gm * p.x / (r2 * r), y: -gm * p.y / (r2 * r), z: -gm * p.z / (r2 * r) }
    };
    (pos, vel) = yoshida4_step(pos, vel, accel, dt);
}
println!(\"闭合漂 = {:?}\", pos); // < 1e-6 * r"
                </code></pre>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "太空场景：mps-cosmos 的辛积子路径" }</h2>
                <p style="color:#aaa;line-height:1.7;">
                    { "把这些积子接进独立物理 world：" }<a href="./cosmos" style="color:#4a9eff;">"mps-cosmos"</a>{ " 的 " }<code>"CosmosWorld::step"</code>{ " 在 " }<code>"OrbitIntegration::Yoshida4"</code>{ " 下，对天体引力 + n-body 互引力用 4 阶辛积子直接写回 translation/linvel，rapier 只跑碰撞/姿态。阻力/光压并入积子的加速度函数。" }
                </p>
                <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig, world::OrbitIntegration};

let mut world = CosmosWorld::new(CosmosWorldConfig {
    dt: 1.0,
    orbit_integration: OrbitIntegration::Yoshida4Kahan, // 4 阶 + Kahan
    ..Default::default()
});"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "1s 步长一圈 LEO：semi-implicit Euler 漂数百 km，Yoshida4 < 0.1% r，Yoshida4Kahan < 0.01% r。各阶误差随 dtⁿ 收敛——缩小 dt 收益随阶数升高而递增（8 阶缩 dt 几乎是白送）。详情见 " }<a href="./cosmos" style="color:#4a9eff;">"太空演算"</a>{ " 页。" }</p>
            </div>
        </div>
    }
}
