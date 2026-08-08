use topcoat::router::page;
use topcoat::view::view;

/// Formula modules page
#[page("/formula")]
pub async fn formula() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ FORMULA"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">"28 个物理模块 + 数值基石"</span><span data-lang="en">"28 Physics Modules + Numerical Foundation"</span></h1>
                    <p class="page-desc"><span data-lang="zh">{ "mps-formula 是 MPS 的纯函数科学核心 —— 28 个物理模块 + 公共数学/误差控制 + last-error 线程槽。每个函数 panic-free、零分配、可独立编译。所有物理量计算（引力、空气动力学、量子力学、相对论...）在此实现，C ABI 与 cosmos 都消费它。" }</span><span data-lang="en">{ "mps-formula is MPS's pure-function scientific core — 28 physics modules + common math/error + last-error thread slot. Every function panic-free, zero-alloc, independently compilable. All physics calculations (gravity, aerodynamics, quantum, relativity...) live here; C ABI and cosmos both consume it." }</span></p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "公共基石 (mps-formula::{math, error, ffi})" }</span><span data-lang="en">{ "Common Foundation (mps-formula::{math, error, ffi})" }</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"模块"</span><span data-lang="en">"Module"</span></th><th><span data-lang="zh">"角色"</span><span data-lang="en">"Role"</span></th><th><span data-lang="zh">"关键导出"</span><span data-lang="en">"Key Exports"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"math"</code></td><td>{ <span data-lang="zh">"数值工具：近似比较、Kahan 补偿求和"</span><span data-lang="en">"Numerical tools: approximate comparison, Kahan compensated summation"</span> }</td><td><code>"approx_eq, KahanSum, KahanVec3"</code></td></tr>
                            <tr><td><code>"error"</code></td><td>{ <span data-lang="zh">"last-error 线程槽（FFI 错误契约）"</span><span data-lang="en">"last-error thread slot (FFI error contract)"</span> }</td><td><code>"set_error, error_code, error_message, clear_error"</code></td></tr>
                            <tr><td><code>"ffi"</code></td><td>{ <span data-lang="zh">"C ABI 桥接类型 (Vec3, Quaternion, Mat3)"</span><span data-lang="en">"C ABI bridge types (Vec3, Quaternion, Mat3)"</span> }</td><td><code>"Vec3 { x, y, z }"</code> { <span data-lang="zh">" 等 #[repr(C)] 结构"</span><span data-lang="en">" and other #[repr(C)] structs"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"全 crate 共享 last-error 线程槽："</span><span data-lang="en">"Full-crate shared last-error thread slot: "</span> }<code>"error::set_error(code, msg)"</code>{ <span data-lang="zh">" 写当前线程的 code+message，"</span><span data-lang="en">" writes current thread's code+message, "</span> }<code>"error_code() / error_message()"</code>{ <span data-lang="zh">" 读。C ABI 在每个入口的失败分支写槽，Java 侧 "</span><span data-lang="en">" reads. C ABI writes slot on failure at every entry, Java via "</span> }<code>"abiLastErrorMessage()"</code>{ <span data-lang="zh">" 取最近消息。"</span><span data-lang="en">" fetches latest message."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "错误码常量 (mps-formula::error)" }</span><span data-lang="en">{ "Error Code Constants (mps-formula::error)" }</span></h2>
                <div class="code-block">
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
                <p class="p-note-top14">{ <span data-lang="zh">"遭 panic 时 ffi_guard 兜底并写 "</span><span data-lang="en">"On panic, ffi_guard catches and writes "</span> }<code>"ERR_INTERNAL"</code>{ <span data-lang="zh">" + 调用栈首行——对 Java 调用者表现为返回 false/0/null 的 abiLastErrorMessage()。从不 unwind 穿越边界（workspace 设 "</span><span data-lang="en">" + first call-stack frame — Java sees return false/0/null + abiLastErrorMessage(). Never unwinds across boundary (workspace sets "</span> }<code>"panic = \"abort\""</code>{ <span data-lang="zh">" 既是保险也是性能暗示）。"</span><span data-lang="en">" as both safety and perf hint)."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "28 个物理模块" }</span><span data-lang="en">{ "28 Physics Modules" }</span></h2>
                <div class="formula-grid">
                    <div class="formula-card">
                        <code>"acoustics"</code><div class="formula-label"><span data-lang="zh">"声学"</span><span data-lang="en">"Acoustics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"aerodynamics"</code><div class="formula-label"><span data-lang="zh">"空气动力学"</span><span data-lang="en">"Aerodynamics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"astrophysics"</code><div class="formula-label"><span data-lang="zh">"天体物理"</span><span data-lang="en">"Astrophysics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"biomechanics"</code><div class="formula-label"><span data-lang="zh">"生物力学"</span><span data-lang="en">"Biomechanics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"celestial_data"</code><div class="formula-label"><span data-lang="zh">"天体参数 (JPL DE441)"</span><span data-lang="en">"Celestial Data (JPL DE441)"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"chaos"</code><div class="formula-label"><span data-lang="zh">"混沌/分形"</span><span data-lang="en">"Chaos/Fractals"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"continuum"</code><div class="formula-label"><span data-lang="zh">"连续介质"</span><span data-lang="en">"Continuum"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"control_theory"</code><div class="formula-label"><span data-lang="zh">"控制理论"</span><span data-lang="en">"Control Theory"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"electromagnetism"</code><div class="formula-label"><span data-lang="zh">"电磁学"</span><span data-lang="en">"Electromagnetism"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"fluid"</code><div class="formula-label"><span data-lang="zh">"流体力学"</span><span data-lang="en">"Fluid Dynamics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"gravitational_models"</code><div class="formula-label"><span data-lang="zh">"引力模型"</span><span data-lang="en">"Gravity Models"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"integrators"</code><div class="formula-label"><span data-lang="zh">"辛积分器 + Kahan + PN"</span><span data-lang="en">"Symplectic Integrators + Kahan + PN"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"material_mechanics"</code><div class="formula-label"><span data-lang="zh">"材料力学"</span><span data-lang="en">"Solid Mechanics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"math"</code><div class="formula-label"><span data-lang="zh">"数值工具"</span><span data-lang="en">"Numerical Tools"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"molecular"</code><div class="formula-label"><span data-lang="zh">"分子动力学"</span><span data-lang="en">"Molecular Dynamics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"nuclear"</code><div class="formula-label"><span data-lang="zh">"核物理"</span><span data-lang="en">"Nuclear Physics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"physchem"</code><div class="formula-label"><span data-lang="zh">"物理化学"</span><span data-lang="en">"Physical Chemistry"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"plasma"</code><div class="formula-label"><span data-lang="zh">"等离子体"</span><span data-lang="en">"Plasma"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"quantum"</code><div class="formula-label"><span data-lang="zh">"量子力学"</span><span data-lang="en">"Quantum Mechanics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"relativity"</code><div class="formula-label"><span data-lang="zh">"相对论"</span><span data-lang="en">"Relativity"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"softbody"</code><div class="formula-label"><span data-lang="zh">"软体物理"</span><span data-lang="en">"Soft-body Physics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"spaceflight"</code><div class="formula-label"><span data-lang="zh">"航天工程"</span><span data-lang="en">"Spaceflight"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"superfluidity"</code><div class="formula-label"><span data-lang="zh">"超流体"</span><span data-lang="en">"Superfluidity"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"thermodynamics"</code><div class="formula-label"><span data-lang="zh">"热力学"</span><span data-lang="en">"Thermodynamics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"topology"</code><div class="formula-label"><span data-lang="zh">"拓扑/几何"</span><span data-lang="en">"Topology/Geometry"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"trajectory"</code><div class="formula-label"><span data-lang="zh">"弹道学"</span><span data-lang="en">"Ballistics"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"transmission"</code><div class="formula-label"><span data-lang="zh">"传动/齿轮"</span><span data-lang="en">"Transmission/Gearing"</span></div>
                    </div>
                    <div class="formula-card">
                        <code>"wave_optics"</code><div class="formula-label"><span data-lang="zh">"波动光学"</span><span data-lang="en">"Wave Optics"</span></div>
                    </div>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"数值基石"</span><span data-lang="en">"Numerical Foundation"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"两个低层工具被太空和地面都依赖："</span><span data-lang="en">"Two low-level tools shared by both cosmos and ground paths:"</span> }</p>
                <ul class="ul-plain">
                    <li><strong><code>"math::approx_eq(a, b, eps_abs, eps_rel)"</code></strong> { <span data-lang="zh">" — 混合绝对/相对误差比较，测试与运行期容差判断都用"</span><span data-lang="en">" — mixed absolute/relative error comparison, used in both tests and runtime tolerance checks"</span> }</li>
                    <li><strong><code>"math::KahanSum / KahanVec3"</code></strong> { <span data-lang="zh">" — Kahan 补偿求和；长弧积分里把有效精度从 15 位提到 ~30 位，详见 "</span><span data-lang="en">" — Kahan compensated summation; raises effective precision from 15 to ~30 digits in long-arc integration, see "</span> }<a href="./integrators" class="link"><span data-lang="zh">"辛积子"</span><span data-lang="en">"Symplectic Integrators"</span></a>{ <span data-lang="zh">"。"</span><span data-lang="en">"."</span> }</li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"最简调用 (Rust)"</span><span data-lang="en">"Minimal Call (Rust)"</span> }</h2>
                <pre><code class="language-rust">
