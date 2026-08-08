use crate::metrics::{CORE_FFI_COUNT, FFI_COLLIDER, FFI_QUERY, FFI_RIGID_BODY, FFI_WORLD};
use topcoat::router::page;
use topcoat::view::view;

/// API reference page
#[page("/api")]
pub async fn api() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">"/ REFERENCE"</div>
                    <h1 class="page-title"><span data-lang="zh">"API 参考"</span><span data-lang="en">"API Reference"</span></h1>
                    <p class="page-desc">
                        <span data-lang="zh">"rigid_body.h 导出的 "{ (CORE_FFI_COUNT) }"+ pub extern \"C\" 函数，按子系统分组。每条同时标注 JNI（mps-jni::RigidBodyNative）/ FFM（test25 downcall）是否就绪。完整签名见 "<code>"crates/mps-core/include/rigid_body.h"</code>"（cbindgen 生成，CI generated-header gate 强提交 diff）。"</span>
                        <span data-lang="en">"Functions exported by rigid_body.h ("{ (CORE_FFI_COUNT) }"+ pub extern \"C\"), grouped by subsystem. Each marks JNI (mps-jni::RigidBodyNative) / FFM (test25 downcall) availability. Full signatures in "<code>"crates/mps-core/include/rigid_body.h"</code>" (cbindgen-generated, CI generated-header gate enforces commit)."</span>
                    </p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title">"🌍 World ("{ (FFI_WORLD) }" fns) — "<span data-lang="zh">"通用物理世界"</span><span data-lang="en">"General Physics World"</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"子组"</span><span data-lang="en">"Subgroup"</span></th><th><span data-lang="zh">"代表函数"</span><span data-lang="en">"Representative Functions"</span></th></tr></thead>
                        <tbody>
                            <tr><td><strong><span data-lang="zh">"生命周期"</span><span data-lang="en">"Lifecycle"</span></strong></td><td><code>"world_create / world_destroy / world_step"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"参数"</span><span data-lang="en">"Parameters"</span></strong></td><td><code>"world_set_gravity / world_set_integration_parameters"</code> <span data-lang="zh">"(dt/solver_iter/ccd_sub) + 对应 _get_*_out"</span><span data-lang="en">"(dt/solver_iter/ccd_sub) + matching _get_*_out"</span></td></tr>
                            <tr><td><strong><span data-lang="zh">"集大小 / 快照"</span><span data-lang="en">"Set Size / Snapshot"</span></strong></td><td><code>"world_get_rigid_body_set_size / _collider_set_size / world_body_snapshot / world_dynamic_body_snapshot"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"插删/复制"</span><span data-lang="en">"Insert/Remove/Copy"</span></strong></td><td><code>"world_insert_rigid_body / world_remove_rigid_body / world_copy_rigid_body / world_remove_collider"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"批更新"</span><span data-lang="en">"Batch Update"</span></strong></td><td><code>"world_update_body_poses / world_update_body_velocities"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"力律注册"</span><span data-lang="en">"Force-Law Registry"</span></strong></td><td><code>"world_set_coulomb_friction_law / _air_drag_law / _external_force_law / _newton_gravity_law"</code> <span data-lang="zh">"+ _clear_*/ + _get_* + world_get_force_registry_count"</span><span data-lang="en">"+ _clear_*/ + _get_* + world_get_force_registry_count"</span></td></tr>
                            <tr><td><strong><span data-lang="zh">"事件 / 回调（init-time-only）"</span><span data-lang="en">"Events / Callbacks (init-time-only)"</span></strong></td><td><code>"world_init_collision_event_ring / _drain_*/ world_register_collision_callback / world_set_event_dispatch_mode / world_unregister_callback"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"共享 Arena"</span><span data-lang="en">"Shared Arena"</span></strong></td><td><code>"world_create_shared_arena / _destroy_shared_arena / _get_shared_arena_address / _reset_shared_arena_events"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"pair filter"</span><span data-lang="en">"pair filter"</span></strong></td><td><code>"world_set_contact_pair_filter_callback / world_set_intersection_pair_filter_callback"</code> <span data-lang="zh">"+ _clear_*"</span><span data-lang="en">"+ _clear_*"</span></td></tr>
                            <tr><td><strong><span data-lang="zh">"破碎"</span><span data-lang="en">"Fracture"</span></strong></td><td><code>"world_replace_body_with_fracture_fragments"</code></td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note"><span data-lang="zh">"每条 set_*/remove_*/insert_* 都有同名 _flag bool 返回变体（=ERR_* 失败时 false）；几乎都过 JNI+FFM。"</span><span data-lang="en">"Each set_*/remove_*/insert_* has a matching _flag bool-return variant (false on ERR_* failure); most go through JNI+FFM."</span></p>
            </div>

            <div class="section-card">
                <h2 class="page-title">"🔩 Rigid Body ("{ (FFI_RIGID_BODY) }" fns)"</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"子组"</span><span data-lang="en">"Subgroup"</span></th><th><span data-lang="zh">"代表函数"</span><span data-lang="en">"Representative Functions"</span></th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Builder"</strong></td><td><code>"rigid_body_builder_create / set_translation / set_rotation / set_pose / set_linvel / set_angvel / set_additional_mass / set_gravity_scale / set_enabled_rotations / set_can_sleep"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"插入"</span><span data-lang="en">"Insert"</span></strong></td><td><code>"world_insert_rigid_body"</code><span data-lang="zh">"（返回 u64 packed handle）"</span><span data-lang="en">" (returns u64 packed handle)"</span></td></tr>
                            <tr><td><strong><span data-lang="zh">"运行时读"</span><span data-lang="en">"Runtime Read"</span></strong></td><td><code>"rigid_body_get_translation_out / get_rotation_out / get_linvel_out / get_angvel_out / get_mass / get_status / is_sleeping"</code> <span data-lang="zh">"（_out 变体把结果写 caller 缓冲）"</span><span data-lang="en">" (_out variants write result into caller buffer)"</span></td></tr>
                            <tr><td><strong><span data-lang="zh">"运行时写"</span><span data-lang="en">"Runtime Write"</span></strong></td><td><code>"rigid_body_set_translation / set_rotation / set_pose / set_linvel / set_angvel / set_status"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"力 / 冲量"</span><span data-lang="en">"Force / Impulse"</span></strong></td><td><code>"rigid_body_add_force / add_force_at_point / add_torque / apply_impulse / apply_torque_impulse / reset_force / reset_torque"</code></td></tr>
                            <tr><td><strong>"CCD / sleep"</strong></td><td><code>"rigid_body_enable_ccd / wake_up / sleep"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"销毁"</span><span data-lang="en">"Destroy"</span></strong></td><td><code>"rigid_body_destroy_raw / rigid_body_builder_destroy"</code></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">"📦 Collider ("{ (FFI_COLLIDER) }" fns)"</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"子组"</span><span data-lang="en">"Subgroup"</span></th><th><span data-lang="zh">"代表函数"</span><span data-lang="en">"Representative Functions"</span></th></tr></thead>
                        <tbody>
                            <tr><td><strong><span data-lang="zh">"Builder 形状"</span><span data-lang="en">"Builder Shapes"</span></strong></td><td><code>"collider_builder_create / _create_halfspace / _create_heightmap / _create_sphere / _create_obb / _create_convex_hull / _create_point_cloud_bounds / _create_double_bv / _create_skewed_obb / _create_voxels / _create_voxels_auto / _create_voxel_aabb / _create_voxel_obb"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"Builder 属性"</span><span data-lang="en">"Builder Properties"</span></strong></td><td><code>"collider_builder_set_density / friction / restitution / sensor / collision_groups / solver_groups / active_events / active_hooks / contact_force_event_threshold / pose / translation / rotation"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"运行时读"</span><span data-lang="en">"Runtime Read"</span></strong></td><td><code>"collider_get_translation_out / get_rotation_out / get_density"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"运行时写"</span><span data-lang="en">"Runtime Write"</span></strong></td><td><code>"collider_set_translation / set_pose / set_friction / set_restitution / set_collision_groups / set_sensor / set_active_events / set_active_hooks / set_contact_force_event_threshold"</code></td></tr>
                            <tr><td><strong><span data-lang="zh">"销毁"</span><span data-lang="en">"Destroy"</span></strong></td><td><code>"collider_destroy_raw / collider_builder_destroy"</code></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">"🔍 Query ("{ (FFI_QUERY) }" fns) — "<span data-lang="zh">"空间查询"</span><span data-lang="en">"Spatial Queries"</span></h2>
                <p class="p-lead"><span data-lang="zh">"三类几何形状的相交 / 投射，每类有 single / _count / _count_all / _all 与 _count_per_body 变体："</span><span data-lang="en">"Intersection / cast for three geometry types; each has single / _count / _count_all / _all and _count_per_body variants:"</span></p>
                <ul class="ul-plain">
                    <li><strong>"Ray cast"</strong> " — "<code>"query_cast_rays"</code></li>
                    <li><strong>"Intersect"</strong> " — "<code>"query_intersect_aabb / capsule / cylinder / ellipsoid / ball / halfspace / convex_polyline / trimesh / heightfield"</code><span data-lang="zh">"（+ rigid_bodies / _count_all 子系列）"</span><span data-lang="en">" (incl. rigid_bodies / _count_all sub-series)"</span></li>
                    <li><strong>"Project / Cast shape"</strong> <span data-lang="zh">" — 点投影、形状投射"</span><span data-lang="en">" — point projection, shape cast"</span></li>
                </ul>
            </div>

            <div class="section-card">
                <h2 class="page-title">"🛰 space* (69 fns) — "<span data-lang="zh">"纯函数航天/太空工程"</span><span data-lang="en">"Pure-Function Spaceflight"</span></h2>
                <p class="p-lead"><span data-lang="zh">"mps-formula::spaceflight 的 #[repr(C)] 出口——大多 "<strong>"纯函数"</strong>"（输入全部数据进 native，无世界状态读取）：Hohmann/Lambert、CW、SGP4-ish 变分推演、Hill-Clohessy-Wiltshire、CMG、磁力矩器、重力梯度力矩、太阳光压、气动阻力、Whipple 盾、热平衡、Sabatier/PEM、EKF 预测更新、Sagnac/GNSS...一应俱全。多数未走 JNI（体量大），仅暴露 C ABI。"</span><span data-lang="en">"mps-formula::spaceflight #[repr(C)] exports — mostly "<strong>"pure functions"</strong>" (all data passed in native, no world state reads): Hohmann/Lambert, CW, SGP4-ish variational propagation, Hill-Clohessy-Wiltshire, CMG, magnetic torquers, gravity gradient torque, solar radiation pressure, atmospheric drag, Whipple shield, thermal balance, Sabatier/PEM, EKF predict/update, Sagnac/GNSS... Most not JNI-exposed (large volume), C ABI only."</span></p>
                <p class="p-lead"><span data-lang="zh">"动态变体（接 world·body）："<code>"space_apply_atmospheric_drag_to_body / _cmg_torque / _gravity_gradient_torque / _magnetic_torquer / _solar_radiation_pressure"</code>"（+ _flag 变体）——直接将导出力加到刚体上，是航天控制律对接物理引擎的入口。"</span><span data-lang="en">"Dynamic variants (touch world·body): "<code>"space_apply_atmospheric_drag_to_body / _cmg_torque / _gravity_gradient_torque / _magnetic_torquer / _solar_radiation_pressure"</code>" (+ _flag variants) — apply exported forces to bodies; entry point for attitude-control laws to the physics engine."</span></p>
            </div>

            <div class="section-card">
                <h2 class="page-title">"🚀 "<span data-lang="zh">"高阶子系统"</span><span data-lang="en">"Advanced Subsystems"</span></h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"前缀"</span><span data-lang="en">"Prefix"</span></th><th><span data-lang="zh">"功能 / 代表函数"</span><span data-lang="en">"Function / Representative"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"aero_* (5)"</code></td><td><span data-lang="zh">"气动升阻面累加：aero_apply_surfaces / _apply_voxel_grid / _estimate_surface_force（+ _flag）"</span><span data-lang="en">"Aero lift/drag surfaces: aero_apply_surfaces / _apply_voxel_grid / _estimate_surface_force (+ _flag)"</span></td></tr>
                            <tr><td><code>"fluid_* (10)"</code></td><td><span data-lang="zh">"流体：Bernoulli 压力 + Navier–Stokes 简化步 + SPH 密度/锐度/粘度。fluid_apply_aabb_forces 接 world"</span><span data-lang="en">"Fluid: Bernoulli pressure + simplified Navier–Stokes + SPH density/sharpness/viscosity. fluid_apply_aabb_forces touches world"</span></td></tr>
                            <tr><td><code>"trajectory_* (6)"</code></td><td><span data-lang="zh">"气动外弹道：glide 估/积、integrate_step、trajectory_apply_forces_to_body"</span><span data-lang="en">"Aero exterior ballistics: glide estimate/accumulate, integrate_step, trajectory_apply_forces_to_body"</span></td></tr>
                            <tr><td><code>"molecular_* (8)"</code></td><td><span data-lang="zh">"分子动力学：Coulomb、Lennard-Jones 势/力、pair interaction。molecular_apply_pair_forces 接 world"</span><span data-lang="en">"Molecular dynamics: Coulomb, Lennard-Jones potential/force, pair interaction. molecular_apply_pair_forces touches world"</span></td></tr>
                            <tr><td><code>"terrain_* (6)"</code></td><td><span data-lang="zh">"不规则引力：DEM 高程引力 + FFT、Werner-Scheeres 多面体、月球 Mascon（GRAIL）"</span><span data-lang="en">"Irregular gravity: DEM elevation gravity + FFT, Werner-Scheeres polyhedron, lunar Mascons (GRAIL)"</span></td></tr>
                            <tr><td><code>"fracture_* (6)"</code></td><td><span data-lang="zh">"断裂力学：Griffith、S-N 寿命、应力强度因子、Miner 损伤 + world_replace_body_with_fracture_fragments"</span><span data-lang="en">"Fracture mechanics: Griffith, S-N life, stress intensity factor, Miner damage + world_replace_body_with_fracture_fragments"</span></td></tr>
                            <tr><td><code>"material_* (3)"</code></td><td><span data-lang="zh">"材料力学：Hertz 接触、应力应变线性、弹性碰撞相对速度"</span><span data-lang="en">"Material mechanics: Hertz contact, stress-strain linear, elastic collision relative velocity"</span></td></tr>
                            <tr><td><code>"anvilkit_* (13)"</code></td><td><span data-lang="zh">"AnvilKit 桥（anvilkit-bridge 特性）：Minecraft 载入结构 → 物理世界；entity↔body/collider、约束、应用到基准世界"</span><span data-lang="en">"AnvilKit bridge (anvilkit-bridge feature): Minecraft load structures → physics world; entity↔body/collider, constraints, apply to benchmark world"</span></td></tr>
                            <tr><td><code>"character_* (10)"</code></td><td>"KinematicCharacterController: offset/slide/autostep/slope/snap/up + solve_impulses + collision_count/get"</td></tr>
                            <tr><td><code>"joint_builder_* (5)"</code></td><td><span data-lang="zh">"关节构建：motor position/velocity、limits、contacts_enabled"</span><span data-lang="en">"Joint construction: motor position/velocity, limits, contacts_enabled"</span></td></tr>
                            <tr><td><code>"voxel_* (5)"</code></td><td><span data-lang="zh">"体素构建统计：voxel_aabb/obb_build_stats(_out)、voxel_build_stats（详见 "</span><span data-lang="en">"Voxel build stats: voxel_aabb/obb_build_stats(_out), voxel_build_stats (see "</span><a href="./voxel" class="link">"voxel"</a><span data-lang="zh">"）"</span><span data-lang="en">")"</span></td></tr>
                            <tr><td><code>"rtree_* / crb_tree_* (18)"</code></td><td><span data-lang="zh">"空间索引 / 碰撞体加速树：insert/update/remove/query_aabb/rebuild/len/clear/destroy"</span><span data-lang="en">"Spatial index / collider accel tree: insert/update/remove/query_aabb/rebuild/len/clear/destroy"</span></td></tr>
                            <tr><td><code>"last_error_* (2)"</code></td><td><span data-lang="zh">"last-error 线程槽："<code>"last_error_clear / last_error_code"</code>"；message 经 "<code>"abiLastErrorMessage"</code>"（JNI）/ "<code>"error_message"</code>"（FFM）"</span><span data-lang="en">"last-error thread slot: "<code>"last_error_clear / last_error_code"</code>"; message via "<code>"abiLastErrorMessage"</code>" (JNI) / "<code>"error_message"</code>" (FFM)"</span></td></tr>
                            <tr><td><code>"neural_* (1)"</code></td><td><span data-lang="zh">"网络辅助：neural_bounds_required_weight_count"</span><span data-lang="en">"Network assist: neural_bounds_required_weight_count"</span></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">"命名约定"</span><span data-lang="en">"Naming Conventions"</span></h2>
                <ul class="ul-plain">
                    <li><strong>"_flag suffix"</strong> <span data-lang="zh">" — 同名带 bool 返回；不带 flag 版本失败静默（仅写 last-error），带 flag 版本显式返回成功。"</span><span data-lang="en">" — bool-returning variant of the same name; non-flag failures silently write last-error; flag variants return success explicitly."</span></li>
                    <li><strong>"_out suffix"</strong> <span data-lang="zh">" — 读路径把结果写 caller 提供的缓冲（Vec3* 等），避免 FFI 结构体返回的 ABI 风险。"</span><span data-lang="en">" — read path writes results into caller-supplied buffer (Vec3* etc.), avoiding FFI struct-return ABI risk."</span></li>
                    <li><strong>"_count / _count_all"</strong> <span data-lang="zh">" — 单步查询返回数量；all 后缀一次拉全部。"</span><span data-lang="en">" — single-step query returns count; _all suffix pulls all at once."</span></li>
                    <li><strong>"builder_* / world_* / runtime_* 分层"</strong> <span data-lang="zh">" — builder 阶段 → build → world 嵌入 → 运行时 set_/get_。"</span><span data-lang="en">" — builder phase → build → world embed → runtime set_/get_."</span></li>
                </ul>
            </div>

            <div class="callout">
                <p><span data-lang="zh">" 完整 API 与每个函数的精确签名/对齐/生命周期都在 "<code>"crates/mps-core/include/rigid_body.h"</code>" —— 由 cbindgen 在 "<code>"mps-core/build.rs"</code>" 生成。CI 强制该文件必须随源码改动同步提交（generated-header gate）。此页只给地图，rigid_body.h 给契约。"</span><span data-lang="en">" Full API and per-function signature/alignment/lifecycle in "<code>"crates/mps-core/include/rigid_body.h"</code>" — generated by cbindgen in "<code>"mps-core/build.rs"</code>". CI enforces the file is committed alongside source changes (generated-header gate). This page is a map; rigid_body.h is the contract."</span></p>
            </div>
        </div>
    }
}
