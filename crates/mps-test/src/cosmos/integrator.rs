//! `mps_cosmos::integrator` 测试 —— 迁移自 `crates/mps-cosmos/src/integrator.rs`。
//!
//! velocity-Verlet 在**点质量中心引力**下推进一整圈应几乎闭合。用 n-body 源
//! （点质量互引力，`-GM·r̂/r²`，不走 `celestial_acceleration` 的 ellipsoid/J2
//! 分支）作为中心引力，直接验证 Verlet 自身的相位精度，排除引力模型误差。

#[cfg(test)]
use mps_cosmos::bodies::satellite_builder;
#[cfg(test)]
use mps_cosmos::gravity::{MassPoint, NBodySource, gm_from_mass};
#[cfg(test)]
use mps_cosmos::integrator::{
    AccelContext, snapshot_source_positions, total_acceleration, verlet_step,
};
#[cfg(test)]
use mps_formula::celestial_data::{CelestialBodyId, get_celestial_body};
#[cfg(test)]
use mps_formula::spaceflight::kepler_period;
#[cfg(test)]
use rapier3d::prelude::{RigidBodyHandle, RigidBodySet, Rotation, Vector};

#[test]
fn verlet_circle_orbit_closes_tight() {
    let earth = get_celestial_body(CelestialBodyId::Earth);
    let gm = earth.gm;
    let r = 7_000_000.0_f64;
    let v = (gm / r).sqrt();
    let period = kepler_period(gm, r).expect("period");
    let dt = 1.0;
    let steps = (period / dt).round() as u32;

    // 用一个固定刚体作为"假地球"n-body 源（点质量互引力）。
    let mut bodies = RigidBodySet::new();
    let earth_hdl = bodies.insert(
        satellite_builder(5.972e24, Vector::ZERO, Vector::ZERO, 1.0)
            .lock_translations()
            .build(),
    );
    let sat_hdl = bodies.insert(
        satellite_builder(
            1000.0,
            Vector::new(r, 0.0, 0.0),
            Vector::new(0.0, v, 0.0),
            1.0,
        )
        .build(),
    );
    let n_body_sources = vec![NBodySource::monopole(earth_hdl, gm_from_mass(5.972e24))];

    // ctx：无 celestials，仅一个 n-body 点质量源。
    let src_pos = snapshot_source_positions(&bodies, &n_body_sources);
    // 纯 monopole 源（无 points）不读 rotation，用 identity 切片填充。
    let src_rot: Vec<Rotation> = (0..bodies.len()).map(|_| Rotation::IDENTITY).collect();
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: &n_body_sources,
        source_positions: &src_pos,
        source_rotations: &src_rot,
        source_pos_gm: &[],
        softening_sq: 0.0,
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
        has_irregular_sources: true,
    };

    let mut a = total_acceleration(
        bodies.get(sat_hdl).unwrap().translation(),
        bodies.get(sat_hdl).unwrap().linvel(),
        1000.0,
        sat_hdl,
        &ctx,
        None,
    );
    for _ in 0..steps {
        let body = bodies.get_mut(sat_hdl).unwrap();
        verlet_step(body, a, &ctx, 1000.0, sat_hdl, None, dt);
        // 子步内源位置快照不变（地球锁定），所以 ctx 可直接复用。
        a = total_acceleration(
            bodies.get(sat_hdl).unwrap().translation(),
            bodies.get(sat_hdl).unwrap().linvel(),
            1000.0,
            sat_hdl,
            &ctx,
            None,
        );
    }

    let body = bodies.get(sat_hdl).unwrap();
    let off = (body.translation() - Vector::new(r, 0.0, 0.0)).length();
    // Verlet(二阶) + 1s 步长 + 纯中心引力，一圈闭合 ~3.6km（相位误差 O(dt²·ω·T)）。
    // 给 0.1% r 放量级余量（7000km → 7km）。
    assert!(
        off / r < 1e-3,
        "Verlet 一圈偏移 {off} 过大 (>0.1% r)，pos={:?}",
        body.translation()
    );
    // 同时验证能量近乎保守（无漂）—— Verlet 不应有系统性能量增减。
    let e0 = 0.5 * v * v - gm / r;
    let final_v = body.linvel().length();
    let final_r = body.translation().length();
    let e1 = 0.5 * final_v * final_v - gm / final_r;
    assert!(
        (e1 - e0).abs() / e0.abs() < 1e-3,
        "Verlet 一圈能量漂移 {e1} vs {e0}"
    );
}

