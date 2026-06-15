# mps_rigid_body 项目分析

## 1. 项目定位

这个项目是一个用 Rust 编写的 3D 刚体物理 native 库，底层依赖 `rapier3d-f64`，也就是 Rapier 3D 物理引擎的双精度版本。项目编译产物包含 `cdylib`，因此主要目标是给 Java 通过 JNI 或 Java FFM 调用。

从 `Cargo.toml` 可以看出几个核心信息：

```toml
name = "mps_rigid_body"
crate-type = ["cdylib", "rlib"]
rapier3d = { package = "rapier3d-f64", version = "0.33.0", features = ["parallel"] }
```

因此它不是一个纯 Rust 应用，而是一个跨语言物理引擎封装层。Rust 负责持有真实物理世界、刚体、碰撞体和查询结构；Java 侧主要持有 `long` 类型的指针或 handle。

整体调用链可以理解为：

```text
Java
  -> JNI / FFM
    -> Rust extern "C" API
      -> 项目内 rapier wrapper
        -> rapier3d-f64
```

## 2. 顶层结构

`src` 下主要分为三层：

```text
src/
  lib.rs
  abi/
    ffm.rs
    jni.rs
    mod.rs
  helper/
    helper.rs
    mod.rs
  rapier/
    ffi.rs
    world.rs
    rigid_body.rs
    collider.rs
    query.rs
    events.rs
    joints.rs
    controller.rs
    bounds.rs
    dop.rs
    neural.rs
    voxel.rs
    compat.rs
    rtree.rs
    crbtree.rs
    error.rs
    mod.rs
```

`lib.rs` 是 crate 入口，设置 `mimalloc` 为全局 allocator，并把 FFI 所需的类型重新导出，例如 `Vec3`、`Quat`、`WorldHandle`、`ShapeDesc`、`RayHit`、`CollisionEventRecord` 等。

`abi` 目录负责 Java 接入。

`helper` 目录提供 JNI 数组转换等辅助能力。

`rapier` 目录是核心实现，里面封装了世界模拟、刚体、碰撞体、查询、事件、关节、角色控制器和扩展几何。

## 3. ABI 和 FFI 设计

`src/rapier/ffi.rs` 是整个项目的 ABI 类型中心。

它定义了大量 `#[repr(C)]` 结构体，保证 Rust 内存布局能被 C ABI、Java FFM 或 JNI 侧稳定理解。例如：

```rust
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Quat {
    pub i: f64,
    pub j: f64,
    pub k: f64,
    pub w: f64,
}
```

项目没有直接把 Rapier 内部类型暴露给外部，而是定义自己的 ABI 类型：

- `Vec3`：三维向量。
- `Quat`：四元数。
- `Bool`：用 `u8` 表示布尔值。
- `ShapeDesc`：基础形状描述。
- `InteractionGroupsDesc`：碰撞组和过滤组。
- `QueryFilterDesc`：查询过滤器。
- `ShapeCastOptionsDesc`：形状扫掠参数。
- `RayHit`：射线命中结果。
- `ShapeCastHit`：形状扫掠命中结果。
- `CollisionEventRecord`：碰撞事件。
- `ContactForceEventRecord`：接触力事件。
- `VoxelColliderOptions`：体素 collider 构建参数。
- `NeuralBoundsDesc`：神经边界体参数。

### 3.1 Opaque Pointer

世界、builder、控制器、树索引等对象都用 opaque pointer 暴露：

```rust
pub struct WorldHandle {
    pub(crate) inner: crate::rapier::world::PhysicsWorld,
}
```

Java 侧只拿到一个 `long`，Rust 侧再转回 `*mut WorldHandle`。

### 3.2 Rapier Handle 打包

Rapier 内部的 `RigidBodyHandle`、`ColliderHandle`、`ImpulseJointHandle` 本质上包含 `id` 和 `generation`。项目把它们打包成 `u64`：

```rust
(((generation as u64) << 32) | (id as u64)).wrapping_add(1)
```

这里加 `1` 是为了把 `0` 保留为 null 或 invalid handle。外部 API 返回 `0` 通常表示失败或不存在。

