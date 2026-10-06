# 数值域

## 主张

已经迁到 `Result<T, FormulaError>` 的公式不碰 FFI 错误槽。NaN 与无穷大输入失败；物理约束按参数声明；非有限结果失败，除非该函数文档点名某个无穷极限。

## 在哪

- 政策与当前已迁移的 API 表：[formula-numeric-domains.md](../formula-numeric-domains.md)
- 实现：`crates/mps-formula/src/domain.rs`
- FFI 映射：`mps-formula` 的 `ffi` 模块（域错误 → `ERR_INVALID_ARGUMENT`，指针错误仍分开）

## 怎么复现

表里每个符号都可以在 `mps-test` 里按名字过滤。例如：

```powershell
cargo test -p mps-test ballistic_coefficient_checked
```

表之外的公式还没有这条保证。不要把未迁移函数的行为写成已经 checked。
