//! `mps_cosmos::arena` 镜像测试 —— 布局不变量守护（零拷贝 ABI 的"模板契约"）。
//!
//! 真正的命令环→状态回读端到端往返在 `ffi.rs` 的 `arena_*` 用例里（直接对
//! arena 共享内存做 native-order 读写，等价于 Java 侧 `ByteBuffer` 映射）。本
//! 文件只守护**布局常量自身的一致性**——任一偏移/步长被改错都会先在这里炸，
//! 比端到端用例更早暴露 ABI 回归：
//! 1. body 槽步长 ≥ 槽内容（gen 8B + pos 24B + vel 24B + angvel 24B + 尾 16B）；
//! 2. 命令槽步长 = 5 × u64（cmd_type, body_index, a0, a1, a2）；
//! 3. header 偏移表不越界（落在 128B header 内）；
//! 4. 命令环基地址（header 在 `new` 时写入的 `OFF_CMD_RING`）随容量连续排布，
//!    与 `flush_all_bodies` 实际写入的区域一致。

#![cfg(test)]

use mps_cosmos::arena::{
    ARENA_MAGIC, ARENA_VERSION, BODY_SLOT_STRIDE, CMD_SLOT_STRIDE, HEADER_SIZE, MAX_ARENA_BODIES,
    MAX_ARENA_COMMANDS, MAX_ARENA_TOTAL_BYTES, OFF_BODY_COUNT, OFF_BODY_SLOT_BASE, OFF_CMD_RING,
    OFF_CMD_WRITE,
};

#[test]
#[allow(clippy::assertions_on_constants)]
fn arena_layout_constants_are_self_consistent() {
    // 128B header 之后紧跟 body 槽；body 槽步长必须容纳槽内容：
    // gen(u64=8) + pos(3×f64=24) + vel(3×f64=24) + angvel(3×f64=24)
    // + body_type(u32) + sleeping(u32) + user_data(u64) = 8+24+24+24+4+4+8 = 96。
    assert_eq!(BODY_SLOT_STRIDE, 96, "body slot stride");
    // 命令槽 = 5 × u64（cmd_type, body_index, a0, a1, a2）= 40。
    assert_eq!(CMD_SLOT_STRIDE, 40, "cmd slot stride");

    // header 偏移表全部落在 128B header 内。
    assert!(OFF_BODY_COUNT < HEADER_SIZE);
    assert!(OFF_CMD_WRITE < HEADER_SIZE);
    assert!(OFF_CMD_RING < HEADER_SIZE);
    assert_eq!(
        OFF_BODY_SLOT_BASE, HEADER_SIZE,
        "body slots start at header end"
    );

    // magic / version 非零且为稳定标识。
    assert_ne!(ARENA_MAGIC, 0);
    assert_eq!(ARENA_VERSION, 1);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn arena_capacity_bounds_are_sane() {
    assert!(MAX_ARENA_BODIES > 0);
    assert!(MAX_ARENA_COMMANDS > 0);
    // 256 MiB 上限必须 ≤ i32::MAX（Java `ByteBuffer.capacity()` 为 int）。
    assert!(MAX_ARENA_TOTAL_BYTES <= i32::MAX as usize);
}

#[test]
fn arena_ring_base_follows_body_slots_contiguously() {
    // 构造一个 arena，校验命令环基地址 = body_slots 末尾 + body_handle_map(8B/体)。
    let arena = mps_cosmos::arena::SharedArena::new(8, 8).expect("arena alloc");
    let addr = arena.address();
    unsafe {
        let base = addr as *const u8;
        let cmd_ring_base = (base.add(OFF_CMD_RING) as *const u64).read_unaligned() as usize;
        let expected = HEADER_SIZE + 8 * BODY_SLOT_STRIDE as usize + 8 * 8;
        assert_eq!(cmd_ring_base, expected, "cmd ring base offset");
    }
}

/// P2（2026-09 多线程适配）——`flush_all_bodies` 并行分支（≥ 512 体）与串行
/// 语义一致：600 个刚体 flush 后逐槽回读，位置/速度/handle-map/header 活跃数
/// 必须与刚体状态逐值一致（槽互不重叠，并行写与串行写逐位相同）。
#[test]
fn flush_all_bodies_parallel_matches_body_state() {
    use mps_cosmos::arena::SharedArena;
    use rapier3d::prelude::{RigidBodyBuilder, RigidBodySet, Vector};

    const N: usize = 600; // ≥ FLUSH_PARALLEL_MIN(512) → rayon 并行分支
    let arena = SharedArena::new(N as u32, 4).expect("arena alloc");
    let mut bodies = RigidBodySet::new();
    let mut expect = Vec::with_capacity(N);
    for i in 0..N {
        let pos = Vector::new(i as f64 * 1.5, -(i as f64) * 0.5, i as f64 * 2.25);
        let vel = Vector::new(i as f64 * 0.25, -(i as f64) * 2.0, i as f64);
        let h = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(pos)
                .linvel(vel)
                .build(),
        );
        expect.push((h, pos, vel));
    }

    arena.flush_all_bodies(&bodies);

    let base = arena.address() as *const u8;
    unsafe {
        // header 活跃数（OFF_BODY_COUNT = 32）。
        let count = base.add(OFF_BODY_COUNT).cast::<u32>().read_unaligned();
        assert_eq!(count as usize, N, "header body count");

        // handle-map（body_slots 之后，8B/体）+ 逐槽 pos/vel 回读。
        let handle_map = base.add(HEADER_SIZE + N * BODY_SLOT_STRIDE as usize);
        for (i, (h, pos, vel)) in expect.iter().enumerate() {
            let slot = base.add(HEADER_SIZE + i * BODY_SLOT_STRIDE as usize);
            let px = slot.add(8).cast::<f64>().read_unaligned();
            let py = slot.add(16).cast::<f64>().read_unaligned();
            let pz = slot.add(24).cast::<f64>().read_unaligned();
            let vx = slot.add(32).cast::<f64>().read_unaligned();
            let vy = slot.add(40).cast::<f64>().read_unaligned();
            let vz = slot.add(48).cast::<f64>().read_unaligned();
            assert_eq!((px, py, pz), (pos.x, pos.y, pos.z), "pos slot {i}");
            assert_eq!((vx, vy, vz), (vel.x, vel.y, vel.z), "vel slot {i}");
            let raw = handle_map.add(i * 8).cast::<u64>().read_unaligned();
            assert_eq!(raw, h.into_raw_parts().0 as u64, "handle map slot {i}");
        }
    }
}
