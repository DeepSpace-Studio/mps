# 0006 — 生成物不手改

## 背景

C 头文件和文档站计数如果手写，会和 Rust 源码漂移，而且漂移要到宿主或网页上才看得见。

## 决定

- `rigid_body.h` 与 `cosmos.h` 只由 `mps-build-common::run_cbindgen` 在 `build.rs` 里生成。
- `crates/mps-web/src/metrics.rs` 只由 `cargo run -p xtask -- dump-metrics` 生成。
- `#[java_struct]` / `#[java_enum]` 的 Java 源只由 `cargo run -p xtask -- gen-java` 生成。
- 生成文件提交进 git。CI 在 Linux 上检查 `rigid_body.h` 与 `cosmos.h` 无 diff。metrics 由 `verify_metrics_sync` 对源码重数。

## 后果

改导出符号或测试数量的变更，必须带上重新生成的文件。冲突时以重新生成为准，不以某一侧的手改为准。Linux CI 在 `cargo build --release` 之后对两份头文件执行 `git diff --exit-code`。Windows 不跑这步，避免 `autocrlf` 把换行报成差异。

## 相关代码

- `crates/mps-build-common/src/lib.rs`
- `crates/mps-core/build.rs`、`crates/mps-cosmos/build.rs`
- `crates/xtask/src/main.rs`
- `crates/mps-test/src/rapier/verify_metrics_sync.rs`
- `.github/workflows/ci.yml` 的 “Check generated C headers are up to date”