## 4. Java 接入层

`src/abi/ffm.rs` 很薄，只提供 ABI 能力探测：

- `abi_version`
- `abi_supports_ffm`
- `abi_supports_jni`

`src/abi/jni.rs` 则把 Rust C ABI 包装成 JNI 函数。它使用宏生成大量 `Java_org_polaris2023_msp_1rigid_1body_RigidBodyNative_*` 方法。

JNI 层主要职责是：

- `jlong` 和裸指针互转。
- `jint` 转 `u32`。
- `jdouble` 转 `f64`。
- Java double array 转 Rust slice。
- Rust `Vec3` / `Quat` 转 Java double array。
- 用 `catch_unwind` 捕获 panic，避免 panic 穿过 JNI 边界。

JNI 层本身不做复杂物理逻辑，它只是参数适配和安全边界。

## 5. PhysicsWorld 和世界模拟

`src/rapier/world.rs` 定义了项目内部的核心世界结构：

```rust
pub(crate) struct PhysicsWorld {
    pub(crate) pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) islands: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) hooks: CallbackPhysicsHooks,
    pub(crate) events: CollectingEventHandler,
}
```

这基本就是 Rapier 运行一帧所需的完整状态集合。

主要 API：

- `world_create`：创建物理世界。
- `world_destroy`：释放世界。
- `world_step`：推进模拟。
- `world_set_gravity` / `world_get_gravity`：设置或读取重力。
- `world_set_integration_parameters`：设置 dt、solver iterations、CCD substeps。
- `world_body_snapshot`：批量导出所有刚体状态。
- `world_dynamic_body_snapshot`：批量导出动态刚体状态。
- `world_update_body_poses`：批量更新刚体位姿。
- `world_update_body_velocities`：批量更新刚体速度。

`world_step` 内部调用 Rapier 的核心模拟：

```rust
world.inner.pipeline.step(
    world.inner.gravity,
    &world.inner.integration_parameters,
    &mut world.inner.islands,
    &mut world.inner.broad_phase,
    &mut world.inner.narrow_phase,
    &mut world.inner.bodies,
    &mut world.inner.colliders,
    &mut world.inner.impulse_joints,
    &mut world.inner.multibody_joints,
    &mut world.inner.ccd_solver,
    &world.inner.hooks,
    &world.inner.events,
);
```

项目限制单步 `delta_seconds` 必须大于 `0` 且不超过 `1.0` 秒，避免异常大步长破坏模拟稳定性。

## 6. 刚体模块

`src/rapier/rigid_body.rs` 负责刚体创建、插入、删除和运行时控制。

支持的刚体类型：

- `Dynamic`：动态刚体，受力、碰撞和重力影响。
- `Fixed`：静态刚体，不移动。
- `KinematicPositionBased`：基于位置的运动学刚体。
- `KinematicVelocityBased`：基于速度的运动学刚体。

典型创建流程：

```text
rigid_body_builder_create
rigid_body_builder_set_translation
rigid_body_builder_set_rotation
rigid_body_builder_set_linvel
rigid_body_builder_build
world_insert_rigid_body
```

builder 支持设置：

- translation
- rotation
- pose
- additional mass
- additional mass properties
- linear velocity
- angular velocity
- gravity scale
- linear damping
- angular damping
- can sleep
- enabled rotations
- user data

运行时支持：

- 获取/设置 body status。
- 获取/设置 translation。
- 获取/设置 rotation。
- 获取/设置 pose。
- 获取/设置 linear velocity。
- 获取/设置 angular velocity。
- add force。
- add torque。
- apply impulse。
- apply torque impulse。
- enable CCD。
- sleep。
- wake up。

实现上，builder 和临时 `RigidBody` 都通过 `Box::into_raw` 传给外部，再通过 `Box::from_raw` 回收或消费。也就是说，Java 侧必须遵守生命周期：builder build 后不能继续使用；未 build 的 builder 应该 destroy。

## 7. 碰撞体模块

`src/rapier/collider.rs` 负责 collider builder 和 collider 的世界操作。

