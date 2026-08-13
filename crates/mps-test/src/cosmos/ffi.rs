//! `mps_cosmos::ffi` 入口测试 —— FFI 批量快照接口（性能分析.MD §11.1/§12.1，
//! M1 + L1 落地的回归守护）。
//!
//! 此前 cosmos FFI 完全没有 batch snapshot 等价：MC mod 端要读 N 个卫星位置
//! 必须 N 次 `cosmos_body_translation_out` 往返 JNI（N=1000 时 ~600µs/tick 的
//! dispatch 开销 + minor GC 抖动）。新加的 `cosmos_world_dynamic_body_snapshot`
//! 把整批合并成一次 native 调用 + 一份连续 f64[]，延迟降到 ~50µs/tick。
//!
//! 本文件直接调 FFI 函数（同 crate 等价于 JNI 路径调 FFI），不经过 JNI 宏，
//! 以便离 JVM 一层运行——这正是 mps-cosmos `ffi.rs` 注释里允诺的 ABI 形态。
//! 验证要点（任一被破坏即视为 ABI 回归）：
//! 1. count 与节点数一致；
//! 2. handle 编码 = (idx << 32) | generation —— 与既有 cosmos per-body handle
//!    一致，可通过 `unpack_handle` 还原；
//! 3. pos 拍平到 `values[i*7..i*7+3]`，与 per-body `cosmos_body_translation_out`
//!    返回值逐项相等；
//! 4. 容量小于 N 时只填前 `capacity` 个体，其它不写超出；
//! 5. null world / null 缓冲 / 0 容量 → 安全返回 0（不 panic / 不 UB）。

#![cfg(test)]

use mps_cosmos::bodies::satellite_builder;
use mps_cosmos::ffi::{
    cosmos_world_dynamic_body_snapshot, cosmos_world_dynamic_body_snapshot_count,
};
use mps_cosmos::world::{CosmosWorld, CosmosWorldConfig, OrbitIntegration, RelativisticCorrection};
use rapier3d::prelude::Vector;

/// 默认 cosmos 配置（不注册任何天体 / sun；用于快照计数 + 拍平布局的纯结构性测试）。
fn empty_world() -> CosmosWorld {
    CosmosWorld::new({
        CosmosWorldConfig {
            gravity: Vector::ZERO,
            dt: 1.0,
            solver_iterations: 4,
            ccd_substeps: 0,
            n_body_softening_sq: 0.0,
            central_body: None,
            orbit_integration: OrbitIntegration::default(),
            verlet_substeps: 1,
            adaptive_substeps: false,
            adaptive_tolerance: 1e-9,
            relativistic_correction: RelativisticCorrection::None,
        }
    })
}

/// 还原 `pack_handle = (idx << 32) | generation`（与 mps-cosmos `ffi.rs::unpack_handle`
/// 完全相同的位运算）—— 测试侧独立写一份避免直接依赖 mps_cosmos::ffi 内部 fn。
fn unpack_handle(packed: u64) -> (u32, u32) {
    let idx = ((packed >> 32) & 0xFFFF_FFFF) as u32;
    let generation = (packed & 0xFFFF_FFFF) as u32;
    (idx, generation)
}

