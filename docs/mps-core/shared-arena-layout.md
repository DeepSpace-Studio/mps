# rapier/shared_arena/layout.rs

## 作用
`shared_arena` 子模块之一，仅定义命令类型枚举 `CommandType`。
原 1028 行的 `shared_arena.rs` 按 OPTIMIZATION.md §N5 拆分后，把命令种类单独抽出成此小文件；`SharedPhysicsArena` 结构体仍留在 `mod.rs`，以便各兄弟 impl 文件能直接构造/解构它。

## 关键导出
- `CommandType` — `#[repr(u32)]` 命令类型枚举，值对应命令环中 `cmd_type` 字段。
  - `AddForce = 0`、`AddTorque = 1`、`SetPose = 2`、`SetVelocity = 3`
  - `ApplyImpulse = 4`、`ApplyTorqueImpulse = 5`、`WakeUp = 6`、`Sleep = 7`
  - `SetRotation = 8`、`SetGravityScale = 9`、`SetLinearDamping = 10`、`SetAngularDamping = 11`
  - `AddForceAtPoint = 12`

（本文件仅此一个 pub 符号，无函数、无 struct。）

## 依赖
- 无运行时依赖，纯枚举定义。
- 语义上被 `crate::rapier::shared_arena::ring` 的 `drain_commands` 与 `crate::rapier::shared_arena::mod.rs` 使用（经 `pub use layout::CommandType`）。