/// D（B3）lock-down：并行归约 `n_body_acceleration_reduce` 必须与串行
/// `total_acceleration` 的 n-body 段**逐位一致**（bit-identical）。
///
/// 正确性依据（非眼测）：f64 加法不满足结合律，故并行归约**不能**做「块内累加再
/// 合并」。本实现并行只「独立求每个源的贡献」（纯位置函数、无跨源浮点运算，
/// 与逐源串行求值逐位一致），随后单线程按源序严格左折叠 `((c0+c1)+c2)+…`，
/// 与 `total_acceleration` 串行主路径的折叠顺序**完全相同** → 二者逐位一致。
///
/// 验证：构造 M 从 1 到 512 的 n-body 源（含「自身源被排除」「零 gm 源」「近场
/// 不规则质量分布源」等边界），对卫星在若干位置处比对两函数返回的加速度——三个
/// 分量必须**全相等**（逐位），容差 0（bit-identical，非近似）。
#[test]
fn inner_m_loop_ordered_parallel_bit_identical() {
    use mps_cosmos::integrator::n_body_acceleration_reduce;

    let sat_hdl = RigidBodyHandle::from_raw_parts(0, 0); // 占位 handle，用于排除自身
    let probe_positions = [
        Vector::new(7.0e6, 0.0, 0.0),
        Vector::new(0.0, 1.3e7, -2.1e6),
        Vector::new(-4.0e6, 5.0e6, 9.0e6),
        Vector::new(3.3e6, -7.7e6, 1.1e6),
    ];

    // 多档真实源数 `real_m`（覆盖阈值边界 M_PARALLEL_MIN=8 与巨大 M）。
    // 外加 2 个「边界源」：自身（排除）+ 零 gm，故总 sources = real_m + 2。
    for &real_m in &[0usize, 1, 7, 8, 16, 64, 256, 512] {
        let mut sources: Vec<NBodySource> = Vec::with_capacity(real_m + 2);
        sources.push(NBodySource::monopole(sat_hdl, 5.972e24)); // 排除项
        sources.push(NBodySource::monopole(
            RigidBodyHandle::from_raw_parts(999, 0),
            0.0,
        )); // 零 gm
        let total = real_m + 2;
        // 真实源 handle 的 arena idx 取 1000+i（与 `positions` 容量错开，模拟真实
        // arena 布局）；`source_positions` 必须覆盖最大 arena idx，否则 irregular
        // 路径 `source_positions[src_idx]` 越界（真实代码里它容量 = bodies.len()）。
        let pos_cap = 1000 + real_m + 2;
        let mut positions: Vec<Vector> = vec![Vector::ZERO; pos_cap];
        let mut pos_gm_full = vec![(Vector::ZERO, 0.0); total];
        pos_gm_full[0] = (Vector::ZERO, 5.972e24);
        pos_gm_full[1] = (Vector::ZERO, 0.0);
        let mut rng = 0x9e3779b97f4a7c15u64 ^ (real_m as u64).wrapping_mul(0x100000001b3);
        for i in 0..real_m {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let a = (rng & 0xffff) as f64 * 2.0 * std::f64::consts::PI / 65535.0;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let b = ((rng >> 16) & 0xffff) as f64 * std::f64::consts::PI / 65535.0;
            let rad = 1.0e7 + ((rng >> 32) & 0xffff) as f64 * 5.0e6;
            let p = Vector::new(
                rad * b.sin() * a.cos(),
                rad * b.cos(),
                rad * b.sin() * a.sin(),
            );
            let h = RigidBodyHandle::from_raw_parts(1000 + i as u32, 0);
            let gm = 1.0e20 + (i as f64) * 1.0e15;
            sources.push(NBodySource::monopole(h, gm));
            let idx = 2 + i;
            positions[1000 + i] = p; // irregular 路径按 handle arena idx 读
            pos_gm_full[idx] = (p, gm); // monopole 路径按 seq 读
        }
        let rots: Vec<Rotation> = (0..pos_cap).map(|_| Rotation::IDENTITY).collect();

        // 全 monopole 路径（has_irregular_sources=false，读 source_pos_gm SOA）。
        let ctx_mono = AccelContext {
            celestials: &[],
            n_body_sources: &sources,
            source_positions: &positions,
            source_rotations: &rots,
            source_pos_gm: &pos_gm_full,
            softening_sq: 0.0,
            central_body: None,
            sun_position: Vector::ZERO,
            relativistic: mps_cosmos::world::RelativisticCorrection::None,
            has_irregular_sources: false,
        };
        for &pos in &probe_positions {
            let serial = total_acceleration(pos, Vector::ZERO, 1000.0, sat_hdl, &ctx_mono, None);
            let parallel = n_body_acceleration_reduce(pos, sat_hdl, &ctx_mono);
            assert_eq!(
                serial, parallel,
                "monopole: real_m={real_m} 并行归约与串行不逐位一致 serial={serial:?} parallel={parallel:?}"
            );
        }

        // 不规则质量分布路径（has_irregular_sources=true，读 source_positions）。
        // 给第 2 个真实源加一组近场 points，并把卫星放到其近场阈值内触发质点求和分支。
        if real_m >= 1 {
            let near_idx = 2; // sources[2] 是第一个真实源
            let near_src_h = sources[near_idx].handle;
            let near_pos = positions[near_idx];
            sources[near_idx] = NBodySource::irregular(
                near_src_h,
                1.0e20,
                vec![
                    MassPoint {
                        local_offset: Vector::new(1.0e5, 0.0, 0.0),
                        gm: 3.0e19,
                    },
                    MassPoint {
                        local_offset: Vector::new(0.0, 1.0e5, 0.0),
                        gm: 3.0e19,
                    },
                    MassPoint {
                        local_offset: Vector::new(0.0, 0.0, 1.0e5),
                        gm: 4.0e19,
                    },
                ],
                1.0e4, // bounding_radius → 近场阈值 8e4，卫星需在其内
            );
            let ctx_irr = AccelContext {
                celestials: &[],
                n_body_sources: &sources,
                source_positions: &positions,
                source_rotations: &rots,
                source_pos_gm: &pos_gm_full,
                softening_sq: 0.0,
                central_body: None,
                sun_position: Vector::ZERO,
                relativistic: mps_cosmos::world::RelativisticCorrection::None,
                has_irregular_sources: true,
            };
            // 卫星放在 near_src 近场阈值内（dist < 8e4），触发 points 求和分支。
            let probe_irr = near_pos + Vector::new(1.0e3, 0.0, 0.0);
            let serial =
                total_acceleration(probe_irr, Vector::ZERO, 1000.0, sat_hdl, &ctx_irr, None);
            let parallel = n_body_acceleration_reduce(probe_irr, sat_hdl, &ctx_irr);
            assert_eq!(
                serial, parallel,
                "irregular: real_m={real_m} 近场并行归约与串行不逐位一致 serial={serial:?} parallel={parallel:?}"
            );
        }
    }
}

