# rapier/joints.rs

## 作用
冲量关节(ImpulseJoint)构造器与相关 C ABI 入口。支持 Fixed/Revolute/Prismatic/Rope/Spring/Spherical 六类关节构造器,提供创建/销毁/启用接触/局部锚点设置/轴限位/电机速度与位置配置,并把构造好的关节插入/移除世界。无 `pub struct`,只有 `pub(crate)` 维护构造器类型的枚举(对外只暴露 opaque `JointBuilderHandle`)。

## 关键导出
- `enum JointBuilderKind`(crate 级)— `Fixed/Revolute/Prismatic/Rope/Spring/Spherical` 各自包一个 Rapier `*JointBuilder`,并定义 `set_contacts_enabled/set_local_anchor1/set_local_anchor2/set_limits/set_motor_velocity/set_motor_position` 等内部分发方法。
- `extern "C"` 入口(~10 项):`joint_builder_create/destroy`、`joint_builder_set_contacts_enabled/set_local_anchor1/set_local_anchor2/set_limits/set_motor_velocity/set_motor_position`、`world_insert_impulse_joint`、`world_remove_impulse_joint`。
- `const EPSILON`、辅助 `valid_axis`(私有)。

## 依赖
- 外部 crate:`rapier3d::prelude::{FixedJointBuilder, ImpulseJointHandle, PrismaticJointBuilder, RevoluteJointBuilder, RopeJointBuilder, SpringJointBuilder, SphericalJointBuilder, Vector}`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(Bool/ImpulseJointHandleRaw/JointAxisDesc/JointBuilderHandle/JointTypeDesc/RigidBodyHandleRaw/Vec3/WorldHandle 及 joint_axis/joint_type 转换与句柄打包辅助)。
