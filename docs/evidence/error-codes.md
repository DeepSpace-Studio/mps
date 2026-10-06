# 错误码一致

## 主张

`mps-formula` 与 `mps-core` 的 `ERR_OK` … `ERR_INTERNAL` 数值相同，且就是 0 到 6。

## 在哪

- 规范字面量：`crates/mps-formula/src/error.rs`
- 为 cbindgen 再声明的一份：`crates/mps-core/src/rapier/error.rs`（文件内 `const _: ()` 断言）
- 守门：`crates/mps-test/src/rapier/error_consistency.rs`

## 怎么复现

```powershell
cargo test -p mps-test error_consistency
```

失败时消息给出两侧定义位置。不要把断言放宽。改码值就两侧一起改，并同步 [BOUNDARY.md](../BOUNDARY.md) 里的码表。决定本身在 [adr/0003-c-abi-error-codes.md](../adr/0003-c-abi-error-codes.md)。
