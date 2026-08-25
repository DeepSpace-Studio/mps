# rapier/spaceflight/thermal.rs

## 作用
热控: 气闸减压、热管热阻、单相流体回路、辐射器散热、Whipple 防护、SPE 氧气产率。7 个 C ABI 入口，纯计算。

## 关键导出
- `space_airlock_depressurization(pressure, ambient_pressure, volume, conductance, dt, out_state) -> Bool` — 气闸减压瞬态 (指数松弛模型)，写入 `AirlockDepressurization { pressure, pressure_rate }`。
- `space_heat_pipe_thermal_resistance(evaporator, vapor, condenser, wick) -> f64` — 热管总热阻 (四段相加)。
- `space_radiator_power` — 辐射器散热功率 (Stefan-Boltzmann)。
- `space_single_phase_loop_heat_transfer` — 单相流体回路换热。
- `space_spe_oxygen_rate` — SPE (质子交换膜) 电解氧产率。
- `space_thermal_balance` — 节点热平衡 (写入 ThermalBalance)。
- `space_whipple_critical_projectile_diameter` — Whipple 防护层临界弹体直径。

## 依赖
- 本 crate 子模块: `super::*` (常量 SIGMA/Stefan-Boltzmann、finite/write_out/ffi_guard)。
- 外部 crate: 无直接 (纯数值)。
