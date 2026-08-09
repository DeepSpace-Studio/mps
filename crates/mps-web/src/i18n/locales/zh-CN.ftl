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
nav-events = 事件
nav-arena = Arena
nav-batch = 批量碰撞体
nav-cosmos = 太空
nav-jni = JNI
nav-ffm = FFM
nav-api = API

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
home-mod-formula-desc = 33 模块 — 航天、天体物理、核物理、相对论、量子等
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
  └─ Rust C ABI (483 functions)
       ├─ mps-formula  — 33 pure formula modules (300+ functions)
       ├─ mps-core     — physics engine + Rapier wrapper (World, bodies, colliders, queries, events)
       ├─ mps-cosmos   — cosmos rigid body (separate world, Verlet orbit integration)
       ├─ mps-jni      — JNI bindings (311 methods, incl. cosmos batch)
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
grav-tag = // MPS
grav-title = 引力模型
grav-desc = 从点质量到月球 Mascon 的完整引力模型清单。
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
int-tag = // MPS
int-title = 辛积分器
int-desc = Leapfrog、Yoshida 4、Forest–Ruth 8 阶辛积分器。
int-why-title = 为何用辛积分器
int-why-lead = 经典 RK4 在长时间轨道演化中能量漂移。
int-why-body = 辛积分器保留哈密顿量结构，能量误差有界且周期性振荡，适合万年级轨道仿真。
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
form-tag = // MPS
form-title = 公式模块
form-desc = 33 个纯 Rust 公式模块，覆盖航天到量子物理。
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
form-support-intro = mps-formula 内部几个共享原语，被 33 个模块复用：
form-call-title = 从 Java 调用
form-call-desc = 所有公式函数都经 C ABI 暴露，无 WorldHandle 依赖：

# ---- Voxel page ----
vox-tag = // MPS
vox-title = 体素系统
vox-desc = 稠密体素栅格 → 碰撞体构建 → 多面体引力桥接。
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
evt-tag = // MPS
evt-title = 事件系统
evt-desc = 碰撞 + 接触力事件，三种派发模式，C 回调 ABI。
evt-types-title = 事件类型
evt-types-lead = 内置两类物理事件，均记录于 mps-formula 的 ffi/types/core.rs。
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
arena-tag = // MPS
arena-title = 共享内存 Arena
arena-desc = DirectByteBuffer — Java 与 Rust 间刚体状态零拷贝桥。
arena-why-title = 为什么需要 Arena
arena-why-lead = 经典 JNI 的每体 get/setField 在 1000 个体场景下吃掉每帧 10 ms+。
arena-why-body = 共享内存 Arena 把刚体 SoA 数据放进一段直接字节缓冲区，Java 端读写不进 Rust 边界。每帧仅触发一次 world_step FFI 调用。
arena-layout-title = 内存布局
arena-layout-desc = rapier::shared_arena 头部 + SoA 槽位数组 + 复用环。
arena-mods-title = 子模块
arena-mod-header = header.rs — magic / version / 容量校验。
arena-mod-layout = layout.rs — BodySlot 字段偏移常量。
arena-mod-ring = ring.rs — SPSC 事件环复用。
arena-mod-holes = holes.rs — 已删除槽位的 O(1) 回收。
arena-flow-title = 每帧协议
arena-flow-step-1 = Java 端 arena_write_* 写入本帧的目标位置 / 力 / 力矩。
arena-flow-step-2 = world_step(world, dt) 单次 FFI 触发仿真。
arena-flow-step-3 = arena_read_* 读取更新后的位置 / 速度 — 直接从 DirectByteBuffer。
arena-flow-note = 全程无 GetFieldID / CallObjectMethod；JNI 引用表为空，无 GC 触发风险。
arena-java-title = Java 侧示例
arena-java-desc = DirectByteBuffer + JNI 是 Java 21 的常规组合：

# ---- Cosmos page ----
cosmos-tag = // MPS
cosmos-title = 太空刚体演算
cosmos-desc = CosmosWorld — 独立轨道尺度物理域。
cosmos-what-title = 什么是 Cosmos
cosmos-what-lead = mps-cosmos 提供一个不共享 World 的轨道物理域。
cosmos-what-body = 与 mps-core 的 Rapier 不同，CosmosWorld 使用 Verlet 辛积分器和 n-body 两两互引力，适合长期轨道演化而无需刚体接触解算。
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
cosmos-jni-desc = mps-jni 暴露 cosmos_batch_* 方法：一次性提交多个飞船状态而非逐体调用，对 tracking 队列友好。

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

# ---- 404 page ----
not-found-title = 页面未找到
not-found-desc = 您访问的页面不存在。请返回首页。
not-found-back = 返回首页

# ---- Footer ----
footer-text = MPS Motion Physics System v{ $version } — GitHub
