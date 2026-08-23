# AGENTS.md — mps_rigid_body

`mps_rigid_body` 是一个 Rust workspace,把 `rapier3d-f64`(f64 精度)封装成单一原生 `cdylib`,供外部项目通过 JNI(`mps-jni`)和 Foreign Function & Memory API(`mps-ffm`)调用。同时附带一个纯 Rust 物理/航天公式库(`mps-formula`,28 个模块)和一个 Dioxus 文档站(`mps-web`)。Workspace = 9 个 crate,edition 2024,版本 0.1.4。

## Crates

- `mps-formula` — 纯计算,28 个模块,无 Rapier/WorldHandle 依赖。`rlib`。
- `mps-core` — 物理世界 + Rapier 封装 + C ABI(`src/rapier/ffi/`)。`cdylib`+`rlib`。通过 cbindgen 生成 `include/rigid_body.h`(`build.rs` → `mps-build-common`)。每个源文件的作用分析见 [docs/mps-core.md](docs/mps-core.md)(逐一链接到 [docs/mps-core/](docs/mps-core/))。
- `mps-cosmos` — 在 `mps-formula` 之上的轨道/飞行动力学。`rlib`。通过 cbindgen 生成 `include/cosmos.h`。
- `mps-jni` — JNI 绑定;lib 名 `mps_rigid_body`。`cdylib`+`rlib`。Java 加载的就是这个。
- `mps-ffm` — Java 25 FFM 元数据/类型。
- `mps-test` — 所有集成测试(718 个 `#[test]`)都在这里,不在源 crate 中。
- `mps-web` — Dioxus 0.7 文档站。`crates/mps-web/src/metrics.rs` 由 xtask 生成(见下)。
- `mps-build-common` — `mps-core` + `mps-cosmos` 共用的 `run_cbindgen()` 辅助。
- `xtask` — workspace 自动化;`dump-metrics` 重新生成 `mps-web/src/metrics.rs`。

## 开发环境

Rust stable,本机用 GNU 工具链。构建前先 `source ~/.hermes_session_env.sh`(把 `~/.cargo/bin` + mingw64/bin 加入 PATH)。MSVC 工具链在此机无 `link.exe`;MSYS 的 `/usr/bin/link` 会覆盖 MSVC link.exe。

## 构建与测试(从 CI `ci.yml` 核实)

```
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                            # 默认 features;全部用例在 mps-test
cargo build --release                 # 整个 workspace
cargo build --release -p mps-jni      # Java 加载的 .dll/.so
cargo run -p xtask -- dump-metrics    # 新增 test/jni/ffi 后重新生成 metrics.rs
```

Feature flags:`default = []`。`anvilkit-bridge`(依赖 `anvilkit`+`bevy_ecs`)和 `relative-force` 默认关闭。CI 只跑默认 features。`--all-features`(`anvilkit-bridge`)目前在 `crates/mps-test/src/rapier/anvilkit.rs` 有既存编译错误,与近期改动无关——除非专门修这个桥接,否则避开。

## 约定(从代码观察到)

- FFI:`pub extern "C" fn <模块>_<动词>(...)` snake_case,如 `aero_apply_surfaces`。很多动词有 `_flag` 变体(多一个布尔模式参数)。大型/可选库用 `#[cfg(feature = "anvilkit-bridge")]` 门控,绝不进 `default`。
- C ABI 错误处理:返回 `u32` 状态码,取自 `error.rs` 常量——`ERR_OK=0`、`ERR_NULL_POINTER=1`、`ERR_INVALID_ARGUMENT=2`、`ERR_NOT_FOUND=3`、`ERR_CAPACITY=4`、`ERR_UNSUPPORTED=5`、`ERR_INTERNAL=6`。JNI 层用 `catch_unwind`+`AssertUnwindSafe` 包住函数体,在边界处兜住 panic。
- Handle:所有权用不透明 `*mut WorldHandle`/builder 指针;Rapier 引用用 packed `u64`(`ColliderHandleRaw`、`RigidBodyHandleRaw`、`JointBuilderHandle`)。JNI 经 `mps-jni/src/lib.rs` 中的 `to_jlong`/`m`/`cp`/`pm` 辅助函数转换。
- 测试:每个 crate 的测试都放在 `mps-test/src/`,与源码布局对应(`src/rapier/collider.rs` ↔ `mps-test/src/rapier/collider.rs`)。模式:`#[cfg(test)] mod tests { ... }` 里 `#[test] fn <动词>_<场景>`。源 crate 内不放内联 `#[cfg(test)]`。
- 公式分层:`mps-formula` = 纯函数(入→出,无 WorldHandle);`mps-core` = C ABI 封装,读取刚体状态、调用公式、施力。保持公式函数不触碰 Rapier 状态。
- 格式化:`rustfmt.toml`——edition 2024、max_width 100、4 空格、soft tabs。`clippy.toml` 抬高了阈值(too-many-args=10、cognitive-complexity=50)以适配 FFI 表面;`doc-valid-idents` 列出项目所用的缩写(FFI、AABB、OBB、SSV、kDOP 等)——文档注释中照原样使用。

## 易踩的坑

- **生成头文件**:`crates/mps-core/include/rigid_body.h` 和 `crates/mps-cosmos/include/cosmos.h` 是 cbindgen 输出(build.rs 生成)。CI 会校验它们已提交且为最新(`git diff --exit-code`,仅 Linux)。改源码→重新构建→提交重新生成的头文件——切勿手改。
- **生成 metrics**:`crates/mps-web/src/metrics.rs` 是 xtask 输出(TEST_COUNT/JNI_METHOD_COUNT/CORE_FFI_COUNT)。文件头注明 "Do NOT edit by hand"——新增 test/jni/ffi 后跑 `cargo run -p xtask -- dump-metrics`。
- **分层规则**:不要给 `mps-formula` 加 Rapier/WorldHandle 依赖——它必须保持纯净以便复用与测试;Rapier 交互一律放 `mps-core`。
- **Windows 工具链**:本机必须用 `stable-x86_64-pc-windows-gnu`。Commit message 是纯日期戳(如 `2026.8.11.20.8`)或简短描述——无 Conventional Commits 前缀。
- **测试很重**:718 个测试,冷启动 `cargo check --workspace` 约 2–3 分钟。跑单个用例用 `cargo test -p mps-test <名称>`。
