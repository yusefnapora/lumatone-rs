#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
pub mod components;
pub mod drawing;
pub mod routes;

use dioxus::prelude::*;
use dioxus::router::{Router, Route};

#[cfg(target_arch = "wasm32")]
fn launch() {
  dioxus::web::launch(app);
}

#[cfg(not(target_arch = "wasm32"))]
fn launch() {
  dioxus::desktop::launch(app);
}

fn main() {
  launch()
}

fn app(cx: Scope) -> Element {
  cx.render(rsx! {
    routes::home { }
  })
}
