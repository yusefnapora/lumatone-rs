use super::{key::Key, layout::Layout, map::KeyMapper};
use dioxus::prelude::*;
use std::collections::HashSet;
use lumatone_core::geometry::coordinates::Hex;
#[derive(Props)]
pub struct BoardProps<'a> {
  layout: Layout,
  coordinates: HashSet<Hex>,

  mapper: Box<dyn KeyMapper>,
  on_hex_clicked: Option<EventHandler<'a, Hex>>,
}

pub fn Board<'a>(cx: Scope<'a, BoardProps<'a>>) -> Element {
  let keys = cx.props.coordinates.iter().map(|c| {
    let dioxus_key = c.to_string();
    if let Some(def) = cx.props.mapper.key_definition_for_coordinate(c) {
      rsx! {
        Key {
          key: "{dioxus_key}",
          fill_color: def.color,
          label: def.label,
          layout: &cx.props.layout,
          coord: *c,
          on_click: move |coord| {
            if let Some(handler) = &cx.props.on_hex_clicked {
              handler.call(coord);
            } else {
              println!("hex clicked: {coord:?}");
            }
          }
        }
      }
    } else {
      rsx! {
        // TODO: is it possible to just return None here?
        g { key: "{dioxus_key}" }
      }
    }
  });

  cx.render(rsx! {
    g {
      keys
    }
  })
}
