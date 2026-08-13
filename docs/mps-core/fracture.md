# rapier/fracture.rs

## 作用
实现断裂力学相关的计算与刚体碎裂操作。纯计算部分提供应力强度因子(Westergaard 公式)、Griffith 断裂准则、Miner 累积损伤法则、S-N 曲线疲劳寿命、能量释放率以及由应力状态判定断裂模式;世界操作部分提供 `world_replace_body_with_fracture_fragments`,将一个刚体替换为若干碎块(并用关节连接)。所有 C ABI 入口均做参数有限性/非负校验,并通过 `ffi_guard` 防 panic。

## 关键导出
- `pub extern "C" fn fracture_stress_intensity_factor(...)` — 计算应力强度因子与临界/安全系数(`StressIntensityReport`)。
- `pub extern "C" fn fracture_griffith_criterion(...)` — Griffith 能量断裂准则判定(`GriffithReport`)。
- `pub extern "C" fn fracture_miner_damage(...)` — Miner 线性累积疲劳损伤(`MinerDamageReport`)。
- `pub extern "C" fn fracture_sn_curve_life(...)` — 基于 S-N 曲线的疲劳寿命估算(`SnCurveReport`)。
- `pub extern "C" fn fracture_energy_release(...)` — 能量释放率计算(`FractureEnergyReport`)。
- `pub extern "C" fn fracture_mode_from_stress(...)` — 由应力张量判定断裂模式(拉/剪)(`FractureModeReport`)。
- `pub extern "C" fn world_replace_body_with_fracture_fragments(...)` — 把刚体替换为碎块并建关节(`FractureReplaceReport`)。
- 内部常量:`EPSILON`、`MAX_FRAGMENTS`;辅助 `material_valid` / `fragment_valid`(非 pub)。

## 依赖
- `rapier3d::prelude::{ColliderBuilder, FixedJointBuilder, RigidBodyBuilder, Vector}` — 碎块刚体/碰撞体/关节构造。
- `crate::rapier::error` — 错误码与 `ffi_guard`、`set_error`、`clear_error`。
- `crate::rapier::ffi` — `Bool`、`FractureEnergyReport`、`FractureFragmentDesc`、`FractureMaterial`、`FractureModeReport`、`FractureReplaceReport`、`GriffithReport`、`ImpulseJointHandleRaw`、`MinerDamageReport`、`RigidBodyHandleRaw`、`SnCurveReport`、`StressIntensityReport`、`WorldHandle`,及 handle 打包/解包、`vec3_finite`、`vec3_to_rapier`。
- `crate::rapier::math::{finite_non_negative, finite_positive}` — 数值有限性校验。
- `std::slice` — 跨 FFI 边界的可读切片构造。
