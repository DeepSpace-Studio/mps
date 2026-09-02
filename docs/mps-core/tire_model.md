# rapier/tire_model.rs

## 作用
Pacejka 简化轮胎模型，叠加在 `vehicle.rs` 射线悬架控制器之上。每个轮胎对应车辆控制器的一个 wheel：法向载荷取自 wheel 的真实悬架力（离地时退化为整车重量均分）；纵向滑移用轮速状态机——`engine_force` 在接触面产生驱动扭矩（F·r），地面反力（上一帧 fx）反向作用，`brake` 作为最大制动冲量把轮速往 0 夹（抱死而不反转）；滑移率 = (轮面速度 − 接触点前向速度)/参考速度，滑移角 = 接触点侧向速度/前向速度的反正切（前向轴 = 接触法线 × 世界轴，与控制器 `update_friction` 同约定，轴已含转向）。力曲线：滑移率/滑移角在峰值前线性、峰值后按 1/|slip| 衰减（Pacejka 形），载荷敏感性 `load^α`，摩擦椭圆以 `ellipse_factor` 封顶组合力（模拟漂移饱和）。**计算并存储**力，由调用方经 `tire_model_get_forces` 读取后自行施加（例如配合刚体冲量 FFI）；需要替代控制器内建摩擦时把 wheel 的 friction_slip 调低。

## 关键导出
- `pub extern "C" fn tire_model_create(world, vehicle_id, wheel_count)` — 绑定到已有车辆控制器（≤ `MAX_TIRE_COUNT = 32` 轮）。
- `pub extern "C" fn tire_model_set_params(world, id, wheel_index, peak_mu_long, peak_mu_lat, peak_slip_ratio, peak_slip_angle, load_sensitivity, ellipse_factor)` — 逐轮参数。
- `pub extern "C" fn tire_model_update(world, id, dt)` — 每帧在 `vehicle_controller_update` **之后**调用，积分轮速状态、计算并存储各轮 fx/fy。
- `pub extern "C" fn tire_model_get_forces(world, id, wheel_index, out_fx*, out_fy*)` — 读取最近一次计算的力。
- `pub extern "C" fn tire_model_remove(world, id)` — 移除（幂等）。
- 内部：`TireModel`/`TireState`（世界内 `tire_models` 哈希表；含 `wheel_omega` 轮速状态）、`WHEEL_MASS_FRACTION = 0.06`（轮转动惯量估算）、`MIN_REF_SPEED = 1.0`（低速防除零）、`TireParams` 默认值（μ=1.2、峰值滑移 0.15、α=0.8、椭圆因子 1.3）。

## 依赖
- `crate::rapier::vehicle::VehicleController` — 读取 `controller.wheels()` 的半径/转速输入/悬架力/世界轴/射线信息与底盘刚体速度（`velocity_at_point`）。
- `crate::rapier::ffi` — `Bool`、`WorldHandle`。
- `crate::rapier::error` — 错误码与 `ffi_guard`。

## 测试
`mps-test/src/rapier/tire_model.rs`：创建校验（未知车辆/轮数越界）、参数校验（越界轮索引/零摩擦/未知 id）、静止零力、驱动产生正纵向力（有界）、制动抱死（fx ≤ 0）、get_forces 的越界/空指针/非法 dt 校验、null world。
