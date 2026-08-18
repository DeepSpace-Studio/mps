//! mps-cosmos 的 C ABI 出口（FFI 层）。
//!
//! 由 `mps-jni`（JNI 路径）与 `test25/RigidBodyFfm`（FFM 路径）共同消费，
//! 作为 mps-cosmos 的唯一 C ABI 来源。本模块的符号由 cbindgen 生成
//! `crates/mps-cosmos/include/cosmos.h` —— 与 `mps-core` 的 `rigid_body.h`
//! 平行、互不耦合（[CLAUDE.md]：mps-cosmos 不介入 mps-core 的 C ABI）。
//!
//! 每个入口都过 `ffi_guard`：panic → `set_error(ERR_INTERNAL, ...)` + 失败
//! 哨兵（与 mps-core/`src/rapier/error.rs` 同模式）。workspace `panic = "abort"`
//! 再兜一层，panics 不可能 unwind 进 JVM。
//!
//! ### Handle 编码
//!
//! `cosmos_world_*` 返回 `*mut CosmosWorld`（Java 拥有，须伴 `cosmos_world_destroy`）。
//! builder 同理返回 `*mut RigidBodyBuilder`。插入后返回的 packed `u64` body
//! handle 与 JNI 侧 `pack_handle` 完全一致：`(idx << 32) | generation`，
//! 顺序与 `RigidBodyHandle::into_raw_parts()` 相同。**注意**：这不是
//! mps-core 的 `+1/-1` 编码，cosmos 用 raw parts。

use std::panic::{AssertUnwindSafe, catch_unwind};

use mps_formula::celestial_data::{celestial_body_id_from_u32, get_celestial_body};
use mps_formula::error::{
    ERR_CAPACITY, ERR_INTERNAL, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, set_error,
};
use mps_formula::ffi::Vec3;
use rapier3d::prelude::{RigidBodyBuilder, RigidBodyHandle, Vector};

use crate::bodies::{fixed_body_builder, satellite_builder};
use crate::gravity::CelestialSource;
use crate::world::{CosmosWorld, CosmosWorldConfig, OrbitIntegration, StepResult, StepSkipReason};

/// 由 `CelestialBodyId`（整数 0..=9）拿 `&'static CelestialBody`；非法则 `None`。
fn celestial_by_id(id: i32) -> Option<&'static mps_formula::celestial_data::CelestialBody> {
    let id = u32::try_from(id).ok()?;
    celestial_body_id_from_u32(id).map(get_celestial_body)
}

/// 单次 `*_snapshot` 调用允许的最大 body 容量（与 mps-core
/// `ffi::convert::MAX_OUTPUT_CAPACITY` 对齐——cosmos 不依赖 mps-core，
/// 此常量在本 crate 内独立定义；改动时请两边同步）。
const MAX_OUTPUT_CAPACITY: u32 = 1_000_000;

/// 把 `RigidBodyHandle` 打包为 `u64`：`(idx << 32) | generation`。
fn pack_handle(h: RigidBodyHandle) -> u64 {
    let (idx, generation) = h.into_raw_parts();
    ((idx as u64) << 32) | (generation as u64)
}

/// 反解 `pack_handle` 产物。
fn unpack_handle(packed: u64) -> RigidBodyHandle {
    let idx = ((packed >> 32) & 0xFFFF_FFFF) as u32;
    let generation = (packed & 0xFFFF_FFFF) as u32;
    RigidBodyHandle::from_raw_parts(idx, generation)
}

/// panic → `ERR_INTERNAL`，返回 `default`（FFI 边界统一兜底）。
fn ffi_guard<R>(default: R, f: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(_) => {
            set_error(ERR_INTERNAL, "internal panic");
            default
        }
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// 构造一个动态刚体 builder（质量 kg、初始位置/速度）。返回 `*mut` 给调用
/// 方；后续交给 `cosmos_world_insert_body` 插入。失败（panic）返回 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_satellite_builder(
    mass: f64,
    px: f64,
    py: f64,
    pz: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    radius: f64,
) -> *mut RigidBodyBuilder {
    ffi_guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(satellite_builder(
            mass,
            Vector::new(px, py, pz),
            Vector::new(vx, vy, vz),
            radius,
        )))
    })
}

