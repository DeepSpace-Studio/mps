use topcoat::router::page;
use topcoat::view::view;

/// Quickstart guide — hands-on from zero to a running sim.
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
                    <p style="font-size:14px; color:#999; line-height:1.7; margin:0;">{ "从零开始使用 MPS 物理引擎 —— 装好工具链、建世界、插入刚体与碰撞体、跑模拟、读结果。Rust C ABI、Java JNI、Java FFM 三条路径各给一份可直接拷贝的最小示例。" }</p>
                </div>
                <div style="font-size:48px; font-weight:700; color:#333; font-family:monospace; line-height:1;">"01"</div>
            </div>

            <div style="background:#16213e; border:1px solid #333; border-radius:8px; padding:24px; margin-bottom:20px;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"0. 环境与构建"</h2>
                <p style="color:#aaa; line-height:1.7; margin:0 0 12px;">{ "工具链：Rust stable（含 cargo），Java 21（JNI 路径）或 Java 25（FFM 路径，预览特性）。本仓库是 cargo workspace，根 Cargo.toml 已配置好内部 rapier3d（f64 后端）与 topcoat 等依赖路径。" }</p>
                <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">"常用命令："</p>
                <pre><code class="language-bash">
"# 全工作区检查（最快验证编译）
cargo check --workspace

# 只构建 C ABI + 绑定产物（mps-core → 静态库 + rigid_body.h，mps-jni → mps_rigid_body.dll/.so）
cargo build --release -p mps-core -p mps-jni

# 跑 342 项集成测试（直接调用 extern C 符号，不经过 Java）
cargo test -p mps-test

# 跑 mps-cosmos 的 19 项精度回归
cargo test -p mps-test --lib cosmos

# 单测指定用例
cargo test -p mps-test -- integration_parameters_and_body_batch_updates_work

# 文档站本地预览（Topcoat）
cargo run -p mps-web"
                </code></pre>
                <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin-top:14px;">
                    <p>{ "根 Cargo.toml 设了 panic = abort。所有 extern C 入口都由 " }<code>"ffi_guard"</code>{ "（mps-core）或 " }<code>"catch_unwind"</code>{ "（mps-jni）兜底，panic 永不 unwind 穿过 FFI 边界。编程模型上：API 失败返回 sentinel（null/0/" }<code>"Bool::FALSE"</code>{ "），并写线程局部 last-error，" }<strong>"绝不 panic"</strong>{ "。" }</p>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"1"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"Rust 路径：C ABI 最小示例"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">{ "mps-test 的集成测试就是最权威的 \"用法范例\"：直接 use mps_core::rapier::* 后调用 extern C 函数。下面这段复刻 integration_parameters_and_body_batch_updates_work 的骨架——建世界、设积分参数、插动态刚体、批量改位姿/速度、拍快照、销毁。" }</p>
                    <pre><code class="language-rust">
"use mps_core::rapier::ffi::{BodyStatus, Bool, Vec3};
use mps_core::rapier::collider::{
    collider_builder_build, collider_builder_create_obb, world_insert_collider, Obb, Quat,
};
use mps_core::rapier::rigid_body::{
    rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
};
use mps_core::rapier::world::{
    world_body_snapshot, world_body_snapshot_count, world_create, world_destroy,
    world_set_integration_parameters, world_step, world_update_body_poses,
};

// 1. 建世界，重力沿 -y（SI 单位：m/s²）
let world = world_create(Vec3 { x: 0.0, y: -9.81, z: 0.0 });
assert!(!world.is_null());

// 2. dt=1/120s，求解器 8 次迭代，CCD 2 子步
assert_eq!(world_set_integration_parameters(world, 1.0 / 120.0, 8, 2), Bool::TRUE);

// 3. 建 Dynamic 刚体 builder → build 成裸指针 → 插入世界拿 packed handle (u64)
let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
let body = rigid_body_builder_build(builder);          // *mut RigidBody（所有权转移）
let handle = world_insert_rigid_body(world, body);     // RigidBodyHandleRaw = u64
assert_ne!(handle, 0);

