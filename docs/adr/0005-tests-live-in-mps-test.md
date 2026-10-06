# 0005 — 测试集中在 mps-test

## 背景

源 crate 同时是 `cdylib`。把集成测试散在各 crate 的 `#[cfg(test)]` 里，会让「加了模块但没人测」变成静默的，也会让 cdylib 的测试配置和库使用者的配置缠在一起。

## 决定

全部集成测试放在 `mps-test`。目录与 `mps-core/src/rapier`、`mps-formula/src`、`mps-cosmos/src` 镜像。源 crate 不另放一套内联测试来代替这个镜像。

守门测试也放在 `mps-test`：模块镜像、错误码、metrics、arena ABI、版本常量。

## 后果

新增或重命名子模块时必须同时增删测试文件，即使测试体一开始只是编译级占位，也要让镜像守门能看见文件。`verify_module_mirror` 只看目录清单，不解析测试内容；有文件不等于有断言，写测试的人仍然要把行为写进函数体。

## 相关代码

- `crates/mps-test/src/lib.rs`
- `crates/mps-test/src/rapier/verify_module_mirror.rs`
