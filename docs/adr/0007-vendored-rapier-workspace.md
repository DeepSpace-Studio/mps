# 0007 — vendored Rapier 保持独立 workspace

## 背景

`rapier/` 是本地的 rapier3d-f64 checkout，自带 `[workspace.package]` 和自己的依赖。把它吸进根 workspace 会让上游锁文件、示例和 testbed 变成 mps 的成员。

## 决定

根 workspace `exclude = ["rapier", "docs"]`。`rapier3d-f64` 只通过 path 依赖进入 `mps-core` 与 `mps-cosmos`，并继续解析它自己的 workspace。

## 后果

在 `rapier/` 里改代码要用它自己的 workspace 命令，或者明确用 `--manifest-path`。不要把 rapier 的示例 crate 加进根 `members`。同步上游是一次单独的变更，不夹在 mps 功能提交里。

## 相关代码

- 根 `Cargo.toml` 的 `exclude` 与 `rapier3d` path
