# rapier/sensor.rs

## 作用
传感器触发区 —— **第四种体类型**：一个非实体的 sensor collider，每步跟踪与其重叠的碰撞体集合，是 rapier 原生的"触发体积"抽象。纯 `mps-core` 层（无 fork 改动）。重叠检测用 `QueryPipeline::intersect_shape` 走 broad-phase BVH——与角色控制器一样，需要先 `world_step` 过至少一次 BVH 才有结果。存在 `world.sensor_zones` 哈希表（稳定 id），`sensor_zone_poll` 在 step 时机内刷新重叠集，随后可查询/消费接触事件。

## 关键导出
- `sensor_zone_create(world, shape, translation)` / `_set_shape` / `_set_translation` — 形状与位姿。
- `sensor_zone_set_enabled` / `_set_edge` — 启停与边缘行为。
- `sensor_zone_poll(world, id)` — 刷新当前重叠集合。
- `sensor_zone_contact_count` / `_get_contacts` — 读取重叠碰撞体。
- `sensor_zone_is_triggered` / `_consume` / `_clear` — 触发状态与一次性消费。
- `sensor_zone_get_translation` / `_destroy`。
- 内部：`SensorZone`。

## 依赖
- `crate::rapier::ffi`（`ShapeDesc`、`ColliderHandleRaw`、`Vec3`）、`crate::rapier::error`。
- Rapier `QueryPipeline::intersect_shape` + sensor collider 标志。

## 测试
`mps-test/src/rapier/sensor.rs` — 触发区进入/离开/消费语义。
