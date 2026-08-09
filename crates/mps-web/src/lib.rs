// MPS Web — Dioxus 0.7 + dioxus-i18n (Fluent)

#![allow(non_snake_case, unused)]

mod i18n;
mod layouts;
mod pages;

mod metrics;

use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::langid;

use layouts::Layout;
use pages::api::Api;
use pages::architecture::Architecture;
use pages::arena::Arena;
use pages::batch::Batch;
use pages::cosmos::Cosmos;
use pages::events::Events;
use pages::ffm::Ffm;
use pages::formula::Formula;
use pages::gravity::Gravity;
use pages::home::Home;
use pages::integrators::Integrators;
use pages::jni::Jni;
use pages::not_found::NotFound;
use pages::quickstart::Quickstart;
use pages::voxel::Voxel;

/// Route enum — Dioxus 0.7 Routable derive auto-generates parsing/rendering.
/// `#[layout(Layout)]` wraps every variant in layouts::Layout; the current
/// route's component is rendered wherever Layout places `Outlet::<Route>`.
/// `#[end_layout]` must be attached to the last variant covered by the layout.
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/quickstart")]
    Quickstart {},
    #[route("/architecture")]
    Architecture {},
    #[route("/gravity")]
    Gravity {},
    #[route("/integrators")]
    Integrators {},
    #[route("/formula")]
    Formula {},
    #[route("/voxel")]
    Voxel {},
    #[route("/events")]
    Events {},
    #[route("/arena")]
    Arena {},
    #[route("/batch")]
    Batch {},
    #[route("/cosmos")]
    Cosmos {},
    #[route("/jni")]
    Jni {},
    #[route("/ffm")]
    Ffm {},
    #[route("/api")]
    Api {},
    #[route("/404")]
    #[end_layout]
    NotFound {},
}

fn app() -> Element {
    // Initialize i18n with Fluent resources embedded via include_str! (WASM-safe static locales).
    use_init_i18n(|| {
        I18nConfig::new(langid!("zh-CN"))
            .with_fallback(langid!("zh-CN"))
            .with_locale((langid!("zh-CN"), include_str!("./i18n/locales/zh-CN.ftl")))
            .with_locale((langid!("en"), include_str!("./i18n/locales/en.ftl")))
    });

    rsx! {
        Router::<Route> {}
    }
}

pub fn main() {
    dioxus::launch(app);
}
