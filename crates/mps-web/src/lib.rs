// MPS Web — single-page documentation (SSR-only, no router).
//
// The site is served fully server-side; there is no client WASM bundle
// (no `dx` CLI in this environment). To avoid the Dioxus `Link` click
// interceptor silently eating navigation under SSR-only, the whole doc is
// ONE inline page: a sticky in-page table-of-contents with plain
// `<a href="#sec-...">` anchors (native scroll, zero JS, no hydration trap).

#![allow(non_snake_case, unused)]

mod i18n;
mod layouts;
mod pages;

mod metrics;

use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::langid;

use pages::home::Home;

pub fn main() {
    dioxus::launch(Home);
}