/// 构造固定（静态）刚体 builder —— 适合做 n-body 引力源中心本体。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_fixed_body_builder(px: f64, py: f64, pz: f64) -> *mut RigidBodyBuilder {
    ffi_guard(std::ptr::null_mut(), || {
        Box::into_raw(Box::new(fixed_body_builder(Vector::new(px, py, pz))))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn cosmos_builder_set_linear_damping(builder: *mut RigidBodyBuilder, value: f64) {
    ffi_guard((), || {
        if let Some(b) = unsafe { builder.as_mut() } {
            b.linear_damping = value;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn cosmos_builder_set_angular_damping(builder: *mut RigidBodyBuilder, value: f64) {
    ffi_guard((), || {
        if let Some(b) = unsafe { builder.as_mut() } {
            b.angular_damping = value;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn cosmos_builder_set_gravity_scale(builder: *mut RigidBodyBuilder, value: f64) {
    ffi_guard((), || {
        if let Some(b) = unsafe { builder.as_mut() } {
            b.gravity_scale = value;
        }
    });
}

/// **激活**平移锁定（动态刚体不再平动，仅可转动）。`RigidBodyBuilder::lock_translations`
/// 是消费 self 的链式 API，这里把裸指针的 builder 取出、调用后再放回原地。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_builder_lock_translations(builder: *mut RigidBodyBuilder) {
    ffi_guard((), || {
        if builder.is_null() {
            return;
        }
        unsafe {
            let b = Box::from_raw(builder);
            let b = b.lock_translations();
            let _ = Box::into_raw(Box::new(b));
        }
    });
}

/// 显式释放一个**未插入**的 builder。插入 `cosmos_world_insert_body` 后所有权
/// 已转移，**不要**再调本函数（会 double-free）。null 是 no-op。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_builder_destroy(builder: *mut RigidBodyBuilder) {
    ffi_guard((), || {
        if !builder.is_null() {
            drop(unsafe { Box::from_raw(builder) });
        }
    });
}

// ---------------------------------------------------------------------------
// World 生命周期与配置
// ---------------------------------------------------------------------------

/// 创建一个 `CosmosWorld`。
///
/// 参数：
/// - `dt`：积分步长（秒），合法范围 `0 < dt ≤ 30`；
/// - `solver_iterations`、`ccd_substeps`：rapier 求解器参数；
/// - `orbit_integration`：0 = `RapierForce`（默认），1 = `Verlet`，
///   2 = `Yoshida4`，3 = `Yoshida4Kahan`，4 = `ForestRuth8`，5 = `ForestRuth8Kahan`；
/// - `verlet_substeps`：Verlet 路径的内部子步数（≥1，仅 `Verlet` 模式生效）；
/// - `n_body_softening_sq`：n-body 互引力软化平方项（m²），0 表示无软化。
///
/// 失败（panic）返回 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_create(
    dt: f64,
    solver_iterations: u32,
    ccd_substeps: u32,
    orbit_integration: u32,
    verlet_substeps: u32,
    n_body_softening_sq: f64,
) -> *mut CosmosWorld {
    ffi_guard(std::ptr::null_mut(), || {
        let orbit_integration = match orbit_integration {
            1 => OrbitIntegration::Verlet,
            2 => OrbitIntegration::Yoshida4,
            3 => OrbitIntegration::Yoshida4Kahan,
            4 => OrbitIntegration::ForestRuth8,
            5 => OrbitIntegration::ForestRuth8Kahan,
            _ => OrbitIntegration::RapierForce,
        };
        let cfg = CosmosWorldConfig {
            gravity: Vector::ZERO,
            dt,
            solver_iterations,
            ccd_substeps,
            n_body_softening_sq,
            central_body: None,
            orbit_integration,
            verlet_substeps: verlet_substeps.max(1),
            ..CosmosWorldConfig::default()
        };
        Box::into_raw(Box::new(CosmosWorld::new(cfg)))
    })
}

/// 销毁 `cosmos_world_create` 产出的世界。null 是 no-op。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_destroy(world: *mut CosmosWorld) {
    ffi_guard((), || {
        if !world.is_null() {
            drop(unsafe { Box::from_raw(world) });
        }
    });
}

/// 设太阳位置（光压方向参考）。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_set_sun_position(world: *mut CosmosWorld, pos: Vec3) {
    ffi_guard((), || {
        if let Some(w) = unsafe { world.as_mut() } {
            w.set_sun_position(Vector::new(pos.x, pos.y, pos.z));
        }
    });
}

/// 设/改 n-body 中心天体（按整数 id：0=Sun..9=Neptune）。`id < 0` 清除。
/// 返回 1 成功 / 0 失败（world 为 null）。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_set_central_body(world: *mut CosmosWorld, id: i32) -> u8 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 0,
        };
        let body = if id < 0 { None } else { celestial_by_id(id) };
        w.set_central_body(body);
        1
    })
}

/// 注册一个天体引力源。`celestial_id` 见 `cosmos_world_set_central_body`；
/// `max_sh_degree` 限制球谐展开最高阶（受 `body.max_degree` 上限约束）。
/// 返回注册到世界中的源索引（≥0 成功；-1 参数错 / world 为 null）。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_add_celestial(
    world: *mut CosmosWorld,
    celestial_id: i32,
    max_sh_degree: u32,
) -> i32 {
    ffi_guard(-1, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return -1,
        };
        let body = match celestial_by_id(celestial_id) {
            Some(t) => t,
            None => return -1,
        };
        let src = CelestialSource::new(body, max_sh_degree);
        w.add_celestial(src) as i32
    })
}

