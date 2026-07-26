use topcoat::router::page;
use topcoat::view::view;

/// JNI page
#[page("/jni")]
pub async fn jni() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ JNI"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"Java JNI 绑定"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"Java 21 JNI 全绑定，~280 个方法覆盖所有核心功能。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"Java 入口"</h2>
                <p style="color:#aaa;line-height:1.7;">"Java 21 使用 RigidBodyNative JNI 方法，位于 org.polaris2023.mps_rigid_body.util 包。"</p>
                <pre><code class="language-java">
"// Java 示例
World world = new World(new Vec3(0, -9.81, 0));
RigidBody body = world.createRigidBody(RigidBodyType.DYNAMIC);
body.setPosition(0, 10, 0);
Collider collider = body.createCollider(ColliderShape.CUBOID, 0.5, 0.5, 0.5);
world.step(1.0 / 60.0);
Vec3 pos = body.getPosition();
System.out.println(\"y = \" + pos.y);"
                </code></pre>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"运行测试"</h2>
                <pre><code class="language-bash">
"cd test21
./gradlew.bat check"
                </code></pre>
            </div>
        </div>
    }
}
