use topcoat::router::page;
use topcoat::view::view;

/// Voxel colliders page
#[page("/voxel")]
pub async fn voxel() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ VOXEL"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">{ "体素碰撞体（大型场景 / 程序化结构）" }</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "把 uint8 体素流编译成 Rapier 复合碰撞体 —— 三种构造模式按规模自适应，支持 DirectByteBuffer 直接内存零拷贝传送。Minecraft 规模地形、AnvilKit 载入结构都用这条路。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "VoxelGrid 数据契约" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "MPS 体素是 " }<strong>"密集uint8数组"</strong>{ "：每字节一格 0=空 / 非0=实心，三轴尺寸独立。索引顺序为 " }<code>"index = y · (size_x · size_z) + z · size_x + x"</code>{ "（y 行主）。每格物理尺寸 voxel_size_xyz 各轴独立，origin 给网格左下角世界坐标。" }</p>
                <div style="overflow-x:auto;margin-top:14px;">
                    <table>
                        <thead><tr><th>"字段"</th><th>"类型"</th><th>"含义"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"voxels"</code></td><td>"const uint8_t*"</td><td>{ "size_x * size_y * size_z 字节，跨调用期间须可读" }</td></tr>
                            <tr><td><code>"size_x/y/z"</code></td><td>"uint32_t"</td><td>{ "三轴体素数；总字节数若超 u32 范围 → ERR_CAPACITY" }</td></tr>
                            <tr><td><code>"voxel_size_x/y/z"</code></td><td>"double"</td><td>{ "每格米数；须有限正数，否则 ERR_INVALID_ARGUMENT" }</td></tr>
                            <tr><td><code>"origin"</code></td><td>"Vec3"</td><td>{ "网格 (0,0,0) 格的世界坐标" }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "三种构造模式 (VoxelColliderMode)" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"模式"</th><th>"内部形态"</th><th>"适用规模 / 代价"</th><th>"用途"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"Cuboids"</code> { "(1)" }</td><td>{ "每实心格一个 AABB cuboid" }</td><td>{ "small_voxel_limit 以下；最简最直" }</td><td>{ "小型结构 / 调试" }</td></tr>
                            <tr><td><code>"GreedyCuboids"</code> { "(2)" }</td><td>{ "同面同向格合并为大 cuboid（贪心 meshing）" }</td><td>{ "中等规模；part 数 ×~4↓" }</td><td>{ "中等动态体" }</td></tr>
                            <tr><td><code>"SurfaceMesh"</code> { "(3)" }</td><td>{ "只建表面三角网格 trimesh" }</td><td>{ "≥ mesh_voxel_limit 大规模；静态" }</td><td>{ "大型静态地形（Minecraft 区块、AnvilKit 结构）" }</td></tr>
                            <tr><td><code>"Auto"</code> { "(0)" }</td><td>{ "按 solid_count / dynamic_body / 阈值自动选" }</td><td>{ "solid ≤ small_voxel_limit → Cuboids；动态且 > limit → Greedy；≥ mesh_voxel_limit → SurfaceMesh" }</td><td>{ "默认推荐" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "阈值在 " }<code>"VoxelColliderOptions {{ mode, dynamic_body, small_voxel_limit, mesh_voxel_limit }}"</code>{ " 传。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "C ABI 构造入口" }</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"函数"</th><th>"用途"</th></tr></thead>
                        <tbody>
                            <tr><td><code>"collider_builder_create_voxels"</code></td><td>{ "建 ColliderBuilder（入 ColliderSet 前可继续配置 mass/friction）" }</td></tr>
                            <tr><td><code>"collider_builder_create_voxels_auto"</code></td><td>{ "同上，强制 Auto 模式（内部调 choose_mode）" }</td></tr>
                            <tr><td><code>"voxel_build_stats"</code></td><td>{ "只统计 VoxelBuildStats 不建体（预演 mode/代价）" }</td></tr>
                            <tr><td><code>"voxel_aabb_build_stats / voxel_obb_build_stats"</code></td><td>{ "给定 AABB/OBB 自动体素化再统计（无需外部体素流）" }</td></tr>
                            <tr><td><code>"collider_builder_create_voxels_packed"</code></td><td>{ "建实心体并把 " }<code>"VoxelBuildStats"</code>{ " 写回 out_stats（一次拿配置回执）" }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p style="color:#aaa;line-height:1.7;margin-top:14px;">{ "Java 侧 DirectByteBuffer/Slice 与 native 共享内存 —— " }<code>"voxel_collider_from_direct_buffer(voxel_address: i64, ...)"</code>{ " 接 ByteBuffer 直接地址，零拷贝。地址失效再调会读乱，文档明确要求「跨调用期间可读」。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "VoxelBuildStats 回执" }</h2>
                <pre><code class="language-c">
"struct VoxelBuildStats {
    uint32_t cell_count;        // 全部体素格数
    uint32_t solid_count;       // 实心格
    uint32_t selected_mode;     // 实际走的 mode（解读 Auto 时关键）
    uint32_t estimated_parts;   // 预期复合体 part 数
    uint32_t estimated_vertices; // 仅 SurfaceMesh 有意义
    uint32_t estimated_triangles;
    uint32_t size_x, size_y, size_z; // 回显
};"
                </code></pre>
                <p style="color:#aaa;line-height:1.7;margin-top:8px;">{ "失败时（ERR_INVALID_ARGUMENT / ERR_CAPACITY）统一返回零化 stats，调用方仅看 error_code 即可。建体函数失败返回 null ColliderBuilderHandle。" }</p>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">{ "AnvilKit 桥（anvilkit-bridge 特性）" }</h2>
                <p style="color:#aaa;line-height:1.7;">{ "默认构建不含 AnvilKit；启用 " }<code>"--features anvilkit-bridge"</code>{ " 后，" }<code>"anvilkit_app_apply_aero_voxel_grid"</code>{ " 把体素网格既作碰撞体又作气动源——同一体素既是几何又是迎风面，" }<a href="./events" style="color:#4a9eff;">"事件系统"</a>{ " 驱动每帧力更新。详见 " }<code>"--features anvilkit-bridge"</code>{ " 构建说明。" }</p>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ " 性能权衡：Cuboids 对碰撞查询最直接但 part 数爆炸（百格千 part）；GreedyCuboids 合并后适合中等动态体；SurfaceMesh trimesh 仅适合静态（Rapier 不支持动态 trimesh 反向解析）。Auto 模式按规模自动切换，绝大多数情况留 Auto 即可。" }</p>
            </div>
        </div>
    }
}
