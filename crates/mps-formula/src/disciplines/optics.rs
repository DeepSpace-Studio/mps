//! 光学 —— 一级学科记录。
//!
//! 与 `scientists` 模块通过 `field_id = "optics"` 对齐。纯数据，不依赖
//! Rapier / `WorldHandle`。

use super::Discipline;

/// 本学科记录。
#[allow(dead_code)]
pub const DISCIPLINE: Discipline = Discipline {
    field_id: "optics",
    name_zh: "光学",
    name_en: "Optics",
    parent_id: "",
    summary: "光的传播、干涉、衍射与波前调控",
    key_symbols: "λ, n, f-number",
};
