# rapier/rtree.rs

## 作用
自实现的 R-Tree 空间索引(节点最大子数 `MAX_CHILDREN = 8`),与 `crbtree.rs` 并列作为独立于 Rapier 广相的可选索引结构。私有 `Aabb` 结构提供 `from_desc` 有限性校验、`union` 包围盒合并、`intersects` 相交判定,索引支持插入/更新/删除以及整体 `rebuild`(重建以恢复查询效率)。条目上限 `MAX_TREE_ENTRIES`,输出上限 `MAX_OUTPUT_CAPACITY`。

## 关键导出
- `struct RTreeIndex`(crate 级)— R-Tree 本体,对外由 `RTreeHandle` 包装;内部提供 new/clear/len/insert/update/remove/rebuild/query 方法。
- `extern "C"` 入口(~10 项):`rtree_create/destroy/clear/len`、`rtree_insert`、`rtree_update`、`rtree_remove`、`rtree_rebuild`、`rtree_query_aabb_count`、`rtree_query_aabb`。
- `const MAX_CHILDREN`(私有,节点扇出)。

## 依赖
- 外部 crate:`smallvec::SmallVec`(节点子列表/查询结果缓冲)。
- 本 crate 子模块:`crate::rapier::error`(ERR_CAPACITY/ERR_INVALID_ARGUMENT/ERR_NOT_FOUND/ERR_NULL_POINTER/clear_error/ffi_guard/set_error)、`crate::rapier::ffi`(`AabbDesc`、`Bool`、`MAX_OUTPUT_CAPACITY`、`MAX_TREE_ENTRIES`、`RTreeHandle`、`Vec3`)。
- 无 Rapier 类型依赖(纯自实现索引)。
