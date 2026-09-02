# rapier/vehicle.rs

## 作用
射线悬架车辆控制器 —— **第五种体类型**，基于 fork 的 `DynamicRayCastVehicleController`（`rapier/src/control/ray_cast_vehicle_controller.rs`，Bullet 风格）。驱动一个动态底盘刚体 + N 个射线投射车轮：每步对每轮沿悬架方向射线检测地面，施加悬架弹簧力，并经 `update_friction` 施加前向/侧向摩擦冲量。纯 `mps-core` 层（无 fork 改动）。每帧时序：`vehicle_controller_update`（更新轮变换 + 悬架 + 摩擦）→ `world_step`。控制器存在 `world.vehicle_controllers` 哈希表（稳定 id）。轮胎模型层见 [tire_model.md](tire_model.md)。

## 关键导出
- `vehicle_controller_create(world, shape, translation)` — 由形状建动态底盘并包一个控制器。
- `vehicle_controller_set_shape(world, id, shape)` — 替换底盘形状。
- `vehicle_controller_add_wheel(world, id, chassis_connection_cs, direction_cs, axle_cs, rest_length, radius, stiffness, compression, damping, friction_slip, max_travel, max_force, side_friction_stiffness)` — 加轮（局部空间连接点/悬架方向/轮轴）。
- `vehicle_controller_set_engine_force` / `_set_brake` / `_set_steering` — 逐轮驱动输入。
- `vehicle_controller_update(world, id, dt)` — 每步更新（轮变换、悬架力、摩擦、rotation 积分）。
- `vehicle_controller_get_translation` / `_get_velocity` / `_wheel_on_ground` / `_wheel_contact_normal` — 状态查询。
- `vehicle_controller_destroy` — 移除控制器（保留底盘刚体）。
- 内部：`VehicleController { controller, body }`。

## 依赖
- fork `rapier::control::DynamicRayCastVehicleController` / `Wheel` / `RayCastInfo`。
- `crate::rapier::ffi`（`ShapeDesc`/`shape_from_desc`/`vec3_*`）、`crate::rapier::error`。

## 测试
`mps-test/src/rapier/vehicle.rs` — 底盘 + 地板端到端：驱动/转向/接地查询/平移查询等。
