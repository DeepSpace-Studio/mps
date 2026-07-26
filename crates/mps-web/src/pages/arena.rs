use topcoat::router::page;
use topcoat::view::view;

/// Arena page
#[page("/arena")]
pub async fn arena() -> topcoat::Result {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:30px;padding-bottom:20px;border-bottom:1px solid #333;">
                <div>
                    <div style="font-size:12px;color:#4a9eff;letter-spacing:3px;text-transform:uppercase;font-family:monospace;margin-bottom:8px;">"/ INTEGRATION"</div>
                    <h1 style="font-size:28px;font-weight:300;color:#fff;margin:0 0 10px;">"共享内存 Arena"</h1>
                    <p style="font-size:14px;color:#999;line-height:1.7;margin:0;">"零 JNI 物理数据访问。Rust 维护一块共享内存区域，Java 通过 DirectByteBuffer 直接读写。每帧仅需 1 次 JNI 调用，整体加速 175×。"</p>
                </div>
                <div style="font-size:48px;font-weight:700;color:#333;font-family:monospace;line-height:1;">"01"</div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"内存布局"</h2>
                <div style="overflow-x:auto;">
                    <table>
                        <thead><tr><th>"区域"</th><th>"偏移"</th><th>"大小"</th><th>"说明"</th></tr></thead>
                        <tbody>
                            <tr><td>"Header"</td><td>"0"</td><td>"64"</td><td>"版本、时间戳、刚体数量"</td></tr>
                            <tr><td>"Body States"</td><td>"64"</td><td>"N×128"</td><td>"位置、旋转、速度等"</td></tr>
                            <tr><td>"Collider States"</td><td>"可变"</td><td>"M×64"</td><td>"碰撞体位置、材质"</td></tr>
                            <tr><td>"Event Queue"</td><td>"可变"</td><td>"可变"</td><td>"碰撞事件环缓冲区"</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <div style="background:#16213e;border:1px solid #333;border-radius:8px;padding:24px;margin-bottom:20px;">
                <h2 style="color:#fff;font-size:20px;font-weight:400;margin:0 0 16px;padding-bottom:10px;border-bottom:1px solid #333;">"性能"</h2>
                <p style="color:#aaa;line-height:1.7;">"共享内存 Arena 消除了每帧数千次 JNI 调用的开销，将整体性能提升 175×。Java 端可直接读取刚体位置、速度等数据，无需 JNI 桥接。"</p>
            </div>
        </div>
    }
}