/// F（C2 复活，bit-identical 版 SIMD）lock-down：4 路 SIMD 远场 `far_field_monopole_simd`
/// 必须与逐源标量循环的 `total_acceleration` 逐位一致（分量全相等，0 容差）。
///
/// 覆盖：M=0..512 多档、全 monopole 路径（SOA 读取）+ 近场不规则源（整组回退标量），
/// 含排除/零-gm 边界。SIMD 版用**标量 sqrt + lane-wise mul/div + 按源序逐 lane 左折叠**，
/// 与串行求和顺序完全一致 → 逐位一致、与 AVX 调度无关。
#[test]
fn far_field_simd_bit_identical_with_serial_scalar_loop() {
    use mps_cosmos::integrator::far_field_monopole_simd;

    let sat_hdl = RigidBodyHandle::from_raw_parts(0, 0);
    let probe_positions = [
        Vector::new(7.0e6, 0.0, 0.0),
        Vector::new(0.0, 1.3e7, -2.1e6),
        Vector::new(-4.0e6, 5.0e6, 9.0e6),
        Vector::new(3.3e6, -7.7e6, 1.1e6),
    ];

    for &real_m in &[0usize, 1, 7, 8, 16, 64, 256, 512] {
        // 真实源 handle 的 arena idx 取 1000+i，模拟真实 arena 布局。
        let pos_cap = 1000 + real_m + 2;
        let mut sources: Vec<NBodySource> = Vec::with_capacity(real_m + 2);
        sources.push(NBodySource::monopole(sat_hdl, 5.972e24)); // 排除项
        sources.push(NBodySource::monopole(
            RigidBodyHandle::from_raw_parts(999, 0),
            0.0,
        )); // 零 gm
        let total = real_m + 2;
        let mut positions: Vec<Vector> = vec![Vector::ZERO; pos_cap];
        let mut pos_gm_full = vec![(Vector::ZERO, 0.0); total];
        pos_gm_full[0] = (Vector::ZERO, 5.972e24);
        pos_gm_full[1] = (Vector::ZERO, 0.0);
        let mut rng = 0x9e3779b97f4a7c15u64 ^ (real_m as u64).wrapping_mul(0x100000001b3);
        for i in 0..real_m {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let a = (rng & 0xffff) as f64 * 2.0 * std::f64::consts::PI / 65535.0;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let b = ((rng >> 16) & 0xffff) as f64 * std::f64::consts::PI / 65535.0;
            let rad = 1.0e7 + ((rng >> 32) & 0xffff) as f64 * 5.0e6;
            let p = Vector::new(
                rad * b.sin() * a.cos(),
                rad * b.cos(),
                rad * b.sin() * a.sin(),
            );
            let h = RigidBodyHandle::from_raw_parts(1000 + i as u32, 0);
            let gm = 1.0e20 + (i as f64) * 1.0e15;
            sources.push(NBodySource::monopole(h, gm));
            positions[1000 + i] = p;
            pos_gm_full[2 + i] = (p, gm);
        }
        let rots: Vec<Rotation> = (0..pos_cap).map(|_| Rotation::IDENTITY).collect();

        let ctx_mono = AccelContext {
            celestials: &[],
            n_body_sources: &sources,
            source_positions: &positions,
            source_rotations: &rots,
            source_pos_gm: &pos_gm_full,
            softening_sq: 0.0,
            central_body: None,
            sun_position: Vector::ZERO,
            relativistic: mps_cosmos::world::RelativisticCorrection::None,
            has_irregular_sources: false,
        };
        for &pos in &probe_positions {
            let serial = total_acceleration(pos, Vector::ZERO, 1000.0, sat_hdl, &ctx_mono, None);
            let simd = far_field_monopole_simd(pos, sat_hdl, &ctx_mono);
            assert_eq!(
                serial, simd,
                "F monopole: real_m={real_m} SIMD 与标量不逐位一致 serial={serial:?} simd={simd:?}"
            );
        }

        // 不规则近场源（整组回退标量）：SIMD 路径对含 points 的 4 元组回退，须仍一致。
        if real_m >= 1 {
            let near_idx = 2;
            let near_pos = positions[near_idx];
            sources[near_idx] = NBodySource::irregular(
                sources[near_idx].handle,
                1.0e20,
                vec![
                    MassPoint {
                        local_offset: Vector::new(1.0e5, 0.0, 0.0),
                        gm: 3.0e19,
                    },
                    MassPoint {
                        local_offset: Vector::new(0.0, 1.0e5, 0.0),
                        gm: 3.0e19,
                    },
                    MassPoint {
                        local_offset: Vector::new(0.0, 0.0, 1.0e5),
                        gm: 4.0e19,
                    },
                ],
                1.0e4,
            );
            let ctx_irr = AccelContext {
                celestials: &[],
                n_body_sources: &sources,
                source_positions: &positions,
                source_rotations: &rots,
                source_pos_gm: &pos_gm_full,
                softening_sq: 0.0,
                central_body: None,
                sun_position: Vector::ZERO,
                relativistic: mps_cosmos::world::RelativisticCorrection::None,
                has_irregular_sources: true,
            };
            let probe_irr = near_pos + Vector::new(1.0e3, 0.0, 0.0);
            let serial =
                total_acceleration(probe_irr, Vector::ZERO, 1000.0, sat_hdl, &ctx_irr, None);
            let simd = far_field_monopole_simd(probe_irr, sat_hdl, &ctx_irr);
            assert_eq!(
                serial, simd,
                "F irregular: real_m={real_m} SIMD 与标量不逐位一致 serial={serial:?} simd={simd:?}"
            );
        }
    }
}

