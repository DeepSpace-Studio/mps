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
home-mod-formula-desc = 28 模块 — 航天、天体物理、核物理、相对论、量子等
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
quickstart-step4-desc = cargo test --workspace 执行 685 项集成测试。
quickstart-step5-title = 生成 C 头文件
quickstart-step5-desc = cargo build -p mps-core 触发 cbindgen 生成 rigid_body.h。

# ---- 404 page ----
not-found-title = 页面未找到
not-found-desc = 您访问的页面不存在。请返回首页。
not-found-back = 返回首页

# ---- Footer ----
footer-text = MPS Motion Physics System v{ $version } — GitHub
