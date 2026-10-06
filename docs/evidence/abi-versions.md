# ABI 版本

## 主张

共享 arena 已发布的布局常量被测试钉死。`ARENA_VERSION` 与 `mps-ffm::ABI_VERSION` 必须一起变，workspace 版本锁也要过。Java 宿主靠这个版本拒绝不匹配的库。

## 在哪

- arena 常量与布局：`crates/mps-core/src/rapier/shared_arena/`
- FFM 探针：`crates/mps-ffm/src/lib.rs` 的 `ABI_VERSION`（当前源码里的值是 `1`；以源码为准）
- 守门：`crates/mps-test/src/rapier/arena_compat.rs`
- 守门：`crates/mps-test/src/rapier/version_consistency.rs`

## 怎么复现

```powershell
cargo test -p mps-test arena_compat
cargo test -p mps-test version_consistency
```

改布局常量却不改测试，是 ABI 破坏，不是测试过时。先决定这是不是一次版本步进，再改两侧。
