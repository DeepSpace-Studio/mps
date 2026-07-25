use topcoat::router::Router;

mod layouts;
mod components;
mod pages;

use pages::home::home;
use pages::quickstart::quickstart;
use pages::architecture::architecture;
use pages::gravity::gravity;
use pages::integrators::integrators;
use pages::formula::formula;
use pages::voxel::voxel;
use pages::events::events;
use pages::arena::arena;
use pages::jni::jni;
use pages::ffm::ffm;
use pages::api::api;
use layouts::root_layout;

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