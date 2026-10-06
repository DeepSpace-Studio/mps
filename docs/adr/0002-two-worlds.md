# 0002 — 两个世界，公式无世界

## 背景

同一套公式既要被通用刚体场景调用，也要被太空演练调用。把太空逻辑塞进 `PhysicsWorld` 会让共享 arena、力律表和事件钩子变成太空路径的负担；把 Rapier 类型放进公式又让公式无法单独测试。

## 决定

- `mps-formula` 只有纯函数和自己的 C ABI。没有 `WorldHandle`，没有 Rapier 依赖。
- `mps-core::PhysicsWorld` 是通用世界：体家族、碰撞、查询、事件、力律、共享 arena。
- `mps-cosmos::CosmosWorld` 自持一套 Rapier 后端。它依赖 `mps-formula` 与 `rapier3d`，不依赖 `mps-core`。
- 两边可以调用同一公式，不可以共用世界、arena 或力律登记表。

## 后果

引力、摄动等若要「施到刚体上」，必须在 core 或 cosmos 各写一层薄封装。看起来重复的是副作用，不是数学。合并两个世界需要新的 ADR，而不是顺手挪模块。

## 相关代码

- `crates/mps-formula/Cargo.toml`（无 rapier）
- `crates/mps-cosmos/src/lib.rs`、`crates/mps-cosmos/src/world.rs`
- `crates/mps-core/src/rapier/world.rs`
