#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
pub mod components;
pub mod drawing;

use components::wheel::ColorWheel;

use dioxus::prelude::*;

fn main() {
  dioxus::desktop::launch(app);
}

fn app(cx: Scope) -> Element {
  cx.render(rsx! {
    div {
      width: "1000px",
      height: "1000px",

      ColorWheel { radius: 500.0 }
    }
  })
}
