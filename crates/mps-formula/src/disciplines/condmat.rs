//! 凝聚态物理 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "condmat"` 对齐。纯数据 + 公式
//! `pub use` 窗口:本文件 re-export 该学科下每位科学家的 `formulas::`
//! 公式函数,使 `crate::disciplines::condmat::*` 即可取得该学科的全部
//! 公式入口。撞名函数保留第一所有者(字典序)的承载,后续同名者以
//! `_<owner>` 别名 re-export,避免 E0252 重复定义。不依赖 Rapier /
//! `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "condmat",
    name_zh: "凝聚态物理",
    name_en: "Condensed Matter Physics",
    parent_id: "",
    summary: "固体与液体中的集体行为、超导与超流",
    key_symbols: "Tc, lattice",
};

/// 该学科下所有科学家的公式函数 re-export 窗口(按字典序排序)。
/// 撞名函数后续所有者以 `_<owner>` 别名导入,避免 E0252 重复定义。
///
/// 涉及以下科学家的 `formulas` 模块:
///   - `crate::scientists::lev_landau::formulas`
#[allow(unused_imports)]
pub use self::__formulas_reexports::*;

/// 私有 re-export 中转模块,承载全部 `pub use crate::scientists::...`。
/// 公开层只通过上一行的 glob 暴露,以隔离 `#[allow(unused_imports)]` 的范围。
mod __formulas_reexports {
    pub use crate::scientists::lev_landau::formulas::*;
}
