# rapier/neural.rs

## 作用
"神经包围壳" (neural bounds) — 一类由小神经网络隐式定义的碰撞形状。`NeuralBoundsDesc` 描述网络结构(hidden_width、hidden_layers、激活函数),`eval_network` 在每个查询方向上展开点积-激活层算出隐函数值,据此定义接触壳,转成 Rapier `ColliderBuilder`/`SharedShape` 复合接触壳。FFI 入口处理构造、相交查询(标准 count/_count_all/_/_all 四变体)。

## 关键导出
- `struct NeuralWeights<'a>`(私有)— 顺序读取网络权重值流,`take()` 顺序消费一个有限 f64,`is_done()` 表征耗尽。
- 私有辅助:`activate(value, NeuralActivation)`(ReLU/Tanh/Sin/Linear)、`required_weight_count`、`eval_layer`、`eval_network`。
- `extern "C"` 入口(6 项):`neural_bounds_required_weight_count`、`collider_builder_create_neural_bounds`、`query_intersect_neural_bounds_count`、`query_intersect_neural_bounds_count_all`、`query_intersect_neural_bounds`、`query_intersect_neural_bounds_all`。
- 上限(私有):`MAX_SAMPLE_RESOLUTION`(128)、`MAX_HIDDEN_WIDTH`(4096)、`MAX_HIDDEN_LAYERS`(16)、`EPSILON`。

## 依赖
- 外部 crate:`rapier3d::math::{Pose, Rotation, Vector}`、`rapier3d::prelude::{ColliderBuilder, SharedShape}`、`smallvec::SmallVec`、`std::slice`。
- 本 crate 子模块:`crate::rapier::error`、`crate::rapier::ffi`(`NeuralActivation`、`NeuralBoundsDesc`、`ColliderBuilderHandle`、`ColliderHandleRaw`、`MAX_OUTPUT_CAPACITY`、`QueryFilterDesc`、`WorldHandle` 等及激活模式、quat/query_filter/vec3 与句柄打包辅助)。
