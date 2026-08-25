# MPS Web 文档站点 — 规划（当前真实状态）

> 注意：本规划曾写 "Topcoat" —— 那已是过时信息。当前 `crates/mps-web` 是
> **Dioxus 0.7 + dioxus-i18n 0.5 (Fluent)** 全栈文档站（fullstack + router + server），
> 不是 Topcoat。下文为 2026-08-20 重做后的实际格局。

## 技术栈（真实）

- **Dioxus 0.7** fullstack：`dioxus = { features = ["fullstack", "router", "server"] }`
- **dioxus-i18n 0.5** + **fluent 0.16**：中英双语，Fluent `.ftl` 资源（`src/i18n/locales/{zh-CN,en}.ftl`）
- SSR（服务端渲染），导航全部用 `<a href>` / `Link`（纯 SSR、无需客户端 hydration JS）
- 本站**不依赖 `dx` CLI** 即可运行：`build.rs` 把 `public/` 拷到二进制旁，`cargo run -p mps-web`
  起 SSR 服务（默认 8080）。

## 为什么之前"用不了"

2026-08-20 排查发现根因：**`mps-web` 整个 crate 编译失败**（6 个 unresolved import）。
`metrics.rs` 由 `xtask dump-metrics` 生成，但当时只导出 `TEST_COUNT/JNI_METHOD_COUNT/CORE_FFI_COUNT`
三个常量，而各页面引用了 `VERSION`、`FFI_WORLD/RIGID_BODY/COLLIDER/QUERY`、
`FORMULA_MODULE_COUNT/CELESTIAL_COUNT/GRAVITY_MODEL_COUNT/INTEGRATOR_COUNT` 等更多符号
→ 编译挂掉 → 任何页面都跑不起来。

修复（根因，非补丁）：扩展 `dump-metrics` 用真实 grep 算出这些常量并一并生成 `metrics.rs`
（保持 "metrics.rs 由 xtask 生成、不手改" 的约定）。重新生成后全站编译通过。

## 站点结构（当前：单页内嵌，无分页路由）

```
crates/mps-web/
├── Cargo.toml
├── build.rs                 # 把 public/ 拷到二进制旁（SSR 资源定位）
├── src/
│   ├── lib.rs               # 入口：dioxus::launch(Home)，**无 Route 枚举 / 无 Router**
│   ├── main.rs              # 转发 mps_web::main()
│   ├── metrics.rs           # xtask 生成
│   ├── i18n.rs              # i18n 初始化 + langs
│   ├── layouts/
│   │   ├── mod.rs           # Toc（顶部锚点导航）+ Footer；**无 Layout/Outlet**
│   │   └── site.css         # 深空科幻主题（克制版）
│   └── pages/               # 14 个页面组件，全部内嵌进 Home
│       ├── home.rs          # 聚合页：Toc + 14 个 <Section> + Footer，内联渲染
│       ├── quickstart.rs … api.rs (13 个)  每个导出 `pub fn X() -> Element`
│       │                    返回 `section { id:"sec-x", class:"doc-section", … }`
└── public/index.html
```

**无分页**：所有章节在一个 HTML 文档内（约 78KB）。导航是顶部 TOC 的
`#sec-xxx` 原生锚点（浏览器原生滚动，**零 JS、无 hydration 拦截**），
彻底消除之前 "菜单栏点击不跳转"（Dioxus `Link` 在前端拦截器因 SSR-only 无 wasm 客户端而死链）的问题。

## 设计（2026-08-20 第三版：单页 + 克制科幻风）

用户反馈前两版"样式太丑"且"分页经常出问题"。本次方向：
1. **砍掉分页**：不要用 router / 多页。整站 = 一个内联页面，TOC 用 `#sec-`
   锚点做页内跳转（原生滚动，绝不会因为 hydration 缺失而点不动）。
2. **克制科幻风**：深空渐变背景 + 安静的星点层（慢漂移动画），顶部 sticky
   TOC 药丸链接，宽松留白，玻璃拟态卡片，柔和的青/紫霓虹（不再刺眼 glow）。
   所有正文类（`.page-head`/`.section-card`/`.module-card`/`.callout`/`.hero`/
   表格/代码块）统一换肤，18 个章节即时获得新外观。
3. **中文优先**：i18n 中文为默认回退；语言切换用 `?lang=` 查询参数。

### 为什么这样稳
- 纯 SSR（build.rs 只拷 `public/index.html`，无 wasm 客户端 bundle，本机无 `dx`）。
- 导航 = 原生 `<a href="#sec-x">`，点击 = 浏览器整页锚点滚动，不依赖任何客户端 JS。
- 无 Dioxus `Link` / `Router` / `Outlet`，因此不存在点击拦截器吞导航的问题。

## 设计（2026-08-20 重做：换格局 + 科幻宇宙风）

旧设计是一套脆弱的 **CSS 轨道星系导航**（首页渲染太阳 + 14 颗环绕行星，纯 CSS 旋转；
其余页面用底部 sheet 模态）。该导航有 z-index 点击穿透 bug，且整站因编译失败根本跑不起来。

新设计（"一切按你的来"）：

- **左侧固定 "星座" 导航栏**：玻璃拟态（backdrop-blur）、霓虹青/紫描边、激活项左侧辉光。
  导航按 `导航 / 物理引擎 / 宇宙 / 绑定` 分组，全部 `<a href>`/`Link`（SSR 安全、无 JS）。
- **深空动态星空背景**：`body` 渐变 + `starfield-bg` 固定层（CSS radial-gradient 星点 + 缓慢漂移动画）。
- **玻璃拟态内容卡片**：`.section-card` / `.module-card` / `.metric-card` 半透明 + 模糊 + 辉光阴影。
- **霓虹遥测风**：等宽字体指标数字带青色 glow；按钮渐变 + 外发光。
- **响应式**：≤880px 时侧栏收起为顶部栏，纯 CSS checkbox 切换（无 JS）。
- **所有 18 页正文零改动**：仅通过重定义 CSS 类（`.page-head`/`.section-card`/`.module-card`/`.callout`/`.hero`/表格/代码块等）整体换皮 —— 全站即时获得新科幻皮肤。
- **首页**：移除原轨道星系块（导航职责移交给侧栏），保留 hero + 指标卡 + 模块目录 + 公式分类 + 特性 + 架构图。

## 验证（2026-08-20）

- `cargo build -p mps-web` 通过（dev + 强制重编）。
- 起 SSR 服务后 `curl /` 与 `curl /api` 均 HTTP 200、返回非空 HTML；
  `/` 含 `mps-sidebar`/`mps-nav-link`/`starfield-bg`/`RIGID BODY`；
  `/api` 含 FFI 前缀真实计数（world 117 / rigid_body 62 / collider 75 / query 58）。
- 运行日志无 error/panic。

## 未做（范围外，按约定只动 Rust 侧）

- 未改 Java / 绑定生成器（与 mps-web 无关）。
- 未重写各页面正文文案（仅换皮）；如需逐页内容升级可另开任务。
