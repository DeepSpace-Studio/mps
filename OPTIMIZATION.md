# OPTIMIZATION.md — 优化工程章节索引

> **来源说明**：本文件的原始版本不在本仓库（也不在 git 历史）中，但仓库内 28 个文件、60 余处代码注释、守门测试与构建脚本按章节号引用它（`OPTIMIZATION.md §N`）。本文件是对这些引用的**还原**：逐条列出每个章节号在代码里被引用时所"治理"的规则、以及对应的守门测试/构建文件。原始正文的完整叙述（背景推导、备选方案讨论）已不可考；本文件以可验证的代码事实为准。新增引用时请同步维护本表。

## 正文章节

### §1 — `ERR_*` 错误码跨 crate 一致性
`ERR_OK`…`ERR_INTERNAL` 七个错误码在 `mps-formula::error` 与 `mps_core::rapier::error` **独立声明**（见 DESIGN.md §3.2 的 cbindgen 规避理由），两侧数值必须一致。
守门：`crates/mps-test/src/rapier/error_consistency.rs`（集中式相等断言，失败时给出双侧定义位置；禁止静默该测试）。

### §2 — `mps-formula` FFI 类型拆分
原 2464 行的 `mps-formula/src/ffi/types.rs` 拆分为 `ffi/types/` 目录（`core.rs`、`physics.rs` 等按域分文件）。
现状：`crates/mps-formula/src/ffi/types/mod.rs` 头注释标注 "Auto-split from original `ffi/types.rs` (2464 lines)"。

### §3 — `mps-core` 空间飞行模块拆分
原 2610 行的 `mps-core/src/rapier/spaceflight.rs` 拆为 8 个 per-domain 子模块（debris/dynamics/gnss/kepler/perturbation/propulsion/rotation/thermal），`mod.rs` 集中共用数值辅助与 FFI 类型 re-export。
现状：`crates/mps-core/src/rapier/spaceflight/mod.rs`；文档见 [docs/mps-core.md](docs/mps-core.md) 空间飞行子模块一节。

### §4 — 共享竞技场事件环（ring.rs）的无锁读风险
`shared_arena/ring.rs` 的 SPSC 环绕缓冲区中，事件读取路径存在与 `event_read` 相关的内存序敏感点；任何对该文件的改动都应当用 Miri 重新验证 SPSC 测试。
守门：`crates/mps-core/src/rapier/shared_arena/ring.rs` 头注释的 Miri 复验要求。

### §5.A — `jni!` / `jni_e_c!` 宏类型表同步
`mps-jni` 中 `jni_e_c!` 复用 `jni!` 的 `@ty`/`@default` 类型表，仅追加 `env`/`class` 两项——两张表必须保持 lockstep，新增类型变体时两处同改。
现状：`crates/mps-jni/src/lib.rs` 宏定义处注释。

### §7 — cbindgen 头文件生成委托
`mps-core` 与 `mps-cosmos` 的 `build.rs` 都把头文件生成委托给 `mps-build-common::run_cbindgen()`（消除两份重复的 cbindgen 配置/调用逻辑）。
现状：`crates/mps-build-common/src/lib.rs`、`crates/mps-core/build.rs`、`crates/mps-cosmos/build.rs`。

### §8 — 测试镜像约定（mps-test ↔ 源 crate）
每当 `mps-core::rapier::*`（及 cosmos）增删/改名子模块，`mps-test/src/<层级>/<name>.rs` 必须同步——测试与源码布局一一镜像，防止"加了模块忘了测试"的静默漏覆盖。此前只是手工约定，`verify_module_mirror` 把它变成 CI 可见断言（仅靠目录清单计算，不做解析）。
守门：`crates/mps-test/src/rapier/verify_module_mirror.rs`；规则原文见 DESIGN.md（模块镜像规则）。

### §10 — `shared_arena` ABI 常量锁定
`ARENA_VERSION` 等共享竞技场 ABI 常量一旦发布即冻结：测试 pin 住具体数值，改动常量必须显式升级（同时 `mps-ffm::ABI_VERSION` 须与 `ARENA_VERSION` 同步步进，Java 侧据此抛 `IllegalStateException` 防呆）。
守门：`crates/mps-test/src/rapier/arena_compat.rs`（ABI 锁定测试，禁止静默）。

### §13 — dead-code 保留理由
`shared_arena` ABI 冻结伴生的 dead_code 项（为保持 ABI 完整而保留的旧常量/字段）的豁免理由集中说明（与 §10 一起被 `arena_compat.rs` 引用）。

## §N 系列（后补条款）

### §N1 — 测试侧镜像拆分（方案 A）
当源侧子模块按 §2/§3/§N8 拆分为目录时，测试侧的对应单文件也建议拆分为同名子目录，保持镜像形状对称。
守门：`verify_module_mirror.rs` 在镜像形状不对称时提示"考虑按 §N1 方案 A 同步拆分"。

### §N3 — `metrics.rs` 生成纪律
`crates/mps-web/src/metrics.rs` 由 `cargo run -p xtask -- dump-metrics` 生成，**禁止手改**；新增/删除 test、JNI 方法或 core FFI 后必须重新生成并提交，否则文档站展示的计数会过期。
守门：`crates/mps-test/src/rapier/verify_metrics_sync.rs`（比对 metrics.rs 与源码实测计数；要求提交生成文件，禁止静默）；生成器：`crates/xtask/src/main.rs`。

### §N6 — 跨 crate 版本常量一致性
`ARENA_VERSION` ↔ `mps-ffm::ABI_VERSION` ↔ workspace `version` 等版本常量必须同步步进；同一"当前值"以一处为规范源，其余（含 `cfg(test)` 下的镜像声明）交叉校验。
守门：`crates/mps-test/src/rapier/version_consistency.rs`（跨 crate 版本锁定测试）。

### §N8 — `mps-formula` 空间飞行模块拆分
原 2040 行的 `mps-formula/src/spaceflight.rs` 拆为 8 个 per-domain 子模块（与 §3 的 mps-core 侧一一镜像），`mod.rs` 集中共用辅助。
现状：`crates/mps-formula/src/spaceflight/mod.rs` 及 8 个子文件头注释 "Split out of the original 2040-line `spaceflight.rs` per OPTIMIZATION.md §N8"。
