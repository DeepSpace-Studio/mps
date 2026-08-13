# rapier/spaceflight/gnss.rs

## 作用
GNSS 与射频链路: 伪距、双差载波相位、Friis 链路预算、雷达测距测速。涉及卫星导航解算与通信链路功率预算的 5 个 C ABI 入口。

## 关键导出
- `space_friis_link(transmit_power, transmit_gain, receive_gain, wavelength, range, system_loss, out_link) -> Bool` — Friis 公式计算接收功率与路径损耗，写入 `FriisLink { received_power, path_loss }`。
- `space_friis_wavelength_from_frequency(frequency) -> f64` — 由频率推自由空间波长 (c/f)。
- `space_gnss_double_difference_carrier_phase` — 站间+星间双差载波相位。
- `space_gnss_pseudorange` — 含电离层/对流层延迟修正的伪距观测值。
- `space_radar_range_rate` — 雷达多普勒测距变化率。

## 依赖
- 本 crate 子模块: `super::*` (常量 SPEED_OF_LIGHT、PI、finite/write_out/ffi_guard)。
- 外部 crate: 间接经 super 的 rapier3d::Vector。
