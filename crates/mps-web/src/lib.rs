// Topcoat `#[component]` functions use PascalCase by framework convention, and
// the macro's generated props struct trips dead-code lints on its fields.
#![allow(non_snake_case, dead_code)]

use topcoat::router::Router;

mod components;
mod layouts;
mod pages;

use layouts::root_layout;
use pages::api::api;
use pages::architecture::architecture;
use pages::arena::arena;
use pages::events::events;
use pages::ffm::ffm;
use pages::formula::formula;
use pages::gravity::gravity;
use pages::home::home;
use pages::integrators::integrators;
use pages::jni::jni;
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
        .page(jni)
        .page(ffm)
        .page(api)
        .build()
}
