use dioxus::prelude::*;
use palette::LinSrgb;
use crate::{
	drawing::Point,
	components::{
  	tabs::{TabContainer, TabItem},
		keyboard::{layout::Layout, coords::gen_full_board_coords, board::Board},
  	wheel::ColorWheel,
	}, 
	harmony::view_model::{Tuning, Scale}
};

use super::keyboard::map::{DebugMapper, LumatoneLocationDebugMapper};

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::c_major();
	let hex_size = Point { x: 25.0, y: 25.0 };
	let layout = Layout::new(hex_size);
  let location_debug_mapper = Box::new(LumatoneLocationDebugMapper{});
  let coord_keymapper = Box::new(DebugMapper { color: LinSrgb::new(1.0, 0.0, 0.0) });

  cx.render(rsx! {
    div {
      width: "100%",
      height: "100%",

      TabContainer {
        tabs: vec![
          TabItem {
            title: "Hex Coords",
            id: "keyboard",
            content: cx.render(rsx! {
							svg {
								width: "2000px",
								height: "1200px",

                Board {
                  layout: layout,
                  coordinates: gen_full_board_coords(),
                  mapper: coord_keymapper,
                }
							}
            })
          },

          TabItem {
            title: "Lumatone Key indices",
            id: "keyboard-indices",
            content: cx.render(rsx! {
							svg {
								width: "2000px",
								height: "1200px",

                Board {
                  layout: layout,
                  coordinates: gen_full_board_coords(),
                  mapper: location_debug_mapper,
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
