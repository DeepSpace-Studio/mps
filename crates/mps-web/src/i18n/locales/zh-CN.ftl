# MPS Motion Physics System — Simplified Chinese translations
# Fluent .ftl format (https://projectfluent.org/)

# ---- Navigation ----
nav-home = 首页
nav-quickstart = 快速入门
nav-architecture = 架构
nav-gravity = 引力模型
nav-integrators = 积分器
nav-formula = 公式模块
nav-voxel = 体素
nav-soft-body = 软体
nav-events = 事件
nav-arena = Arena
nav-batch = 批量碰撞体
nav-cosmos = 太空
nav-cosmos-class = 功能分类
nav-jni = JNI
nav-ffm = FFM
nav-api = API
nav-more = 更多 ▾

# ── 短标签,用于星系行星球的圆形按钮内(每球 ≈ 60px 直径,字数 2-4 最佳) ──
nav-planet-home = 首页
nav-planet-quickstart = 入门
nav-planet-architecture = 架构
nav-planet-gravity = 引力
nav-planet-integrators = 积分
nav-planet-formula = 公式
nav-planet-api = API
nav-planet-voxel = 体素
nav-planet-events = 事件
nav-planet-arena = Arena
nav-planet-batch = 批量
nav-planet-cosmos = 太空
nav-planet-jni = JNI
nav-planet-ffm = FFM

# ---- Language switcher ----
lang-zh = 中文
lang-en = English

# ---- Home page ----
home-hero-tag = / MPS 物理观测台
home-hero-title = 运动物理系统 (米每秒)
home-hero-desc = 基于 { $rapier } 的高精度 Rust 物理引擎。通过 C FFI ({ $ffi } 函数) 和 Java JNI ({ $jni } 方法) 暴露完整 API。支持 { $tests } 项测试、{ $gravity } 种引力模型、{ $integrators } 种辛积分器、共享内存零拷贝 Arena、{ $modules } 个公式模块和 { $bodies } 个太阳系天体。
home-cta-quickstart = 快速入门
home-cta-api = API 参考

home-stat-tests = 集成测试
home-stat-formula-fns = 纯公式函数
home-stat-formula-modules = 公式模块
home-stat-celestial = 太阳系天体

home-section-directory = 模块目录
home-section-formula-modules = 公式模块 ({ $count })
home-section-key-features = 核心特性
home-section-architecture = 架构设计

home-mod-core-title = 核心引擎
home-mod-core-desc = World、刚体、碰撞体、关节、查询、控制器
home-mod-cosmos-title = 太空刚体演算
home-mod-cosmos-desc = CosmosWorld、Verlet 轨道积分、n-body 互引力、环境扰动
home-mod-physics-title = 物理系统
home-mod-physics-desc = 引力、地形、力注册表、事件系统、空气动力学、流体
home-mod-formula-title = 领域公式
home-mod-formula-desc = 107 个纯公式模块（557 函数）— 航天、天体物理、核物理、相对论、量子等。
home-mod-integration-title = 集成方案
home-mod-integration-desc = Arena 共享内存、JNI/FFM 绑定、Java 生态
home-mod-reference-title = 参考资料
home-mod-reference-desc = 完整 API 表、精度与性能、优化指南

home-feat-gravity-title = 高精度引力
home-feat-gravity-desc = 球谐展开 (EGM2008 8×8)、椭球引力、J2-J6 带谐、四极张量。自动根据轨道高度选择最优模型。
home-feat-integrators-title = 辛积分器
home-feat-integrators-desc = Leapfrog、Yoshida 4 阶、Forest-Ruth 8 阶。Kahan 补偿精度从 15 位→30 位有效数字。后牛顿 1PN+2PN 相对论修正。
home-feat-celestial-title = 内置天体
home-feat-celestial-desc = 太阳系 10 天体精密参数 (JPL DE441)。地球 EGM2008、月球 LP165 + 12 Mascon (GRAIL)、火星 Mars50c。
home-feat-terrain-title = 地形引力
home-feat-terrain-desc = 多面体引力 (Werner-Scheeres)、DEM 地形质量分布、FFT 加速。月球 Mascon 模型防止低轨坠毁。
home-feat-registry-title = ForceRegistry
home-feat-registry-desc = 类型化力注册表。任意力实现 ForceLaw trait 后自动调度，世界步进内自动聚合报告，无需手写分发逻辑。
home-feat-jni-title = JNI + 共享内存
home-feat-jni-desc = Java 21 JNI 全绑定 ({ $count } 方法)。共享内存 Arena (DirectByteBuffer) 零 JNI 读写，每帧仅 1 次 world_step 调用。

home-callout = 全部公式位于独立 crate { $crate } — 纯 Rust 实现，不依赖 Rapier 或 WorldHandle。

# ---- Formula mini-stat labels (home module grid) ----
formula-cat-spaceflight = 航天
formula-cat-nuclear = 核物理
formula-cat-mechanics = 力学
formula-cat-astrophysics = 天体物理
formula-cat-relativity = 相对论
formula-cat-quantum = 量子
formula-cat-electromagnetism = 电磁学
formula-cat-fluid = 流体力学

# ---- Quickstart page ----
quickstart-tag = / 快速入门
quickstart-title = 快速入门
quickstart-desc = 从零搭建 MPS 物理引擎开发环境。
quickstart-step1-title = 安装 Rust 工具链
quickstart-step1-desc = 安装 Rust 1.75+ 和 cargo。推荐用 rustup。
quickstart-step2-title = 克隆仓库
quickstart-step2-desc = git clone 后进入 rigid-body 目录。
quickstart-step3-title = 构建核心库
quickstart-step3-desc = cargo build --workspace 编译全部 crate。
quickstart-step4-title = 运行测试
quickstart-step4-desc = cargo test --workspace 执行 { $tests } 项集成测试。
quickstart-step5-title = 生成 C 头文件
quickstart-step5-desc = cargo build -p mps-core 触发 cbindgen 生成 rigid_body.h。

# ---- Architecture page ----
arch-tag = // MPS
arch-title = 架构概览
arch-desc = 自 Java 顶部到 Rapier 底部的 crate 分层与每帧数据流。
arch-stack-title = crate 堆栈
arch-stack-lead = 自顶向下：Java 绑定 → C ABI → mps-core → Rapier3D-f64。
arch-stack-diagram = Java 21 JNI / Java 25 FFM
  └─ Rust C ABI (720 functions)
       ├─ mps-formula  — { $modules } pure formula modules (557 functions)
       ├─ mps-core     — physics engine + Rapier wrapper (World, bodies, colliders, queries, events)
       ├─ mps-cosmos   — cosmos rigid body (separate world, Verlet orbit integration)
       ├─ mps-jni      — JNI bindings ({ $methods } methods, incl. cosmos batch)
       ├─ mps-ffm      — FFM metadata
       └─ mps-test     — integration tests (incl. cosmos 19)
arch-layers-title = 各层职责
arch-layer-formula-title = mps-formula
arch-layer-formula-desc = { $modules } 纯公式模块（航天 / 天体物理 / 核物理等）。无 WorldHandle 依赖，可独立调用。
arch-layer-core-title = mps-core
arch-layer-core-desc = Rapier3D-f64 封装 + 共享 Arena + C-ABI。World、刚体、碰撞体、关节、查询、力注册表、事件系统。
arch-layer-cosmos-title = mps-cosmos
arch-layer-cosmos-desc = CosmosWorld 独立物理域：Verlet 轨道积分 + n-body 互引力 + 环境扰动。不与 mps-core 共享 World。
arch-layer-jni-title = mps-jni
arch-layer-jni-desc = Java 21 JNI 全绑定，共 { $methods } 个方法（含 cosmos 批量 API）。
arch-layer-test-title = mps-test
arch-layer-test-desc = { $tests } 项集成测试，并作为模块镜像 crate 校验源码结构。
arch-layer-ffm-title = mps-ffm
arch-layer-ffm-desc = Java 25 Foreign Function & Memory API 元数据，逐步替代 JNI。
arch-flow-title = 每帧数据流
arch-flow-lead = 从 Java 帧渲染侧看 MPS 的 5 步循环：
arch-flow-step-1 = Java 端读取 DirectByteBuffer 中的 Arena 数据，更新刚体目标位置 / 力。
arch-flow-step-2 = 触发一次 world_step(world, dt) FFI 调用 — 跨边界的唯一同步点。
arch-flow-step-3 = mps-core 内部调度：ForceRegistry 聚合力 → Rapier 碰撞解算 → 方向约束求解 → 事件派发。
arch-flow-step-4 = Arena 布局就位：位置 / 速度 / 力累积器 SoA 更新被直接写回 DirectByteBuffer。
arch-flow-step-5 = Java 端零拷贝读取 Arena，渲染 / 物理抽取。事件通过 ring buffer 拉取。
arch-tenets-title = 设计准则
arch-tenet-zero-copy = 零拷贝：Java 与 Rust 通过共享内存 Arena 交换刚体状态，避免逐对象 JNI 读写。
arch-tenet-formula-pure = 公式层纯净：mps-formula 不依赖 WorldHandle 或 Rapier，可在任意上下文纯函数式调用。
arch-tenet-ffi-stable = C ABI 稳定：cbindgen 生成 rigid_body.h，所有破坏性变更必须更新 banner 版本号。
arch-build-title = 构建管线
arch-build-cbindgen = mps-core 构建时通过 build.rs 触发 cbindgen，将 pub C-ABI 类型/函数写入 rigid_body.h。
arch-build-xtask = xtask 自动统计 TEST_COUNT / JNI_METHOD_COUNT / CORE_FFI_COUNT，并刷新 mps-web/src/metrics.rs。

