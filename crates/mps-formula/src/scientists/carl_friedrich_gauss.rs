//! Carl Friedrich Gauss —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "carl_friedrich_gauss",
    name: "Carl Friedrich Gauss",
    birth_year: Some(1777),
    death_year: Some(1855),
    field_id: "mathphys",
    nationality: "German",
    contribution: "Gauss's law; least squares; Gaussian units",
    key_constants: "",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;

    /// Associated Legendre function of the first kind P̄ₙₘ (4π-normalized).
    ///
    /// Recurrence relation (Holmes & Featherstone 2002):
    ///   P̄₀₀ = 1
    ///   P̄_{n,n} = √((2n+1)/(2n)) · cos(φ) · P̄_{n-1,n-1}
    ///   P̄_{n+1,n} = √(2n+3) · sin(φ) · P̄_{n,n}
    ///   P̄_{n,m} = a_{n,m} · sin(φ) · P̄_{n-1,m} - b_{n,m} · P̄_{n-2,m}
    ///
    /// where φ is the geocentric latitude (sin φ = z/r).
    ///
    /// Returns a vector `pnm` indexed as pnm[n*(n+1)/2 + m] for n=0..max_degree.

    pub fn normalized_legendre(sin_phi: f64, max_degree: u32) -> Vec<f64> {
        let n_max = max_degree as usize;
        let size = (n_max + 1) * (n_max + 2) / 2;
        let mut pnm = vec![0.0; size];

        pnm[0] = 1.0; // P̄₀₀

        if n_max == 0 {
            return pnm;
        }

        let cos_phi = (1.0 - sin_phi * sin_phi).sqrt().max(0.0);

        // Standard Holmes & Featherstone (2002) recurrence:
        // For each n, first compute P̄_{n,n} (sectoral), then P̄_{n,0..n-1}

        for n in 1..=n_max {
            let nf = n as f64;

            // ---- Sectoral term: P̄_{n,n} ----
            let idx_nn = n * (n + 1) / 2 + n;
            if n == 1 {
                // P̄₁₁ = √3 · cos φ
                pnm[idx_nn] = (3.0_f64).sqrt() * cos_phi;
            } else {
                let idx_prev_nn = (n - 1) * n / 2 + (n - 1);
                // P̄_{n,n} = √((2n+1)/(2n)) · cos φ · P̄_{n-1,n-1}
                let factor = ((2.0 * nf + 1.0) / (2.0 * nf)).sqrt();
                pnm[idx_nn] = factor * cos_phi * pnm[idx_prev_nn];
            }

            // ---- Tesseral terms: P̄_{n,m} for m = 0..n-1 ----
            // P̄_{n,m} = a_{n,m} · sin φ · P̄_{n-1,m} - b_{n,m} · P̄_{n-2,m}
            // where:
            //   a_{n,m} = √((2n-1)(2n+1) / ((n-m)(n+m)))
            //   b_{n,m} = √((2n+1)(n+m-1)(n-m-1) / ((2n-3)(n-m)(n+m)))
            for m in 0..n {
                let mf = m as f64;
                let idx = n * (n + 1) / 2 + m;

                if n == 1 {
                    // P̄₁₀ = √3 · sin φ
                    // index = n(n+1)/2 + m = 1 for P̄₁₀
                    pnm[1] = (3.0_f64).sqrt() * sin_phi;
                    continue;
                }

                let nm1_idx = (n - 1) * n / 2 + m;

                // a_{n,m}
                let a = {
                    let denom = (nf - mf) * (nf + mf);
                    if denom <= 0.0 {
                        // m = n gives sectoral (already done above), m=n-1 needs near-sectoral
                        continue;
                    }
                    ((2.0 * nf - 1.0) * (2.0 * nf + 1.0) / denom).sqrt()
                };

                // b_{n,m}
                let b = if n >= 2 && m < n - 1 {
                    let _nm2_idx = (n - 2) * (n - 1) / 2 + m;
                    let denom = (2.0 * nf - 3.0) * (nf - mf) * (nf + mf);
                    if denom <= 0.0 {
                        0.0
                    } else {
                        ((2.0 * nf + 1.0) * (nf + mf - 1.0) * (nf - mf - 1.0) / denom).sqrt()
                    }
                } else {
                    0.0
                };

                let nm2_idx = if n >= 2 { (n - 2) * (n - 1) / 2 + m } else { 0 };

                pnm[idx] = a * sin_phi * pnm[nm1_idx];
                if n >= 2 && b != 0.0 {
                    pnm[idx] -= b * pnm[nm2_idx];
                }
            }
        }

        pnm
    }
}