// 4. 挂一个 OBB 碰撞体（half_extents 0.5/1.0/1.5）
let obb = Obb {
    center: Vec3 { x: 0.0, y: 10.0, z: 0.0 },
    half_extents: Vec3 { x: 0.5, y: 1.0, z: 1.5 },
    rotation: Quat { i: 0.0, j: 0.0, k: 0.0, w: 1.0 },
};
let collider = world_insert_collider(world, collider_builder_build(collider_builder_create_obb(obb)));
assert_ne!(collider, 0);

// 5. 推进一步（rapier 内部 semi-implicit Euler 会下落）
world_step(world, 1.0 / 120.0);

// 6. 批量更新位姿：handles[] + 扁平 f64[]（xyz + ijkw = 7 个/体）。返回成功写入个数
let updated = world_update_body_poses(
    world,
    [handle].as_ptr(),
    [0.0, 12.0, 0.0, 0.0, 0.0, 0.0, 1.0].as_ptr(),
    1, Bool::TRUE, // wake_up = true
);
assert_eq!(updated, 1);

// 7. 快照：13 个 f64/体 = 平移(3) + 旋转(4) + 线速度(3) + 角速度(3)
let mut out_handles = [0u64; 1];
let mut values = [0.0_f64; 13];
assert_eq!(world_body_snapshot_count(world), 1);
assert_eq!(
    world_body_snapshot(world, out_handles.as_mut_ptr(), values.as_mut_ptr(), 1),
    1,
);
assert_eq!(out_handles[0], handle);
assert_eq!(&values[..3], &[0.0, 12.0, 0.0]);

world_destroy(world);"</code></pre>
                    <p style="color:#777; line-height:1.7; margin:8px 0 0; font-size:13px;">{ "句柄约定：刚体/碰撞体返回的是 " }<code>"u64"</code>{ "（高 32 位 arena index + 低 32 位 generation，防悬挂引用），传回 API 时原样塞回；builder / world 则返回裸指针 " }<code>"*mut"</code>{ "，" }<strong>"builder 在 build 后所有权转移到世界，调用方不再持有"</strong>{ "。" }</p>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"2"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">{ "读取错误（last-error 线程槽）" }</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">{ "任一 FFI 调用失败都不会 panic，而是返回 sentinel（null pointer / 0 / " }<code>"Bool::FALSE"</code>{ "）并把错误码+消息写进 " }<strong>"线程局部"</strong>{ " 错误槽（定义在 mps-formula::error，被 mps-core re-export）。下次同线程任意 FFI 调用会覆盖它。" }</p>
                    <pre><code class="language-rust">
"use mps_core::rapier::error::{
    ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, last_error_code, last_error_message,
};

