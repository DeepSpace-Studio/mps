// Topcoat `#[component]` functions use PascalCase by framework convention, and
// the macro's generated props struct trips dead-code lints on its fields.
#![allow(non_snake_case, dead_code)]

use topcoat::router::Router;

/// Auto-generated metrics constants (see `xtask dump-metrics`,
/// OPTIMIZATION.md §N3). Used by `pages/home.rs` to keep the displayed test
/// counts/JNI method counts/FFI counts in sync with the source.
pub mod metrics;

mod i18n;
mod layouts;
mod pages;

use layouts::root_layout;
use pages::api::api;
use pages::architecture::architecture;
use pages::arena::arena;
use pages::cosmos::cosmos;
use pages::events::events;
use pages::ffm::ffm;
use pages::formula::formula;
use pages::gravity::gravity;
use pages::home::home;
use pages::integrators::integrators;
use pages::jni::jni;
use pages::page_not_found::page_not_found;
use pages::quickstart::quickstart;
use pages::voxel::voxel;

/// Build and return the application router.
pub fn app() -> Router {
    Router::builder()
        .layout(root_layout)
        .page(home)
        .page(quickstart)
        .page(architecture)
        .page(gravity)
        .page(integrators)
        .page(formula)
        .page(voxel)
        .page(events)
        .page(arena)
        .page(cosmos)
        .page(jni)
        .page(ffm)
        .page(api)
        .page(page_not_found)
        .build()
}