基础形状由 `ShapeDesc` 或 `shape_type + Vec3` 创建，支持：

- ball
- cuboid
- capsule Y/X/Z
- cylinder
- round cylinder
- cone
- round cone
- round cuboid

额外构造器支持：

- OBB
- sphere
- heightmap
- convex hull
- point cloud bounds
- double BV
- skewed OBB
- discrete OBB
- fused collapsing bounds
- edge BVH
- medial spheres

builder 可设置：

- translation
- rotation
- pose
- sensor
- friction
- restitution
- density
- collision groups
- solver groups
- active events
- active hooks
- contact force event threshold

插入方式：

- `world_insert_collider`：插入独立 collider。
- `world_insert_collider_with_parent`：把 collider 挂到某个 rigid body 上。

多数真实物理对象会用 parent body 方式：刚体负责运动状态，collider 负责几何形状、密度、摩擦和碰撞过滤。

## 8. 查询模块

`src/rapier/query.rs` 提供对物理世界的空间查询。

主要功能：

- `query_cast_ray`：单条射线检测。
- `query_cast_rays`：批量射线检测。
- `query_project_point`：把点投影到最近 collider。
- `query_intersect_point_count`：点包含查询。
- `query_intersect_aabb_count`：AABB 交集数量。
- `query_intersect_obb`：OBB 交集列表。
- `query_intersect_sphere`：球体交集列表。
- `query_cast_shape`：形状扫掠。

查询内部会基于世界的 broad phase 和 narrow phase 构造 query pipeline：

```rust
world.inner.broad_phase.as_query_pipeline(
    world.inner.narrow_phase.query_dispatcher(),
    &world.inner.bodies,
    &world.inner.colliders,
    query_filter_from_desc(filter),
)
```

`QueryFilterDesc` 支持：

- flags
- interaction groups
- exclude collider
- exclude rigid body

这让 Java 侧可以实现“射线忽略自己”、“只检测某个碰撞组”、“排除某个刚体”等常见逻辑。

## 9. 事件模块

`src/rapier/events.rs` 实现 Rapier 的 `EventHandler`，用于收集模拟中的事件。

收集两类事件：

- collision event：碰撞开始或结束。
- contact force event：接触力事件。

事件存储在 `parking_lot::Mutex<Vec<_>>` 中，最大记录数是 `16_384`。

对外 API：

- `world_clear_events`
- `world_collision_event_count`
- `world_get_collision_event`
- `world_get_collision_events`
- `world_contact_force_event_count`
- `world_get_contact_force_event`
- `world_get_contact_force_events`

文件里也有 contact pair callback 和 intersection callback 的 API，但当前实现会返回 unsupported。原因是外部 ABI 回调涉及函数指针、生命周期和线程安全，项目目前选择禁用，保留默认 hooks。

## 10. 关节模块

`src/rapier/joints.rs` 封装 Rapier impulse joints。

支持的 joint 类型：

- fixed
- revolute
- prismatic
- rope
- spring
- spherical

builder 支持设置：

- contacts enabled
- local anchor1
- local anchor2
- limits
- motor velocity
- motor position

插入 joint 的流程：

```text
joint_builder_create
joint_builder_set_local_anchor1
joint_builder_set_local_anchor2
joint_builder_set_limits
world_insert_impulse_joint
```

`world_insert_impulse_joint` 会消费 builder，并把两个 rigid body handle 连接起来。

## 11. 角色控制器

`src/rapier/controller.rs` 包装 Rapier 的 `KinematicCharacterController`。

它不是动态刚体求解器，而是游戏中常见的角色移动控制器。主要用于角色胶囊体、玩家移动、斜坡、台阶和贴地逻辑。

支持设置：

- up 方向
- offset absolute
- offset relative
- slide
- autostep
- snap to ground
- slope angles

移动流程：

```text
character_controller_move_shape
character_controller_collision_count
character_controller_get_collision
character_controller_solve_impulses
```

`character_controller_move_shape` 返回 `EffectiveCharacterMovement`：