# ---- Gravity page ----
grav-tag = // mps-core
grav-title = 引力模型
grav-desc = mps-core 的引力栈是可插拔的 ForceLaw：每个模型通过 world_set_*_gravity 注册，支持从牛顿点质量、球谐（EGM2008）、椭球、带谐 J2–J6、四极张量，到多面体（Werner–Scheeres）与月球 Mascon（GRAIL）共 8 类。
grav-models-title = 模型目录
grav-models-lead = 共 5 类，按轨道高度和精度需求自动选择。
grav-col-name = 名称
grav-col-use = 适用场景
grav-col-cost = 复杂度
grav-row-newton = 深空巡航；两体近似
grav-row-sh = 低轨高精度；EGM2008 8×8 模型
grav-row-ellipsoid = 地球扁率近似；快速椭圆体引力
grav-row-zonal = J2 主项（扁率）到 J6 高阶带谐
grav-row-quad = 任意四极张量积分
grav-row-poly = 小行星 / 不规则天体表面（Werner–Scheeres）
grav-row-mascon = 月球低轨（GRAIL 12 块质量异常）
grav-bodies-title = 内置天体
grav-body-earth-title = 地球 EGM2008
grav-body-earth-desc = 8×8 球谐系数截断，Jason/CHAMP/GRACE 卫星轨道仿真级别。
grav-body-moon-title = 月球 LP165 + Mascon
grav-body-moon-desc = LP165 球谐 + 12 块 GRAIL Mascon 模型，防止低轨仿真坠毁。
grav-body-mars-title = 火星 Mars50c
grav-body-mars-desc = 50 阶球谐截止，含极区椭率与季节性 CO₂ 极冠近似。
grav-body-sun-title = 太阳点源
grav-body-sun-desc = JPL DE441 太阳引力参数 GM=1.32712440018e20 m³/s²， Crimea 全部行星摄动起源。
grav-auto-title = 自动模型选择
grav-auto-lead = 引擎依据当前轨道高度和参考半径自适应切换模型。
grav-auto-note = 低轨 (< 200 km) 用球谐 + Mascon；中轨用 J2-J6；高轨 / 深空切回点质量。
grav-api-title = C ABI
grav-api-desc = 引力相关函数均经 mps-core C-ABI 暴露：
grav-bodies-grid-title = 天体清单

# ---- Integrators page ----
int-tag = // mps-core
int-title = 辛积分器
int-desc = Leapfrog、Yoshida-4 与 Forest–Ruth 8 阶辛步进器，外加 Kahan 补偿变体，以及后牛顿修正 — 全部位于 mps-formula::integrators。
int-why-title = 为何用辛积分器
int-why-lead = 经典 RK4 在长时间轨道弧段会缓慢流失能量，轨道逐渐内旋。
int-why-body = 辛步进器保留哈密顿量结构：能量误差保持有界并振荡而非持续增长，因此万年级传播仍闭合。每个步进器都提供 _kahan 变体，将 f64 精度从约 15 位提升至约 30 位有效数字。
int-catalog-title = 积分器目录
int-col-name = 积分器
int-col-order = 阶数
int-col-notes = 备注
int-row-leapfrog = Kick-Drift-Kick；2 阶时间反转对称；轨道最常用。
int-row-yoshida4 = 4 阶分步对称（3 子步）；能量误差 ~O(dt⁴)。
int-row-forest-ruth = 8 阶 Forest–Ruth 类；深空高精度近似行星运动。
int-kahan-title = Kahan 误差补偿
int-kahan-lead = 全部辛积分器有 _kahan 变体，通过 Kahan 算法补偿 f64 截断。
int-kahan-li-1 = 标准版能量稳定在 15 位有效数字。
int-kahan-li-2 = Kahan 版提升到 30 位有效数字。
int-kahan-li-3 = 仅在需要 long double 精度相当时使用 — 性能差 ~2×。
int-kahan-note = Kahan 算法对所有求和步骤维护补偿项 c = (sum - t) - y。
int-pn-title = 后牛顿相对论修正
int-pn-lead = 在 GM 中心天体附近，相对论效应可观测。
int-pn-li-1pn = post_newtonian_1pn — 一阶后牛顿（Mercury 近日点）。
int-pn-li-2pn = post_newtonian_2pn — 二阶后牛顿（高偏心率轨道 Lense–Thirring）。
int-pn-li-full = post_newtonian_full — 全组合 PN 项。
int-adaptive-title = 自适应步长
int-adaptive-desc = adaptive_step_size + step_accepted 控制积分器步长：
int-diag-title = 数值诊断
int-diag-energy = 比能量 ε = v²/2 - GM/r — 应保持守恒。
int-diag-am = 比角动量 h = r × v — 辛积分器严格保持。
int-diag-kepler = keplerian_elements() 转六根数，监控半长轴漂移。

# ---- Formula page ----
form-tag = // mps-formula
form-title = 公式模块
form-desc = 独立纯 Rust crate：107 个公式模块、557 个公开函数，覆盖航天到量子物理，零 Rapier / WorldHandle 依赖。
form-intro-pure = 全部公式位于独立 crate mps-formula — 纯 Rust 实现，不依赖 Rapier 或 WorldHandle。
form-mod-kepler = kepler.rs — 开普勒方程迭代解 / 六根数转换
form-mod-dynamics = dynamics.rs — 轨道动力学 / 双中心近似
form-mod-perturbation = perturbation.rs = 第三体摄动 / 大气阻力 / 太阳光压
form-mod-propulsion = propulsion.rs — 齐奥尔科夫斯基 / 推进剂预算
form-mod-rotation = rotation.rs — 刚体姿态动力学 / 四元数
form-mod-thermal = thermal.rs — 热平衡 / 太阳辐照
form-mod-debris = debris.rs — 碎片云演化 / 碰撞概率
form-mod-gnss = gnss.rs — GNSS 伪距 / 多频解算
form-mod-trajectory = trajectory.rs — Lambert 问题 / 转移轨道
form-mod-astrophysics = astrophysics.rs — 恒星结构 / 光度函数
form-mod-stellar = stellar.rs — 主序 / 赫罗图 / 演化轨迹
form-mod-galactic = galactic_dynamics.rs — 旋转曲线 / 密度波理论
form-mod-cosmology = cosmology.rs — FLRW 度规 / 哈勃流
form-mod-helio = heliophysics.rs — 太阳风 / 日冕参数
form-mod-high-energy = high_energy_astro.rs — 黑体辐射 / 同步辐射 / 逆 Compton
form-mod-celestial = celestial_data.rs — JPL DE441 10 天体精密参数
form-mod-planetary = planetary_science.rs — 行星内部结构 / 潮汐
form-mod-mechanics = material_mechanics.rs — 应力应变 / 屈服准则
form-mod-material = material_mechanics (子集) — 弹性张量 / 各向异性
form-mod-biomech = biomechanics.rs — 关节力矩 / 肌肉模型
form-mod-control = control_theory.rs — PID / LQR / 状态空间
form-mod-chaos = chaos.rs — Lorenz / Rössler 吸引子
form-mod-topology = topology.rs — 同伦 / 拓扑不变量
form-mod-softbody = softbody.rs — 软体 / 弹簧-质点
form-mod-relativity = relativity.rs — 洛伦兹变换 / 时间膨胀
form-mod-transmission = transmission.rs — 信号传输 / 链路预算
form-mod-quantum = quantum.rs — 薛定谔方程 / 哈密顿量 / 自旋
form-mod-em = electromagnetism.rs — 麦克斯韦方程 / 坡印廷矢量
form-mod-nuclear = nuclear.rs — 衰变 / 半衰期 / 反应截面
form-mod-fluid = fluid.rs — Navier–Stokes / Euler / Bernoulli
form-mod-plasma = plasma.rs — 等离子体频率 / 磁流体
form-mod-superfluidity = superfluidity.rs — 超流氦 / 二流体模型
form-mod-continuum = continuum.rs — 连续介质力学
form-mod-physchem-title = 物理 + 化学
form-mod-physchem = physchem.rs — 反应动力学 / 平衡常数
form-mod-thermo = thermodynamics.rs — 熵 / 焓 / 热机效率
form-mod-molecular = molecular.rs — 理想气体 / 分子动力学
form-mod-wave-optics = wave_optics.rs — 干涉 / 衍射 / 偏振
form-mod-acoustics = acoustics.rs — 声学 / 多普勒
form-mod-aero = aerodynamics.rs — 升力 / 阻力 / 翼型
form-support-title = 辅助模块
form-support-intro = mps-formula 内部几个共享原语，被 107 个模块复用：
form-call-title = 从 Java 调用
form-call-desc = 所有公式函数都经 C ABI 暴露，无 WorldHandle 依赖：

