# 文档站计数

## 主张

`crates/mps-web/src/metrics.rs` 里的测试数、JNI 入口数、core `extern "C"` 数、版本号和若干分项，与源码上重新数出来的一致。

## 在哪

- 生成器：`crates/xtask/src/main.rs` 的 `dump-metrics`
- 生成结果：`crates/mps-web/src/metrics.rs`（文件头写明计数口径）
- 守门：`crates/mps-test/src/rapier/verify_metrics_sync.rs`

当前口径（以生成文件头为准，若实现改了以代码为准）：

- `TEST_COUNT`：`mps-test` 里 `#[test]` 的个数
- `JNI_METHOD_COUNT`：`mps-jni/src/lib.rs` 里 `jni!(` / `jni_e_c!(` 的个数
- `CORE_FFI_COUNT`：`mps-core/src/rapier` 里 `pub extern "C"` 的个数
- `VERSION`：根 `Cargo.toml` 的 workspace `version`

## 怎么复现

```powershell
cargo run -p xtask -- dump-metrics
cargo test -p mps-test verify_metrics
```

先生成再测。手改 `metrics.rs` 会在下一次生成时被覆盖，守门也会在数字对不上时失败。
