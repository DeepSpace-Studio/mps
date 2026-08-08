use topcoat::router::page;
use topcoat::view::view;

/// Integrators page
#[page("/integrators")]
pub async fn integrators() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ INTEGRATORS"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">{ "辛积分器 + Kahan + 后牛顿" }</span><span data-lang="en">{ "Symplectic Integrators + Kahan + Post-Newtonian" }</span></h1>
                    <p class="page-desc">{ <span data-lang="zh">"MPS 在 mps-formula 里提供 4 阶/8 阶辛积分器、Kahan 补偿累加、1PN/2PN 后牛顿修正，作为纯函数；mps-cosmos 把这些积子接进独立物理 world。rapier 走 semi-implicit Euler，仅适用地面/短弧。"</span><span data-lang="en">"MPS provides 4th/8th-order symplectic integrators, Kahan compensated accumulation, and 1PN/2PN post-Newtonian corrections as pure functions in mps-formula; mps-cosmos connects these to its standalone world. rapier uses semi-implicit Euler, suitable only for ground/short arcs."</span> }</p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "积子选型表（mps-formula::integrators）" }</span><span data-lang="en">{ "Integrator Selection Table (mps-formula::integrators)" }</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"函数"</span><span data-lang="en">"Function"</span></th><th>{ <span data-lang="zh">"阶 / 每步误差"</span><span data-lang="en">"Order / Error per Step"</span> }</th><th>{ <span data-lang="zh">"子步评估数"</span><span data-lang="en">"Substep Evals"</span> }</th><th><span data-lang="zh">"适用"</span><span data-lang="en">"Use Case"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"leapfrog_step"</code></td><td>{ <span data-lang="zh">"2 阶辛 ~10⁻¹⁰"</span><span data-lang="en">"Order 2 symplectic ~1e-10"</span> }</td><td>"1"</td><td>{ <span data-lang="zh">"短弧快速估算 / mps-cosmos Verlet"</span><span data-lang="en">"Short-arc quick estimate / mps-cosmos Verlet"</span> }</td></tr>
                            <tr><td><code>"leapfrog_step_kahan"</code></td><td>{ <span data-lang="zh">"2 阶 + Kahan ~10⁻¹³"</span><span data-lang="en">"Order 2 + Kahan ~1e-13"</span> }</td><td>"1"</td><td>{ <span data-lang="zh">"高精度长弧 Verlet"</span><span data-lang="en">"High-precision long-arc Verlet"</span> }</td></tr>
                            <tr><td><code>"yoshida4_step"</code></td><td>{ <span data-lang="zh">"4 阶辛 ~10⁻¹⁴（默认）"</span><span data-lang="en">"Order 4 symplectic ~1e-14 (default)"</span> }</td><td>"3"</td><td>{ <span data-lang="zh">"主流轨道积分 / mps-cosmos Yoshida4"</span><span data-lang="en">"Mainstream orbit integration / mps-cosmos Yoshida4"</span> }</td></tr>
                            <tr><td><code>"yoshida4_step_kahan"</code></td><td>{ <span data-lang="zh">"4 阶 + Kahan ~10⁻¹⁷"</span><span data-lang="en">"Order 4 + Kahan ~1e-17"</span> }</td><td>"3"</td><td>{ <span data-lang="zh">"长弧无能量漂"</span><span data-lang="en">"Long-arc zero energy drift"</span> }</td></tr>
                            <tr><td><code>"forest_ruth8_step"</code></td><td>{ <span data-lang="zh">"8 阶辛 ~10⁻¹⁶"</span><span data-lang="en">"Order 8 symplectic ~1e-16"</span> }</td><td>"15"</td><td>{ <span data-lang="zh">"f64 极限精度 / mps-cosmos ForestRuth8"</span><span data-lang="en">"f64 near-limit precision / mps-cosmos ForestRuth8"</span> }</td></tr>
                            <tr><td><code>"forest_ruth8_step_kahan"</code></td><td>{ <span data-lang="zh">"8 阶 + Kahan ~10⁻¹⁸"</span><span data-lang="en">"Order 8 + Kahan ~1e-18"</span> }</td><td>"15"</td><td>{ <span data-lang="zh">"超长弧高精导航"</span><span data-lang="en">"Ultra-long-arc high-precision navigation"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"所有积子函数签名一致："</span><span data-lang="en">"All integrators share the signature: "</span> }<code>"(position: Vec3, velocity: Vec3, accel_fn: F, dt: f64) -> (Vec3, Vec3)"</code>{ <span data-lang="zh">"，accel_fn 返回当前位置的加速度。纯函数、无世界状态、无 panic——可在任何上下文（含 WebAssembly）独立使用。每个 "</span><span data-lang="en">", accel_fn returns acceleration at current position. Pure functions, no world state, no panics — usable in any context (incl. WebAssembly). Each "</span> }<code>"*_kahan"</code>{ <span data-lang="zh">" 版本叠加 "</span><span data-lang="en">" variant adds "</span> }<code>"KahanSum / KahanVec3"</code>{ <span data-lang="zh">"（mps-formula::math）做位置/速度增量的补偿累加。"</span><span data-lang="en">" (mps-formula::math) for compensated accumulation of position/velocity increments."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"Kahan 补偿求和"</span><span data-lang="en">"Kahan Compensated Summation"</span> }</h2>
                <p class="p-lead">{ "f64 加法在累加数千步后较低位会丢失——长弧轨道积分里这表现为能量缓慢漂移（非物理）。Kahan 补偿用补偿位 \"捡回\" 每次加法的低位误差，把有效精度从 15 位提升到接近 30 位。MPS 在 " }<code>"mps-formula::math"</code>{ <span data-lang="zh">" 提供 "</span><span data-lang="en">" provides "</span> }<code>"KahanSum"</code>{ <span data-lang="zh">"（标量）与 "</span><span data-lang="en">" (scalar) and "</span> }<code>"KahanVec3"</code>{ <span data-lang="zh">"（3 维）。"</span><span data-lang="en">" (3D)."</span> }</p>
                <pre><code class="language-rust">
