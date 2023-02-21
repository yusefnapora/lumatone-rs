#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
pub(crate) mod components;
pub(crate) mod drawing;
pub(crate) mod harmony;
pub(crate) mod hooks;

use components::scratchpad::Scratchpad;

use dioxus::prelude::*;
use hooks::useuniqueid::use_unique_id_provider;

fn main() {
  hot_reload_init!();
  dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
  use_unique_id_provider(cx);

  cx.render(rsx! {
    style { include_str!("./app.css") },
    Scratchpad { }
  })
}