/// F 路由 lock-down：通过真实 env 开关 `COSMOS_FARFIELD_SIMD` 切换 `total_acceleration`
/// 的串行/SIMD 路径，二者输出必须逐位一致（进程内切换 env，覆盖真实路由分支）。
#[test]
fn far_field_simd_env_gate_routes_bit_identical() {
    let sat_hdl = RigidBodyHandle::from_raw_parts(0, 0);
    let real_m = 256usize;
    let pos_cap = 1000 + real_m + 2;
    let mut sources: Vec<NBodySource> = Vec::with_capacity(real_m + 2);
    sources.push(NBodySource::monopole(sat_hdl, 5.972e24));
    sources.push(NBodySource::monopole(
        RigidBodyHandle::from_raw_parts(999, 0),
        0.0,
    ));
    let total = real_m + 2;
    let mut positions: Vec<Vector> = vec![Vector::ZERO; pos_cap];
    let mut pos_gm_full = vec![(Vector::ZERO, 0.0); total];
    pos_gm_full[0] = (Vector::ZERO, 5.972e24);
    pos_gm_full[1] = (Vector::ZERO, 0.0);
    let mut rng = 0x1234567890abcdefu64;
    for i in 0..real_m {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let a = (rng & 0xffff) as f64 * 2.0 * std::f64::consts::PI / 65535.0;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let b = ((rng >> 16) & 0xffff) as f64 * std::f64::consts::PI / 65535.0;
        let rad = 1.0e7 + ((rng >> 32) & 0xffff) as f64 * 5.0e6;
        let p = Vector::new(
            rad * b.sin() * a.cos(),
            rad * b.cos(),
            rad * b.sin() * a.sin(),
        );
        let h = RigidBodyHandle::from_raw_parts(1000 + i as u32, 0);
        let gm = 1.0e20 + (i as f64) * 1.0e15;
        sources.push(NBodySource::monopole(h, gm));
        positions[1000 + i] = p;
        pos_gm_full[2 + i] = (p, gm);
    }
    let rots: Vec<Rotation> = (0..pos_cap).map(|_| Rotation::IDENTITY).collect();
    let ctx = AccelContext {
        celestials: &[],
        n_body_sources: &sources,
        source_positions: &positions,
        source_rotations: &rots,
        source_pos_gm: &pos_gm_full,
        softening_sq: 0.0,
        central_body: None,
        sun_position: Vector::ZERO,
        relativistic: mps_cosmos::world::RelativisticCorrection::None,
        has_irregular_sources: false,
    };
    let probe = Vector::new(7.0e6, -2.0e6, 3.5e6);

    unsafe { std::env::remove_var("COSMOS_FARFIELD_SIMD") };
    let serial = total_acceleration(probe, Vector::ZERO, 1000.0, sat_hdl, &ctx, None);
    unsafe { std::env::set_var("COSMOS_FARFIELD_SIMD", "1") };
    let simd = total_acceleration(probe, Vector::ZERO, 1000.0, sat_hdl, &ctx, None);
    unsafe { std::env::remove_var("COSMOS_FARFIELD_SIMD") };
    assert_eq!(
        serial, simd,
        "F 路由：COSMOS_FARFIELD_SIMD=1 与串行不逐位一致 serial={serial:?} simd={simd:?}"
    );
}
