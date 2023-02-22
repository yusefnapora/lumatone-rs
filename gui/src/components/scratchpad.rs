use dioxus::prelude::*;
use crate::{components::{
  tabs::{TabContainer, TabItem},
  wheel::ColorWheel,
}, harmony::view_model::{Tuning, Scale}};

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::c_major();

  cx.render(rsx! {
    div {
      width: "100%",
      height: "100%",

      TabContainer {
        tabs: vec![
          TabItem {
            title: "Wheel",
            id: "wheel",
            content: cx.render(rsx! {
              div {
                max_width: "600px",
                max_height: "600px",
              
              ColorWheel {
                tuning: tuning,
                scale: scale,
              }
            }
            })
          },

          TabItem {
            title: "Keyboard",
            id: "keyboard",
            content: cx.render(rsx! {
              div { 
                "bar"
              }
            })
          },

          TabItem {
            title: "Baz",
            id: "baz",
            content: cx.render(rsx! {
              div { 
                "baz"
              }
            })
          }          
        ]
      }
    }
  })
}
