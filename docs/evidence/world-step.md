# world_step 矩阵

## 主张

`mps-test` 里有一条被 ignore 的 release 矩阵，用来记 `world_step` 的均值、P50、P95 和进程 CPU。它不进默认 `cargo test`，因此默认 CI 绿并不代表性能没有回退。

## 在哪

跑法和 CSV 列的定义在 [world-step-benchmarks.md](../world-step-benchmarks.md)。刚体中心索引的相关滤镜在 [body-spatial-index.md](../body-spatial-index.md)。

产物：

- `target/world-step-matrix.csv`
- `target/world-step-matrix-raw.csv`

两次运行会互相覆盖。数字不要抄进本目录。

## 怎么复现

按专题文档里的命令，在空闲的 release 下单独跑。比较两次结果时记录 CPU 型号、电源策略和 `rustc -Vv`。开 `profiler` 只填充分阶段列，不用于绝对时间验收。
