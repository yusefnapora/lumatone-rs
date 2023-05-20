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

use super::keyboard::map::DebugMapper;

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::c_major();
	let hex_size = Point { x: 25.0, y: 25.0 };
	let layout = Layout::new(hex_size);
  let keymapper = Box::new(DebugMapper { color: "blue".to_string() });

  cx.render(rsx! {
    div {
      width: "100%",
      height: "100%",

      TabContainer {
        tabs: vec![
          TabItem {
            title: "Keyboard",
            id: "keyboard",
            content: cx.render(rsx! {
							svg {
								width: "2000px",
								height: "1200px",

                Board {
                  layout: layout,
                  coordinates: gen_full_board_coords(),
                  mapper: keymapper,
                }
							}
            })
          },

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
        ]
      }
    }
  })
}
