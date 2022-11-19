use dioxus::prelude::*;

use crate::components::wheel::ColorWheel;

/// Home "/" route
pub fn home(cx: Scope) -> Element {
  cx.render(rsx! {
    div {
      width: "1000px",
      height: "1000px",

      ColorWheel { radius: 300.0 }
    }
  })
}