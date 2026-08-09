#ifndef COSMOS_H
#define COSMOS_H

#pragma once

/* Generated with cbindgen:0.29.4 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * 默认近场阈值倍率：|d| ≤ 8·bounding_radius 时走质点求和。8 给到 r² 误差 ~1.5%
 * 的 monopole，足够典型的薄壳/扁平分布过渡到 monopole。
 */
#define NEAR_FIELD_FACTOR 8.0

/**
 * 太空物理世界。所有公开 API 自行管理内部 `RigidBodySet` 等。
 *
 * 手写 `Clone`（而非 derive）因为 `PhysicsPipeline` 不实现 `Clone`——它是
 * 无状态的工作对象（每次 `step` 内部重建临时结构），克隆时用 `::new()`
 * 恢复即可。用途：场景快照/回滚（演练器 undo、Monte Carlo 多世界并行）。
 * 成本是深拷贝整个 body/collider set；超大规模场景应考虑 `Arc` 共享只读配置。
 */
typedef struct CosmosWorld CosmosWorld;



#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * 构造一个动态刚体 builder（质量 kg、初始位置/速度）。返回 `*mut` 给调用
 * 方；后续交给 `cosmos_world_insert_body` 插入。失败（panic）返回 null。
 */
RigidBodyBuilder *cosmos_satellite_builder(double mass,
                                           double px,
                                           double py,
                                           double pz,
                                           double vx,
                                           double vy,
                                           double vz,
                                           double radius);

/**
 * 构造固定（静态）刚体 builder —— 适合做 n-body 引力源中心本体。
 */
RigidBodyBuilder *cosmos_fixed_body_builder(double px, double py, double pz);

void cosmos_builder_set_linear_damping(RigidBodyBuilder *builder, double value);

void cosmos_builder_set_angular_damping(RigidBodyBuilder *builder, double value);

void cosmos_builder_set_gravity_scale(RigidBodyBuilder *builder, double value);

/**
 * **激活**平移锁定（动态刚体不再平动，仅可转动）。`RigidBodyBuilder::lock_translations`
 * 是消费 self 的链式 API，这里把裸指针的 builder 取出、调用后再放回原地。
 */
void cosmos_builder_lock_translations(RigidBodyBuilder *builder);

/**
 * 显式释放一个**未插入**的 builder。插入 `cosmos_world_insert_body` 后所有权
 * 已转移，**不要**再调本函数（会 double-free）。null 是 no-op。
 */
void cosmos_builder_destroy(RigidBodyBuilder *builder);

/**
 * 创建一个 `CosmosWorld`。
 *
 * 参数：
 * - `dt`：积分步长（秒），合法范围 `0 < dt ≤ 30`；
 * - `solver_iterations`、`ccd_substeps`：rapier 求解器参数；
 * - `orbit_integration`：0 = `RapierForce`（默认），1 = `Verlet`，
 *   2 = `Yoshida4`，3 = `Yoshida4Kahan`，4 = `ForestRuth8`，5 = `ForestRuth8Kahan`；
 * - `verlet_substeps`：Verlet 路径的内部子步数（≥1，仅 `Verlet` 模式生效）；
 * - `n_body_softening_sq`：n-body 互引力软化平方项（m²），0 表示无软化。
 *
 * 失败（panic）返回 null。
 */
struct CosmosWorld *cosmos_world_create(double dt,
                                        uint32_t solver_iterations,
                                        uint32_t ccd_substeps,
                                        uint32_t orbit_integration,
                                        uint32_t verlet_substeps,
                                        double n_body_softening_sq);

/**
 * 销毁 `cosmos_world_create` 产出的世界。null 是 no-op。
 */
void cosmos_world_destroy(struct CosmosWorld *world);

/**
 * 设太阳位置（光压方向参考）。
 */
void cosmos_world_set_sun_position(struct CosmosWorld *world, Vec3 pos);

/**
 * 设/改 n-body 中心天体（按整数 id：0=Sun..9=Neptune）。`id < 0` 清除。
 * 返回 1 成功 / 0 失败（world 为 null）。
 */
uint8_t cosmos_world_set_central_body(struct CosmosWorld *world, int32_t id);

/**
 * 注册一个天体引力源。`celestial_id` 见 `cosmos_world_set_central_body`；
 * `max_sh_degree` 限制球谐展开最高阶（受 `body.max_degree` 上限约束）。
 * 返回注册到世界中的源索引（≥0 成功；-1 参数错 / world 为 null）。
 */
