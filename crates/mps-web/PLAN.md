# MPS Web 文档站点 — 规划

## 概述

使用 **Topcoat**（Rust 全栈 Web 框架）在 `crates/mps-web/` 下构建 MPS Motion Physics System 的文档站点。
Topcoat 使用 Rust 的 `view!` 宏编写 HTML 模板，`#[page]` 定义路由，所有页面代码用 Rust 编写。

## 技术栈

- **Topcoat** = Rust 全栈框架（server-rendered，支持 `view!` 模板宏、`#[component]` 组件、`#[layout]` 布局、`#[page]` 路由）
- 纯 Rust 代码，无需手写 HTML/CSS/JS 文件
- 通过 GitHub Actions 构建并部署到 GitHub Pages

## 站点结构（Rust 模块）

```
crates/mps-web/src/
├── lib.rs              # 入口：创建 Router，启动服务
├── pages/
│   ├── mod.rs          # 模块路由导出
│   ├── home.rs         # 首页 — 项目介绍、核心数据
│   ├── quickstart.rs   # 快速入门教程
│   ├── architecture.rs # 架构概述
│   ├── gravity.rs      # 引力模型
│   ├── integrators.rs  # 辛积分器
│   ├── formula.rs      # 公式模块 (28 modules)
│   ├── voxel.rs        # Voxel 碰撞体
│   ├── events.rs       # 事件系统
│   ├── arena.rs        # 共享内存 Arena
│   ├── jni.rs          # Java JNI 绑定
│   ├── ffm.rs          # Java FFM 绑定
│   └── api.rs          # API 参考
├── components/
│   ├── mod.rs
│   ├── header.rs       # 导航栏组件
│   ├── footer.rs       # 页脚组件
│   ├── metric_card.rs  # 指标卡片组件
│   ├── stat_grid.rs    # 统计网格组件
│   ├── module_card.rs  # 模块卡片组件
│   ├── doc_module.rs   # 文档模块容器
│   └── layout.rs       # 全局布局（含语言切换）
└── layouts/
    └── mod.rs          # 根布局（HTML 骨架、导航、页脚）
```

## 设计原则

1. 使用 Topcoat 的 `view!` 宏和 `#[component]` 组件构建所有页面
2. 深色主题，与物理引擎的 "太空/天文" 调性一致
3. 中英双语内容，通过 JS 信号切换
4. 组件化：导航栏、页脚、卡片等均封装为可复用组件

## 执行批次

### Batch 1: 基础框架 + 首页
- 配置 `Cargo.toml` 依赖（topcoat, tokio）
- 创建 `lib.rs` 入口 + 路由
- 创建 `layouts/` 根布局（HTML 骨架、导航、页脚）
- 创建 `components/` 基础组件（header, footer, metric_card, module_card 等）
- 创建 `pages/home.rs` 首页

### Batch 2: 核心功能页面
- 创建 `pages/architecture.rs`（架构概述）
- 创建 `pages/quickstart.rs`（快速入门）
- 创建 `pages/gravity.rs`（引力模型）
- 创建 `pages/integrators.rs`（辛积分器）

### Batch 3: 公式与高级功能页面
- 创建 `pages/formula.rs`（28 公式模块）
- 创建 `pages/voxel.rs`（Voxel 碰撞体）
- 创建 `pages/events.rs`（事件系统）

### Batch 4: 集成与 API 页面
- 创建 `pages/jni.rs`（Java JNI）
- 创建 `pages/ffm.rs`（Java FFM）
- 创建 `pages/arena.rs`（共享内存 Arena）
- 创建 `pages/api.rs`（API 参考）

### Batch 5: 部署配置
- 创建 `Cargo.toml` 完整配置
- 创建 `index.html` 占位（用于 GitHub Pages 入口）
- 创建 GitHub Actions 部署 workflow
- 更新 workspace `Cargo.toml` 确认配置
- `cargo check` 验证编译

## 内容来源

从 `README.md` 和现有 `docs/` 目录提取内容，转为 Topcoat `view!` 宏语法。