/// 把已插入的刚体登记为 n-body 互引力质点源（给定质量 kg）。
/// `body` 是 `cosmos_world_insert_body` 返回的 packed handle。返回 1 / 0。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_add_n_body(world: *mut CosmosWorld, body: u64, mass: f64) -> u8 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 0,
        };
        w.add_n_body(unpack_handle(body), mass);
        1
    })
}

/// 插入 builder，返回打包的 body 句柄（0 = 失败）。builder 所有权转移。
/// 连带把质量登记为 n-body 源。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_insert_body_as_gravity_source(
    world: *mut CosmosWorld,
    builder: *mut RigidBodyBuilder,
    mass: f64,
) -> u64 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 0,
        };
        if builder.is_null() {
            return 0;
        }
        let b = *unsafe { Box::from_raw(builder) };
        pack_handle(w.insert_body_as_gravity_source(b, mass))
    })
}

/// 插入 builder，返回打包的 body 句柄（0 = 失败）。builder 所有权转移。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_insert_body(
    world: *mut CosmosWorld,
    builder: *mut RigidBodyBuilder,
) -> u64 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 0,
        };
        if builder.is_null() {
            return 0;
        }
        let b = *unsafe { Box::from_raw(builder) };
        pack_handle(w.insert_body(b))
    })
}

/// 设置某刚体的环境扰动配置（大气阻力 + 太阳光压 + 太阳风动压 +
/// Chandrasekhar 动力学摩擦）。返回 1 / 0。
///
/// `sun_position` 通过 `cosmos_world_set_sun_position` 单独设置；太阳风方向
/// 复用 `sun_position → 刚体位置` 的世界方向。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_set_perturbation(
    world: *mut CosmosWorld,
    body: u64,
    drag_coefficient: f64,
    area: f64,
    enable_drag: i32,
    reflectivity: f64,
    optical_area: f64,
    enable_solar: i32,
    solar_wind_proton_density: f64,
    solar_wind_speed: f64,
    solar_wind_area: f64,
    enable_solar_wind: i32,
    friction_background_density: f64,
    friction_coulomb_log: f64,
    enable_dynamical_friction: i32,
) -> u8 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 0,
        };
        w.set_perturbation(
            unpack_handle(body),
            crate::world::PerturbationConfig {
                drag_coefficient,
                area,
                enable_drag: enable_drag != 0,
                reflectivity,
                optical_area,
                enable_solar: enable_solar != 0,
                solar_wind_proton_density,
                solar_wind_speed,
                solar_wind_area,
                enable_solar_wind: enable_solar_wind != 0,
                friction_background_density,
                friction_coulomb_log,
                enable_dynamical_friction: enable_dynamical_friction != 0,
            },
        );
        1
    })
}

