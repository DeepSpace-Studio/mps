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
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"辛积分器"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"MPS 提供 3 种辛积分器，支持 Kahan 补偿求和和后牛顿 1PN+2PN 相对论修正。"</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"支持积分器"</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;">"Leapfrog"</strong> " — 2 阶，速度 Verlet，适合基本应用"</li>
                    <li><strong style="color:#ddd;">"Yoshida 4 阶"</strong> " — 4 阶精度，适合中等精度需求"</li>
                    <li><strong style="color:#ddd;">"Forest-Ruth 8 阶"</strong> " — 8 阶精度，适合高精度轨道积分"</li>
                </ul>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"Kahan 补偿求和"</h2>
                <p style="color:#aaa;line-height:1.7;">"通过 Kahan 补偿求和算法，将精度从 15 位有效数字提升至 30 位，显著减少长周期积分中的累积误差。"</p>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"后牛顿修正"</h2>
                <p style="color:#aaa;line-height:1.7;">"支持 1PN 和 2PN 后牛顿相对论修正，适用于水星轨道进动等强引力场场景。"</p>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"太空场景：mps-cosmos 的 velocity-Verlet"</h2>
                <p style="color:#aaa;line-height:1.7;">
                    "上面 Leapfrog 即 velocity-Verlet 在纯函数层面的实现。" <a href="./cosmos" style="color:#4a9eff;">"mps-cosmos"</a> " 把它接进了独立物理 world：" <code>"CosmosWorld::step"</code> " 在 " <code>"orbit_integration = Verlet"</code> " 下，对天体引力 + n-body 互引力用二阶辛 velocity-Verlet 直接写回 " <code>"translation"</code> " / " <code>"linvel"</code> "，rapier 只跑碰撞 / 姿态。阻力 / 光压并入 Verlet 的加速度函数。1s 步长一圈 LEO 闭合误差 < 0.1% r，远优于 rapier " <code>"add_force"</code> " 路径的 semi-implicit Euler（漂数百公里）。"
                </p>
            </div>
        </div>
    }
}
