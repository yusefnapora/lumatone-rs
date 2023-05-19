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

	// the "pointy hex" layout needs to be rotated to match the layout
	// of the physical keyboard. A much better solution would be to
	// actually apply a rotation to the matrix in the layout struct,
	// but I'm not sure how to do that yet...
	// let transform = "rotate(-15)";

	cx.render(
    rsx! {
      g { 
				// transform: transform,
				keys 
			}
    }
  )
}
