use crate::components::keyboard::key::Key;
use crate::components::wheel::ColorWheel;
use crate::drawing::Point;
use crate::harmony::view_model::{Scale, Tuning};
use dioxus::prelude::*;

pub fn Scratchpad(cx: Scope<()>) -> Element {
  // let tuning = Tuning::edo_12();
  // let scale = Scale::d_major();

  let center = Point { x: 50.0, y: 50.0 };
  let fill_color = String::from("red");

  cx.render(rsx! {
    div {
      width: "600px",
      height: "600px",

      // ColorWheel { radius: 300.0, tuning: tuning, scale: scale }
      svg {
        width: "600px",
        height: "600px",

        Key {
          center: center,
          size: 50.0,
          fill_color: fill_color,
          label: "C"
        }
    }
    }
  })
}
