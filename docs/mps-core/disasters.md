# rapier/disasters.rs

## 作用
自然灾害模拟模块 — 台风（热带气旋，含飓风，同一模型南北半球用 `spin` 符号区分）、龙卷风、冰雹三类灾害源，挂接在 `WorldHandle` 上。纯场公式（风场剖面、冰雹弹道）全部放在 `mps_formula::disasters`（不触碰 Rapier 状态），本模块只持有逐世界的灾害源注册表、采样合成风场，并把场转成刚体力/冰雹冲击脉冲，经 `disaster_*` C ABI 暴露。

典型帧序：
```text
disaster_advance(world, dt)       // 风暴中心随 translation 平移、冰雹 RNG 时钟走格
disaster_apply_forces(world, …)   // 风阻力 + 冰雹冲击（走 add_force 管线）
world_step(world, dt)
```

## 关键导出
- `pub enum Disaster` — 灾害源：`Cyclone { params: CycloneParams }`（Rankine 涡 + 径向入流 + 风暴平移 + 边界层幂律高度剖面）、`Tornado { params: TornadoParams }`（垂直轴 Rankine 组合涡 + 核心集中上升气流 + 漏斗顶以上线性衰减）、`Hail { center, radius, intensity, hail_radius, rng_state }`（水平圆盘内的冰雹落区）。
- `pub extern "C" fn disaster_add_typhoon(...)` — 注册台风，返回稳定 id（`inflow_fraction`/`height_exponent`/`reference_height` 传 0 用默认 0.15/0.11/10 m）。
- `pub extern "C" fn disaster_add_tornado(...)` — 注册龙卷风（垂直轴过 `base`）。
- `pub extern "C" fn disaster_add_hail(...)` — 注册冰雹云团（`intensity` = 每平方米参考面积每秒期望冲击数）。
- `pub extern "C" fn disaster_remove / disaster_clear / disaster_advance(...)` — 生命周期与推进。
- `pub extern "C" fn disaster_sample_wind(world, point, out_wind)` — 合成风场（台风 + 龙卷风求和，冰雹不产风）。
- `pub extern "C" fn disaster_sample_pressure_drop(world, point, air_density, out_drop)` — 各涡旋气旋式压降 `Δp = ½ρv²` 求和。
- `pub extern "C" fn disaster_apply_forces(world, air_density, drag_coefficient, reference_area, dt, wake_up, out_body_count, out_impact_count)` — 对全部动态刚体施加风阻力 `F = ½ρCdA|v_rel|v_rel`，对落区内动态体施加确定性（splitmix64）冰雹捕获脉冲 `m·v_terminal`（以 `总脉冲/dt` 的等效力求走 `add_force` 管线）。
- 所有入口返回 `u8` 状态码（`ERR_OK = 0`），`ffi_guard` 包裹防 panic 跨界。

## 依赖
- `mps_formula::disasters` — 纯公式：`CycloneParams`/`TornadoParams`、`cyclone_wind_at`、`tornado_wind_at`、`cyclostrophic_pressure_drop`、`hail_mass/_terminal_velocity/_impulse`、`wind_drag_force`、常量 `AIR_DENSITY_SEA_LEVEL`/`ICE_DENSITY`。
- `crate::rapier::error` — `ERR_*`、`ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `Bool`、`Vec3`、`MAX_OUTPUT_CAPACITY`、`vec3_finite`、`vec3_to_rapier`、`vec3_from_rapier`。
- `crate::rapier::world::PhysicsWorld` — 新增字段 `disasters: IdRegistry<Disaster>`（id 单调不复用）。

## 实现注意
- **采样点用 `body.translation()` 而非 `center_of_mass()`**：`mprops.world_com` 只在 step 时刷新，刚插入的 body 读到过期的原点，会导致风场全部采在原点（冰雹落区判定同理）。
- **冰雹脉冲走 `add_force(总脉冲/dt)` 而非 `apply_impulse`**：刚插入 body 的 `effective_inv_mass` 同样未刷新，直接改速度会在首帧丢失；力管线在 step 积分时使用已刷新的质量属性，线性动量传递等价。
- 冰雹冲击数与横向散布由 `splitmix64`（以 registry 计数器 + body handle 混种）确定性生成 — 同一输入永远同一结果，利于回放/测试。
- `air_density` 传 0 取 ISA 海平面默认 1.225 kg/m³；冰雹终端速度球体系数 `Cd = 0.47` 固定。