/// 推进一步，返回一个 `int` 编码的 `StepResult`：
/// - `>0`：`Stepped(n)` —— 实际推进的秒数 ×1000；
/// - `-1`：`Substepped`（拆子步完成）；
/// - `-2`：`Skipped(NonFinite)`（dt 为 NaN/Inf）；
/// - `-3`：`Skipped(NonPositive)`（dt ≤ 0）；
/// - `-4`：`Skipped(TooLarge)`（dt 超过 30s 硬上限）。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_step(world: *mut CosmosWorld, dt: f64) -> i32 {
    ffi_guard(-2, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return -2,
        };
        match w.step(dt) {
            StepResult::Stepped(n) => ((n * 1000.0).round() as i64).max(1) as i32,
            StepResult::Substepped { .. } => -1,
            StepResult::Skipped(StepSkipReason::NonFinite) => -2,
            StepResult::Skipped(StepSkipReason::NonPositive) => -3,
            StepResult::Skipped(StepSkipReason::TooLarge) => -4,
        }
    })
}

/// 循环 `n` 次推进 `dt`，任一步非法整批拒。
/// 返回 0 = 成功；1 = NonFinite；2 = NonPositive；3 = TooLarge。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_step_n(world: *mut CosmosWorld, dt: f64, n: u32) -> i32 {
    ffi_guard(1, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => return 1,
        };
        match w.step_n(dt, n) {
            Ok(()) => 0,
            Err(StepSkipReason::NonFinite) => 1,
            Err(StepSkipReason::NonPositive) => 2,
            Err(StepSkipReason::TooLarge) => 3,
        }
    })
}

/// 创建共享内存 arena（Java 零拷贝命令通道 + 状态回读）。
///
/// 写入 `out_address` / `out_size`（传 `null` 可跳过对应输出）；返回的 `*mut CosmosWorld`
/// 不变。一个世界最多一个 arena，已存在则原样保留并返回 `false`。容量必须 >0 且
/// 不超过上限，总分配 ≤ 256 MiB。Java 侧用 `out_address`/`out_size` 把这块内存
/// 映射成 native-order 的 `ByteBuffer`，命令环写入 + body 槽零拷贝读取都走它。
///
/// # Safety
/// `world` 须为 `cosmos_world_create` 产出的有效指针或 null；`out_address` /
/// `out_size` 若为非负则指向 8 字节可写内存。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_create_shared_arena(
    world: *mut CosmosWorld,
    max_bodies: u32,
    max_commands: u32,
    out_address: *mut u64,
    out_size: *mut u64,
) -> i32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_mut() } {
            Some(t) => t,
            None => {
                set_error(ERR_NULL_POINTER, "cosmos world is null");
                return 0;
            }
        };
        if !out_address.is_null() {
            unsafe {
                *out_address = 0;
            }
        }
        if !out_size.is_null() {
            unsafe {
                *out_size = 0;
            }
        }
        if !w.create_shared_arena(max_bodies, max_commands) {
            set_error(
                ERR_INVALID_ARGUMENT,
                "arena create failed (exists or bad capacity)",
            );
            return 0;
        }
        if !out_address.is_null() {
            unsafe {
                *out_address = w.shared_arena_address();
            }
        }
        if !out_size.is_null() {
            unsafe {
                *out_size = w.shared_arena_size();
            }
        }
        1
    })
}

/// 销毁共享 arena（若有的话）。`null` world 是 no-op。销毁前 Java 必须已释放映射
/// 该 arena 的 `ByteBuffer`，否则会 use-after-free。
///
/// # Safety
/// `world` 须为 `cosmos_world_create` 产出的有效指针或 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_destroy_shared_arena(world: *mut CosmosWorld) {
    ffi_guard((), || {
        if let Some(w) = unsafe { world.as_mut() } {
            w.destroy_shared_arena();
        }
    })
}

