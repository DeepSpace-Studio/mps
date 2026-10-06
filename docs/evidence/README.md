# 证据

每条记录一件**已经被仓库钉住**的事：哪段代码、哪条命令、失败时会怎样。不记录「应该再测什么」。

数字会过期。计数以 `crates/mps-web/src/metrics.rs` 为准，那份文件由 xtask 生成；本目录不复制那些数字。

| 记录 | 钉住了什么 |
| --- | --- |
| [error-codes.md](error-codes.md) | 七个 `ERR_*` 双侧相等 |
| [module-mirror.md](module-mirror.md) | 测试目录与源模块一致 |
| [metrics.md](metrics.md) | 文档站计数与源码一致 |
| [abi-versions.md](abi-versions.md) | arena ABI 与 FFM 版本一起动 |
| [world-step.md](world-step.md) | 步进性能矩阵的命令和 CSV |
| [numeric-domains.md](numeric-domains.md) | 已迁移的 checked 公式 |

CI 本身（fmt、clippy、`cargo test`、release 构建、Linux 上的 `rigid_body.h` 与 `cosmos.h` diff、`web_audit.py`）记在 `.github/workflows/ci.yml`，不在这里重复步骤。
