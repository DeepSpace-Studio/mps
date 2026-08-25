# lib.rs

## 作用
crate 根出境入口。对外重新导出 `rapier3d` crate 与 `rapier` 子模块全部公开接口,并把 `rapier::ffi::*` 直通到 crate 顶层,供 `mps-jni` / Java 调用。`jni_api` 子模块将这些再导出集中包装为面向 JNI 的别名集合(`aero/fl/mol/traj` 等)。`anvilkit-bridge` feature 下额外暴露 `AnvilKitAppHandle`。

## 关键导出
- `pub extern crate rapier3d` — 直接重新导出 Rapier3d,下游无需单独声明依赖。
- `pub mod rapier` — 物理引擎主体子模块。
- `pub use rapier::ffi::*` — 将 FFI 句柄与 C ABI 函数平铺到 crate 根。
- `pub mod jni_api` — 集中再导出供 `mps-jni` 使用的各 Rapier 子模块别名与 `ffi::*`。
- `pub use rapier::ffi::AnvilKitAppHandle`(feature `anvilkit-bridge`)— AnvilKit 应用级桥句柄。
- `jni_api::anvilkit`(feature `anvilkit-bridge`)— 桥子模块别名。

## 依赖
- 外部 crate:`rapier3d`。
- 本 crate 子模块:`rapier::ffi`、`rapier::anvilkit`(条件),以及 `aerodynamics/bounds/bridge/collider/compat/controller/crbtree/dop/error/events/fluid/joints/molecular/neural/query/rigid_body/rtree/spaceflight/trajectory/voxel/world`。