/// 取 arena 基地址（无 arena 时返回 0）。供 Java 映射 `ByteBuffer` 的地址来源。
///
/// # Safety
/// `world` 须为 `cosmos_world_create` 产出的有效指针或 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_get_shared_arena_address(world: *const CosmosWorld) -> u64 {
    ffi_guard(0, || {
        unsafe { world.as_ref() }.map_or(0, |w| w.shared_arena_address())
    })
}

/// 取 arena 总字节大小（无 arena 时返回 0）。供 Java 映射 `ByteBuffer` 的容量来源。
///
/// # Safety
/// `world` 须为 `cosmos_world_create` 产出的有效指针或 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_get_shared_arena_size(world: *const CosmosWorld) -> u64 {
    ffi_guard(0, || {
        unsafe { world.as_ref() }.map_or(0, |w| w.shared_arena_size())
    })
}

/// 取刚体当前位置（3×f64）。`out` 指向 24 字节 native 缓冲（`Vec3`）。
/// 返回 1 成功 / 0 句柄无效或 world 为 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_body_translation_out(
    world: *const CosmosWorld,
    body: u64,
    out: *mut Vec3,
) -> i32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return 0,
        };
        let Some(p) = w.body_translation(unpack_handle(body)) else {
            return 0;
        };
        if let Some(o) = unsafe { out.as_mut() } {
            o.x = p.x;
            o.y = p.y;
            o.z = p.z;
        }
        1
    })
}

/// 取刚体当前线速度（3×f64）。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_body_linvel_out(
    world: *const CosmosWorld,
    body: u64,
    out: *mut Vec3,
) -> i32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return 0,
        };
        let Some(v) = w.body_linvel(unpack_handle(body)) else {
            return 0;
        };
        if let Some(o) = unsafe { out.as_mut() } {
            o.x = v.x;
            o.y = v.y;
            o.z = v.z;
        }
        1
    })
}

/// 取刚体质量（kg）。`NaN` 表示句柄无效 / world 为 null。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_body_mass(world: *const CosmosWorld, body: u64) -> f64 {
    ffi_guard(f64::NAN, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return f64::NAN,
        };
        w.body_mass(unpack_handle(body)).unwrap_or(f64::NAN)
    })
}

/// Hill 球半径（m）：刚体作为卫星时其自引力主导范围，与 Roche 极限互补。
///
/// 主星质量由 `cosmos_world_set_central_body` 注册的天体 GM/G 反算；卫星质量
/// 取自刚体本身；`semi_major_axis`（m）与 `eccentricity`（0..=1）由调用方提
/// 供。`NaN` 表示无 `central_body` / 句柄无效 / 参数非法。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_hill_radius_for(
    world: *const CosmosWorld,
    body: u64,
    semi_major_axis: f64,
    eccentricity: f64,
) -> f64 {
    ffi_guard(f64::NAN, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return f64::NAN,
        };
        w.hill_radius_for(unpack_handle(body), semi_major_axis, eccentricity)
            .unwrap_or(f64::NAN)
    })
}

/// 当前动态刚体数量。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_dynamic_body_count(world: *const CosmosWorld) -> u32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return 0,
        };
        w.dynamic_body_count() as u32
    })
}

/// 动态刚体数量（用于 sizing `cosmos_world_dynamic_body_snapshot` 调用）。
///
/// 与 [`cosmos_world_dynamic_body_count`] 在当前实现里返回相同数；
/// 单独导出独立计法以与 mps-core `world_dynamic_body_snapshot_count`
/// 的 ABI 形态对齐——Java 端可以以完全相同的 Java 代码模式先用
/// `cosmosWorldDynamicBodySnapshotCount` 拿到 N，分配 `long[N]` 与
/// `double[N * 7]`，再调 `cosmosWorldDynamicBodySnapshot` 拉一次全数据。
///
/// # Safety
/// `world` 可为 null（返回 0），其余情形须是 `cosmos_world_create` 产出的有效指针。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_dynamic_body_snapshot_count(world: *const CosmosWorld) -> u32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => return 0,
        };
        w.dynamic_body_count() as u32
    })
}