# ---- Voxel page ----
vox-tag = // mps-core
vox-title = 体素系统
vox-desc = 稠密体素栅格 → 碰撞体构建 → 多面体引力桥接，全部位于 mps-core。
vox-overview-title = 概览
vox-overview-lead = VoxelGrid + build_voxel_collider 提供从体素地图到仿真的端到端管线。
vox-overview-body = 适用于月球着陆器表面表示、不规则地形高程图，以及需要快速近场碰撞的环绕飞行仿真。
vox-grid-title = VoxelGrid 数据模型
vox-grid-desc = rapier::voxel::VoxelGrid<'a> 持有 cells / dims / origin / scale 引用：
vox-grid-li-1 = cells: 八位或十六位位密度图（0=空，非 0=固体）
vox-grid-li-2 = dims + origin + scale：世界坐标系尺寸 / 原点 / 米每体素
vox-grid-li-3 = 借用语义：网格不拥有 cells，可直接来自 mmap 或 Java ByteBuffer
vox-build-title = build_voxel_collider
vox-build-lead = 入口函数将栅格转换为 Rapier 碰撞体并送入 World。
vox-build-note = 内部走 MarchingCubes + 凸分解：约 1 m³ → 1 个 convex hull，性能 O(N log N)。
vox-terrain-title = 地形引力桥接
vox-terrain-desc = 体素栅格可直接喂入 rapier::terrain_gravity 多面体引力：
vox-terrain-li-direct = terrain_gravity_direct — 暴力 O(N²) 顶点求和，验证基准。
vox-terrain-li-fft = terrain_gravity_fft — 频域卷积 O(N log N)，大型栅格首选。
vox-terrain-li-poly = polyhedron_gravity — Werner–Scheeres 多面体引力，源码级别 API。
vox-cases-title = 典型用例
vox-case-lunar-title = 月球表面着陆
vox-case-lunar-desc = 月球 100 m 网格 + 12 块 Mascon，仿真着陆器中途引力梯度扰动。
vox-case-terrain-title = 地形区域映射
vox-case-terrain-desc = DEM 高程 → VoxelGrid → 仿真器；与陆地飞行轨迹适配。
vox-case-proximity-title = 近场碰撞表现
vox-case-proximity-desc = 由于凸分解后碰撞体数量有限，n-body 结构体碰撞开销 O(N×log M)。

# ---- Events page ----
evt-tag = // mps-core
evt-title = 事件系统
evt-desc = 碰撞事件与接触力事件，三种派发模式，C 回调 ABI — 基于无锁 SPSC ring buffer。
evt-types-title = 事件类型
evt-types-lead = world_step 发出两类物理事件记录，经事件 ring 取出。
evt-col-type = 类型
evt-col-fields = 字段
evt-row-collision = started, collider1, collider2, sensor, removed
evt-row-contact = collider1, collider2, total_force, max_force_direction, max_force_magnitude
evt-modes-title = 派发模式
evt-mode-poll-title = Poll
evt-mode-poll-desc = 事件压入后台 Vec，应用层主动 drain / 查询。
evt-mode-callback-title = Callback
evt-mode-callback-desc = 直接调度 unsafe extern "C" 回调，实时但要在回调内避免重 Rust 调用。
evt-mode-both-title = Both
evt-mode-both-desc = 同时压入队列并触发回调，用于测试和热回放。
evt-ring-title = Ring Buffer
evt-ring-desc = 高频路径使用 SPSC EventRing<T>，无锁单生产者 / 单消费者。
evt-ring-li-1 = MAX_EVENT_RECORDS = 16 384 — 单 ring 上限。
evt-ring-li-2 = 生产者：world_step 内的 Rapier 回调线程。
evt-ring-li-3 = 消费者：Java 渲染帧线程，drain() 返回最新 N 条。
evt-forces-title = ForceLaw 派发
evt-forces-lead = rapier::forces::ForceRegistry 类型化注册，每个 step 聚合 ForceReport。
evt-force-coulomb = CoulombFrictionLaw — 动 / 静摩擦 + 速度阈值。
evt-force-airdrag = AirDragLaw — Reynolds 自适应：Stokes / Newton 流区切换。
evt-force-external = ExternalForceLaw — 浮力 + 电磁 + 弹簧 + 万有引力组合。
evt-force-newton = NewtonGravityLaw — body-body N-body 两两吸引，可缩放 G。
evt-force-custom = 自定义 ForceLaw trait — 任意力结构在 register() 后自动调度。
evt-forces-note = ForceReport 含每体合力 / 力矩 / 力类型标签，可拼接到 UI 调试覆盖。
evt-abi-title = C 回调 ABI
evt-abi-desc = 碰撞 / 接触力回调签名均以 unsafe extern "C" 暴露：

# ---- Arena page ----
arena-tag = // mps-core
arena-title = 共享内存 Arena
arena-desc = DirectByteBuffer — Java 与 Rust 间刚体状态零拷贝桥；每帧仅一次 world_step FFI，无逐体 JNI。
arena-why-title = 为什么需要 Arena
arena-why-lead = 经典 JNI 的每体 get/setField 在 1000 个体场景下吃掉每帧 10 ms+。
arena-why-body = 共享 Arena 把刚体 SoA 数据排布进一段直接字节缓冲区，Java 端在读写时不穿越 Rust 边界。每帧只发生一次 world_step FFI 调用。
arena-layout-title = 内存布局
arena-layout-desc = rapier::shared_arena 头部 + SoA 槽位数组 + 复用环。
arena-mods-title = 子模块
arena-mod-header = header.rs — magic / version / 容量校验。
arena-mod-layout = layout.rs — BodySlot 字段偏移常量。
arena-mod-ring = ring.rs — SPSC 事件环复用。
arena-mod-holes = holes.rs — 已删除槽位的 O(1) 回收。
arena-flow-title = 每帧协议
arena-flow-step-1 = Java 端：arena_write_* 写入本帧的目标位置 / 力 / 力矩。
arena-flow-step-2 = 单次 world_step(world, dt) FFI 触发仿真。
arena-flow-step-3 = arena_read_* 直接从 DirectByteBuffer 读取更新后的位置 / 速度。
arena-flow-note = 全程无 GetFieldID / CallObjectMethod；JNI 引用表为空，因此无 GC 风险。
arena-java-title = Java 侧示例
arena-java-desc = DirectByteBuffer + JNI 是 Java 21 的常规组合：

