//! 天体物理与宇宙学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "astro"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::astro::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "astro",
    name_zh: "天体物理与宇宙学",
    name_en: "Astrophysics & Cosmology",
    parent_id: "",
    summary: "恒星、星系、宇宙大尺度结构与演化",
    key_symbols: "H0, Ω, Λ",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::johannes_kepler::formulas`
///   - `crate::scientists::pierre_simon_laplace::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::johannes_kepler::formulas::*;
    pub use crate::scientists::pierre_simon_laplace::formulas::einstein_de_sitter_age;
    pub use crate::scientists::pierre_simon_laplace::formulas::einstein_radius;
    pub use crate::scientists::pierre_simon_laplace::formulas::flat_universe_lookback_time;
    pub use crate::scientists::pierre_simon_laplace::formulas::friedmann_hubble_distance;
    pub use crate::scientists::pierre_simon_laplace::formulas::hawking_temperature;
    pub use crate::scientists::pierre_simon_laplace::formulas::hubble_distance;
    pub use crate::scientists::pierre_simon_laplace::formulas::hubble_flow_velocity;
    pub use crate::scientists::pierre_simon_laplace::formulas::hubble_recession_velocity;
    pub use crate::scientists::pierre_simon_laplace::formulas::luminosity_distance_hubble;
}
