# articulation.rs — 铰接体(多刚体伺服链,组合层)

`crates/mps-core/src/rapier/articulation.rs`。一个**铰接体**是沿直线排布的球链刚体(`link_count` 个),相邻刚体用 **multibody revolute 关节**连接,每个关节带**隐式弹簧伺服**指向目标角。与 `soft_chain_create` / `cloth` / `rope` 同属组合层——不发明新物理。

## 机制

- **链**:link i 的球心 = `base + dir·i·spacing`(spacing = 2·link_radius);球 collider `density(0)` + `additional_mass`。**link 0 固定**(肩部锚定)——自由悬浮的链无法靠自身关节扭矩到达目标位形。
- **关节**:multibody revolute(`MultibodyJointSet::insert`,非 impulse joint)绕 `joint_axis`;anchors 在两球交界(`±dir·spacing/2`)。
- **伺服**:每个关节 `set_spring(3, stiffness, target)` —— rapier Multibody 的**隐式 backward-Euler 关节弹簧**(`-k·(q−rest) − k·dt·v`),无条件稳定;显式位置电机会往系统注入能量炸掉(fork `Multibody` 文档明确点名)。`joint_axis` 不得平行 `dir`(否则退化)。
- **关节接触关闭**(相邻壳体出生即相切),`contacts_enabled(false)`。

## 为何不用 impulse-joint 电机关节

fork 继承自上游的 3D wide 路径把角电机行硬编码跳过(`joint_constraint_builder.rs` 的 `#[cfg(feature = "dim3")] ang_motor_params = None`),而带未锁轴电机的 joint 会被 `supports_simd_constraints()` 踢出 SIMD、走标量路径——标量路径虽有实现,实测行为不稳定。multibody 隐式弹簧是 fork 中被有意设计为"替代显式电机"的路径,物理正确且无爆稳定性。

## FFI

- `articulation_body_create(world, base, dir, joint_axis, link_count, link_radius, link_mass, target_angles*, targets_len, stiffness, damping_unused) -> u32`;`link_count ∈ [2, 256]`;`target_angles` 可空/短于关节数(缺省 0)
- `articulation_body_link_handle(world, id, link_index) -> RigidBodyHandleRaw`(0 = base;可与全部 `rigid_body_*` / 力 FFI 互操作)
- `articulation_body_link_count(world, id) -> u32`
- `articulation_body_set_joint_target(world, id, joint_index, target_angle) -> Bool`(运行时重定位弹簧 rest,链唤醒)

## 测试

`crates/mps-test/src/rapier/articulation.rs`:链构建(间距/句柄/越界)、电机折叠(3×π/2 目标 → 末端 reach 显著缩短)、运行时重定位、坏参数表 + 未知 id。

## JNI

`softArticulationCreate / softArticulationLinkHandle / softArticulationLinkCount / softArticulationSetJointTarget`。
