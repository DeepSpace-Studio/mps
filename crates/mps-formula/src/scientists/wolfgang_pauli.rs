//! Wolfgang Pauli —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "wolfgang_pauli",
    name: "Wolfgang Pauli",
    birth_year: Some(1900),
    death_year: Some(1958),
    field_id: "quantum_mechanics",
    nationality: "Austrian/Swiss",
    contribution: "Pauli exclusion; spin matrices",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;

    /// Pauli sigma_x matrix-vector multiply.

    pub fn pauli_sigma_x(_spinor: (f64, f64)) -> ((f64, f64), (f64, f64)) {
        ((0.0, 1.0), (1.0, 0.0))
    }

    /// Pauli sigma_y matrix-vector multiply.

    pub fn pauli_sigma_y(spinor: (f64, f64)) -> ((f64, f64), (f64, f64)) {
        // ((0, -i), (i, 0))
        ((-spinor.1, spinor.0), (spinor.1, -spinor.0))
    }

    /// Pauli sigma_z matrix-vector multiply.

    pub fn pauli_sigma_z(spinor: (f64, f64)) -> (f64, f64) {
        (spinor.0, -spinor.1)
    }

    /// Spin expectation value in direction n from spinor.

    pub fn spin_expectation(spinor: (f64, f64)) -> (f64, f64, f64) {
        let (a, b) = spinor;
        let norm2 = a * a + b * b;
        if norm2 < 1.0e-15 {
            return (0.0, 0.0, 0.0);
        }
        let sx = 2.0 * (a * b) / norm2;
        let sy = 0.0; // simplified
        let sz = (a * a - b * b) / norm2;
        (sx, sy, sz)
    }
}
