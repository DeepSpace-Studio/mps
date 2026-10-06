# docs

设计约束和证据。给人读的项目简要在仓库根 [README.md](../README.md)；给代理的操作清单在 [AGENTS.md](../AGENTS.md)。

代码注释里的 `DESIGN.md §N` / `OPTIMIZATION.md §N` 仍指向仓库根的两份章节索引。那两个文件的章节号是守门测试的锚，不搬进这里。

## 读哪一份

| 文档 | 回答的问题 |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统现在长什么样，数据怎么过边界 |
| [BOUNDARY.md](BOUNDARY.md) | 什么不许跨、什么必须成对修改 |
| [ROADMAP.md](ROADMAP.md) | 现在做什么、明确不做哪些 |
| [CHECKPOINT_RECOVERY.md](CHECKPOINT_RECOVERY.md) | 长任务断了之后从哪接着查 |
| [adr/](adr/README.md) | 已经做出的决定，以及为什么 |
| [evidence/](evidence/README.md) | 哪些行为被测试或基准钉住 |
| [mps-core.md](mps-core.md) | `mps-core` 每个源文件做什么 |

## 专题（实现契约，不是路线）

| 文档 | 内容 |
| --- | --- |
| [formula-numeric-domains.md](formula-numeric-domains.md) | 纯公式的数值域：`FormulaError`，不写 FFI 错误槽 |
| [world-collision-mode.md](world-collision-mode.md) | 默认碰撞体策略（None / Simple / Compound / Adaptive） |
| [body-spatial-index.md](body-spatial-index.md) | 刚体中心 BVH：区域查询，不替代碰撞体 BVH |
| [world-step-benchmarks.md](world-step-benchmarks.md) | `world_step` 性能矩阵怎么跑、CSV 是什么 |

`mps-core/` 下是按源文件拆开的导览，只从 [mps-core.md](mps-core.md) 进入。不要在新文档里再抄一份文件清单。
