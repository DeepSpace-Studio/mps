use topcoat::router::page;
use topcoat::view::view;

/// Voxel colliders page
#[page("/voxel")]
pub async fn voxel() -> topcoat::Result {
    view! {
        <div>
            <div class="page-head">
                <div>
                    <div class="page-tag">
                        "/ VOXEL"
                    </div>
                    <h1 class="page-title"><span data-lang="zh">{ "体素碰撞体（大型场景 / 程序化结构）" }</span><span data-lang="en">{ "Voxel Colliders (Large Scenes / Procedural Structures)" }</span></h1>
                    <p class="page-desc"><span data-lang="zh">{ "把 uint8 体素流编译成 Rapier 复合碰撞体 —— 三种构造模式按规模自适应，支持 DirectByteBuffer 直接内存零拷贝传送。Minecraft 规模地形、AnvilKit 载入结构都用这条路。" }</span><span data-lang="en">{ "Compile uint8 voxel streams into Rapier composite colliders — 3 construction modes auto-adapt by scale, DirectByteBuffer direct-memory zero-copy transfer. Minecraft-scale terrain and AnvilKit loaded structures use this path." }</span></p>
                </div>
                <div class="page-index">"01"</div>
            </div>

            <div class="section-card">
                <h2 class="page-title"><span data-lang="zh">{ "VoxelGrid 数据契约" }</span><span data-lang="en">{ "VoxelGrid Data Contract" }</span></h2>
                <p class="p-lead">{ <span data-lang="zh">"MPS 体素是 "</span><span data-lang="en">"MPS voxels are "</span> }<strong><span data-lang="zh">"密集uint8数组"</span><span data-lang="en">"dense uint8 array"</span></strong>{ <span data-lang="zh">"：每字节一格 0=空 / 非0=实心，三轴尺寸独立。索引顺序为 "</span><span data-lang="en">": one byte per voxel, 0=empty / non-0=solid, three axes independent. Index order "</span> }<code>"index = y · (size_x · size_z) + z · size_x + x"</code>{ <span data-lang="zh">"（y 行主）。每格物理尺寸 voxel_size_xyz 各轴独立，origin 给网格左下角世界坐标。"</span><span data-lang="en">" (y-major). Voxel physical size voxel_size_xyz independent per axis, origin gives the world coordinate of the grid's bottom-left corner."</span> }</p>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"字段"</span><span data-lang="en">"Field"</span></th><th><span data-lang="zh">"类型"</span><span data-lang="en">"Type"</span></th><th><span data-lang="zh">"含义"</span><span data-lang="en">"Meaning"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"voxels"</code></td><td>"const uint8_t*"</td><td>{ <span data-lang="zh">"size_x * size_y * size_z 字节，跨调用期间须可读"</span><span data-lang="en">"size_x * size_y * size_z bytes, must be readable across calls"</span> }</td></tr>
                            <tr><td><code>"size_x/y/z"</code></td><td>"uint32_t"</td><td>{ <span data-lang="zh">"三轴体素数；总字节数若超 u32 范围 → ERR_CAPACITY"</span><span data-lang="en">"Voxel count on three axes; total bytes exceeding u32 range → ERR_CAPACITY"</span> }</td></tr>
                            <tr><td><code>"voxel_size_x/y/z"</code></td><td>"double"</td><td>{ <span data-lang="zh">"每格米数；须有限正数，否则 ERR_INVALID_ARGUMENT"</span><span data-lang="en">"Meters per voxel; must be finite positive, else ERR_INVALID_ARGUMENT"</span> }</td></tr>
                            <tr><td><code>"origin"</code></td><td>"Vec3"</td><td>{ <span data-lang="zh">"网格 (0,0,0) 格的世界坐标"</span><span data-lang="en">"World coordinate of grid (0,0,0) cell"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"三种构造模式 (VoxelColliderMode)"</span><span data-lang="en">"Three Construction Modes (VoxelColliderMode)"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"模式"</span><span data-lang="en">"Mode"</span></th><th><span data-lang="zh">"内部形态"</span><span data-lang="en">"Internal Form"</span></th><th><span data-lang="zh">"适用规模 / 代价"</span><span data-lang="en">"Scale / Cost"</span></th><th><span data-lang="zh">"用途"</span><span data-lang="en">"Purpose"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"Cuboids"</code> { "(1)" }</td><td>{ <span data-lang="zh">"每实心格一个 AABB cuboid"</span><span data-lang="en">"One AABB cuboid per solid cell"</span> }</td><td>{ <span data-lang="zh">"small_voxel_limit 以下；最简最直"</span><span data-lang="en">"Below small_voxel_limit; simplest, most direct"</span> }</td><td>{ <span data-lang="zh">"小型结构 / 调试"</span><span data-lang="en">"Small structures / debug"</span> }</td></tr>
                            <tr><td><code>"GreedyCuboids"</code> { "(2)" }</td><td>{ <span data-lang="zh">"同面同向格合并为大 cuboid（贪心 meshing）"</span><span data-lang="en">"Merges same-face same-direction cells into one cuboid (greedy meshing)"</span> }</td><td>{ <span data-lang="zh">"中等规模；part 数 ×~4↓"</span><span data-lang="en">"Mid scale; part count ×~4 lower"</span> }</td><td>{ <span data-lang="zh">"中等动态体"</span><span data-lang="en">"Mid-size dynamic bodies"</span> }</td></tr>
                            <tr><td><code>"SurfaceMesh"</code> { "(3)" }</td><td>{ <span data-lang="zh">"只建表面三角网格 trimesh"</span><span data-lang="en">"Builds only surface triangle mesh trimesh"</span> }</td><td>{ <span data-lang="zh">"≥ mesh_voxel_limit 大规模；静态"</span><span data-lang="en">"≥ mesh_voxel_limit large-scale; static"</span> }</td><td>{ <span data-lang="zh">"大型静态地形（Minecraft 区块、AnvilKit 结构）"</span><span data-lang="en">"Large static terrain (Minecraft chunks, AnvilKit structures)"</span> }</td></tr>
                            <tr><td><code>"Auto"</code> { "(0)" }</td><td>{ <span data-lang="zh">"按 solid_count / dynamic_body / 阈值自动选"</span><span data-lang="en">"Auto-selected by solid_count / dynamic_body / thresholds"</span> }</td><td>{ <span data-lang="zh">"solid ≤ small_voxel_limit → Cuboids；动态且 > limit → Greedy；≥ mesh_voxel_limit → SurfaceMesh"</span><span data-lang="en">"solid ≤ small_voxel_limit → Cuboids; dynamic and > limit → Greedy; ≥ mesh_voxel_limit → SurfaceMesh"</span> }</td><td>{ <span data-lang="zh">"默认推荐"</span><span data-lang="en">"Default recommended"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"阈值在 "</span><span data-lang="en">"Thresholds passed in "</span> }<code>"VoxelColliderOptions {{ mode, dynamic_body, small_voxel_limit, mesh_voxel_limit }}"</code>{ <span data-lang="zh">" 传。"</span><span data-lang="en">"."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"C ABI 构造入口"</span><span data-lang="en">"C ABI Construction Entry Points"</span> }</h2>
                <div class="table-wrap">
                    <table>
                        <thead><tr><th><span data-lang="zh">"函数"</span><span data-lang="en">"Function"</span></th><th><span data-lang="zh">"用途"</span><span data-lang="en">"Purpose"</span></th></tr></thead>
                        <tbody>
                            <tr><td><code>"collider_builder_create_voxels"</code></td><td>{ <span data-lang="zh">"建 ColliderBuilder（入 ColliderSet 前可继续配置 mass/friction）"</span><span data-lang="en">"Create ColliderBuilder (still configurable mass/friction before entering ColliderSet)"</span> }</td></tr>
                            <tr><td><code>"collider_builder_create_voxels_auto"</code></td><td>{ <span data-lang="zh">"同上，强制 Auto 模式（内部调 choose_mode）"</span><span data-lang="en">"Same, but force Auto mode (internally calls choose_mode)"</span> }</td></tr>
                            <tr><td><code>"voxel_build_stats"</code></td><td>{ <span data-lang="zh">"只统计 VoxelBuildStats 不建体（预演 mode/代价）"</span><span data-lang="en">"Only compute VoxelBuildStats without building (preview mode/cost)"</span> }</td></tr>
                            <tr><td><code>"voxel_aabb_build_stats / voxel_obb_build_stats"</code></td><td>{ <span data-lang="zh">"给定 AABB/OBB 自动体素化再统计（无需外部体素流）"</span><span data-lang="en">"Auto-voxelize given AABB/OBB and compute stats (no external voxel stream needed)"</span> }</td></tr>
                            <tr><td><code>"collider_builder_create_voxels_packed"</code></td><td>{ <span data-lang="zh">"建实心体并把 "</span><span data-lang="en">"Build solid body and write "</span> }<code>"VoxelBuildStats"</code>{ <span data-lang="zh">" 写回 out_stats（一次拿配置回执）"</span><span data-lang="en">" into out_stats (returns a config receipt in one call)"</span> }</td></tr>
                        </tbody>
                    </table>
                </div>
                <p class="p-note-top14">{ <span data-lang="zh">"Java 侧 DirectByteBuffer/Slice 与 native 共享内存 —— "</span><span data-lang="en">"Java-side DirectByteBuffer/Slice shares native memory — "</span> }<code>"voxel_collider_from_direct_buffer(voxel_address: i64, ...)"</code>{ <span data-lang="zh">" 接 ByteBuffer 直接地址，零拷贝。地址失效再调会读乱，文档明确要求「跨调用期间可读」。"</span><span data-lang="en">" accepts the ByteBuffer direct address, zero-copy. Calls after the address becomes invalid will read garbage; docs explicitly require \"readable across calls\"."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"VoxelBuildStats 回执"</span><span data-lang="en">"VoxelBuildStats Report"</span> }</h2>
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
                <p class="p-note">{ <span data-lang="zh">"失败时（ERR_INVALID_ARGUMENT / ERR_CAPACITY）统一返回零化 stats，调用方仅看 error_code 即可。建体函数失败返回 null ColliderBuilderHandle。"</span><span data-lang="en">"On failure (ERR_INVALID_ARGUMENT / ERR_CAPACITY) returns uniform zeroed stats; caller only needs error_code. Build functions return null ColliderBuilderHandle on failure."</span> }</p>
            </div>

            <div class="section-card">
                <h2 class="page-title">{ <span data-lang="zh">"AnvilKit 桥（anvilkit-bridge 特性）"</span><span data-lang="en">"AnvilKit Bridge (anvilkit-bridge feature)"</span> }</h2>
                <p class="p-lead">{ <span data-lang="zh">"默认构建不含 AnvilKit；启用 "</span><span data-lang="en">"Default build excludes AnvilKit; enabling "</span> }<code>"--features anvilkit-bridge"</code>{ <span data-lang="zh">" 后，"</span><span data-lang="en">" makes "</span> }<code>"anvilkit_app_apply_aero_voxel_grid"</code>{ <span data-lang="zh">" 把体素网格既作碰撞体又作气动源——同一体素既是几何又是迎风面，"</span><span data-lang="en">" treat the voxel grid as both collider and aero source — same voxels are both geometry and windward surface; "</span> }<a href="./events" class="link"><span data-lang="zh">"事件系统"</span><span data-lang="en">"event system"</span></a>{ <span data-lang="zh">" 驱动每帧力更新。详见 "</span><span data-lang="en">" drives per-frame force updates. See "</span> }<code>"--features anvilkit-bridge"</code>{ <span data-lang="zh">" 构建说明。"</span><span data-lang="en">" build instructions."</span> }</p>
            </div>

            <div class="callout">
                <p>{ <span data-lang="zh">" 性能权衡：Cuboids 对碰撞查询最直接但 part 数爆炸（百格千 part）；GreedyCuboids 合并后适合中等动态体；SurfaceMesh trimesh 仅适合静态（Rapier 不支持动态 trimesh 反向解析）。Auto 模式按规模自动切换，绝大多数情况留 Auto 即可。"</span><span data-lang="en">" Performance trade-off: Cuboids is the most direct for collision query but part count explodes (hundreds of cells → thousands of parts); GreedyCuboids merges, suited to mid-size dynamic bodies; SurfaceMesh trimesh is static-only (Rapier doesn't support reverse parsing of dynamic trimesh). Auto mode auto-switches by scale — leave it on Auto in most cases."</span> }</p>
            </div>
        </div>
    }
}
