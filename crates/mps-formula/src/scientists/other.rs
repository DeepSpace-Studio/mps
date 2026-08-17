//! 跨科学家共享公式集合。
//!
//! 某些公式由多位科学家独立给出（或作为公共物理定律被多人使用），
//! 不应在每位科学家的文件里各写一份相同实现。本模块集中存放这类
//! "共同公式"，各科学家文件通过 `pub use crate::scientists::other::formulas::*;`
//! （或具名 `pub use`）引用，避免重复定义与 E0252 冲突。
//!
//! 纯函数，不依赖 Rapier / `WorldHandle`。

use crate::error::*;
use crate::ffi::*;
use crate::math::*;

/// 普朗克常数 h = 6.62607015e-34 J·s（2019 SI 精确值）。
pub const PLANCK: f64 = 6.626_070_15e-34;

/// 跨科学家共享公式集合。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    use super::PLANCK;

    /// 德布罗意波长（物质波）：λ = h / (m·v)。
    ///
    /// 由 de Broglie (1924) 提出，是波粒二象性的核心关系，被 Schrödinger、
    /// Dirac 等多位量子力学奠基者共用，故置于本共享模块。
    /// `mass` 与 `velocity` 必须为有限正值。
    pub fn de_broglie_wavelength(mass: f64, velocity: f64) -> Option<f64> {
        if !mass.is_finite() || mass <= 0.0 || !velocity.is_finite() || velocity <= 0.0 {
            return None;
        }
        Some(PLANCK / (mass * velocity))
    }
}