- `translation`：实际可应用位移。
- `grounded`：是否接地。
- `is_sliding_down_slope`：是否正在沿斜坡下滑。

碰撞详情保存在 controller 内部的 `collisions` 列表里，可以逐个读取。

## 12. Bounds 扩展形状

`src/rapier/bounds.rs` 提供一些额外边界体构造和查询：

- capsule
- SSV
- ellipsoid
- prism
- cylinder
- spherical shell

实现方式：

- capsule / SSV：本质上使用 Rapier capsule。
- cylinder：使用 Rapier cylinder。
- ellipsoid：采样球面点后按 radii 缩放，再构造 convex hull。
- prism：生成上下两个多边形环，再构造 convex hull。
- spherical shell：当前实现实际使用 outer radius 的 ball，并没有真实表示空心壳内部。

这些形状既可以创建 collider builder，也可以直接用于 intersection 查询。

## 13. DOP / FDH 模块

`src/rapier/dop.rs` 实现方向包围体。

支持 k-DOP preset：

- K6
- K14
- K18
- K26

还支持 FDH，也就是自定义方向集合。

实现流程：

1. 读取点云。
2. 读取或生成方向集合。
3. 对每个方向计算所有点的投影 min/max。
4. 得到一组 slab。
5. 枚举平面三元组求交点。
6. 保留落在所有 slab 内的点。
7. 用这些点构造 convex hull。

这种方式适合把复杂点云近似成受控方向的凸包，而不是直接生成任意 convex hull。

## 14. Neural Bounds 模块

`src/rapier/neural.rs` 实现了一个特殊的神经边界体。

输入参数包括：

- center
- half extents
- rotation
- sample resolution
- hidden width
- hidden layers
- activation
- output scale
- padding
- weights

大致算法：

1. 先加入 OBB 的 8 个角点。
2. 在球面上采样多个方向。
3. 把方向输入一个小 MLP。
4. MLP 输出该方向的 expansion。
5. 在基础 OBB 半径上叠加 expansion 和 padding。
6. 得到扩展后的点集。
7. 用点集生成 convex hull。

它提供：

- `neural_bounds_required_weight_count`
- `collider_builder_create_neural_bounds`
- `query_intersect_neural_bounds`

这个模块适合把某些复杂或学习得到的边界近似成可用于物理查询的凸包。

## 15. Voxel 模块

`src/rapier/voxel.rs` 把体素网格转换成 collider。

输入是 `u8` 数组，非 0 表示 solid voxel。

支持模式：

- `Cuboids`：每个 solid voxel 生成一个 cuboid。
- `GreedyCuboids`：把连续 solid voxel 合并成长方体。
- `SurfaceMesh`：只生成外表面 mesh。
- `Auto`：根据体素数量和是否 dynamic body 自动选择。

`Auto` 的倾向：

- solid 数量较小时用 cuboids。
- dynamic body 倾向 greedy cuboids。
- 大型静态体素场景倾向 surface mesh。

模块中有多个上限保护：

- 最大体素数：`262_144`
- 最大 compound parts：`100_000`
- 最大 surface vertices：`1_000_000`
- 最大 surface triangles：`500_000`

这些限制用于防止 Java 侧传入过大数据导致 native 层内存或耗时失控。

## 16. Compat 模块

`src/rapier/compat.rs` 提供偏业务和性能优化的批量接口。

主要功能：

- `world_insert_dynamic_cuboids`
- `world_insert_static_trimesh`
- `query_intersect_aabb_rigid_body_count`
- `query_intersect_aabb_rigid_bodies`

`world_insert_dynamic_cuboids` 会一次创建一个 dynamic body，并把多个 cuboid collider 挂到它下面。这适合复杂物体由很多盒子组成的场景，能减少 Java 到 native 的调用次数。

`world_insert_static_trimesh` 会创建 fixed body，再挂一个 trimesh collider。适合静态地形或场景网格。

`query_intersect_aabb_rigid_bodies` 先查 AABB 命中的 collider，再通过 collider parent 去重得到 rigid body handles。

## 17. RTree 和 CRbTree

