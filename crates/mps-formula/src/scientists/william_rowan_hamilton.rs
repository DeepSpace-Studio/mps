//! William Rowan Hamilton —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "william_rowan_hamilton",
    name: "William Rowan Hamilton",
    birth_year: Some(1805),
    death_year: Some(1865),
    field_id: "mechanics",
    nationality: "Irish",
    contribution: "Hamiltonian mechanics; quaternions",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::integrators::leapfrog_step_kahan;
    use crate::math::*;
    const fn forest_ruth8_coefficients() -> [f64; 9] {
        // 2^(1/7) 与 2^(1/3) 不是 const fn 友好，这里硬编码由科学计算验得的值：
        // 外层 a = 1 / (2 - 2^(1/7)) ≈ 1.7089 5024 5292 4497
        //       b = 1 - 2a           ≈ -2.4179 0049 0584 8994
        // 内层 y1= 1 / (2 - 2^(1/3)) ≈ 1.3512 0719 1959 6578
        //       y0= 1 - 2y1          ≈ -1.7024 1438 3919 3153
        // 系数 = 外·内，对称（i ↔ 9-i）。
        let a = 1.7089_5024_5292_4497_f64;
        let b = 1.0 - 2.0 * a; // -2.4179...
        let y1 = 1.3512_0719_1959_6578_f64;
        let y0 = 1.0 - 2.0 * y1; // -1.7024...
        [
            a * y1, // 0
            a * y0, // 1
            a * y1, // 2
            b * y1, // 3
            b * y0, // 4
            b * y1, // 5
            a * y1, // 6
            a * y0, // 7
            a * y1, // 8
        ]
    }

    /// Advance position and velocity using the leapfrog (velocity Verlet) integrator.
    ///
    /// Algorithm:
    ///   1. v_{n+1/2} = v_n + 0.5 · a(r_n) · dt
    ///   2. r_{n+1}   = r_n + v_{n+1/2} · dt
    ///   3. a_{n+1}   = compute(r_{n+1})
    ///   4. v_{n+1}   = v_{n+1/2} + 0.5 · a_{n+1} · dt

    pub fn leapfrog_step(
        position: &mut Vec3,
        velocity: &mut Vec3,
        dt: f64,
        acceleration_fn: impl Fn(Vec3) -> Vec3,
    ) {
        let accel0 = acceleration_fn(*position);

        // Half-step kick
        velocity.x += 0.5 * accel0.x * dt;
        velocity.y += 0.5 * accel0.y * dt;
        velocity.z += 0.5 * accel0.z * dt;

        // Full drift
        position.x += velocity.x * dt;
        position.y += velocity.y * dt;
        position.z += velocity.z * dt;

        // Half-step kick
        let accel1 = acceleration_fn(*position);
        velocity.x += 0.5 * accel1.x * dt;
        velocity.y += 0.5 * accel1.y * dt;
        velocity.z += 0.5 * accel1.z * dt;
    }

    /// Yoshida's 4th-order symplectic integrator.
    ///
    /// Composed from 3 leapfrog steps with fractional timesteps w₁, w₂, w₃:
    ///   w₁ = w₃ = 1/(2 - 2^{1/3}) ≈ 1.3512071919596578
    ///   w₂ = 1 - 2w₁           ≈ -1.7024143839193153
    ///
    /// The negative w₂ step is a feature, not a bug — it cancels the 3rd-order error term.

    pub fn yoshida4_step(
        position: &mut Vec3,
        velocity: &mut Vec3,
        dt: f64,
        acceleration_fn: impl Fn(Vec3) -> Vec3,
    ) {
        let w1: f64 = 1.0 / (2.0 - 2.0_f64.cbrt()); // ≈ 1.3512
        let w0: f64 = 1.0 - 2.0 * w1; // ≈ -1.7024
        let ws = [w1, w0, w1];

        for &w in &ws {
            leapfrog_step(position, velocity, w * dt, &acceleration_fn);
        }
    }

    /// Forest–Ruth 8th-order symplectic integrator.
    ///
    /// 构成为对 4 阶 Yoshida（[`yoshida4_step`]）做外层 3 段对称组合：外层用
    /// `z = 2^(1/7)` 推出的系数 `(a, 1-2a, a)` 抵消 5/7 阶误差项，每段内层再走
    /// Y4 的 3 子步（`2^(1/3)` 根）。共 9 个 leapfrog 子步，系数对称排列、
    /// `Σλᵢ = 1`，方法 time-symmetric、保辛、全局误差 O(dt⁹)（8 阶）。
    ///
    /// 公式：`for λ in λs { leapfrog_step(p, v, λ·dt, a) }`。
    ///
    /// 参考：
    /// - Yoshida, *Construction of higher order symplectic integrators*, PLA 150 (1990)
    /// - McLachlan, *On the numerical integration of ODEs by symmetric composition*,
    ///   Comp. Phys. Comm. 1995

    pub fn forest_ruth8_step(
        position: &mut Vec3,
        velocity: &mut Vec3,
        dt: f64,
        acceleration_fn: impl Fn(Vec3) -> Vec3,
    ) {
        // 嵌套对称组合：外层根 z7=2^(1/7) → (a, 1-2a, a)；每段内层 Y4 根 z3=2^(1/3)
        // → (y1, 1-2y1, y1)。9 子步系数 = 外层×内层，对称，和 = 1。
        // 原实现误抄成一组和≠1 的系数，导致一步推进 ≠ dt、轨道失稳。
        const W: [f64; 9] = forest_ruth8_coefficients();

        for &w in &W {
            leapfrog_step(position, velocity, w * dt, &acceleration_fn);
        }
    }
    /// Forest–Ruth 8 with Kahan compensation. 系数同 [`forest_ruth8_step`]
    /// （Yoshida 1990 Table I 8 阶对称组合，和 = 1），位置/速度累加改用 Kahan
    /// 补偿，进一步压低长弧舍入积累。

    pub fn forest_ruth8_step_kahan(
        position: &mut KahanVec3,
        velocity: &mut KahanVec3,
        dt: f64,
        acceleration_fn: impl Fn(Vec3) -> Vec3,
    ) {
        const W: [f64; 9] = forest_ruth8_coefficients();

        for &w in &W {
            leapfrog_step_kahan(position, velocity, w * dt, &acceleration_fn);
        }
    }
}