/// 批量快照动态刚体的 handle + pose（7 f64/body：pos3 + quat4）。
///
/// 与 mps-core `world_dynamic_body_snapshot` 完全平行，目的同样：把每 tick
/// Java 端原本要按 N 次 `cosmos_body_translation_out` 往返取所有 pos 的
/// 路径合并成**一次 JNI 调用 + 一份连续 f64[]**——N=1000 卫星的取位延迟
/// 从 ~600 µs/tick 砍到 ~50 µs/tick（见 `性能分析.MD` §11.1 / §12.1，
/// M1 + L1 同根改动）。
///
/// # 布局
/// - `out_handles[i]`: `pack_handle = (idx << 32) | generation` —— 与
///   `cosmos_insert_body` / `cosmos_body_translation_out` 等所有 cosmos
///   body handle ABI 一致（**注意**：与 mps-core 的 `+1/-1` 编码不同）。
/// - `out_values[i * 7 .. i * 7 + 3]`：位置 `pos.x, pos.y, pos.z`
/// - `out_values[i * 7 + 3 .. i * 7 + 7]`：旋转 `quat.i, quat.j, quat.k, quat.w`
///   （与 rapier3d `Rotation::xyzw` 顺序一致，Java 端可以直接映射到
///   `Quatd(i, j, k, w)` 或 JOML `Quaterniond`）。
///
/// 只写动态刚体（`is_dynamic() == true`），跳过 fixed / kinematic —— 与
/// `dynamic_body_count` / `cosmosWorldDynamicBodySnapshotCount` 一致。容量
/// 不够时只填到 `capacity` 为止并返回实际数量；调用方应按 count 先分配。
///
/// # 返回值
/// 实际写入的 body 数；任一前置参数非法返回 0（并 `set_error`）：
/// - `world` null → `ERR_NULL_POINTER`，返回 0
/// - `out_handles` / `out_values` null，或 `capacity == 0`，
///   或 `capacity > MAX_OUTPUT_CAPACITY` → `ERR_CAPACITY`，返回 0
///
/// # Safety
/// `out_handles` 指向至少 `capacity` 个 `u64` 可写内存；`out_values` 指向
/// 至少 `capacity * 7` 个 `f64` 可写内存。`world` 须为 `cosmos_world_create`
/// 产出的指针或 null。调用方应在写入完成前不让另一线程同时操作这两个缓冲。
#[unsafe(no_mangle)]
pub extern "C" fn cosmos_world_dynamic_body_snapshot(
    world: *const CosmosWorld,
    out_handles: *mut u64,
    out_values: *mut f64,
    capacity: u32,
) -> u32 {
    ffi_guard(0, || {
        let w = match unsafe { world.as_ref() } {
            Some(t) => t,
            None => {
                set_error(ERR_NULL_POINTER, "cosmos world is null");
                return 0;
            }
        };
        if out_handles.is_null() || out_values.is_null() {
            set_error(ERR_NULL_POINTER, "snapshot output buffer is null");
            return 0;
        }
        if capacity == 0 || capacity > MAX_OUTPUT_CAPACITY {
            set_error(ERR_CAPACITY, "invalid snapshot capacity");
            return 0;
        }

        let capacity = capacity as usize;
        let Some(value_capacity) = capacity.checked_mul(7) else {
            set_error(ERR_CAPACITY, "snapshot capacity overflow");
            return 0;
        };
        let handles = unsafe { std::slice::from_raw_parts_mut(out_handles, capacity) };
        let values = unsafe { std::slice::from_raw_parts_mut(out_values, value_capacity) };

        let mut written = 0usize;
        for (handle, body) in w.bodies().iter() {
            if !body.is_dynamic() {
                continue;
            }
            if written >= capacity {
                break;
            }

            let pos = body.translation();
            let rot = body.rotation();
            handles[written] = pack_handle(handle);
            let off = written * 7;
            values[off] = pos.x;
            values[off + 1] = pos.y;
            values[off + 2] = pos.z;
            values[off + 3] = rot.x; // i
            values[off + 4] = rot.y; // j
            values[off + 5] = rot.z; // k
            values[off + 6] = rot.w; // w
            written += 1;
        }

        written as u32
    })
}