"use mps_formula::math::{KahanSum, KahanVec3};

let mut sum = KahanSum::new();
for i in 0..100_000 {
    sum.add(1.0e-10); // 极小量累加
}
// naive 求和会漂 1e-5 量级，Kahan 保留至机器精度
assert!((sum.value() - 100_000.0 * 1.0e-10).abs() < 1.0e-15);"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"积子内置 Kahan 版本把位置/速度增量走 KahanVec3 累加。长弧闭合误差比裸版降 1-3 个量级（见 mps-test/cosmos/orbit.rs 的闭合容差回归）。"</span><span data-lang="en">"Built-in *_kahan variants route position/velocity increments through KahanVec3. Long-arc closure error drops 1-3 orders vs. plain variants (see mps-test/cosmos/orbit.rs closure tolerance regression)."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"后牛顿 (PN) 相对论修正"</span><span data-lang="en">"Post-Newtonian (PN) Relativistic Corrections"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"MPS 在 "</span><span data-lang="en">"MPS exposes scalar functions in "</span> }<code>"integrators::post_newtonian_*"</code>{ <span data-lang="zh">" 暴露标量函数返回中心引力项的 PN 修正加速度，叠加在牛顿 "</span><span data-lang="en">" returning PN correction acceleration for the central gravity term, added on top of Newtonian "</span> }<code>"a = -GM·r̂/r²"</code>{ <span data-lang="zh">" 之上："</span><span data-lang="en">":"</span> }</p>
                <ul class="ul-plain">
                    <li><strong><span data-lang="zh">"1PN"</span><span data-lang="en">"1PN"</span></strong> { <span data-lang="zh">" — post_newtonian_1pn，近日点进动主导项；适合水星。每步 ~10⁻⁸"</span><span data-lang="en">" — post_newtonian_1pn, perihelion precession dominant term; for Mercury. ~1e-8 per step"</span> }</li>
                    <li><strong><span data-lang="zh">"2PN"</span><span data-lang="en">"2PN"</span></strong> { <span data-lang="zh">" — post_newtonian_2pn，二阶修正；适合太阳系内高精历表"</span><span data-lang="en">" — post_newtonian_2pn, second-order correction; for high-precision solar-system ephemeris"</span> }</li>
                    <li><strong><span data-lang="zh">"Full"</span><span data-lang="en">"Full"</span></strong> { <span data-lang="zh">" — post_newtonian_full，1PN + 2PN 全修"</span><span data-lang="en">" — post_newtonian_full, full 1PN + 2PN corrections"</span> }</li>
                </ul>
                <p class="p-note">{ <span data-lang="zh">"mps-cosmos 的 "</span><span data-lang="en">"mps-cosmos "</span> }<code>"RelativisticCorrection"</code>{ <span data-lang="zh">" 枚举叠在 "</span><span data-lang="en">" enum overlays correction on "</span> }<code>"total_acceleration"</code>{ <span data-lang="zh">" 的中心天体引力项上（n-body 与扰动项不做修正——多体相对论算法复杂、物理意义弱）。"</span><span data-lang="en">"'s central-body gravity term in total_acceleration (n-body and perturbations not corrected — multi-body relativistic algorithms complex, weak physical meaning)."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"纯函数用法（无 World）"</span><span data-lang="en">"Pure-Function Usage (No World)"</span> }</h2>
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

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"太空场景：mps-cosmos 的辛积子路径"</span><span data-lang="en">"Space scenario: mps-cosmos symplectic integrator path"</span> }</h2>
                <p class="p-lead">
                    { <span data-lang="zh">"把这些积子接进独立物理 world："</span><span data-lang="en">"Connect these integrators to a standalone world: "</span> }<a href="./cosmos" class="link">"mps-cosmos"</a>{ <span data-lang="zh">" 的 "</span><span data-lang="en">"'s "</span> }<code>"CosmosWorld::step"</code>{ <span data-lang="zh">" 在 "</span><span data-lang="en">" under "</span> }<code>"OrbitIntegration::Yoshida4"</code>{ <span data-lang="zh">" 下，对天体引力 + n-body 互引力用 4 阶辛积子直接写回 translation/linvel，rapier 只跑碰撞/姿态。阻力/光压并入积子的加速度函数。"</span><span data-lang="en">" directly writes back translation/linvel for celestial + n-body via 4th-order symplectic integrator; rapier runs only collision/pose. Drag/solar-pressure folded into the integrator's acceleration function."</span> }
                </p>
                <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig, world::OrbitIntegration};

let mut world = CosmosWorld::new(CosmosWorldConfig {
    dt: 1.0,
    orbit_integration: OrbitIntegration::Yoshida4Kahan, // 4 阶 + Kahan
    ..Default::default()
});"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"1s 步长一圈 LEO：semi-implicit Euler 漂数百 km，Yoshida4 < 0.1% r，Yoshida4Kahan < 0.01% r。各阶误差随 dtⁿ 收敛——缩小 dt 收益随阶数升高而递增（8 阶缩 dt 几乎是白送）。详情见 "</span><span data-lang="en">"1s step around LEO one orbit: semi-implicit Euler drifts hundreds of km, Yoshida4 < 0.1% r, Yoshida4Kahan < 0.01% r. Per-order error converges as dtⁿ — shrinking dt pays more at higher orders (8th-order shrinking dt is almost free). Detail on "</span> }<a href="./cosmos" class="link"><span data-lang="zh">"太空演算"</span><span data-lang="en">"Cosmos page"</span></a>{ <span data-lang="zh">" 页。"</span><span data-lang="en">" page."</span> }</p>
            </div>
        </div>
    }
}
