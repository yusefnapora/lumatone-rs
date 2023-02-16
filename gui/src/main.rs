#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
pub mod components;
pub mod drawing;
pub mod harmony;

use components::scratchpad::Scratchpad;

use dioxus::prelude::*;

fn main() {
  dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
  cx.render(rsx! {
    Scratchpad { }
  })
}