int32_t cosmos_world_add_celestial(struct CosmosWorld *world,
                                   int32_t celestial_id,
                                   uint32_t max_sh_degree);

/**
 * 把已插入的刚体登记为 n-body 互引力质点源（给定质量 kg）。
 * `body` 是 `cosmos_world_insert_body` 返回的 packed handle。返回 1 / 0。
 */
uint8_t cosmos_world_add_n_body(struct CosmosWorld *world, uint64_t body, double mass);

/**
 * 插入 builder，返回打包的 body 句柄（0 = 失败）。builder 所有权转移。
 * 连带把质量登记为 n-body 源。
 */
uint64_t cosmos_world_insert_body_as_gravity_source(struct CosmosWorld *world,
                                                    RigidBodyBuilder *builder,
                                                    double mass);

/**
 * 插入 builder，返回打包的 body 句柄（0 = 失败）。builder 所有权转移。
 */
uint64_t cosmos_world_insert_body(struct CosmosWorld *world, RigidBodyBuilder *builder);

/**
 * 设置某刚体的环境扰动配置（大气阻力 + 太阳光压 + 太阳风动压 +
 * Chandrasekhar 动力学摩擦）。返回 1 / 0。
 *
 * `sun_position` 通过 `cosmos_world_set_sun_position` 单独设置；太阳风方向
 * 复用 `sun_position → 刚体位置` 的世界方向。
 */
uint8_t cosmos_world_set_perturbation(struct CosmosWorld *world,
                                      uint64_t body,
                                      double drag_coefficient,
                                      double area,
                                      int32_t enable_drag,
                                      double reflectivity,
                                      double optical_area,
                                      int32_t enable_solar,
                                      double solar_wind_proton_density,
                                      double solar_wind_speed,
                                      double solar_wind_area,
                                      int32_t enable_solar_wind,
                                      double friction_background_density,
                                      double friction_coulomb_log,
                                      int32_t enable_dynamical_friction);

/**
 * 推进一步，返回一个 `int` 编码的 `StepResult`：
 * - `>0`：`Stepped(n)` —— 实际推进的秒数 ×1000；
 * - `-1`：`Substepped`（拆子步完成）；
 * - `-2`：`Skipped(NonFinite)`（dt 为 NaN/Inf）；
 * - `-3`：`Skipped(NonPositive)`（dt ≤ 0）；
 * - `-4`：`Skipped(TooLarge)`（dt 超过 30s 硬上限）。
 */
int32_t cosmos_world_step(struct CosmosWorld *world, double dt);

/**
 * 循环 `n` 次推进 `dt`，任一步非法整批拒。
 * 返回 0 = 成功；1 = NonFinite；2 = NonPositive；3 = TooLarge。
 */
int32_t cosmos_world_step_n(struct CosmosWorld *world, double dt, uint32_t n);

/**
 * 取刚体当前位置（3×f64）。`out` 指向 24 字节 native 缓冲（`Vec3`）。
 * 返回 1 成功 / 0 句柄无效或 world 为 null。
 */
int32_t cosmos_body_translation_out(const struct CosmosWorld *world, uint64_t body, Vec3 *out);

/**
 * 取刚体当前线速度（3×f64）。
 */
int32_t cosmos_body_linvel_out(const struct CosmosWorld *world, uint64_t body, Vec3 *out);

/**
 * 取刚体质量（kg）。`NaN` 表示句柄无效 / world 为 null。
 */
double cosmos_body_mass(const struct CosmosWorld *world, uint64_t body);

/**
 * Hill 球半径（m）：刚体作为卫星时其自引力主导范围，与 Roche 极限互补。
 *
 * 主星质量由 `cosmos_world_set_central_body` 注册的天体 GM/G 反算；卫星质量
 * 取自刚体本身；`semi_major_axis`（m）与 `eccentricity`（0..=1）由调用方提
 * 供。`NaN` 表示无 `central_body` / 句柄无效 / 参数非法。
 */
double cosmos_hill_radius_for(const struct CosmosWorld *world,
                              uint64_t body,
                              double semi_major_axis,
                              double eccentricity);

/**
 * 当前动态刚体数量。
 */
uint32_t cosmos_world_dynamic_body_count(const struct CosmosWorld *world);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* COSMOS_H */