// 传一个 null world
world_step(std::ptr::null_mut(), 1.0 / 60.0);
assert_eq!(last_error_code(), ERR_NULL_POINTER);
// last_error_message() 返回 *const c_char，NUL 结尾，无效化前有效
let msg = unsafe { std::ffi::CStr::from_ptr(last_error_message()) };
assert!(msg.to_string_lossy().contains(\"null\"));"
                    </code></pre>
                    <p style="color:#aaa; line-height:1.7; margin:8px 0 0;">{ "错误码：ERR_OK=0、ERR_NULL_POINTER=1、ERR_INVALID_ARGUMENT=2、ERR_NOT_FOUND=3、ERR_CAPACITY=4、ERR_UNSUPPORTED=5、ERR_INTERNAL=6。" }</p>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"3"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"Java 21 JNI 路径（test21）"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">{ "原生方法集中在 org.polaris2023.mps_rigid_body.RigidBodyNative（符号名 Java_org_polaris2023_mps_1rigid_1body_RigidBodyNative_<method>，由 mps-jni/src/lib.rs 的 jni! / jni_e_c! 宏生成）。上层提供 Fluent API：PhysicsWorld / RigidBody / Collider 全在 util 包。最小循环：" }</p>
                    <pre><code class="language-java">
"import org.polaris2023.mps_rigid_body.util.PhysicsWorld;
import org.polaris2023.mps_rigid_body.util.RigidBody;
import org.polaris2023.mps_rigid_body.util.Collider;

try (PhysicsWorld world = new PhysicsWorld(0, -9.81, 0)) {
    world.integrationParameters(1.0 / 120.0, 8, 2);

    // Dynamic 刚体 + cuboid 碰撞体
    RigidBody body = world.body(RigidBody.DYNAMIC)
            .translation(0, 10, 0)
            .insert();
    Collider cuboid = world.cuboidCollider(0.5, 0.5, 0.5)
            .friction(0.7)
            .restitution(0.1)
            .attach(body);

    for (int i = 0; i < 600; i++) {                  // 5 秒
        world.step();                                 // deltaSeconds 默认 1/60
        double[] p = body.translation();
        if (i % 60 == 0) System.out.printf(\"t=%.2f y=%.3f%n\", i / 60.0, p[1]);
    }
} // close() 自动 worldDestroy —— 句柄清零防 use-after-free"
                    </code></pre>
                    <p style="color:#aaa; line-height:1.7; margin:8px 0 0;">{ "链路：PhysicsWorld.body() → RigidBody.Builder → insert() 内部 RigidBodyNative.worldInsertRigidBody。world.step() 会 panic-guard（JNI 侧 catch_unwind），native panic 转成 abiLastErrorMessage() 返回的 ERR_INTERNAL。" }</p>
                    <p style="color:#aaa; line-height:1.7; margin:8px 0 0;">{ "验证：" }<code>"cd test21 && ./gradlew.bat check"</code>{ "。详情见 " }<a href="./jni" style="color:#4a9eff;">"JNI 绑定"</a>{ " 页。" }</p>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"4"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">"Java 25 FFM 路径（test25）"</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">{ "FFM（Foreign Function & Memory API）不经 JNI 符号，改用 java.lang.foreign.Linker 直查 rigid_body.h 声明的 C 符号，参数用 MemorySegment。绑定在 RigidBodyFfm，烟雾测试 FfmSmokeTest。最关键是 SharedPhysicsArena —— 它把 world_get_shared_arena_address 返回的 native 地址包成 MemorySegment，" }<strong>"此后读刚体位置/速度无需任何 downcall"</strong>{ "，每帧只剩一次 worldStep。" }</p>
                    <pre><code class="language-java">
"import org.polaris2023.mps_rigid_body.ffm.RigidBodyFfm;
import org.polaris2023.mps_rigid_body.ffm.SharedPhysicsArena;
import java.lang.foreign.MemorySegment;

long world = RigidBodyFfm.worldCreate(0, -9.81, 0);
RigidBodyFfm.worldSetIntegrationParameters(world, 1.0 / 120.0, 8, 2);
long arenaAddr = RigidBodyFfm.worldCreateSharedArena(world, /* maxBodies */ 4096);

MemorySegment seg = MemorySegment.ofAddress(arenaAddr).reinterpret(/* size */ 1L << 20);
SharedPhysicsArena arena = new SharedPhysicsArena(seg);

long body = RigidBodyFfm.worldInsertRigidBody(world, /* ... */);
for (int i = 0; i < 600; i++) {
    RigidBodyFfm.worldStep(world, 1.0 / 60.0);
    // 零 downcall 直接读第 0 个 body 槽位的 y
    double y = arena.getBodyPY(0);
    if (i % 60 == 0) System.out.printf(\"t=%.2f y=%.3f%n\", i / 60.0, y);
}"
                    </code></pre>
                    <p style="color:#aaa; line-height:1.7; margin:8px 0 0;">{ "验证：" }<code>"cd test25 && ./gradlew.bat check"</code>{ "。Arena 内存布局、命令环协议见 " }<a href="./arena" style="color:#4a9eff;">"共享内存 Arena"</a>{ " 页。" }</p>
                </div>
            </div>

            <div style="margin-bottom:24px;">
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:12px;">
                    <span style="background:#4a9eff; color:#1a1a2e; width:32px; height:32px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-weight:700; font-size:14px; flex-shrink:0;">"5"</span>
                    <h3 style="margin:0; font-size:18px; color:#fff; font-weight:500;">{ "下一步：太空轨道用 mps-cosmos" }</h3>
                </div>
                <div style="padding-left:44px;">
                    <p style="color:#aaa; line-height:1.7; margin:0 0 8px;">{ "上面是通用/地面场景的 mps-core PhysicsWorld。要做长弧轨道（卫星、行星、n-body），用 " }<a href="./cosmos" style="color:#4a9eff;">"mps-cosmos"</a>{ " —— 它自带 CosmosWorld，把轨道推进从 rapier 的 semi-implicit Euler 里抽出来改走辛积分器（Yoshida 4 / Forest-Ruth 8 / + Kahan），长弧相位误差随 dt⁴ / dt⁸ 收敛。Rust 端是纯 pub API（不经 C ABI），Java 端有 cosmos* 系列 JNI 绑定。" }</p>
                    <pre><code class="language-rust">
"use mps_cosmos::{CosmosWorld, CosmosWorldConfig, world::OrbitIntegration};
use mps_cosmos::gravity::CelestialSource;
use mps_cosmos::bodies::satellite_builder;
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
use rapier3d::prelude::Vector;

let earth = get_celestial_body(CelestialBodyId::Earth);
let mut world = CosmosWorld::new(CosmosWorldConfig {
    dt: 1.0, orbit_integration: OrbitIntegration::Yoshida4,
    central_body: Some(earth), ..Default::default()
});
world.add_celestial(CelestialSource::new(earth, 8));
let _sat = world.insert_body_as_gravity_source(
    satellite_builder(1000.0, Vector::new(7e6, 0.0, 0.0), Vector::new(0.0, 7800.0, 0.0), 1.0),
    1000.0,
);
for _ in 0..5400 { let _ = world.step(1.0); }  // 一圈 LEO，闭合误差 < 0.1% r"
                    </code></pre>
                </div>
            </div>

            <div class="callout" style="background:#0f1a2e; border-left:4px solid #4a9eff; padding:14px 18px; border-radius:4px; margin:20px 0;">
                <p>{ "所有 API 均通过 C FFI 暴露，Java 开发者可通过 JNI 或 FFM 调用。Java 代码中可使用 " }<span class="hi" style="color:#4a9eff; font-family:monospace;">"RigidBodyNative"</span>{ " 或 " }<span class="hi" style="color:#4a9eff; font-family:monospace;">"RigidBodyFfm"</span>{ " 类。太空轨道场景用 " }<a href="./cosmos" style="color:#4a9eff;">"mps-cosmos"</a>{ "，提供 " }<span class="hi" style="color:#4a9eff; font-family:monospace;">"CosmosWorld"</span>{ " 与辛轨道积分，JNI 端见 " }<span class="hi" style="color:#4a9eff; font-family:monospace;">"cosmos*"</span>{ " 系列。" }</p>
            </div>

            <div style="margin:20px 0;">
                <h2 style="font-size:20px; color:#fff; font-weight:400; margin:0 0 16px; padding-bottom:10px; border-bottom:1px solid #333;">"集成测试"</h2>
                <p style="color:#aaa; line-height:1.7;">{ "项目在 mps-test 内置 342 项集成测试，直接调 extern C 符号覆盖所有功能（含 cosmos 19）。这是验证安装、也是 \"函数到底怎么用\" 的最权威来源：" }</p>
                <pre><code class="language-bash">
"cargo test -p mps-test                       # 342 integration tests
cargo test -p mps-test --lib cosmos           # mps-cosmos 精度回归
cargo test -p mps-test -- <test_name>        # 跑单个用例
cargo check --workspace                       # 全工作区编译"
                </code></pre>
            </div>
        </div>
    }
}
