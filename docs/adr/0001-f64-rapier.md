# 0001 — 模拟用 f64 Rapier

## 背景

长时轨道、航天和多体模拟里，单精度刚体状态会漂。Rapier 上游同时提供 f32 与 f64 crate。

## 决定

物理后端固定为 `rapier3d-f64`，依赖名 `rapier3d`，path 指向 `rapier/crates/rapier3d-f64`，并打开 `parallel`。不设 f32 feature，不在 ABI 上提供第二套世界。

## 后果

内存和一部分算子更贵。这是默认成本，不是可选模式。公式层同样用 f64；checked API 的溢出政策见 [formula-numeric-domains.md](../formula-numeric-domains.md)。

## 相关代码

- 根 `Cargo.toml` 的 `[workspace.dependencies] rapier3d`
- `crates/mps-core`、`crates/mps-cosmos` 的 `rapier3d` 依赖
