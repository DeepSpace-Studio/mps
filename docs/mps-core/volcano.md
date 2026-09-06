# rapier/volcano.rs

## 作用
火山喷发模拟 — 喷发烟柱（plume）流场、热暴露与刚体**渐进熔化**，挂接在 `WorldHandle` 上。纯场公式（烟柱流场/温度剖面、冰雹式弹道冲量、牛顿热交换、超热熔化速率）全部放在 `mps_formula::volcano`（不触碰 Rapier 状态）；本模块持有逐世界的火山源注册表与逐刚体热/熔化状态，把场转成力与质量损失，经 `volcano_*` C ABI 暴露。

典型帧序：
```text
volcano_advance(world, dt)       // 时钟/喷发 RNG 走格
volcano_apply(world, dt, …)      // 烟柱拖拽 + 火山弹 + 加热/熔化
world_step(world, dt)
```

## 熔化模型
- 每个被追踪刚体带温度（K，初值 `AMBIENT_TEMPERATURE = 293.15`）与剩余质量比例 `mass_scale`（初值 1.0）。
- 加热：`dT/dt = heat_exchange_rate · (T_exposure − T_body)`（牛顿交换，`heat_exchange_rate` 单位 1/s）；烟柱外暴露温度为环境温度，即自然冷却。
- 熔化：`T > melt_point` 时每秒损失 `melt_rate · (T − T_melt) / T_melt` 比例的剩余质量（超热线性）。
- 质量收缩通过 `set_additional_mass(scale · 原始总质量 − 碰撞体质量部分)` 实现；`mass_scale ≤ 0.05` 视为完全熔化：`set_enabled(false)` 原地禁用，并把打包刚体句柄写入 `out_melted`。

## 关键导出
- `pub struct Volcano` — 火山源：`PlumeParams`（喷口位置、柱底半径、柱高、最大上升气流、熔岩温度）+ `ejecta_rate`（火山弹/秒）+ 确定性 RNG。
- `pub struct VolcanoBody` — 逐体热/熔化状态（温度、原始总质量、碰撞体质量部分、mass_scale、melted）。
- `pub extern "C" fn volcano_add(...)` — 注册火山（`lava_temperature` 传 0 用玄武岩默认 1473.15 K；`max_updraft` 可为 0 —— 熔岩湖类无风但全热暴露）。
- `pub extern "C" fn volcano_remove / _clear / _advance(...)` — 生命周期与推进（clear 同时重置逐体热状态）。
- `pub extern "C" fn volcano_sample_flow / _sample_temperature(...)` — 任意点采样烟柱流场（m/s）与暴露温度（K，不低于环境温度）。
- `pub extern "C" fn volcano_body_temperature / _set_body_temperature(...)` — 查询/脚本化覆盖刚体温度。
- `pub extern "C" fn volcano_apply(world, dt, melt_point, melt_rate, heat_exchange_rate, wake_up, out_bomb_count, out_melted, melted_capacity, out_melted_count)` — 对全部动态刚体施加烟柱拖拽 `F = ½ρ|v_rel|v_rel`（Cd·A 折算 1 m²）、近喷口区确定性火山弹冲量（splitmix64，`上主 + 径向外偏 + 抖动`），并执行加热/熔化。
- 所有入口返回 `u8` 状态码（`ERR_OK = 0`），`ffi_guard` 包裹防 panic 跨界。**`out_bomb_count`/`out_melted_count` 是"本次调用"计数**，跨帧统计由调用方累加。

## 依赖
- `mps_formula::volcano` — 纯公式：`PlumeParams`、`plume_flow_at`、`plume_temperature_at`、`body_heating_rate`、`melt_mass_loss_rate`、`lava_bomb_mass/_impulse`、常量 `AMBIENT_TEMPERATURE`/`LAVA_TEMPERATURE`/`ROCK_DENSITY`/`BOMB_RADIUS`/`BOMB_SPEED`。
- `mps_formula::disasters` — 复用 `wind_drag_force`（烟柱拖拽）与 `AIR_DENSITY_SEA_LEVEL`。
- `crate::rapier::error` — `ERR_*`、`ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `Bool`、`Vec3`、`RigidBodyHandleRaw`、`MAX_OUTPUT_CAPACITY`、`vec3_finite`、`vec3_to_rapier`、`pack_rigid_body_handle`。
- `crate::rapier::world::PhysicsWorld` — 新增字段 `volcanoes: IdRegistry<Volcano>`、`volcano_bodies: HashMap<RigidBodyHandleRaw, VolcanoBody>`。

## 实现注意
- **状态表键用打包句柄 `pack_rigid_body_handle`**（与 `volcano_body_temperature` 等查询入口、Java 侧持有的一致）；最初用裸 index 导致 set 与 apply 各写一条状态互不相通。
- **采样点用 `body.translation()`**；`world_com` 只在 step 刷新（同 disaster 模块的坑）。
- **质量基准惰性捕获**：`body.mass()`（= `inv(inv_mass)`）在首次 step 前报 0，所以 `original_total_mass ≤ 0` 时每帧重试捕获；捕获成功前只加热/弹射、不缩放质量。
- **熔化的质量缩放走 `set_additional_mass`**，其效果在下一个 step 才生效（标记 `LOCAL_MASS_PROPERTIES` 由管线重算）；末帧过冲会使最终质量略低于 5% 阈值，属预期。
- 火山弹冲量以 `总冲量/dt` 的等效力走 `add_force` 管线（新插入 body 的 `effective_inv_mass` 未刷新，直接 `apply_impulse` 首帧会丢，同 disaster 模块）。
