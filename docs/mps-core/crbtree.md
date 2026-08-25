# rapier/crbtree.rs

## 作用
基于 `std::collections::BTreeMap`(标准库红黑/B 树)的轻量 AABB 索引结构,独立于 Rapier 的广相之外,供上层做按 id 键的包围盒登记与区域查询。内部私有 `Aabb` 结构做有限性校验(`from_desc`)与相交判定(`intersects`),查询为线性遍历 BTreeMap 条目取交集。条目上限受 `MAX_TREE_ENTRIES` 约束、输出受 `MAX_OUTPUT_CAPACITY` 约束。

## 关键导出
- `struct CRbTreeIndex`(crate 级)— 内含 `BTreeMap<u64, Aabb>`;提供 new/clear/len/insert/update/remove/query 内部方法,对外通过 `CRbTreeHandle` 暴露。
- `extern "C"` 入口(~10 项):`crb_tree_create/destroy/clear/len`、`crb_tree_insert(_flag)`、`crb_tree_update`、`crb_tree_remove`、`crb_tree_query_aabb_count`、`crb_tree_query_aabb`。

## 依赖
- 标准库:`std::collections::BTreeMap`。
- 本 crate 子模块:`crate::rapier::error`(ERR_CAPACITY/ERR_INVALID_ARGUMENT/ERR_NOT_FOUND/ERR_NULL_POINTER/clear_error/ffi_guard/set_error)、`crate::rapier::ffi`(`AabbDesc`、`Bool`、`CRbTreeHandle`、`MAX_OUTPUT_CAPACITY`、`MAX_TREE_ENTRIES`、`Vec3`)。
- 无 Rapier 类型依赖(纯自实现索引)。
