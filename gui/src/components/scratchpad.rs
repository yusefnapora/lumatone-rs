use crate::{
  components::{
    keyboard::{board::Board, layout::Layout},
    tabs::{TabContainer, TabItem},
    wheel::ColorWheel,
  },
  harmony::view_model::{Scale, Tuning},
};
use lumatone_core::geometry::Point;
use lumatone_core::geometry::coordinates::gen_full_board_coords;
use dioxus::prelude::*;
use palette::LinSrgb;

use super::keyboard::map::{DebugMapper, LumatoneLocationDebugMapper};

pub fn Scratchpad(cx: Scope<()>) -> Element {
  let tuning = Tuning::edo_12();
  let scale = Scale::c_major();
  let hex_size = Point { x: 25.0, y: 25.0 };
  let layout = Layout::new(hex_size);
  let location_debug_mapper = Box::new(LumatoneLocationDebugMapper {});
  let coord_keymapper = Box::new(DebugMapper {
    color: LinSrgb::new(1.0, 0.0, 0.0),
  });

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
