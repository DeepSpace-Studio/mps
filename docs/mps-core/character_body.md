# rapier/character_body.rs

## 作用
端到端**角色体** —— **第三种体类型**：由 `KinematicCharacterController` 驱动的运动学刚体。胶囊/球碰撞体可以行走、滑动、自动上台阶（autostep）、贴地（snap-to-ground），解析后的位置每步写回运动学刚体，因此能推动其它体、并像普通体一样被查询。复用 `controller.rs` 的碰撞查询构造；存在 `world.character_bodies` 哈希表（稳定 id）。

## 关键导出
- `character_body_create(world, ...)` / `character_body_set_shape` — 创建与换形。
- `character_body_move(world, id, displacement, dt)` — 施加移动意图并求解。
- `character_body_set_up(world, id, x, y, z)` — 上方向（默认 +Y）。
- `character_body_set_offset_absolute` / `_relative` — 碰撞体偏移。
- `character_body_set_autostep` / `_set_snap_to_ground` / `_set_slope_angles` / `_set_slide` — 步高、贴地、坡度限制、滑动开关。
- `character_body_is_grounded` / `_is_sliding_down_slope` / `_is_on_ground` — 接地状态。
- `character_body_collision_count` / `_get_collision` — 本步碰撞列表。
- `character_body_solve_impulses` / `_set_apply_impulses_to_dynamic_bodies` — 对动态体的推挤。
- `character_body_move_with_terrain` — 移动并带运动地台的搬运。
- `character_body_get_translation` / `_destroy`。
- 内部：`CharacterBody`。

## 依赖
- Rapier `KinematicCharacterController` + 查询管线。
- `crate::rapier::ffi`（`ShapeDesc`、`CharacterCollision`、handle 打包）、`crate::rapier::error`。

## 测试
`mps-test/src/rapier/character_body.rs` — 行走/接地/坡度/推动动态体等端到端。
