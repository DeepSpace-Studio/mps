# rapier/shared_arena/header.rs

## 作用
`shared_arena` 子模块之一，扩展 `SharedPhysicsArena` 的头部访问与区域写入方法。
提供对原始头部字节的 `u32`/`u64` 读写，以及 flags 的原子 set/clear，并把积分参数、聚合力汇总两个固定区域直接刷入竞技场内存，供 Java 侧以零 JNI 方式读取。

## 关键导出
- `header_u32` — 以 `u32` 读取头部某偏移（pub）。
- `header_u64` — 以 `u64` 读取头部某偏移（pub）。
- `set_header_u32` — 以 `u32` 写入头部某偏移（pub(super)）。
- `set_flags` — 原子 `fetch_or` 设置头部 flags 位（offset 12，pub）。
- `clear_flags` — 原子 `fetch_and(!flags)` 清除头部 flags 位（pub）。
- `flush_integration_params` — 把 dt、求解迭代次数、CCD 子步、重力写入 integration_params 区域（pub）。
- `flush_force_report` — 把最大雷诺数、总外力、总阻力及各自 body 计数写入 force_summary 区域（pub）。

## 依赖
- `std::sync::atomic::{AtomicU32, Ordering}`。
- `super::SharedPhysicsArena`（本文件所有方法均为其 `impl` 块）。
- `rapier3d::prelude::Vector`、`crate::rapier::ffi::Vec3`（力汇总写入时用到）。
- `super` 中的 `OFF_*` 常量与 `integration_params_offset`/`force_summary_offset` 字段。
