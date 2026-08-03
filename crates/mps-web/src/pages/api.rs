use topcoat::router::page;
use topcoat::view::view;

/// API reference page
#[page("/api")]
pub async fn api() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ REFERENCE"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"API 参考"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">{ "rigid_body.h 导出的 373+ pub extern \"C\" 函数，按子系统分组。每条同时标注 JNI（mps-jni::RigidBodyNative）/ FFM（test25 downcall）是否就绪。完整签名见 " }<code>"crates/mps-core/include/rigid_body.h"</code>{ "（cbindgen 生成，CI generated-header gate 强提交 diff）。" }</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "🌍 World (66 fns) — 通用物理世界" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"子组"</th><th>"代表函数"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"生命周期"</strong></td><td><code>"world_create / world_destroy / world_step"</code></td></tr>
                            <tr><td><strong>"参数"</strong></td><td><code>"world_set_gravity / world_set_integration_parameters"</code> { "(dt/solver_iter/ccd_sub) + 对应 _get_*_out" }</td></tr>
                            <tr><td><strong>"集大小 / 快照"</strong></td><td><code>"world_get_rigid_body_set_size / _collider_set_size / world_body_snapshot / world_dynamic_body_snapshot"</code></td></tr>
                            <tr><td><strong>"插删/复制"</strong></td><td><code>"world_insert_rigid_body / world_remove_rigid_body / world_copy_rigid_body / world_remove_collider"</code></td></tr>
                            <tr><td><strong>"批更新"</strong></td><td><code>"world_update_body_poses / world_update_body_velocities"</code></td></tr>
                            <tr><td><strong>"力律注册"</strong></td><td><code>"world_set_coulomb_friction_law / _air_drag_law / _external_force_law / _newton_gravity_law"</code> { "+ _clear_*/ + _get_* + world_get_force_registry_count" }</td></tr>
                            <tr><td><strong>"事件 / 回调（init-time-only）"</strong></td><td><code>"world_init_collision_event_ring / _drain_*/ world_register_collision_callback / world_set_event_dispatch_mode / world_unregister_callback"</code></td></tr>
                            <tr><td><strong>"共享 Arena"</strong></td><td><code>"world_create_shared_arena / _destroy_shared_arena / _get_shared_arena_address / _reset_shared_arena_events"</code></td></tr>
                            <tr><td><strong>"pair filter"</strong></td><td><code>"world_set_contact_pair_filter_callback / world_set_intersection_pair_filter_callback"</code> { "+ _clear_*" }</td></tr>
                            <tr><td><strong>"破碎"</strong></td><td><code>"world_replace_body_with_fracture_fragments"</code></td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "每条 set_*/remove_*/insert_* 都有同名 _flag bool 返回变体（=ERR_* 失败时 false）；几乎都过 JNI+FFM。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "🔩 Rigid Body (51 fns)" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"子组"</th><th>"代表函数"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Builder"</strong></td><td><code>"rigid_body_builder_create / set_translation / set_rotation / set_pose / set_linvel / set_angvel / set_additional_mass / set_gravity_scale / set_enabled_rotations / set_can_sleep"</code></td></tr>
                            <tr><td><strong>"插入"</strong></td><td><code>"world_insert_rigid_body"</code>{ "（返回 u64 packed handle）" }</td></tr>
                            <tr><td><strong>"运行时读"</strong></td><td><code>"rigid_body_get_translation_out / get_rotation_out / get_linvel_out / get_angvel_out / get_mass / get_status / is_sleeping"</code> { "（_out 变体把结果写 caller 缓冲）" }</td></tr>
                            <tr><td><strong>"运行时写"</strong></td><td><code>"rigid_body_set_translation / set_rotation / set_pose / set_linvel / set_angvel / set_status"</code></td></tr>
                            <tr><td><strong>"力 / 冲量"</strong></td><td><code>"rigid_body_add_force / add_force_at_point / add_torque / apply_impulse / apply_torque_impulse / reset_force / reset_torque"</code></td></tr>
                            <tr><td><strong>"CCD / 睡眠"</strong></td><td><code>"rigid_body_enable_ccd / wake_up / sleep"</code></td></tr>
                            <tr><td><strong>"销毁"</strong></td><td><code>"rigid_body_destroy_raw / rigid_body_builder_destroy"</code></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "📦 Collider (37 fns)" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"子组"</th><th>"代表函数"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Builder 形状"</strong></td><td><code>"collider_builder_create / _create_halfspace / _create_heightmap / _create_sphere / _create_obb / _create_convex_hull / _create_point_cloud_bounds / _create_double_bv / _create_skewed_obb / _create_voxels / _create_voxels_auto / _create_voxel_aabb / _create_voxel_obb"</code></td></tr>
                            <tr><td><strong>"Builder 属性"</strong></td><td><code>"collider_builder_set_density / friction / restitution / sensor / collision_groups / solver_groups / active_events / active_hooks / contact_force_event_threshold / pose / translation / rotation"</code></td></tr>
                            <tr><td><strong>"运行时读"</strong></td><td><code>"collider_get_translation_out / get_rotation_out / get_density"</code></td></tr>
                            <tr><td><strong>"运行时写"</strong></td><td><code>"collider_set_translation / set_pose / set_friction / set_restitution / set_collision_groups / set_sensor / set_active_events / set_active_hooks / set_contact_force_event_threshold"</code></td></tr>
                            <tr><td><strong>"销毁"</strong></td><td><code>"collider_destroy_raw / collider_builder_destroy"</code></td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "🔍 Query (52 fns) — 空间查询" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "三类几何形状的相交 / 投射，每类有 single / _count / _count_all / _all 与 _count_per_body 变体：" }</p>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong>"Ray cast"</strong> { " — " }<code>"query_cast_rays"</code></li>
                    <li><strong>"Intersect"</strong> { " — " }<code>"query_intersect_aabb / capsule / cylinder / ellipsoid / ball / halfspace / convex_polyline / trimesh / heightfield"</code>{ "（+ rigid_bodies / _count_all 子系列）" }</li>
                    <li><strong>"Project / Cast shape"</strong> { " — 点投影、形状投射" }</li>
                </ul>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "🛰 space* (69 fns) — 纯函数航天/太空工程" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "mps-formula::spaceflight 的 #[repr(C)] 出口——大多 " }<strong>"纯函数"</strong>{ "（输入全部数据进 native，无世界状态读取）：Hohmann/Lambert、CW、SGP4-ish 变分推演、Hill-Clohessy-Wiltshire、CMG、磁力矩器、重力梯度力矩、太阳光压、气动阻力、Whipple 盾、热平衡、Sabatier/PEM、EKF 预测更新、Sagnac/GNSS...一应俱全。多数未走 JNI（体量大），仅暴露 C ABI。" }</p>
                <p style="color:#aaa;line-height:1.7;">{ "动态变体（接 world·body）：" }<code>"space_apply_atmospheric_drag_to_body / _cmg_torque / _gravity_gradient_torque / _magnetic_torquer / _solar_radiation_pressure"</code>{ "（+ _flag 变体）——直接将导出力加到刚体上，是航天控制律对接物理引擎的入口。" }</p>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "🚀 高阶子系统" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"前缀"</th><th>"功能 / 代表函数"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"aero_* (5)"</code></td><td>{ "气动升阻面累加：aero_apply_surfaces / _apply_voxel_grid / _estimate_surface_force（+ _flag）" }</td></tr>
                            <tr><td><code>"fluid_* (10)"</code></td><td>{ "流体：Bernoulli 压力 + Navier–Stokes 简化步 + SPH 密度/锐度/粘度。fluid_apply_aabb_forces 接 world" }</td></tr>
                            <tr><td><code>"trajectory_* (6)"</code></td><td>{ "气动外弹道：glide 估/积、integrate_step、trajectory_apply_forces_to_body" }</td></tr>
                            <tr><td><code>"molecular_* (8)"</code></td><td>{ "分子动力学：Coulomb、Lennard-Jones 势/力、pair interaction。molecular_apply_pair_forces 接 world" }</td></tr>
                            <tr><td><code>"terrain_* (6)"</code></td><td>{ "不规则引力：DEM 高程引力 + FFT、Werner-Scheeres 多面体、月球 Mascon（GRAIL）" }</td></tr>
                            <tr><td><code>"fracture_* (6)"</code></td><td>{ "断裂力学：Griffith、S-N 寿命、应力强度因子、Miner 损伤 + world_replace_body_with_fracture_fragments" }</td></tr>
                            <tr><td><code>"material_* (3)"</code></td><td>{ "材料力学：Hertz 接触、应力应变线性、弹性碰撞相对速度" }</td></tr>
                            <tr><td><code>"anvilkit_* (13)"</code></td><td>{ "AnvilKit 桥（anvilkit-bridge 特性）：Minecraft 载入结构 → 物理世界；entity↔body/collider、约束、应用到基准世界" }</td></tr>
                            <tr><td><code>"character_* (10)"</code></td><td>{ "KinematicCharacterController：offset/slide/autostep/slope/snap/up + solve_impulses + collision_count/get" }</td></tr>
                            <tr><td><code>"joint_builder_* (5)"</code></td><td>{ "关节构建：motor position/velocity、limits、contacts_enabled" }</td></tr>
                            <tr><td><code>"voxel_* (5)"</code></td><td>{ "体素构建统计：voxel_aabb/obb_build_stats(_out)、voxel_build_stats（详见 " }<a href="./voxel" style="color:#4a9eff;">"voxel"</a>{ "）" }</td></tr>
                            <tr><td><code>"rtree_* / crb_tree_* (18)"</code></td><td>{ "空间索引 / 碰撞体加速树：insert/update/remove/query_aabb/rebuild/len/clear/destroy" }</td></tr>
                            <tr><td><code>"last_error_* (2)"</code></td><td>{ "last-error 线程槽：" }<code>"last_error_clear / last_error_code"</code>{ "；message 经 " }<code>"abiLastErrorMessage"</code>{ "（JNI）/ " }<code>"error_message"</code>{ "（FFM）" }</td></tr>
                            <tr><td><code>"neural_* (1)"</code></td><td>{ "网络辅助：neural_bounds_required_weight_count" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "命名约定" }</h2>
                <ul style="color:#999;line-height:2;padding-left:20px;">
                    <li><strong>"_flag 后缀"</strong> { " — 同名带 bool 返回；不带 flag 版本失败静默（仅写 last-error），带 flag 版本显式返回成功。" }</li>
                    <li><strong>"_out 后缀"</strong> { " — 读路径把结果写 caller 提供的缓冲（Vec3* 等），避免 FFI 结构体返回的 ABI 风险。" }</li>
                    <li><strong>"_count / _count_all"</strong> { " — 单步查询返回数量；all 后缀一次拉全部。" }</li>
                    <li><strong>"builder_* / world_* / runtime_* 分层"</strong> { " — builder 阶段 → build → world 嵌入 → 运行时 set_/get_。" }</li>
                </ul>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 完整 API 与每个函数的精确签名/对齐/生命周期都在 " }<code>"crates/mps-core/include/rigid_body.h"</code>{ " —— 由 cbindgen 在 " }<code>"mps-core/build.rs"</code>{ " 生成。CI 强制该文件必须随源码改动同步提交（generated-header gate）。此页只给地图，rigid_body.h 给契约。" }</p>
            </div>
        </div>
    }
}
