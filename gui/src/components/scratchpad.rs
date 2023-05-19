use dioxus::prelude::*;
use crate::{
	drawing::Point,
	components::{
  	tabs::{TabContainer, TabItem},
		keyboard::{layout::Layout, coords::gen_full_board_coords, board::Board},
  	wheel::ColorWheel,
	}, 
	harmony::view_model::{Tuning, Scale}
};

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::c_major();
	let hex_size = Point { x: 30.0, y: 30.0 };
	let layout = Layout::new(hex_size);

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
							svg {
								width: "1200px",
								height: "1200px",

                Board {
                  layout: layout,
                  coordinates: gen_full_board_coords(),
                }
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
