use topcoat::router::page;
use topcoat::view::view;

/// Formula modules page
#[page("/formula")]
pub async fn formula() -> topcoat::Result {
    let modules = [
        (
            "88",
            "航天工程 (spaceflight)",
            "轨道力学、姿态控制、热控、推进、环境",
        ),
        ("23", "核物理 (nuclear)", "衰变、结合能、裂变/聚变、中子学"),
        (
            "26",
            "材料力学 (material_mechanics)",
            "弹性、塑性、断裂、疲劳、梁理论",
        ),
        (
            "19",
            "天体物理 (astrophysics)",
            "N体、Barnes-Hut、FMM、Lane-Emden、Eddington",
        ),
        (
            "23",
            "相对论 (relativity)",
            "Lorentz、Schwarzschild、Kerr、ISCO、引力红移",
        ),
        ("20", "量子力学 (quantum)", "波函数、隧穿、谐振子、氢原子"),
        (
            "16",
            "电磁学 (electromagnetism)",
            "Lorentz、Faraday、Maxwell、Biot-Savart",
        ),
        (
            "18",
            "流体力学 (fluid)",
            "浮力/阻力、SPH、Navier-Stokes、Bernoulli、湍流",
        ),
        ("7", "声学 (acoustics)", "模态分析、波动方程、共振、空间化"),
        (
            "8",
            "分子动力学 (molecular)",
            "Lennard-Jones、Coulomb、对势相互作用",
        ),
        ("5", "空气动力学 (aerodynamics)", "表面力、体素气动、力估算"),
        ("4", "生物力学 (biomechanics)", "Hill 肌肉模型、关节约束"),
        (
            "6",
            "混沌理论 (chaos)",
            "Lorenz 吸引子、双摆、Lyapunov 指数",
        ),
        ("5", "连续介质力学 (continuum)", "FEM 形函数、应变/应力张量"),
        ("7", "控制理论 (control_theory)", "PID、状态空间、MPC、LQR"),
        (
            "6",
            "引力模型 (gravitational_models)",
            "球谐展开、椭球、多面体",
        ),
        (
            "7",
            "积分器 (integrators)",
            "Leapfrog、Yoshida 4、Forest-Ruth 8、后牛顿",
        ),
        ("4", "物理化学 (physchem)", "Gray-Scott 反应扩散、催化"),
        ("7", "等离子体物理 (plasma)", "Debye 屏蔽、Vlasov、PIC、MHD"),
        ("5", "软体 (softbody)", "XPBD 约束、超弹性本构模型"),
        (
            "4",
            "超流 (superfluidity)",
            "Gross-Pitaevskii、涡旋晶格、量子化环流",
        ),
        ("3", "拓扑学 (topology)", "持续同调、Betti 数"),
        ("6", "弹道学 (trajectory)", "6DOF 弹道/滑翔轨迹、RK4 积分"),
        ("3", "传动 (transmission)", "齿轮比、扭矩分配"),
        (
            "5",
            "波动光学 (wave_optics)",
            "Kirchhoff 衍射、Fresnel 传播、干涉",
        ),
    ];

    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ FORMULA MODULES"
                    </div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"公式模块"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"mps-formula 提供 28 个模块 300+ 纯公式函数，覆盖物理、航天、工程等领域。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div class="callout" style="background:#0f1a2e;border-left:4px solid #4a9eff;padding:14px 18px;border-radius:4px;margin:20px 0;">
                <p>"所有公式为纯 Rust 函数，不依赖 Rapier 或 WorldHandle。可在无物理引擎的环境中独立使用。"</p>
            </div>

            for (count, name, desc) in modules {
                <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:16px 20px;margin-bottom:10px;display:flex;align-items:center;gap:16px;">
                    <span style="background:#4a9eff;color:#1a1a2e;padding:4px 10px;border-radius:4px;font-weight:700;font-size:14px;white-space:nowrap;">(count)</span>
                    <div style="flex:1;">
                        <strong style="color:#fff;font-size:15px;">(name)</strong>
                        <br>
                        <small style="color:#888;font-size:13px;">(desc)</small>
                    </div>
                </div>
            }
        </div>
    }
}
