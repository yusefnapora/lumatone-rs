#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
pub(crate) mod components;
pub(crate) mod drawing;
pub(crate) mod harmony;
pub(crate) mod hooks;

#[macro_use]
extern crate lazy_static;

use components::scratchpad::Scratchpad;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use hooks::useuniqueid::use_unique_id_provider;

fn main() {
  // hot_reload_init!();
  let config = Config::default()
    .with_window(
      WindowBuilder::new()
        .with_maximized(true)
        .with_title("Lumatone Playground")
    );
  dioxus_desktop::launch_cfg(app, config);
}

fn app(cx: Scope) -> Element {
  use_unique_id_provider(cx);

  cx.render(rsx! {
    style { include_str!("./app.css") },
    Scratchpad { }
  })
}