# ---- Cosmos page ----
cosmos-tag = // mps-cosmos
cosmos-title = 太空刚体演算
cosmos-desc = CosmosWorld — 独立的轨道尺度物理域，不与 mps-core 的 Rapier World 共享。
cosmos-what-title = 什么是 Cosmos
cosmos-what-lead = mps-cosmos 提供一个不共享 World 的轨道物理域。
cosmos-what-body = 与 mps-core 的 Rapier 步进不同，CosmosWorld 使用 Verlet 积分（verlet_step）与两两 n-body 互引力（n_body_acceleration_reduce），并配有 SIMD 远场单极路径（far_field_monopole_simd）应对大数量级。它专为长期轨道演化而生，无需刚体接触解算。
cosmos-mods-title = 子模块
cosmos-mod-kepler-title = kepler.rs
cosmos-mod-kepler-desc = 开普勒方程迭代 / 六根数 ↔ 笛卡尔转换。
cosmos-mod-dynamics-title = dynamics.rs
cosmos-mod-dynamics-desc = 双中心近似 / 第三体摄动 / 大气阻力。
cosmos-mod-perturbation-title = perturbation.rs
cosmos-mod-perturbation-desc = 潮汐力 / Yarkovsky / Poynting–Robertson。
cosmos-mod-propulsion-title = propulsion.rs
cosmos-mod-propulsion-desc = 齐奥尔科夫斯基方程 / 比冲 / 推进剂预算。
cosmos-mod-rotation-title = rotation.rs
cosmos-mod-rotation-desc = 四元数姿态动力学 / 自旋稳定。
cosmos-mod-thermal-title = thermal.rs
cosmos-mod-thermal-desc = 热平衡 / 太阳辐照 / 阴影进出循环。
cosmos-mod-debris-title = debris.rs
cosmos-mod-debris-desc = 碎片云演化 / Kessler 综合征模型。
cosmos-mod-gnss-title = gnss.rs
cosmos-mod-gnss-desc = L1/L5 伪距 / 多频消电离层解算。
cosmos-nbody-title = n-body 互引力
cosmos-nbody-lead = 每个 CosmosWorld 帧对全部天体两两求 GM·m/r²，O(N²)。
cosmos-nbody-note = N≤20 时直接两两法比 BH 树更快；这正是太阳系级仿真的规模。
cosmos-bodies-title = 天体目录 ({ $count })
cosmos-bodies-desc = JPL DE441 提供 10 个主要天体的 GM、轨道根数、相位，初始化时灌入 CosmosWorld。
cosmos-jni-title = JNI 集成
cosmos-jni-desc = mps-jni 暴露 cosmosWorld* 方法;批量状态读回经 cosmosWorldDynamicBodySnapshot / cosmosWorldDynamicBodySnapshotCount 走 Arena(无逐体 JNI)。
cosmos-arena-title = 共享内存 Arena（与 core 一致）
cosmos-arena-desc = Cosmos 自带 SharedArena（magic COSMAREN）—— 即 mps-core shared_arena 的轨道尺度孪生版。Java 通过 DirectByteBuffer 读写刚体状态，无需逐体 JNI；cosmosWorldGetArenaDirectByteBuffer 与 core 的 worldGetArenaDirectByteBuffer 平行。
cosmos-class-title = 功能分类
cosmos-class-lead = Cosmos 在轨道尺度复用了多个 mps-core 能力。以下每个类目把一项能力映射到其源码模块。
cosmos-class-world-title = World 与天体
cosmos-class-world-desc = world.rs / bodies.rs — CosmosWorld、中心体 + 太阳、10 天体 DE441 目录，以及批量插入（cosmos_world_add_n_body）。
cosmos-class-gravity-title = 引力
cosmos-class-gravity-desc = gravity.rs — 牛顿点质量与两两 n-body 互引力（n_body_acceleration_reduce），并配 SIMD 远场单极路径（far_field_monopole_simd）。
cosmos-class-integrator-title = 积分器
cosmos-class-integrator-desc = integrator.rs — verlet_step、n-body 加速度归约，以及高阶 + Kahan 补偿步进（advance_highorder_kahan）用于长弧段。
cosmos-class-orbit-title = 轨道与诊断
cosmos-class-orbit-desc = orbit.rs / orbit_diagnostics.rs — 六根数转换、Hill 半径（cosmos_hill_radius_for），以及用于监控漂移的状态快照。
cosmos-class-flight-title = 飞行与摄动
cosmos-class-flight-desc = flight/*（动力学、配平、稳定性）与 perturbation/* — 双中心动力学、第三体 / 光压 / 大气摄动，以及姿态配平。
cosmos-class-arena-title = 共享内存 Arena
cosmos-class-arena-desc = arena.rs + ffi.rs — SharedArena（COSMAREN），带 seqlock 保护的体槽；经 cosmos_world_get_shared_arena_address / _size 作为 DirectByteBuffer 暴露给 Java，与 core 的 arena 桥接平行。

# ---- Batch collider page ----
batch-tag = // Box3D
batch-title = 批量碰撞体管线
batch-desc = Box3D 风格批量插入 + 同材质合并 + 物理感预设，一次 ColliderSet::insert 摊销 N 个形状。
batch-pipeline-title = 管线流程
batch-pipeline-lead = 上层将 ColliderRequest 记录压入 ColliderBatch 管理器，合并兼容的静态形状为单个 compound，然后一次性插入。
batch-step-1-title = 构造请求
batch-step-1-desc = 填充 ColliderRequest 数组 — 形状、位姿、材质、碰撞组、父刚体。
batch-step-2-title = 选择预设
batch-step-2-desc = 传入 Box3DPreset 设定默认摩擦/弹性/密度/侵蚀/阻尼/CCD 步数/求解器迭代。
batch-step-3-title = 合并与插入
batch-step-3-desc = 同材质静态形状合并为单个 compound collider；不同材质或动态形状分组插入。
batch-step-4-title = 返回句柄
batch-step-4-desc = 返回每个生成的 collider 的 ColliderHandleRaw，上层可直接用于查询和后续操作。
batch-request-title = ColliderRequest 字段
batch-request-lead = 每条请求是一个 #[repr(C)] 扁平结构体，可构造连续数组传 (ptr, count) 给 FFI。
batch-col-field = 字段
batch-col-type = 类型
batch-col-desc = 说明
batch-col-scenario = 场景
batch-col-result = 结果
batch-field-shape = 形状描述符（shape_type + a/b/c/d 四个浮点）
batch-field-translation = 相对合并 collider 原点的局部平移
batch-field-rotation = 单位四元数局部旋转
batch-field-friction = 库仑摩擦系数（≥ 0）
batch-field-restitution = 恢复系数（≥ 0，通常 < 1）
batch-field-density = 质量密度（≥ 0，静态形状忽略）
batch-field-collision-groups = 碰撞组成员位掩码
batch-field-solver-groups = 求解器组成员位掩码
batch-field-body-parent = 非零时绑定到指定刚体
batch-field-is-sensor = 非零时为传感器（无碰撞响应）
batch-field-erosion-margin = 侵蚀裕度，仅对圆角形状有效；0 = 无侵蚀
batch-preset-title = Box3D 物理感预设
batch-preset-lead = 三种内置预设覆盖常见沙盒物理场景，也可通过 FFI 构造器获取。
batch-preset-default-title = Default
batch-preset-default-desc = 平衡型 — 中等摩擦、轻微弹性、适度阻尼。适合通用沙盒。
batch-preset-sticky-title = Sticky
batch-preset-sticky-desc = 不反弹、高摩擦。适合地面/墙壁等静态几何体。
batch-preset-bouncy-title = Bouncy
batch-preset-bouncy-desc = 低摩擦、高弹性、更多 CCD 子步。适合弹跳/堆积演示。
batch-merge-title = 合并策略
batch-merge-lead = 管理器按材质、碰撞组、传感器标志、父刚体分组，同组静态形状合并为 compound。
batch-merge-same-material = 同材质 + 同碰撞组 + 静态
batch-merge-compound = 合并为单个 compound collider（一次 insert）
batch-merge-diff-material = 不同材质或不同碰撞组
batch-merge-separate = 各自独立 collider（多次 insert）
batch-merge-dynamic-parent = 绑定到动态刚体
batch-merge-attach = 通过 insert_with_parent 附着到父刚体
batch-merge-sensor = 传感器标志为真
batch-merge-sensor-result = 传感器 collider 不参与碰撞响应，仅触发事件
batch-erosion-title = 侵蚀 (Erosion)
batch-erosion-lead = Rapier/parry 无内置 clone_eroded API，我们重建形状为圆角变体，border_radius = erosion_margin。
batch-erosion-cuboid = 将硬边长方体转为圆角长方体，堆积时减少抖动。
batch-erosion-cylinder = 圆柱体转圆角圆柱，边缘接触更平滑。
batch-erosion-cone = 圆锥转圆角圆锥，尖端钝化避免穿透。
batch-erosion-note = Ball / Capsule 等形状本身已圆滑，侵蚀不改变其几何。Ball 和不支持的形状回退到 shape_from_desc。
batch-ffi-title = FFI 入口
batch-ffi-lead = 全部 pub extern "C" fn，载荷为 #[repr(C)] 扁平结构体，cbindgen 生成 rigid_body.h 头文件。
batch-limits-title = 容量限制
batch-limit-max-requests = MAX_BATCH_REQUESTS = 100 000 — 单批次最大请求数。
batch-limit-max-compound = MAX_COMPOUND_PARTS = 50 000 — 单个 compound 最大部件数。
batch-limit-erosion-zero = erosion_margin = 0 时跳过圆角重建，直接使用原始形状。
batch-example-title = Rust 使用示例
batch-example-lead = 构造 ColliderRequest 数组，传入 batch_add_colliders，同材质自动合并为 compound。

# ---- JNI page ----
jni-tag = // mps-jni
jni-title = Java JNI 绑定
jni-desc = mps-jni 经 jni!/jni_e_c! 宏导出 { $methods } 个方法到 org.polaris2023.mps.rapier.RapierNative。
jni-codegen-title = 宏代码生成
jni-codegen-lead = 两套声明式宏把 Rust 闭包自动包成 JNI export_named 符号，类型表在 @ty / @default 内集中维护。
jni-codegen-body = jni! 用于不需要 JNIEnv 的普通方法；jni_e_c! 给需要 env / class 的回调安装类方法补充两个类型条目，并复用 jni! 的类型表（见 OPTIMIZATION.md §5.A）。
jni-codegen-note = 宏体把 closure 用 catch_unwind(AssertUnwindSafe) 包住，导出失败时回落到 @default 而不是让 JVM 进程崩。
jni-panic-title = panic 隔离
jni-panic-lead = 任何 Rust 端 panic 都被 catch_unwind 兜成 ERR_INTERNAL，并返回该返回类型的零值。
jni-panic-body = JVM 进程一旦 abort 几乎不可恢复，故每个 export fn 内嵌 panic 兜底；副作用是脏状态——调用方应把可疑调用夹在 world_step 之间并检查 last_error_code()。
jni-mangle-title = 符号重整（symbol mangling）
jni-mangle-desc = export_name 必须与 Java 类 FQN 严格对齐：'.' 转下划线，标识符内的 '_' 转成 '_1'。
jni-col-class = Java 类
jni-col-symbol = 导出符号前缀
jni-mangle-note = 漏写 _1 会让 System.load 成功但 dlsym 失败 → UnsatisfiedLinkError；RigidBodyNative 必须写 mps_1rigid_1body，RapierNative 不写 _1。
jni-groups-title = API 分组（{ $ffi } 个 extern C 入口）
jni-group-abi-title = ABI / 版本
jni-group-abi-desc = abiVersion / abiSupportsFfm / abiSupportsJni / last_error_code / last_error_message / clear。
jni-group-world-title = world_*
jni-group-world-desc = 创建 / 步进 / 销毁 / 重力 / 力律安装 / 事件注册。117 个 FFI 入口。
jni-group-rb-title = rigid_body_*
jni-group-rb-desc = builder 链 / 状态读写 / 锁定轴 / mass_properties。62 个 FFI 入口。
jni-group-collider-title = collider_*
jni-group-collider-desc = 形状构造 / 摩擦弹性 / 碰撞组 / 传感器。75 个 FFI 入口。
jni-group-query-title = query_*
jni-group-query-desc = ray_cast / shape_cast / point / intersection / R树剔除。58 个 FFI 入口。
jni-group-events-title = events / ForceLaw
jni-group-events-desc = 碰撞事件 + 接触力事件 ring buffer；Coulomb / AirDrag / Newton / 自定义力律安装。
jni-group-forces-title = 物理力律（C1–C4 扩展）
jni-group-forces-desc = 太阳风动压 / Eddington 光压 / X 射线辐照 / 脉冲星磁偶极力矩 / Jeans 逃逸 / MOND 引力。
jni-group-aero-title = 空气动力学 / 流体
jni-group-aero-desc = aero_apply_surfaces / aero_apply_voxel_grid / 流体 AABB 阻力与浮力。
jni-group-arena-title = Arena 零拷贝桥
jni-group-arena-desc = arenaAsDirectByteBuffer / arena_read_double / arena_write_double —— 绕过 JNI 往返读写物理状态。
jni-group-cosmos-title = cosmos_*
jni-group-cosmos-desc = CosmosWorld 创建 / 天体注册 / n-body 互引力 / Verlet 推进 / step_n 批量。
jni-group-spaceflight-title = spaceflight_*
jni-group-spaceflight-desc = 轨道摄动 / 比冲 / 推进剂预算 / 出加速结果到 native 缓冲（out_accel）。
jni-handle-title = 句柄打包
jni-handle-lead = RigidBodyHandle 折成单个 jlong：高 32 位存 index，低 32 位存 generation，对应 Rapier 的 into_raw_parts() 顺序。
jni-handle-note = 不拆成两个 jint 是为了与 RigidBodyHandleRaw 的 ABI（单 u64）对齐，避免 JNI 端两次读之间的 generation race。
jni-arena-title = 零拷贝 Arena 桥
jni-arena-lead = arenaAsDirectByteBuffer 用 NewDirectByteBuffer（Java 1.4 起的标准 JNI API）把原生 Arena 内存直接暴露成 java.nio.ByteBuffer。
jni-arena-body = Java 侧纯用 DoubleBuffer 读写，不再走 JNI upcall；per-frame 的 native→jdoubleArray 复制消失，热路径里几乎看不见 JNI 调度。
jni-deploy-title = 部署
jni-deploy-lib = cargo build --release -p mps-jni 产物 mps_rigid_body.dll 放进 src/main/resources/natives/。
jni-deploy-load = Java 端按架构 System.load("mps_rigid_body")，缺失时 UnsatisfiedLinkError 会带出错的符号名。
jni-deploy-version = ABI 通过 abiVersion() 商谈；当前 mps-web 自报 v{ $version }，运行时不匹配应由调用方主动 abort。
# ---- FFM page ----
ffm-tag = // MPS
ffm-title = Java FFM 绑定
ffm-desc = Foreign Function & Memory API（JEP 454）的 ABI 探测与版本协商入口，面向 Java 25+ 调用方。
ffm-what-title = FFM 在这里的角色
ffm-what-lead = mps-ffm 是个三函数的轻 crate：只把 ABI 版本号与两端能力位暴露出去，提供运行时协商。
ffm-what-body = 真正的 rigid_body.h 入口仍由 Java 端用 Linker.downcallHandle 直接 down-call；crates/mps-ffm 不复制任何方法签名。它的职责是让 Java 在加载 .dll/.so 的第一时间拿到 abi_version() 并判断是否支持 FFM / JNI 走哪条路径。
ffm-surface-title = ABI 表面
ffm-surface-lead = 三个 #[unsafe(no_mangle)] extern C 函数，#[repr(C)] Bool 为返回值。
ffm-surface-note = 不调用世界的纯 ABI 探测；Java 端 Linker 直接 downcall 这三个然后再切到对应的 world_create / world_step 等。
ffm-vs-title = JNI vs FFM 对比
ffm-col-feature = 特性
ffm-row-min-java = 最低 Java
ffm-row-binding = 绑定方式
ffm-row-jni-bind = Java 端 native 方法 + javah 生成头文件
ffm-row-ffm-bind = Linker.downcallHandle + FunctionDescriptor
ffm-row-overhead = 调用开销
ffm-row-jni-over = 高（每次调用经 JNIEnv）
ffm-row-ffm-over = 近原生（编译期 ABI 直链）
ffm-row-memory = 内存管理
ffm-row-jni-mem = GetXxxArrayElements + 释放
ffm-row-ffm-mem = MemorySegment 直接切片 Arena
ffm-row-panic = panic 兜底
ffm-row-jni-panic = catch_unwind 兜 ERR_INTERNAL
ffm-row-ffm-panic = 调用方必须自洽约 Rust 约定（未定义行为）
ffm-layout-title = Linker downcall 布局
ffm-layout-desc = Java 端按 C ABI 写 FunctionDescriptor 描述参数与返回类型，Linker 生成方法句柄，invokeExact 调用。
ffm-header-title = C ABI 输入
ffm-header-lead = Java 把 rigid_body.h 当作契约，自己用 memory layout 镜像 #[repr(C)] 结构体。
ffm-header-cbindgen = cbindgen 生成的 rigid_body.h 5400+ 行，它是两单一的事实依据。
ffm-header-structs = Vec3 / Quat / ShapeDesc / 事件记录等全部 #[repr(C)] 平铺，Java MemoryLayout 计算偏移。
ffm-header-load = Linker + SymbolLookup.loaderLookup() 加载 mps_rigid_body 没 JNIEnv 依赖。
ffm-header-note = cbindgen 改了任何字段，Java 端的 layout 描述必须同步更新；abi_version() 是版本守门人。
ffm-alloc-title = 分配策略
ffm-alloc-segment-title = MemorySegment
ffm-alloc-segment-desc = 每次调用临时申请一个 segment，arena 分配器统一关闭，生命周期明确。
ffm-alloc-arena-title = 共享 Arena
ffm-alloc-arena-desc = 大量状态读写走 mps-core 的 Arena，Java 直接拿到 DirectByteBuffer 封装，零拷贝。
ffm-alloc-shared-title = Foundation
ffm-alloc-shared-desc = JNI 与 FFM 两条路径共用 mps-core 的 Arena 状态，殊途同归。
ffm-status-title = 当前状态
ffm-status-body = mps-ffm 仍是能力探测 crate；JEP 454 的全量 downcall 绑定由 Java 端运行时构造，未占生成器路径。
# ---- API Reference page ----
api-tag = // MPS
api-title = API 参考
api-desc = crates/mps-core/include/rigid_body.h 公开 { $total } 个 pub extern C 入口。
api-header-title = 头文件表面
api-header-lead = cbindgen 0.29.4 由 mps-core build.rs 生成，{ $total } 个 FFI 函数 + 全部 #[repr(C)] 类型一并列出。
api-header-body = 严禁手改——任何改动来自 mps-core rapier 模块的 pub extern C，build 时自动重生成。
api-prefix-title = 函数前缀分组
api-prefix-lead = 导出函数名按物理子系统定前缀，grep 一下前缀就能列出该模块的全部入口。
api-col-prefix = 前缀
api-col-count = 数量
api-col-domain = 职责
api-row-world = World / 引力 / 力律 / 事件 / 步进
api-row-rigid = RigidBody 状态 / mass / 锁定
api-row-collider = 形状 / 摩擦 / 碰撞组 / 传感器
api-row-query = Ray cast / shape cast / 点投 / 缝击
api-prefix-note = 合计与 CORE_FFI_COUNT 一致；未列出前缀（aero_ / fluid_ / trajectory_ / anvilkit_ / cosmos_ / molecular_ / events / force law）走各域小节。
api-handles-title = 常用句柄类型
api-col-type = 类型
api-col-scope = 作用域
api-handle-world = 物理世界，生命周期由 cargo 创建到 world_destroy。
api-handle-rigid = 刚体索引+代，u64 打包，跨调用复用。
api-handle-collider = 碰撞体索引+代，与 parent rigid body 可能解绑。
api-handle-rb-build = builder 链，insert_with_parent 后所有权转移成功。
api-handle-col-build = 碰撞体 builder，build_insert 后所有权转移给 world。
api-handle-joint = 关节 builder，world_add_impulse_joint 成功后失效。
api-handle-rtree = 批量剔除 R 树，用于 collider/query 的 broadphase。
api-handle-crbtree = 关节 / 碰撞对的 C-side 红黑树，用于 O(log n) 查询。
api-handle-cc = 角色控制器句柄，封装 capsule scan + 接触解算。
api-records-title = 扁平记录类型
api-records-lead = 全部 #[repr(C)] 平铺，Java/JNI/FFM 三端兼容写入。
api-record-vec3 = Vec3 — 三轴 f64，按 x/y/z 顺序。
api-record-quat = Quat — (i, j, k, w) 四元数，builder 会转轴角。
api-record-aabb = AabbDesc — min/max + 用户标签，查询范围用。
api-record-shape = ShapeDesc — shape_type + a/b/c/d 参数，覆盖球 / 长方体 / 胶囊 / 椎体等。
api-record-event = CollisionEventRecord / ContactForceEventRecord — 事件 ring buffer 载荷。
api-record-filter = QueryFilterDesc / InteractionGroupsDesc — 查询过滤位掩码。
api-error-title = 错误报告
api-error-lead = 线程局部错误码 + 消息，函数返回值一定要先检查。
api-error-note = 同一帧后续调用覆盖前次错误；读完最后立刻 last_error_clear() 才能判定下一次失败。
api-lifecycle-title = World 生命周期调用样例
api-stability-title = ABI 稳定性约定
api-stability-cbindgen = rigid_body.h 由 cbindgen 自动生成，跳过人手编辑是硬错。
api-stability-repr = 全部公共结构体强制 #[repr(C)]，字段顺序、对齐、填充固定。
api-stability-version = abi_version() 是强制谈判入口，运行时不匹配调用方主动 abort。
api-stability-redline = 公式模块只能暴露 crate-internal pub fn；只有 pub extern C fn 入 rigid_body.h。valsharkin C ABI 的红线由 cargo build -p mps-core + git diff header 守护。
# ---- 404 page ----
not-found-title = 页面未找到
not-found-desc = 您访问的页面不存在。请返回首页。
not-found-back = 返回首页

# ---- Footer ----
footer-text = MPS Motion Physics System v{ $version } — GitHub
nav-group-overview = 概览
nav-group-core = mps-core
nav-group-cosmos = mps-cosmos
nav-group-formula = mps-formula
nav-group-jni = mps-jni
nav-group-ffm = mps-ffm
# ---- 软体(Phase 0–21) ----
soft-tag = 软体
soft-title = 软体物理
soft-desc = XPBD / MassSpring 可变形体 —— 布料、四面体体积网格、体素地形,Phase 0–21 的 22 项能力升级,外加一条零 fork 的 FFI 安全线(Phase 22–25):接触力回读、单粒子冲量、AABB 回读、深拷贝、二进制状态序列化、逐粒子速度写入。

soft-overview-title = 概述
soft-overview-lead = 软体是一组质点,由距离约束(XPBD)和/或弹簧(MassSpring)连接,可外裹三角壳或四面体体积网格。
soft-overview-body = 每个软体拥有独立的重力场、休眠/唤醒状态,以及一组 XPBD 距离约束加 MassSpring 弹簧。求解器经 soft_body_configure_solver 按体切换 —— XPBD 用于刚性结构布料/肉体,MassSpring 用于弹性绳索/果冻。全部状态经 Arena 可读写,Java 以零逐对象 JNI 读取质点/四面体/三角/边。

soft-solver-title = 求解器
soft-solver-desc = 两套求解器共用同一质点缓冲;用 soft_body_configure_solver(world, id, solver, iterations, dt) 切换。
soft-solver-li-1 = XPBD —— 刚性柔度距离约束,逐约束设定 compliance 与压缩;配合四面体体积守恒实现不可压肉体。
soft-solver-li-2 = MassSpring —— 胡克弹簧(soft_body_add_spring),逐弹簧刚度;绳索、布料、凝胶便宜又稳定。
soft-solver-li-3 = 逐约束各向异性柔度(soft_body_set_distance_constraint_compliance)让一条边沿轴向抗拉伸不同,实现定向刚度。

soft-data-title = 数据模型
soft-data-desc = 软体是四个并行数组加两组约束;全部可由 Arena 读取。
soft-data-li-1 = 质点 —— soft_body_add_particle(pos, inv_mass, pinned);经 soft_body_read_particles 读取。
soft-data-li-2 = 四面体 —— soft_body_add_tetrahedron(a,b,c,d) 构体积网格;静止体积缓存用于守恒;经 soft_body_read_tetrahedra 读取。
soft-data-li-3 = 三角 —— soft_body_add_triangle(a,b,c) 构外壳;经 soft_body_read_triangles 读取。
soft-data-li-4 = 边 —— 弹簧与距离约束;经 soft_body_read_edges 读取。

soft-cap-title = 能力矩阵(Phase 0–21)
soft-cap-lead = 每张卡片对应工作区中一个真实的 soft_body_* FFI。Phase 22–25 另加一条零 fork 的 FFI 安全线(见下)。

soft-cap-01-title = 基础体与质点
soft-cap-01-desc = soft_body_create + soft_body_add_particle;自由或钉住的质点带独立 inv_mass。后续所有功能的地基。
soft-cap-02-title = 三角外壳
soft-cap-02-desc = soft_body_add_triangle 把 3 条结构边登记为距离约束;外壳驱动布料与表面接触。
soft-cap-03-title = 四面体体积网格
soft-cap-03-desc = soft_body_add_tetrahedron + soft_body_build_tetra_mesh 构不可压体积体;静止体积缓存用于体积守恒。
soft-cap-04-title = 弹簧(MassSpring)
soft-cap-04-desc = soft_body_add_spring + soft_body_set_spring_stiffness —— 胡克链接,用于绳索、布料、凝胶,刚度可调。
soft-cap-05-title = 距离约束
soft-cap-05-desc = soft_body_add_distance_constraint 带逐约束 compliance 与压缩;XPBD 结构骨架。
soft-cap-06-title = 布料与弯曲
soft-cap-06-desc = soft_body_add_bending 在三角壳之上加角度弯曲抗力,用于硬挺布料与可折叠表面。
soft-cap-07-title = 风力场
soft-cap-07-desc = soft_body_apply_wind + soft_body_clear_wind —— 用三角法线对外壳做气动阻力;标志位经 soft_body_apply_wind_flag。
soft-cap-08-title = 休眠诊断
soft-cap-08-desc = soft_body_is_sleeping / soft_body_sleep / soft_body_wake —— 持久岛屿休眠态让静止体退出求解。
soft-cap-09-title = 锚定刚体
soft-cap-09-desc = soft_body_attach_particle / soft_body_detach_particle 把质点绑到刚体(固定点、绳端、钉住的布角)。
soft-cap-10-title = 撕裂
soft-cap-10-desc = soft_body_set_tear_strain —— 距离约束应变超阈值即断裂,布料在载荷下撕裂。
soft-cap-11-title = 塑性
soft-cap-11-desc = soft_body_set_plasticity —— 约束保留一部分形变为永久偏移,软体凹陷并维持凹陷。
soft-cap-12-title = 充气
soft-cap-12-desc = soft_body_set_pressure —— 内部压力把四面体网格吹胀(气球、气囊、膀胱)。
soft-cap-13-title = 自碰撞
soft-cap-13-desc = soft_body_set_self_collision —— 质点在半径内互斥;防止折叠体穿透自身。
soft-cap-14-title = 软软碰撞
soft-cap-14-desc = soft_body_set_cross_collision —— 两个软体互相解算接触(堆叠、挤压、互压)。
soft-cap-15-title = 独立重力
soft-cap-15-desc = soft_body_set_gravity —— 每个体带自己的重力向量,与世界给刚体的重力解耦。
soft-cap-16-title = 体积守恒
soft-cap-16-desc = soft_body_set_volume_conservation + soft_body_total_volume —— XPBD 约束维持四面体总体积(不可压肉体、水球)。
soft-cap-17-title = 黏连
soft-cap-17-desc = soft_body_set_cohesion —— 捕获半径内近邻质点相互吸引(表面张力、黏滴、湿沙)。
soft-cap-18-title = 结构阻尼
soft-cap-18-desc = soft_body_set_damping —— 速度比例阻尼抑制抖动,让体趋于静止。
soft-cap-19-title = 各向异性柔度
soft-cap-19-desc = soft_body_set_distance_constraint_compliance —— 逐边定向柔度,用于硬经软纬布料与定向肉体。
soft-cap-20-title = 软软摩擦
soft-cap-20-desc = soft_body_set_self_collision_friction + soft_body_set_cross_collision_friction —— 自碰撞与跨体接触上的库伦切向阻尼(μ ∈ [0,1])。
soft-cap-21-title = 自适应四面体细分
soft-cap-21-desc = soft_body_subdivide_tetrahedra —— 最长边超阈值的四面体做重心 1→4 细分;子体积之和恰为父体积,体积守恒。
soft-cap-22-title = 读写 API
soft-cap-22-desc = soft_body_read_particles / _read_tetrahedra / _read_triangles / _read_edges + soft_body_get_particle —— 完整体状态经零拷贝 Arena 流动。

soft-p25-title = FFI 安全线(Phase 22–25)
soft-p25-lead = 六个纯 mps-core 增量 —— 每个都直接遍历 SoftBody 的 pub 字段,无一改动 rapier3d fork。暴露状态回读、克隆、二进制(反)序列化与直接速度写入,支撑存档/读档、回放、联网软体快照。
soft-p25-1-title = 接触力回读
soft-p25-1-desc = soft_body_read_contact_force —— 每个质点来自碰撞代理的合接触冲量,按击中哪个碰撞体拆分。只读诊断,用于抓握/挤压力。
soft-p25-2-title = 单粒子冲量
soft-p25-2-desc = soft_body_apply_particle_impulse —— p.vel += J·inv_mass;钉住(inv_mass==0)为空操作。踢单个节点而不重建体。
soft-p25-3-title = AABB / 质心回读
soft-p25-3-desc = soft_body_read_aabb —— 由质点位置算最小/最大角与质心;任一输出指针可传 null 跳过。
soft-p25-4-title = 深拷贝
soft-p25-4-desc = soft_body_clone —— SoftBody::clone 到新 id 且 collide=false,副本独立自积分、绝不共享源代理。
soft-p25-5-title = 二进制状态存/取
soft-p25-5-desc = soft_body_state_size + soft_body_save_state + soft_body_restore_state —— 手写小端字节块覆盖全部 pub 字段(Option/enum/RigidBodyHandle 经 into_raw_parts 打包)。坏 magic/版本/截断返回 FALSE 且不残留半体。
soft-p25-6-title = 逐粒子速度写入
soft-p25-6-desc = soft_body_set_particle_velocity —— 覆盖 particle.vel;钉住/越界/未知 id 返回 FALSE。是 soft_body_get_particle 的写入对偶。

soft-p25-map-title = Phase 25 FFI ↔ JNI(零 fork,仅 mps-core)
soft-p25-map-note = 每个 C FFI 与 Java JNI 方法一一对应。八个都直接遍历 SoftBody 的 pub 字段,绝不改动 rapier3d fork。返回/守卫列给出成功类型与失败路径。
soft-p25-map-body = FFI                                      JNI                               ret / guard
  soft_body_read_contact_force        softBodyReadContactForce        u32 count / 坏 id -> 0
  soft_body_apply_particle_impulse     softBodyApplyParticleImpulse     bool / 钉住跳过,坏 id -> false
  soft_body_read_aabb                  softBodyReadAabb                 bool / 输出指针可 null
  soft_body_clone                      softBodyClone                    u32 新 id / 失败 -> u32::MAX
  soft_body_state_size                 softBodyStateSize                u32 字节数 / 失败 -> u32::MAX
  soft_body_save_state                 softBodySaveState                u32 写入数 / 缓冲过小 -> u32::MAX
  soft_body_restore_state              softBodyRestoreState             u32 新 id / 坏 magic -> u32::MAX
  soft_body_set_particle_velocity      softBodySetParticleVelocity      bool / 钉住|越界|坏 id -> false

soft-api-title = FFI 接口面
soft-api-desc = 软体子系统在 C FFI、Java JNI、集成测试三方对称暴露。
soft-api-stat-ffi = C FFI 函数
soft-api-stat-jni = JNI 方法
soft-api-stat-tests = 集成测试

# ---- Cosmos sub-pages (Plan D split) ----
cosmos-land-title = 特性分页
cosmos-land-lead = Cosmos 拆成六个能力分页 —— 点进去看每个分类背后的真实函数与 FFI。
nav-cosmos-world = 世界与天体
nav-cosmos-gravity = 重力与 n 体
nav-cosmos-integrator = 积分器
nav-cosmos-orbit = 轨道与诊断
nav-cosmos-flight = 飞行与摄动
nav-cosmos-arena = Arena 与 JNI
cw-tag = // mps-cosmos
cw-title = 世界与天体
cw-desc = CosmosWorld —— 独立的轨道尺度世界,与 mps-core 的 Rapier World 分离。
cw-overview-title = 概述
cw-overview-lead = 一个 CosmosWorld 拥有自己的积分域:中心天体、太阳,以及天体编目。
cw-overview-body = 与 mps-core 不同,这里没有刚体接触求解器 —— CosmosWorld 仅靠重力 + 积分器推进天体。天体可逐个插入(cosmos_world_insert_body)或作为引力源插入(cosmos_world_insert_body_as_gravity_source),太阳/中心天体经 cosmos_world_set_sun_position / cosmos_world_set_central_body 配置。
cw-bodies-title = 天体编目
cw-bodies-desc = cosmos_world_add_celestial 从 JPL DE441 数据集注入天体(GM、根数、相位)。10 个主天体在初始化时登记,用户天体再叠加其上。
cw-batch-title = 批量插入
cw-batch-desc = cosmos_world_add_n_body 一次提交多个航天器状态,而非逐体;对应 cosmosWorldDynamicBodySnapshot 批量读回模式,适合追踪队列负载。
cw-ffi-title = C FFI 接口面
cw-ffi-desc = 世界层暴露 create / build / insert / step 生命周期。
cw-ffi-1 = cosmos_world_create / cosmos_world_destroy —— 拥有轨道世界。
cw-ffi-2 = cosmos_satellite_builder / cosmos_fixed_body_builder —— 航天器或固定(天体)体。
cw-ffi-3 = cosmos_builder_set_gravity_scale / _set_linear_damping / _set_angular_damping / _lock_translations —— 体参数。
cw-ffi-4 = cosmos_world_insert_body / cosmos_world_insert_body_as_gravity_source —— 加入世界。
cw-ffi-5 = cosmos_world_set_central_body / cosmos_world_set_sun_position / cosmos_world_set_perturbation —— 域配置。
cw-ffi-6 = cosmos_world_step / cosmos_world_step_n —— 推进一帧或 N 帧。
cg-tag = // mps-cosmos
cg-title = 重力与 n 体
cg-desc = 牛顿点质量重力、两两 n 体互引力,以及 SIMD 远场单极路径。
cg-overview-title = 概述
cg-overview-lead = 每一帧计算所有天体的 GM·m/r² —— CosmosWorld 的标志性特征。
cg-overview-body = gravity.rs 建模三种情形:单点质量加速度(point_mass_acceleration)、完整两两互和(n_body_acceleration / n_body_acceleration_reduce,O(N²)),以及 SIMD 远场单极近似(far_field_monopole_simd)用于高数量级。近/远场切换由 near_field_threshold、monopole、irregular 控制。
cg-fn-title = 函数
cg-fn-desc = gravity.rs 中的纯函数;天体在 step 中调用它们。
cg-fn-1 = point_mass_acceleration / celestial_acceleration —— 来自单个引力源的加速度。
cg-fn-2 = n_body_acceleration / n_body_acceleration_reduce —— 所有天体两两互引力。
cg-fn-3 = far_field_monopole_simd —— 远场 SIMD 单极求和。
cg-fn-4 = gm_from_mass —— 由质量经引力常数得 GM。
cg-fn-5 = monopole / irregular / near_field_threshold —— 切换近场与远场处理。
cg-ffi-title = 引力源 FFI
cg-ffi-desc = cosmos_world_insert_body_as_gravity_source 让一个体吸引其他体而不被积分;cosmos_hill_radius_for(轨道页)给出影响球半径。
ci-tag = // mps-cosmos
ci-title = 积分器
ci-desc = Verlet 步进,外加高阶与 Kahan 补偿步进器,用于长弧推进。
ci-overview-title = 概述
ci-overview-lead = CosmosWorld 默认用速度 Verlet 积分,另有高阶选项提升精度。
ci-overview-body = integrator.rs 以 verlet_step 为基线,explicit_highorder_step / advance_highorder 提供高阶精度。advance_highorder_kahan / explicit_highorder_kahan_step 加入 Kahan 求和补偿,使长时间弧段(数年仿真)不因浮点舍入漂移。total_acceleration 把重力 + 摄动合成一个向量;snapshot_source_positions 缓存引力源位置供远场使用。
ci-fn-title = 函数
ci-fn-desc = integrator.rs 中的步进器族。
ci-fn-1 = verlet_step —— 基线速度 Verlet 推进。
ci-fn-2 = explicit_highorder_step / advance_highorder —— 高阶积分。
ci-fn-3 = explicit_highorder_kahan_step / advance_highorder_kahan —— Kahan 补偿,长弧低漂移。
ci-fn-4 = total_acceleration —— 重力 + 摄动求和。
ci-fn-5 = snapshot_source_positions —— 缓存引力源位置供远场。
ci-toggle-title = 并行与 SIMD 开关
ci-toggle-desc = nb_parallel_enabled 将两两求和切到 rayon 并行路径;ff_simd_enabled 开启 SIMD 远场单极。两者在太阳系尺度默认开启。
co-tag = // mps-cosmos
co-title = 轨道与诊断
co-desc = 六根数转换、Hill 半径、平均运动、偏心率向量、Kozai 周期,以及状态快照。
co-overview-title = 概述
co-overview-lead = orbit.rs / orbit_diagnostics.rs 把状态向量与经典根数互转,并暴露监控漂移的诊断量。
co-overview-body = 六个开普勒根数经 orbit.rs 与笛卡尔坐标互转。orbit_diagnostics.rs 增加 mean_motion、mean_motion_ratio、eccentricity_vector,以及 kozai_period(Kozai–Lidov 周期)。cosmos_hill_radius_for 给出一个体的 Hill 影响球半径。状态快照(cosmos_world_dynamic_body_snapshot / _count)让 Java 读取每个体的位置/速度,无需逐体 JNI。
co-fn-title = 函数
co-fn-desc = 根数与诊断辅助函数。
co-fn-1 = 六根数 ↔ 笛卡尔互转(orbit.rs)。
co-fn-2 = cosmos_hill_radius_for —— 一个体的 Hill 球半径。
co-fn-3 = mean_motion / mean_motion_ratio —— 轨道角速率与共振比。
co-fn-4 = eccentricity_vector —— 轨道形状/朝向。
co-fn-5 = kozai_period —— Kozai–Lidov 振荡周期。
co-snap-title = 状态快照
co-snap-desc = cosmos_world_dynamic_body_snapshot_count + cosmos_world_dynamic_body_snapshot 经 Arena 批量导出每个动态体的状态,用于漂移监控与绘图。
cf-tag = // mps-cosmos
cf-title = 飞行与摄动
cf-desc = 双中心动力学、配平、纵向稳定性,以及环境摄动。
cf-dyn-title = 飞行动力学
cf-dyn-lead = flight/dynamics.rs —— 双中心近似、第三体摄动与阻力。
cf-dyn-desc = total_forces_and_moments 与 simulate_one_step 积分航天器状态;from_body / linvel_body 转换坐标系;flat_plate_area / default_airfoil 给出升力面尺寸。经 valid 校验输入。
cf-trim-title = 配平
cf-trim-desc = flight/trim.rs —— hover_target / level_flight_target 定义期望平衡;trim 求解对应操纵面。
cf-stab-title = 稳定性
cf-stab-desc = flight/stability.rs —— linearize 构建状态矩阵;longitudinal_modes / longitudinal_submatrix 暴露短周期/长周期模态;power_iteration 求主特征值。
cf-pert-title = 摄动
cf-pert-desc = perturbation/* —— 大气阻力与太阳光压作为力(复用 mps_formula::spaceflight),在每步前注入。
ca-tag = // mps-cosmos
ca-title = Arena 与 JNI
ca-desc = Cosmos 自带 SharedArena(COSMAREN)—— mps-core shared_arena 的轨道尺度孪生体。
ca-overview-title = 概述
ca-overview-lead = Java 经 DirectByteBuffer 读写天体状态,无需逐体 JNI 调用。
ca-overview-body = arena.rs 持有一个 seqlock 守护的体槽;ffi.rs 暴露地址与大小,Java 将其映射为 DirectByteBuffer。这镜像 mps-core 的 arena 桥,但按轨道天体尺寸定制。cosmos_world_get_shared_arena_address / _size 平行 worldGetArenaDirectByteBuffer。
ca-ffi-title = Arena FFI
ca-ffi-desc = 创建 / 销毁 / 查询共享 Arena。
ca-ffi-1 = cosmos_world_create_shared_arena / cosmos_world_destroy_shared_arena —— 拥有 Arena。
ca-ffi-2 = cosmos_world_get_shared_arena_address / cosmos_world_get_shared_arena_size —— 映射到 Java。
ca-ffi-3 = cosmos_world_dynamic_body_snapshot(_count) —— 经 Arena 批量天体状态。
ca-jni-title = JNI 批量 API
ca-jni-desc = mps-jni 暴露 cosmosWorld* 方法;批量状态读回经 cosmosWorldDynamicBodySnapshot / cosmosWorldDynamicBodySnapshotCount 走 Arena(无逐体 JNI)。

# ---- Cosmos sub-page FFI<->JNI maps ----
cw-map-title = FFI <-> JNI  (src: 实现模块)
cw-map-note = C FFI(snake_case)与 Java JNI(camelCase)一一对应;builder 插入后所有权转移给世界。src 列标注 mps-cosmos 中的真实实现模块。
cw-map-body = FFI                                   JNI                          src
  cosmos_world_create                  cosmosWorldCreate            world.rs
  cosmos_world_destroy                 cosmosWorldDestroy           world.rs
  cosmos_satellite_builder            cosmosSatelliteBuilder       bodies.rs
  cosmos_fixed_body_builder           cosmosFixedBodyBuilder       bodies.rs
  cosmos_builder_set_gravity_scale     cosmosBuilderSetGravityScale bodies.rs
  cosmos_builder_set_linear_damping    cosmosBuilderSetLinearDamping bodies.rs
  cosmos_builder_set_angular_damping   cosmosBuilderSetAngularDamping bodies.rs
  cosmos_builder_lock_translations     cosmosBuilderLockTranslations bodies.rs
  cosmos_world_insert_body             cosmosWorldInsertBody        world.rs
  cosmos_world_insert_body_as_gravity_source cosmosWorldInsertBodyAsGravitySource world.rs + gravity.rs
  cosmos_world_set_central_body        cosmosWorldSetCentralBody    world.rs
  cosmos_world_set_sun_position        cosmosWorldSetSunPosition    world.rs
  cosmos_world_set_perturbation        cosmosWorldSetPerturbation   world.rs + perturbation/
  cosmos_world_step                    cosmosWorldStep              world.rs -> integrator.rs
  cosmos_world_step_n                  cosmosWorldStepN             world.rs -> integrator.rs
  cosmos_world_add_celestial          cosmosWorldAddCelestial       world.rs + gravity.rs
  cosmos_world_add_n_body              cosmosWorldAddNBody          world.rs + gravity.rs
cg-map-title = FFI <-> JNI  (src: 实现模块)
cg-map-note = 本层唯一 FFI 是引力源注册;加速度函数在 cosmosWorldStep 内部调用。src 列标注其所在模块。
cg-map-body = FFI                                                       JNI  src
  cosmos_world_insert_body_as_gravity_source  cosmosWorldInsertBodyAsGravitySource  world.rs -> gravity.rs
  # point_mass_acceleration / n_body_acceleration_reduce / far_field_monopole_simd
  #   在 cosmosWorldStep 内部调用 —— 实现于 gravity.rs + integrator.rs,无独立 FFI/JNI。
ci-map-title = FFI <-> JNI  (src: 实现模块)
ci-map-note = 本层唯一 FFI 是步进;步进器族由 orbit_integration / verlet_substeps 内部选择。src 列标注其所在模块。
ci-map-body = FFI                          JNI                 src
  cosmos_world_step    cosmosWorldStep    world.rs -> integrator.rs
  cosmos_world_step_n  cosmosWorldStepN  world.rs -> integrator.rs
  # verlet_step / explicit_highorder_step / advance_highorder_kahan
  #   位于 integrator.rs,按 orbit_integration + verlet_substeps 选择,无独立 JNI。
co-map-title = FFI <-> JNI  (src: 实现模块)
co-map-note = Hill 半径仅 FFI(内部诊断,world.rs -> orbit_diagnostics.rs);快照是 JNI 批量路径(ffi.rs + arena.rs 布局)。
co-map-body = FFI                                                   JNI  src
  cosmos_hill_radius_for  (仅 FFI)  world.rs -> orbit_diagnostics.rs
  cosmos_world_dynamic_body_snapshot        cosmosWorldDynamicBodySnapshot       ffi.rs + arena.rs
  cosmos_world_dynamic_body_snapshot_count  cosmosWorldDynamicBodySnapshotCount  ffi.rs + arena.rs
  # mean_motion / eccentricity_vector / kozai_period 为 orbit.rs / orbit_diagnostics.rs 纯函数,
  # 经快照 / Arena 读回。
cf-map-title = FFI <-> JNI  (src: 实现模块)
cf-map-note = flight/* 与 perturbation/* 仅计算;在 cosmosWorldStep 前注入,无独立 FFI/JNI。src 列标注其所在模块。
cf-map-body = 模块                         函数                                   src
  flight/dynamics   总力与力矩 total_forces_and_moments / simulate_one_step  flight/dynamics.rs
  flight/trim      trim(hover_target / level_flight_target)          flight/trim.rs
  flight/stability linearize / longitudinal_modes / power_iteration  flight/stability.rs
  perturbation     大气阻力 atmospheric_drag_force / 光压              perturbation/
  # 均在 cosmosWorldStep 前注入;结果经 cosmosWorldDynamicBodySnapshot 经 Arena(arena.rs)读回。
ca-map-title = FFI <-> JNI  (src: 实现模块)
ca-map-note = Arena 把地址/大小作为 DirectByteBuffer 暴露给 Java;快照是批量读回路径(arena.rs)。
ca-map-body = FFI                                              JNI                              src
  cosmos_world_create_shared_arena     cosmosWorldCreateSharedArena     world.rs -> arena.rs
  cosmos_world_destroy_shared_arena    cosmosWorldDestroySharedArena    world.rs -> arena.rs
  cosmos_world_get_shared_arena_address cosmosWorldGetSharedArenaAddress arena.rs
  cosmos_world_get_shared_arena_size     cosmosWorldGetSharedArenaSize     arena.rs
  cosmos_world_dynamic_body_snapshot        cosmosWorldDynamicBodySnapshot       ffi.rs + arena.rs
  cosmos_world_dynamic_body_snapshot_count  cosmosWorldDynamicBodySnapshotCount  ffi.rs + arena.rs
