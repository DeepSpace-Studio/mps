use topcoat::router::page;
use topcoat::view::view;

/// Formula modules page
#[page("/formula")]
pub async fn formula() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ FORMULA"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">"28 个物理模块 + 数值基石"</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "mps-formula 是 MPS 的纯函数科学核心 —— 28 个物理模块 + 公共数学/误差控制 + last-error 线程槽。每个函数 panic-free、零分配、可独立编译。所有物理量计算（引力、空气动力学、量子力学、相对论...）在此实现，C ABI 与 cosmos 都消费它。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "公共基石 (mps-formula::{math, error, ffi})" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"模块"</th><th>"角色"</th><th>"关键导出"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"math"</code></td><td>{ "数值工具：近似比较、Kahan 补偿求和" }</td><td><code>"approx_eq, KahanSum, KahanVec3"</code></td></tr>
                            <tr><td><code>"error"</code></td><td>{ "last-error 线程槽（FFI 错误契约）" }</td><td><code>"set_error, error_code, error_message, clear_error"</code></td></tr>
                            <tr><td><code>"ffi"</code></td><td>{ "C ABI 桥接类型 (Vec3, Quaternion, Mat3)" }</td><td><code>"Vec3 { x, y, z }"</code> { " 等 #[repr(C)] 结构" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "全 crate 共享 last-error 线程槽：" }<code>"error::set_error(code, msg)"</code>{ " 写当前线程的 code+message，" }<code>"error_code() / error_message()"</code>{ " 读。C ABI 在每个入口的失败分支写槽，Java 侧 " }<code>"abiLastErrorMessage()"</code>{ " 取最近消息。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "错误码常量 (mps-formula::error)" }</h2>
                <div style="background:#0d0d2b; border:1px solid #333; border-radius:6px; padding:12px 16px;">
                    <pre><code class="language-rust">
"pub const ERR_OK: u32               = 0;  // 无错
pub const ERR_NULL_POINTER: u32     = 1;  // 入参 *const T 为 null
pub const ERR_INVALID_ARGUMENT: u32 = 2;  // 越界 / NaN / 非法枚举
pub const ERR_NOT_FOUND: u32        = 3;  // handle 不在注册表
pub const ERR_CAPACITY: u32         = 4;  // arena 容量超限
pub const ERR_UNSUPPORTED: u32      = 5;  // 该构建模式不支持
pub const ERR_INTERNAL: u32         = 6;  // panic 被 ffi_guard 接住"
                    </code></pre>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "遭 panic 时 ffi_guard 兜底并写 " }<code>"ERR_INTERNAL"</code>{ " + 调用栈首行——对 Java 调用者表现为返回 false/0/null 的 abiLastErrorMessage()。从不 unwind 穿越边界（workspace 设 " }<code>"panic = \"abort\""</code>{ " 既是保险也是性能暗示）。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "28 个物理模块" }</h2>
                <div style="display:grid; grid-template-columns:repeat(auto-fill, minmax(240px, 1fr)); gap:8px;">
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"acoustics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"声学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"aerodynamics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"空气动力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"astrophysics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"天体物理"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"biomechanics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"生物力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"celestial_data"</code><div style="color:#999; font-size:12px; margin-top:2px;">"天体参数 (JPL DE441)"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"chaos"</code><div style="color:#999; font-size:12px; margin-top:2px;">"混沌/分形"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"continuum"</code><div style="color:#999; font-size:12px; margin-top:2px;">"连续介质"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"control_theory"</code><div style="color:#999; font-size:12px; margin-top:2px;">"控制理论"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"electromagnetism"</code><div style="color:#999; font-size:12px; margin-top:2px;">"电磁学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"fluid"</code><div style="color:#999; font-size:12px; margin-top:2px;">"流体力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"gravitational_models"</code><div style="color:#999; font-size:12px; margin-top:2px;">"引力模型"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"integrators"</code><div style="color:#999; font-size:12px; margin-top:2px;">"辛积分器 + Kahan + PN"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"material_mechanics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"材料力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"math"</code><div style="color:#999; font-size:12px; margin-top:2px;">"数值工具"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"molecular"</code><div style="color:#999; font-size:12px; margin-top:2px;">"分子动力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"nuclear"</code><div style="color:#999; font-size:12px; margin-top:2px;">"核物理"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"physchem"</code><div style="color:#999; font-size:12px; margin-top:2px;">"物理化学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"plasma"</code><div style="color:#999; font-size:12px; margin-top:2px;">"等离子体"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"quantum"</code><div style="color:#999; font-size:12px; margin-top:2px;">"量子力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"relativity"</code><div style="color:#999; font-size:12px; margin-top:2px;">"相对论"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"softbody"</code><div style="color:#999; font-size:12px; margin-top:2px;">"软体物理"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"spaceflight"</code><div style="color:#999; font-size:12px; margin-top:2px;">"航天工程"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"superfluidity"</code><div style="color:#999; font-size:12px; margin-top:2px;">"超流体"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"thermodynamics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"热力学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"topology"</code><div style="color:#999; font-size:12px; margin-top:2px;">"拓扑/几何"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"trajectory"</code><div style="color:#999; font-size:12px; margin-top:2px;">"弹道学"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"transmission"</code><div style="color:#999; font-size:12px; margin-top:2px;">"传动/齿轮"</div>
                    </div>
                    <div style="background:#0d0d2b; border:1px solid #333; border-radius:4px; padding:10px 14px;">
                        <code style="color:#4a9eff; font-size:13px;">"wave_optics"</code><div style="color:#999; font-size:12px; margin-top:2px;">"波动光学"</div>
                    </div>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "数值基石" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "两个低层工具被太空和地面都依赖：" }</p>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong style="color:#ddd;"><code>"math::approx_eq(a, b, eps_abs, eps_rel)"</code></strong> { " — 混合绝对/相对误差比较，测试与运行期容差判断都用" }</li>
                    <li><strong style="color:#ddd;"><code>"math::KahanSum / KahanVec3"</code></strong> { " — Kahan 补偿求和；长弧积分里把有效精度从 15 位提到 ~30 位，详见 " }<a href="./integrators" style="color:#4a9eff;">"辛积子"</a>{ "。" }</li>
                </ul>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "最简调用 (Rust)" }</h2>
                <pre><code class="language-rust">
"use mps_formula::spaceflight::orbital_velocity;       // 例：圆轨道速度
use mps_formula::math::approx_eq;

let v = orbital_velocity(3.986e14, 6.7e6);            // GM, r
assert!(approx_eq(v, 7707.0, 1e-3, 1e-9));"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "C ABI 的对外函数（mps-core/mps-formula::ffi）只是这些 pub fn 的 #[repr(C)] 包装：算前 clear_error()，失败 set_error() + 返回 0/false/null，成功返回值 + 不动 error 槽。Java/JNI 与 Java/FFM 都走同一条 ABI。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 全 crate " }<strong>"panic-free"</strong>{ "：I/O、文件、分配从不出现在热路径；除 fmt::Debug 外不依赖 std::io。可在 no_std 路径上裁剪（仅 celestrial_data + math + error 子集），适配嵌入式 GNC。" }</p>
            </div>
        </div>
    }
}
