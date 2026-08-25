# rapier/controller.rs

## 作用
运动学角色控制器(`KinematicCharacterController`)的 C ABI 封装层。提供控制器的创建/销毁/配置(up 朝向、绝对/相对偏移、滑动、autostep、贴地、坡角阈值)、移动求解(`character_controller_move_shape`)、碰撞记录读取(`character_controller_collision_count/get_collision`)与冲量求解(`character_controller_solve_impulses`)。内部用 `CharacterControllerState` 持有 Rapier 控制器与碰撞缓冲。

## 关键导出
- `struct CharacterControllerState`(crate 级)— 包装 `KinematicCharacterController` 与 `Vec<RapierCharacterCollision>`,带 `#[derive(Default)]`。
- `extern "C"` 入口(~13 项):`character_controller_create/destroy`、`set_up`、`set_offset_absolute/relative`、`set_slide`、`set_autostep`、`set_snap_to_ground`、`set_slope_angles`、`move_shape`、`collision_count`、`get_collision`、`solve_impulses`。

## 依赖
- 外部 crate:`rapier3d::control::{CharacterAutostep, CharacterCollision as RapierCharacterCollision, CharacterLength, KinematicCharacterController}`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(Bool/CharacterControllerHandle/CharacterCollision as FfiCharacterCollision/EffectiveCharacterMovement/Quat/ShapeDesc/Vec3/WorldHandle 及 shape_desc_valid/shape_from_desc/quat/vec3 转换与句柄打包)。