项目自己实现了两个 AABB 空间索引，它们不依赖 Rapier 世界。

### 17.1 RTree

`src/rapier/rtree.rs` 实现了一个简单 RTree 风格索引。

特点：

- 存储 `id + AABB`。
- 插入和更新后标记 dirty。
- 查询前懒 rebuild。
- rebuild 时按最长轴排序并分块。
- 查询时递归遍历 branch 和 leaf。

它适合数据量较大、需要空间剪枝的 AABB 查询。

### 17.2 CRbTree

`src/rapier/crbtree.rs` 使用 `BTreeMap<u64, Aabb>`。

特点：

- 查询是线性扫描。
- 输出按 id 顺序稳定。
- 实现简单，确定性好。

它适合数据量较小但希望查询结果顺序稳定的场景。

## 18. 错误处理

`src/rapier/error.rs` 提供 thread-local last error。

对外 API：

- `last_error_code`
- `last_error_message`
- `last_error_clear`

错误类型包括：

- null pointer
- invalid argument
- capacity
- not found
- unsupported

需要注意的是，并不是所有 API 都会设置 last error。很多 C ABI 风格函数遇到非法参数时只是返回 `0`、`false` 或默认结构体。

## 19. 典型使用流程

### 19.1 创建动态刚体和碰撞体

```text
world_create
rigid_body_builder_create(Dynamic)
rigid_body_builder_set_translation
rigid_body_builder_build
world_insert_rigid_body

collider_builder_create(Cuboid)
collider_builder_set_density
collider_builder_set_friction
collider_builder_build
world_insert_collider_with_parent

world_step
rigid_body_get_translation
```

### 19.2 做射线检测

```text
query_cast_ray(
    world,
    origin,
    direction,
    max_toi,
    solid,
    filter
)
```

返回 `RayHit`，其中包含：

- 命中的 collider handle
- time of impact
- normal
- feature id

### 19.3 同步大量刚体状态

如果 Java 侧每个 body 都调用一次 `rigid_body_get_translation`，跨语言调用成本会很高。因此项目提供：

```text
world_body_snapshot
world_dynamic_body_snapshot
world_update_body_poses
world_update_body_velocities
```

这些接口用连续数组一次性读写多个 body 的状态。

## 20. 实现特点和注意点

1. 项目是 ABI 优先设计，接口大量使用裸指针、`u64` handle 和 `#[repr(C)]` 数据结构。

2. 物理求解核心交给 Rapier，本项目主要做跨语言封装、数据转换、批量 API 和扩展几何。

3. `0` handle 通常表示失败或空值，真实 Rapier handle 会被打包后加 `1`。

4. 输入校验比较多，包括 null pointer、NaN、Inf、负尺寸、容量上限等。

5. builder 类对象通常会被 build 函数消费，外部必须遵守生命周期，否则容易出现 use-after-free。

6. 很多扩展形状最终都会转成 convex hull，例如 ellipsoid、prism、k-DOP、FDH 和 neural bounds。

7. voxel collider 会根据模式生成 compound cuboids 或 trimesh，适合体素场景。

8. event collection 是内置收集式设计，不支持外部 callback。

9. last error 不是所有函数都会设置，调用方不能完全依赖 error code 判断所有失败原因。

10. 这个库明显针对 Java 调用性能做了优化，批量 snapshot、批量 ray、批量 cuboid 插入都是为了减少 JNI/FFM 往返次数。

## 21. 总结

这个项目可以概括为：一个面向 Java 的 Rapier 3D 双精度物理封装库。

它提供了完整的刚体物理能力：

- 世界模拟
- 刚体
- 碰撞体
- 关节
- 查询
- 事件
- 角色控制器

同时还提供了一批扩展功能：

- 体素 collider
- k-DOP / FDH
- 神经网络边界体
- 自定义 AABB 空间索引
- 批量状态同步
- 批量复杂碰撞体插入

整体设计重点不是重写物理引擎，而是把 Rapier 的能力稳定、高效地暴露给 Java，并补充一些适合业务侧使用的批量和复杂几何接口。
