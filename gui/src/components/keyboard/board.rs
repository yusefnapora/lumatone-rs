use dioxus::prelude::*;
use std::collections::HashSet;
use super::{coords::Hex, layout::Layout, key::Key};
#[derive(PartialEq, Props)]
pub struct BoardProps {
  layout: Layout,
  coordinates: HashSet<Hex>,
}

pub fn Board(cx: Scope<BoardProps>) -> Element {
  let keys = cx.props.coordinates.iter().map(|c| {
		rsx! {
			Key {
				fill_color: "red".into(), // TODO: get from delegate fn in props
				layout: &cx.props.layout,
				coord: *c,
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
