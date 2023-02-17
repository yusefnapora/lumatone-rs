use crate::components::wheel::ColorWheel;
use crate::harmony::view_model::{Scale, Tuning};
use dioxus::prelude::*;

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::d_major();
  cx.render(rsx! {
    div {
      width: "600px",
      height: "600px",

      ColorWheel { radius: 300.0, tuning: tuning, scale: scale }
    }
  })
}
