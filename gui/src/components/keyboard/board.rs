use dioxus::prelude::*;
use std::collections::HashSet;
use super::{coords::Hex, layout::Layout, key::Key};
#[derive(Props)]
pub struct BoardProps<'a> {
  layout: Layout,
  coordinates: HashSet<Hex>,

	on_hex_clicked: Option<EventHandler<'a, Hex>>
}

pub fn Board<'a>(cx: Scope<'a, BoardProps<'a>>) -> Element {
	
  let keys = cx.props.coordinates.iter().map(|c| {
		rsx! {
			Key {
				fill_color: "red".into(), // TODO: get from delegate fn in props
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
	});


	cx.render(
    rsx! {
      g { 
				keys 
			}
    }
  )
}
