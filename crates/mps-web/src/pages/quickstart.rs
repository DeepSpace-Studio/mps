use topcoat::router::page;
use topcoat::view::view;

/// Quickstart guide
#[page("/quickstart")]
pub async fn quickstart() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:30px; padding-bottom:20px; border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px; color:#4a9eff; letter-spacing:3px; text-transform:uppercase; font-family:monospace; margin-bottom:8px;">
                        "/ QUICKSTART"
                    </div>
                    <h1 style="font-size:28px; font-weight:300; color:#fff; margin:0 0 10px;">"快速入门"</h1>
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">"从零开始使用 MPS 物理引擎 — 三行代码运行完整物理模拟。"</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"1"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"创建世界"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">"使用 world_create 函数创建物理世界，设置重力向量。返回世界指针，所有后续操作均基于此指针。"</p>
                    <pre><code class="language-rust">
"let world = world_create(Vec3 { x: 0.0, y: -9.81, z: 0.0 });"
                    </code></pre>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"2"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"配置积分参数"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">"设置时间步长、求解器迭代次数和 CCD 子步数，平衡精度与性能。"</p>
                    <pre><code class="language-rust">
"world_set_integration_parameters(world, 1.0 / 120.0, 8, 2);"
                    </code></pre>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"3"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"插入刚体与碰撞体"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">"创建刚体构建器，配置位置、速度、形状，然后插入世界。"</p>
                    <pre><code class="language-rust">
"let body = rigid_body_builder_create(RigidBodyType::Dynamic);
rigid_body_builder_set_pos(body, 0.0, 10.0, 0.0);
let collider = collider_builder_create(ColliderShape::Cuboid);
collider_builder_set_half_extents(collider, 0.5, 0.5, 0.5);
let handle = world_insert_rigid_body(world, body, collider);"
                    </code></pre>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"4"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"运行模拟循环"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">"在循环中调用 world_step 推进模拟，每次步进更新所有刚体和碰撞体状态。"</p>
                    <pre><code class="language-rust">
"loop {
    world_step(world, 1.0 / 60.0);
    // 读取刚体位置
    let mut pos = Vec3::default();
    rigid_body_get_position(world, handle, &mut pos);
    println!(\"y = {}\", pos.y);
}"
                    </code></pre>
                </div>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p><strong>"提示："</strong> "所有 API 均通过 C FFI 暴露，Java 开发者可通过 JNI 或 FFM 调用。Java 代码中可使用 " <span class="hi" style="color:#4a9eff; font-family:monospace;">"RigidBodyNative"</span> " 或 " <span class="hi" style="color:#4a9eff; font-family:monospace;">"RigidBodyFfm"</span> " 类。"</p>
            </div>

            <div style="margin:20px 0;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"集成测试"</h2>
                <p style="color:#aaa; line-height:1.7;">"项目包含 342 个集成测试覆盖所有功能。运行以下命令验证安装："</p>
                <pre><code class="language-bash">
"cargo test -p mps-test              # 342 integration tests
cargo check --workspace              # full workspace check"
                </code></pre>
            </div>
        </div>
    }
}