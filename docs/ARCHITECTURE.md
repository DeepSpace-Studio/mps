# 架构

本文描述仓库里**已经存在**的结构。决定的理由在 [adr/](adr/README.md)；不许跨越的线在 [BOUNDARY.md](BOUNDARY.md)。

## 仓库里有两个 workspace

根 `Cargo.toml` 的 members 是 `crates/` 下 10 个包：9 个 `mps-*` 加上不发布的 `xtask`。`exclude = ["rapier", "docs"]`：`rapier/` 是上游 `rapier3d-f64` 的 vendored fork，自带 workspace；`docs/` 不是 crate。`rapier3d` 这个依赖名指向 `rapier/crates/rapier3d-f64`，并打开 `parallel`。

```text
Java 宿主
  └─ mps_rigid_body   crates/mps-jni        cdylib，库名固定
       ├─ mps-core    通用物理世界 + C ABI   include/rigid_body.h
       ├─ mps-cosmos  太空世界 + C ABI        include/cosmos.h
       └─ mps-ffm     abi_version / abi_supports_*
            │
            ├─ mps-formula   纯函数，无 Rapier
            └─ rapier3d-f64  只被 mps-core 与 mps-cosmos 直接依赖

mps-web  ──只依赖──▶ mps-formula（文档站，不进模拟）
mps-test ──依赖──▶ formula + core + cosmos + ffm + rapier3d
```

`mps-cosmos` 不依赖 `mps-core`。两边各自持有 `RigidBodySet` / `PhysicsPipeline`。公式可以共享，世界不能共享。

## 三层职责

### 公式 — `mps-formula`

`rlib`。公开模块见 `crates/mps-formula/src/lib.rs`：轨道与航天、流体、连续介质、电磁、量子、相对论、核、等离子体、天体物理，以及 `scientists/`、`disciplines/` 两套按人和按学科拆开的条目。

两条返回路径：

- 已检查的纯函数返回 `Result<T, FormulaError>`（`domain.rs`）。输入必须有限；正值、非负、对数、开方、除零各自显式。这条路径**不**写线程局部错误槽。
- `ffi` 模块把同一批计算包成 C ABI。`formula_result` 把域错误映射成 `ERR_INVALID_ARGUMENT`，指针错误仍是 `ERR_NULL_POINTER`。标量失败可以返回 NaN；报告结构失败返回 false，且不改调用者的输出缓冲。

### 通用世界 — `mps-core`

`cdylib` + `rlib`。入口 `lib.rs` 再导出 `rapier`。`rapier/mod.rs` 里一部分是本 crate 的模拟模块，一部分是 `pub use mps_formula::...` 把公式再导出给上层。

世界本体是 `PhysicsWorld`（`world.rs`）：Rapier 的 pipeline、刚体集、碰撞体集、关节、积分参数、事件、力律登记、共享 arena。步进之后，力复位循环顺手刷新刚体中心空间索引里已变动的叶子。

体不是只有刚体。当前各自有模块和 FFI 前缀的包括：

| 模块 | 是什么 |
| --- | --- |
| `rigid_body` | 刚体：创建、位姿、速度、力、冲量、CCD、睡眠 |
| `soft_body` | 软体：骨骼链与质点体 |
| `cloth` / `rope` / `balloon` / `hair` / `rope_knot` | 布料、缆绳、气囊、毛发、绳结 |
| `fluid` / `fluid_sph` | 流体力，以及 SPH 流体体 |
| `granular` | 颗粒 DEM |
| `articulation` | 铰接体 |
| `character_body` | 运动学角色体 |
| `sensor` | 传感器重叠区 |
| `vehicle` / `tire_model` | 射线悬架车与轮胎 |
| `servo_body` | PD/PID 伺服体 |
| `fracture` / `fracture_mesh` | 断裂与可碎复合刚体 |

碰撞与查询在 `collider`、`query`、`voxel`、`bounds`、`dop`、`neural`。空间索引有紧凑树（`crbtree`）、R 树（`rtree`）和世界私有的刚体中心 BVH（`body_spatial_index`，不是碰撞体 BVH）。

`spaceflight/` 是把公式施到**这个世界里的刚体**上的一层（开普勒、摄动、推进、姿态、热控、GNSS、碎片、相对运动）。纯公式在 `mps-formula/src/spaceflight/`，两边按域拆开，但不是同一个类型系统。

### 太空世界 — `mps-cosmos`

`rlib`。`CosmosWorld` 自己拿着一套 Rapier 后端，加上天体引力源、n-body 质点和可选的环境扰动力。`step` 在每个物理子步之前把「天体引力 + 互引力 + 扰动力」累加成力，再交给 `PhysicsPipeline::step`。

它没有 `mps-core` 的共享 arena、力律登记表和事件钩子。C 符号前缀是 `cosmos_*`。`COSMOS_PROFILE=1` 时 `explicit_substep` 打四段耗时；不设置时这条路径零开销。

飞行子模块在 `flight/`（动力学、配平、稳定性），轨道诊断在 `orbit_diagnostics`，无线电在 `radio`。

## 过 ABI 的东西

| 种类 | 表示 | 谁持有 |
| --- | --- | --- |
| 世界、builder | 不透明 `*mut` | Rust 分配，调用者用成对的 destroy |
| 刚体、碰撞体、关节 | packed `u64` | Rapier arena 的一代一代句柄压进整数 |
| 错误 | 返回值里的哨兵 + 线程局部 `last_error_*` | 槽在 `mps-formula::error`，core 再导出访问器 |
| 大块状态 | 共享 arena / force queue 的 DirectByteBuffer | 布局由 arena 常量钉死 |

`mps-jni` 用 `jni!` / `jni_e_c!` 把上述符号接成 JNI，并再套一层 `catch_unwind`。`mps-ffm` 只导出三个探针：`abi_version`、`abi_supports_ffm`、`abi_supports_jni`。Java 侧源码不在本仓库。

头文件由 `build.rs` → `mps-build-common::run_cbindgen` 写出：

- `crates/mps-core/include/rigid_body.h`
- `crates/mps-cosmos/include/cosmos.h`

cbindgen 只解析当前 crate，所以要出现在头文件里的常量必须在该 crate 内写成字面量，不能 `pub use`。

## 并行与 profile

`mps-core` 的 `parallel` 用 Rayon 做逐体力、成对力和快照。`rapier3d` 依赖已开 `parallel`。

根 profile：dev 与 release 都是 `panic = "unwind"`。release 另有 `lto = "thin"`、`codegen-units = 1`、`opt-level = 3`、`strip = true`。`panic = "abort"` 会让 `ffi_guard` 失效，宿主进程会被 panic 杀掉。

## 测试与文档站怎么挂上架构

`mps-test` 的目录镜像源码：`crates/mps-test/src/rapier/` 对 `mps-core/src/rapier/` 与 `mps-formula/src/` 的并集，`src/cosmos/` 对 `mps-cosmos/src/`。守门测试也住在这里，不是单独的 CI 脚本。

`mps-web` 是 Dioxus 0.7 全栈 SSR，单页锚点，不走客户端路由。页面上的计数来自 xtask 生成的 `metrics.rs`，生成规则在 [evidence/metrics.md](evidence/metrics.md)。

## 源文件导览

`mps-core` 每个文件的作用写在 [mps-core.md](mps-core.md)，链到 `mps-core/*.md`。那是导览，不是第二份架构说明。