#[test]
fn snapshot_count_matches_world_dynamic_body_count() {
    let mut world = empty_world();
    // 起先无 body。
    assert_eq!(cosmos_world_dynamic_body_snapshot_count(&world), 0);

    // 插入 3 个体（前两个动态卫星 + 第三个固定体）。
    let _sat1 = world.insert_body(satellite_builder(
        1.0,
        Vector::new(1.0, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));
    let _sat2 = world.insert_body(satellite_builder(
        2.0,
        Vector::new(2.0, 0.0, 0.0),
        Vector::ZERO,
        0.1,
    ));
    let _ = world.insert_body(mps_cosmos::bodies::fixed_body_builder(Vector::new(
        0.0, 0.0, 0.0,
    )));

    // 只数动态体（fixed 不算），与 `world.dynamic_body_count()` 一致。
    assert_eq!(cosmos_world_dynamic_body_snapshot_count(&world), 2);
}

#[test]
fn snapshot_writes_pos_and_handles_in_expected_layout() {
    let mut world = empty_world();
    let sat1 = world.insert_body(satellite_builder(
        1.0,
        Vector::new(10.0, 20.0, 30.0),
        Vector::ZERO,
        0.1,
    ));
    let sat2 = world.insert_body(satellite_builder(
        2.0,
        Vector::new(40.0, 50.0, 60.0),
        Vector::ZERO,
        0.1,
    ));

    // 预分配：2 体 × 7 f64 + 2 个 u64 handle。
    let mut handles = vec![0u64; 2];
    let mut values = vec![0f64; 2 * 7];
    let n =
        cosmos_world_dynamic_body_snapshot(&world, handles.as_mut_ptr(), values.as_mut_ptr(), 2);
    assert_eq!(n, 2, "snapshot should write both bodies");

    // handle 还原后应能映射回 RigidBodyHandle（与 per-body 路径一致）。
    let (id1, gen1) = unpack_handle(handles[0]);
    let (id2, _gen2) = unpack_handle(handles[1]);
    let (expect1, expect2) = (sat1.into_raw_parts(), sat2.into_raw_parts());
    assert_eq!((id1, gen1), expect1, "handle[0] encoding");
    assert_eq!((id2, _gen2), expect2, "handle[1] encoding");

    // pos 拍平验证（values[i*7..i*7+3]），与 per-body `body_translation` 等价。
    // bodies.iter() 按插入序输出 → sat1 在前，sat2 在后。
    assert_eq!(values[0..3], [10.0, 20.0, 30.0], "sat1 pos");
    assert_eq!(values[7..10], [40.0, 50.0, 60.0], "sat2 pos");

    // 旋转槽 [3..7] 应为 identity (0,0,0,1)——satellite_builder 不自定义 rotation。
    assert_eq!(values[3..7], [0.0, 0.0, 0.0, 1.0], "sat1 identity quat");
    assert_eq!(values[10..14], [0.0, 0.0, 0.0, 1.0], "sat2 identity quat");
}

#[test]
fn snapshot_truncates_to_capacity_without_overflow() {
    let mut world = empty_world();
    for i in 0..5 {
        let _ = world.insert_body(satellite_builder(
            1.0,
            Vector::new(i as f64, 0.0, 0.0),
            Vector::ZERO,
            0.1,
        ));
    }
    assert_eq!(cosmos_world_dynamic_body_snapshot_count(&world), 5);

    // 容量 3：只能写入 3 个体，不超出缓冲。
    let mut handles = vec![u64::MAX; 5];
    let mut values = vec![f64::NAN; 5 * 7];
    let n =
        cosmos_world_dynamic_body_snapshot(&world, handles.as_mut_ptr(), values.as_mut_ptr(), 3);
    assert_eq!(n, 3);
    // 未写入部分保持 sentinel（"未污染 capacity 之外"——FFI 不应越界写入）。
    assert_eq!(handles[3], u64::MAX);
    assert_eq!(handles[4], u64::MAX);
    assert!(values[3 * 7].is_nan());
    assert!(values[4 * 7].is_nan());
}

#[test]
fn snapshot_handles_null_world_and_buffers_safely() {
    use mps_cosmos::ffi::cosmos_world_dynamic_body_count;
    use std::ptr;

    // null world → 0，且不 panic。
    assert_eq!(cosmos_world_dynamic_body_snapshot_count(ptr::null()), 0);
    assert_eq!(cosmos_world_dynamic_body_count(ptr::null()), 0);
    let mut h = vec![0u64; 1];
    let mut v = vec![0f64; 7];
    assert_eq!(
        cosmos_world_dynamic_body_snapshot(ptr::null(), h.as_mut_ptr(), v.as_mut_ptr(), 1),
        0
    );

    // 有效 world + null 缓冲 → 0。
    let world = empty_world();
    assert_eq!(
        cosmos_world_dynamic_body_snapshot(&world, ptr::null_mut(), v.as_mut_ptr(), 1),
        0
    );
    assert_eq!(
        cosmos_world_dynamic_body_snapshot(&world, h.as_mut_ptr(), ptr::null_mut(), 1),
        0
    );

    // 0 容量 → 0。
    assert_eq!(
        cosmos_world_dynamic_body_snapshot(&world, h.as_mut_ptr(), v.as_mut_ptr(), 0),
        0
    );
}

#[test]
fn snapshot_round_trip_against_per_body_translation_out() {
    // 验证 batch snapshot 与逐体 `cosmos_body_translation_out` 返回 pos 完全一致
    // —— 这是 M1+L1 改动的核心契约：Java 端可以从 N 次 jni 往返迁到 1 次 batch
    // 调用，得到完全相同的 pos 数值（这是替代 per-body 路径的前提）。
    use mps_cosmos::ffi::cosmos_body_translation_out;
    use mps_formula::ffi::Vec3;

    let mut world = empty_world();
    let h1 = world.insert_body(satellite_builder(
        1.0,
        Vector::new(1.5, 2.5, 3.5),
        Vector::ZERO,
        0.1,
    ));
    let h2 = world.insert_body(satellite_builder(
        2.0,
        Vector::new(-4.0, 0.5, 7.2),
        Vector::ZERO,
        0.1,
    ));

    // per-body 路径：循环 N 次取 pos。
    let mut per_body_pos = [Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }; 2];
    let pack = |h: rapier3d::prelude::RigidBodyHandle| -> u64 {
        let (idx, generation) = h.into_raw_parts();
        ((idx as u64) << 32) | (generation as u64)
    };
    // FFI 签名是 `out: *mut Vec3`——这里直接传裸指针，C-ABI 入口在 panic=abort
    // 防护下不会 UB（`world` 也是 `*const CosmosWorld`，Rust 引用强转裸指针对齐）。
    let _ =
        cosmos_body_translation_out(&world as *const _, pack(h1), &mut per_body_pos[0] as *mut _);
    let _ =
        cosmos_body_translation_out(&world as *const _, pack(h2), &mut per_body_pos[1] as *mut _);

    // batch 路径：一次拉。
    let mut handles = vec![0u64; 2];
    let mut values = vec![0f64; 2 * 7];
    let n =
        cosmos_world_dynamic_body_snapshot(&world, handles.as_mut_ptr(), values.as_mut_ptr(), 2);
    assert_eq!(n, 2);

    // 两个 body 的 pos 必须无误差相等。
    assert_eq!(
        values[0..3],
        [per_body_pos[0].x, per_body_pos[0].y, per_body_pos[0].z]
    );
    assert_eq!(
        values[7..10],
        [per_body_pos[1].x, per_body_pos[1].y, per_body_pos[1].z]
    );
}
