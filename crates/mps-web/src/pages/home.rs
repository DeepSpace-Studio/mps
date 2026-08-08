use topcoat::{router::page, view::view};

use crate::metrics::{
    CELESTIAL_COUNT, CORE_FFI_COUNT, FORMULA_MODULE_COUNT, GRAVITY_MODEL_COUNT,
    INTEGRATOR_COUNT, JNI_METHOD_COUNT, TEST_COUNT,
};

/// Home page — MPS Physics System overview
#[page("/")]
pub async fn home() -> topcoat::Result {
    view! {
        <div class="hero">
            <div class="hero-tag">"/ MPS PHYSICS OBSERVATORY"</div>
            <h1 class="hero-title">
                <span data-lang="zh">"运动物理系统 (米每秒)"</span>
                <span data-lang="en">"Motion Physics System (Meters Per Second)"</span>
            </h1>
            <p class="hero-desc">
                <span data-lang="zh">{ "基于 " }<strong class="text-hl">"Rapier3D-f64"</strong>{ " 的高精度 Rust 物理引擎。通过 C FFI (" }<strong class="text-hl">{ (CORE_FFI_COUNT) }</strong>{ " 函数) 和 Java JNI (" }<strong class="text-hl">{ (JNI_METHOD_COUNT) }</strong>{ " 方法) 暴露完整 API。支持 " }<strong class="text-hl">{ (TEST_COUNT) }</strong>{ " 项测试、" }<strong class="text-hl">{ (GRAVITY_MODEL_COUNT) }</strong>{ " 种引力模型、" }<strong class="text-hl">{ (INTEGRATOR_COUNT) }</strong>{ " 种辛积分器、共享内存零拷贝 Arena、" }<strong class="text-hl">{ (FORMULA_MODULE_COUNT) }</strong>{ " 个公式模块和 " }<strong class="text-hl">{ (CELESTIAL_COUNT) }</strong>{ " 个太阳系天体。" }</span>
                <span data-lang="en">{ "High-precision Rust physics engine based on " }<strong class="text-hl">"Rapier3D-f64"</strong>{ ". Full API exposed via C FFI (" }<strong class="text-hl">{ (CORE_FFI_COUNT) }</strong>{ " functions) and Java JNI (" }<strong class="text-hl">{ (JNI_METHOD_COUNT) }</strong>{ " methods). " }<strong class="text-hl">{ (TEST_COUNT) }</strong>{ " tests, " }<strong class="text-hl">{ (GRAVITY_MODEL_COUNT) }</strong>{ " gravity models, " }<strong class="text-hl">{ (INTEGRATOR_COUNT) }</strong>{ " symplectic integrators, zero-copy shared-memory Arena, " }<strong class="text-hl">{ (FORMULA_MODULE_COUNT) }</strong>{ " formula modules and " }<strong class="text-hl">{ (CELESTIAL_COUNT) }</strong>{ " celestial bodies." }</span>
            </p>
            <div class="hero-actions">
                <a href="quickstart" class="btn-primary">
                    <span data-lang="zh">"快速入门"</span>
                    <span data-lang="en">"Quickstart"</span>
                </a>
                <a href="api" class="btn-outline">
                    <span data-lang="zh">"API 参考"</span>
                    <span data-lang="en">"API Reference"</span>
                </a>
            </div>
        </div>

        <div class="metric-grid">
            <div class="metric-card">
                <strong class="num">{ (TEST_COUNT) }</strong>
                <span class="label"><span data-lang="zh">"集成测试"</span><span data-lang="en">"Tests"</span></span>
            </div>
            <div class="metric-card">
                <strong class="num">"300+"</strong>
                <span class="label"><span data-lang="zh">"纯公式函数"</span><span data-lang="en">"Formula Fns"</span></span>
            </div>
            <div class="metric-card">
                <strong class="num">{ (FORMULA_MODULE_COUNT) }</strong>
                <span class="label"><span data-lang="zh">"公式模块"</span><span data-lang="en">"Formula Modules"</span></span>
            </div>
            <div class="metric-card">
                <strong class="num">{ (CELESTIAL_COUNT) }</strong>
                <span class="label"><span data-lang="zh">"太阳系天体"</span><span data-lang="en">"Celestial Bodies"</span></span>
            </div>
        </div>

        <div class="text-center section-divider">
            <div class="hero-tag">"/ MODULE DIRECTORY"</div>
            <h2 class="section-heading-lg">
                <span data-lang="zh">"模块目录"</span>
                <span data-lang="en">"Module Directory"</span>
            </h2>

            <div class="module-grid">
                <a href="architecture" class="module-card">
                    <span class="idx">"01"</span>
                    <strong class="title"><span data-lang="zh">"核心引擎"</span><span data-lang="en">"Core Engine"</span></strong>
                    <small class="desc"><span data-lang="zh">"World、刚体、碰撞体、关节、查询、控制器"</span><span data-lang="en">"World, rigid bodies, colliders, joints, queries, controllers"</span></small>
                    <em class="arrow">"↗"</em>
                </a>
                <a href="cosmos" class="module-card">
                    <span class="idx">"06"</span>
                    <strong class="title"><span data-lang="zh">"太空刚体演算"</span><span data-lang="en">"Cosmos Rigid Body"</span></strong>
                    <small class="desc"><span data-lang="zh">"CosmosWorld、Verlet 轨道积分、n-body 互引力、环境扰动"</span><span data-lang="en">"CosmosWorld, Verlet orbit integration, n-body gravity, perturbations"</span></small>
                    <em class="arrow">"↗"</em>
                </a>
                <a href="gravity" class="module-card">
                    <span class="idx">"02"</span>
                    <strong class="title"><span data-lang="zh">"物理系统"</span><span data-lang="en">"Physics Systems"</span></strong>
                    <small class="desc"><span data-lang="zh">"引力、地形、力注册表、事件系统、空气动力学、流体"</span><span data-lang="en">"Gravity, terrain, force registry, events, aerodynamics, fluid"</span></small>
                    <em class="arrow">"↗"</em>
                </a>
                <a href="formula" class="module-card">
                    <span class="idx">"03"</span>
                    <strong class="title"><span data-lang="zh">"领域公式"</span><span data-lang="en">"Domain Formulas"</span></strong>
                    <small class="desc"><span data-lang="zh">"28 模块 — 航天、天体物理、核物理、相对论、量子等"</span><span data-lang="en">"28 modules — spaceflight, astrophysics, nuclear, relativity, quantum, etc."</span></small>
                    <em class="arrow">"↗"</em>
                </a>
                <a href="arena" class="module-card">
                    <span class="idx">"04"</span>
                    <strong class="title"><span data-lang="zh">"集成方案"</span><span data-lang="en">"Integration"</span></strong>
                    <small class="desc"><span data-lang="zh">"Arena 共享内存、JNI/FFM 绑定、Java 生态"</span><span data-lang="en">"Arena shared memory, JNI/FFM bindings, Java ecosystem"</span></small>
                    <em class="arrow">"↗"</em>
                </a>
                <a href="api" class="module-card">
                    <span class="idx">"05"</span>
                    <strong class="title"><span data-lang="zh">"参考资料"</span><span data-lang="en">"Reference"</span></strong>
                    <small class="desc"><span data-lang="zh">"完整 API 表、精度与性能、优化指南"</span><span data-lang="en">"Full API tables, precision & performance, optimization guide"</span></small>
                    <em class="arrow">"↗"</em>
                </a>
            </div>
        </div>

        <div class="section-divider">
            <h2 class="section-heading">
                <span data-lang="zh">{ "公式模块 (" }{ (FORMULA_MODULE_COUNT) }{ ")" }</span>
                <span data-lang="en">{ "Formula Modules (" }{ (FORMULA_MODULE_COUNT) }{ ")" }</span>
            </h2>
            <div class="mini-stat-grid">
                <div class="stat-card"><span class="num">"88"</span><span class="label"><span data-lang="zh">"航天工程"</span><span data-lang="en">"Spaceflight"</span></span></div>
                <div class="stat-card"><span class="num">"23"</span><span class="label"><span data-lang="zh">"核物理"</span><span data-lang="en">"Nuclear"</span></span></div>
                <div class="stat-card"><span class="num">"26"</span><span class="label"><span data-lang="zh">"材料力学"</span><span data-lang="en">"Mechanics"</span></span></div>
                <div class="stat-card"><span class="num">"19"</span><span class="label"><span data-lang="zh">"天体物理"</span><span data-lang="en">"Astrophysics"</span></span></div>
                <div class="stat-card"><span class="num">"23"</span><span class="label"><span data-lang="zh">"相对论"</span><span data-lang="en">"Relativity"</span></span></div>
                <div class="stat-card"><span class="num">"20"</span><span class="label"><span data-lang="zh">"量子力学"</span><span data-lang="en">"Quantum"</span></span></div>
                <div class="stat-card"><span class="num">"16"</span><span class="label"><span data-lang="zh">"电磁学"</span><span data-lang="en">"Electromagnetism"</span></span></div>
                <div class="stat-card"><span class="num">"18"</span><span class="label"><span data-lang="zh">"流体力学"</span><span data-lang="en">"Fluid Dynamics"</span></span></div>
            </div>
        </div>

        <div class="callout">
            <p>
                <span data-lang="zh">{ "全部公式位于独立 crate " }<span class="hi">"mps-formula"</span>{ " — 纯 Rust 实现，不依赖 Rapier 或 WorldHandle。" }</span>
                <span data-lang="en">{ "All formulas live in a standalone crate " }<span class="hi">"mps-formula"</span>{ " — pure Rust, no Rapier or WorldHandle dependency." }</span>
            </p>
        </div>

        <div class="section-divider">
            <h2 class="section-heading">
                <span data-lang="zh">"核心特性"</span>
                <span data-lang="en">"Key Features"</span>
            </h2>
            <div class="feature-grid">
                <div class="feature-card">
                    <h3><span data-lang="zh">"高精度引力"</span><span data-lang="en">"High-Precision Gravity"</span></h3>
                    <p><span data-lang="zh">"球谐展开 (EGM2008 8×8)、椭球引力、J2-J6 带谐、四极张量。自动根据轨道高度选择最优模型。"</span><span data-lang="en">"Spherical harmonics (EGM2008 8×8), ellipsoidal gravity, J2-J6 zonal harmonics, quadrupole tensor. Auto-selects optimal model by orbital altitude."</span></p>
                </div>
                <div class="feature-card">
                    <h3><span data-lang="zh">"辛积分器"</span><span data-lang="en">"Symplectic Integrators"</span></h3>
                    <p><span data-lang="zh">{ "Leapfrog、Yoshida 4 阶、Forest-Ruth 8 阶。Kahan 补偿精度从 15 位→30 位有效数字。后牛顿 1PN+2PN 相对论修正。" }<a href="./cosmos" class="link">"mps-cosmos"</a>{ " 另提供 velocity-Verlet 轨道积分，长弧相位误差随 dt² 收敛。" }</span><span data-lang="en">{ "Leapfrog, Yoshida 4th order, Forest-Ruth 8th order. Kahan compensation: 15→30 significant digits. Post-Newtonian 1PN+2PN corrections." }<a href="./cosmos" class="link">"mps-cosmos"</a>{ " adds velocity-Verlet orbit integration with dt² phase error convergence." }</span></p>
                </div>
                <div class="feature-card">
                    <h3><span data-lang="zh">"内置天体"</span><span data-lang="en">"Built-in Celestials"</span></h3>
                    <p><span data-lang="zh">"太阳系 10 天体精密参数 (JPL DE441)。地球 EGM2008、月球 LP165 + 12 Mascon (GRAIL)、火星 Mars50c。"</span><span data-lang="en">"10 solar system bodies with precision data (JPL DE441). Earth EGM2008, Moon LP165 + 12 Mascons (GRAIL), Mars Mars50c."</span></p>
                </div>
                <div class="feature-card">
                    <h3><span data-lang="zh">"地形引力"</span><span data-lang="en">"Terrain Gravity"</span></h3>
                    <p><span data-lang="zh">"多面体引力 (Werner-Scheeres)、DEM 地形质量分布、FFT 加速。月球 Mascon 模型防止低轨坠毁。"</span><span data-lang="en">"Polyhedral gravity (Werner-Scheeres), DEM terrain mass distribution, FFT acceleration. Lunar Mascon model prevents low-orbit decay."</span></p>
                </div>
                <div class="feature-card">
                    <h3>"ForceRegistry"</h3>
                    <p><span data-lang="zh">"类型化力注册表。任意力实现 ForceLaw trait 后自动调度，世界步进内自动聚合报告，无需手写分发逻辑。"</span><span data-lang="en">"Typed force registry. Any force implementing ForceLaw trait auto-dispatches; world step auto-aggregates reports, no manual dispatch needed."</span></p>
                </div>
                <div class="feature-card">
                    <h3><span data-lang="zh">"JNI + 共享内存"</span><span data-lang="en">"JNI + Shared Memory"</span></h3>
                    <p><span data-lang="zh">{ "Java 21 JNI 全绑定 (" }<strong class="text-hl">{ (JNI_METHOD_COUNT) }</strong>{ " 方法)。共享内存 Arena (DirectByteBuffer) 零 JNI 读写，每帧仅 1 次 world_step 调用。" }</span><span data-lang="en">{ "Java 21 JNI full binding (" }<strong class="text-hl">{ (JNI_METHOD_COUNT) }</strong>{ " methods). Shared-memory Arena (DirectByteBuffer) for zero-JNI read/write, only 1 world_step call per frame." }</span></p>
                </div>
            </div>
        </div>

        <div class="section-divider">
            <h2 class="section-heading">
                <span data-lang="zh">"架构设计"</span>
                <span data-lang="en">"Architecture"</span>
            </h2>
            <pre><code class="language-text">
<span data-lang="zh">"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI ("</span><span data-lang="en">"Java 21 JNI / Java 25 FFM
  └─ Rust C ABI ("</span>{ (CORE_FFI_COUNT) }<span data-lang="zh">" 函数)
       ├─ mps-formula  — 28 纯公式模块 (300+ 函数)
       ├─ mps-core     — 物理引擎 + Rapier 封装 (World, 刚体, 碰撞体, 查询, 事件)
       ├─ mps-cosmos   — 太空刚体演算 (独立 world, Verlet 轨道积分)
       ├─ mps-jni      — JNI 绑定 ("</span><span data-lang="en">" functions)
       ├─ mps-formula  — 28 pure formula modules (300+ functions)
       ├─ mps-core     — physics engine + Rapier wrapper (World, bodies, colliders, queries, events)
       ├─ mps-cosmos   — cosmos rigid body (separate world, Verlet orbit integration)
       ├─ mps-jni      — JNI bindings ("</span>{ (JNI_METHOD_COUNT) }<span data-lang="zh">" 方法, 含 cosmos 一批)
       ├─ mps-ffm      — FFM 元数据
       └─ mps-test     — 集成测试 (含 cosmos 19)"</span><span data-lang="en">" methods, incl. cosmos batch)
       ├─ mps-ffm      — FFM metadata
       └─ mps-test     — integration tests (incl. cosmos 19)"</span>
            </code></pre>
        </div>
    }
}
