# 模块镜像

## 主张

`mps-test/src/rapier` 的文件集合等于 `mps-core/src/rapier` 与 `mps-formula/src` 的并集（去掉 `mod.rs` / `lib.rs`，加上测试专用适配文件）。`mps-test/src/cosmos` 镜像 `mps-cosmos/src`（去掉 `lib.rs` 与 `ffi.rs`）。源是目录而测试仍是单文件时，守门允许过渡，但会提示；测试是目录而源还是单文件则失败。

## 在哪

`crates/mps-test/src/rapier/verify_module_mirror.rs`

比较的是目录清单，不解析 `#[test]` 正文。

## 怎么复现

```powershell
cargo test -p mps-test verify_module_mirror
```

新增子模块却没有对应测试文件时，这条失败。补文件，而不是把名字加进忽略列表，除非那个文件本来就是测试专用适配（`extra_ok` 在源码里）。
