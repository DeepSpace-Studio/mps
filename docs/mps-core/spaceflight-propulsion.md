# rapier/spaceflight/propulsion.rs

## 作用
推进与电源: CO2 质量平衡、Hall 推力器性能、Sabatier 甲烷合成、太阳能板功率、电池等效电路、结构自然频率、接触 (Hunt-Crossley) 力。7 个 C ABI 入口，纯计算不操作 Rapier 状态。

## 关键导出
- `space_battery_equivalent_circuit(open_circuit_voltage, current, ohmic_resistance, rc_voltage, rc_resistance, rc_capacitance, capacity_coulombs, out_battery) -> Bool` — 电池 Thevenin+RC 等效电路: 端压、RC 电压导数、SOC 导数。
- `space_co2_mass_balance` — CO2 还原质量守恒。
- `space_hall_thruster_performance` — 霍尔推力器比冲/推力/效率。
- `space_sabatier_methane_rate` — Sabatier 反应甲烷产率。
- `space_solar_panel_power` — 太阳能板功率 (光照角/退化)。
- `space_structural_natural_frequency` — 结构基频 (悬臂梁简化)。
- `space_contact_force_hunt_crossley` — Hunt-Crossley 非线性接触力模型。

## 依赖
- 本 crate 子模块: `super::*`。
- 外部 crate: 无直接 (纯数值)。