"use mps_formula::spaceflight::orbital_velocity;       // 例：圆轨道速度
use mps_formula::math::approx_eq;

let v = orbital_velocity(3.986e14, 6.7e6);            // GM, r
assert!(approx_eq(v, 7707.0, 1e-3, 1e-9));"
                </code></pre>
                <p class="p-note">{ <span data-lang="zh">"C ABI 的对外函数（mps-core/mps-formula::ffi）只是这些 pub fn 的 #[repr(C)] 包装：算前 clear_error()，失败 set_error() + 返回 0/false/null，成功返回值 + 不动 error 槽。Java/JNI 与 Java/FFM 都走同一条 ABI。"</span><span data-lang="en">"C ABI exports (mps-core/mps-formula::ffi) are #[repr(C)] wrappers around these pub fn: clear_error() before compute, set_error() + return 0/false/null on failure, return value + leave error slot untouched on success. Java/JNI and Java/FFM use the same ABI path."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" 全 crate "</span><span data-lang="en">" Full crate "</span> }<strong>"panic-free"</strong>{ <span data-lang="zh">"：I/O、文件、分配从不出现在热路径；除 fmt::Debug 外不依赖 std::io。可在 no_std 路径上裁剪（仅 celestrial_data + math + error 子集），适配嵌入式 GNC。"</span><span data-lang="en">": no I/O, files, or allocations on hot paths; no std::io dependency except fmt::Debug. Can be trimmed for no_std (subset of celestial_data + math + error), suitable for embedded GNC."</span> }</p>
            </div>
        </div>
    }
}
