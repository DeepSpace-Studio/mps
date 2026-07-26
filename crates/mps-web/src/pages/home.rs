use topcoat::{
    router::page,
    view::view,
};

/// Home page — MPS Physics System overview
#[page("/")]
pub async fn home() -> topcoat::Result {
    view! {
        <div style="text-align:center; padding:60px 20px 40px;">
            <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; margin-bottom:12px; font-family:monospace;">
                "/ MPS PHYSICS OBSERVATORY"
            </div>
            <h1 style="font-size:36px; font-weight:300; color:#fff; margin:0 0 16px;">
                "运动物理系统 (米每秒)"
            </h1>
            <p style="font-size:16px; color:#aaa; max-width:720px; margin:0 auto 30px; line-height:1.7;">
                "基于 " <strong style="color:#e0e0e0;">"Rapier3D-f64"</strong> " 的高精度 Rust 物理引擎。通过 C FFI (~480 函数) 和 Java JNI (~280 方法) 暴露完整 API。支持 "
                <strong style="color:#e0e0e0;">"332 项测试"</strong> "、" <strong style="color:#e0e0e0;">"5 种引力模型"</strong> "、" <strong style="color:#e0e0e0;">"3 种辛积分器"</strong> "、" <strong style="color:#e0e0e0;">"共享内存零拷贝 Arena"</strong> "、" <strong style="color:#e0e0e0;">"28 个公式模块"</strong> " 和 " <strong style="color:#e0e0e0;">"10 个太阳系天体"</strong> "。"
            </p>
            <div style="display:flex; gap:12px; justify-content:center; flex-wrap:wrap;">
                <a href="./quickstart" style="background:#4a9eff; color:#fff; padding:12px 24px; border-radius:6px; text-decoration:none; font-weight:500;">"快速入门"</a>
                <a href="./api" style="border:1px solid #4a9eff; color:#4a9eff; padding:12px 24px; border-radius:6px; text-decoration:none; font-weight:500;">"API 参考"</a>
            </div>
        </div>

        <div style="display:flex; gap:16px; justify-content:center; flex-wrap:wrap; margin:40px 0;">
            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px 28px; text-align:center; min-width:120px;">
                <strong style="display:block;font-size:28px;color:#4a9eff;font-weight:300;">"332"</strong>
                <span style="font-size:12px;color:#888;text-transform:uppercase;letter-spacing:1px;">"集成测试"</span>
            </div>
            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px 28px; text-align:center; min-width:120px;">
                <strong style="display:block;font-size:28px;color:#4a9eff;font-weight:300;">"300+"</strong>
                <span style="font-size:12px;color:#888;text-transform:uppercase;letter-spacing:1px;">"纯公式函数"</span>
            </div>
            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px 28px; text-align:center; min-width:120px;">
                <strong style="display:block;font-size:28px;color:#4a9eff;font-weight:300;">"28"</strong>
                <span style="font-size:12px;color:#888;text-transform:uppercase;letter-spacing:1px;">"公式模块"</span>
            </div>
            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px 28px; text-align:center; min-width:120px;">
                <strong style="display:block;font-size:28px;color:#4a9eff;font-weight:300;">"10"</strong>
                <span style="font-size:12px;color:#888;text-transform:uppercase;letter-spacing:1px;">"太阳系天体"</span>
            </div>
        </div>

        <div style="text-align:center; margin:40px 0;">
            <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; margin-bottom:12px; font-family:monospace;">
                "/ MODULE DIRECTORY"
            </div>
            <h2 style="font-size:24px; font-weight:300; color:#fff; margin:0 0 24px;">"模块目录"</h2>

            <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(280px, 1fr)); gap:16px;">
                <a href="./architecture" style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px;">
                    <span style="font-family:monospace; font-size:12px; color:#4a9eff;">"01"</span>
                    <strong style="font-size:16px; color:#fff;">"核心引擎"</strong>
                    <small style="font-size:13px; color:#888; line-height:1.5;">"World、刚体、碰撞体、关节、查询、控制器"</small>
                    <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
                </a>
                <a href="./gravity" style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px;">
                    <span style="font-family:monospace; font-size:12px; color:#4a9eff;">"02"</span>
                    <strong style="font-size:16px; color:#fff;">"物理系统"</strong>
                    <small style="font-size:13px; color:#888; line-height:1.5;">"引力、地形、力注册表、事件系统、空气动力学、流体"</small>
                    <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
                </a>
                <a href="./formula" style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px;">
                    <span style="font-family:monospace; font-size:12px; color:#4a9eff;">"03"</span>
                    <strong style="font-size:16px; color:#fff;">"领域公式"</strong>
                    <small style="font-size:13px; color:#888; line-height:1.5;">"28 模块 — 航天、天体物理、核物理、相对论、量子等"</small>
                    <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
                </a>
                <a href="./arena" style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px;">
                    <span style="font-family:monospace; font-size:12px; color:#4a9eff;">"04"</span>
                    <strong style="font-size:16px; color:#fff;">"集成方案"</strong>
                    <small style="font-size:13px; color:#888; line-height:1.5;">"Arena 共享内存、JNI/FFM 绑定、Java 生态"</small>
                    <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
                </a>
                <a href="./api" style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px; text-decoration:none; color:#ccc; display:flex; flex-direction:column; gap:8px;">
                    <span style="font-family:monospace; font-size:12px; color:#4a9eff;">"05"</span>
                    <strong style="font-size:16px; color:#fff;">"参考资料"</strong>
                    <small style="font-size:13px; color:#888; line-height:1.5;">"完整 API 表、精度与性能、优化指南"</small>
                    <em style="font-style:normal; font-size:18px; color:#4a9eff; text-align:right; margin-top:auto;">"↗"</em>
                </a>
            </div>
        </div>

        <div style="margin:40px 0;">
            <h2 style="font-size:20px; font-weight:300; color:#fff; margin:0 0 16px;">"公式模块 (28)"</h2>
            <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(140px, 1fr)); gap:12px;">
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"88"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"航天工程"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"23"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"核物理"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"26"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"材料力学"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"19"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"天体物理"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"23"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"相对论"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"20"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"量子力学"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"16"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"电磁学"</span>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:6px; padding:16px; text-align:center;">
                    <span style="display:block;font-size:22px;color:#4a9eff;font-weight:600;">"18"</span>
                    <span style="font-size:11px;color:#888;text-transform:uppercase;letter-spacing:1px;">"流体力学"</span>
                </div>
            </div>
        </div>

        <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
            <p>"全部公式位于独立 crate " <span class="hi" style="color:#4a9eff; font-family:monospace;">"mps-formula"</span> " — 纯 Rust 实现，不依赖 Rapier 或 WorldHandle。"</p>
        </div>

        <div style="margin:40px 0;">
            <h2 style="font-size:20px; font-weight:300; color:#fff; margin:0 0 16px;">"核心特性"</h2>
            <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(300px, 1fr)); gap:16px;">
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"高精度引力"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"球谐展开 (EGM2008 8×8)、椭球引力、J2-J6 带谐、四极张量。自动根据轨道高度选择最优模型。"</p>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"辛积分器"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"Leapfrog、Yoshida 4 阶、Forest-Ruth 8 阶。Kahan 补偿精度从 15 位→30 位有效数字。后牛顿 1PN+2PN 相对论修正。"</p>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"内置天体"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"太阳系 10 天体精密参数 (JPL DE441)。地球 EGM2008、月球 LP165 + 12 Mascon (GRAIL)、火星 Mars50c。"</p>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"地形引力"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"多面体引力 (Werner-Scheeres)、DEM 地形质量分布、FFT 加速。月球 Mascon 模型防止低轨坠毁。"</p>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"ForceRegistry"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"类型化力注册表。任意力实现 ForceLaw trait 后自动调度，世界步进内自动聚合报告，无需手写分发逻辑。"</p>
                </div>
                <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:20px;">
                    <h3 style="font-size:16px; color:#fff; margin:0 0 8px;">"JNI + 共享内存"</h3>
                    <p style="font-size:14px; color:#999; line-height:1.6; margin:0;">"Java 21 JNI 全绑定 (~280 方法)。共享内存 Arena (DirectByteBuffer) 零 JNI 读写，每帧仅 1 次 world_step 调用。"</p>
                </div>
            </div>
        </div>

        <div style="margin:40px 0;">
            <h2 style="font-size:20px; font-weight:300; color:#fff; margin:0 0 16px;">"架构设计"</h2>
            <pre style="background:#0d0d2b; border:1px solid #333; border-radius:6px; padding:16px; font-size:13px; line-height:1.5;">
                <code class="language-text">
"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI (~480 函数)
       ├─ mps-formula  — 28 纯公式模块 (300+ 函数)
       ├─ mps-core     — 物理引擎 + Rapier 封装 (World, 刚体, 碰撞体, 查询, 事件)
       ├─ mps-jni      — JNI 绑定 (~280 方法)
       ├─ mps-ffm      — FFM 元数据
       └─ mps-test     — 332 集成测试"
                </code>
            </pre>
        </div>
    }
}