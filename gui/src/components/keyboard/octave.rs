//! This module contains a dioxus component that renders a single "octave" of the Lumatone layout,
//! where an octave is a 56-key section of the board. 
//!
//! If we number the rows from top to bottom, with the origin at top-left,
//! each octave is layed out as a rectangle with
//! 11 rows of six columns, with a few grid locations "missing" in rows 0, 1, 9, and 10.
//! 
//!
//!  0: <><>            - row 0 only has two keys
//!  1:  <><><><><>     - row 1 has 5 keys
//!  2: <><><><><><>    - rows 2-8 have 6 keys
//!  3:  <><><><><><>
//!  4: <><><><><><>
//!  5:  <><><><><><>
//!  6: <><><><><><>
//!  7:  <><><><><><>
//!  8: <><><><><><>
//!  9:    <><><><><>   - row 9 has 5 keys
//! 10:         <><>    - row 10 has 2 keys
//!
//! The `octave_num` prop affects the coordinate space covered by this component.
//! Each successive octave effectively shifts the origin 6 columns to the right
//! and two columns down.
//!
//! Thinking in "offset coordinates", where coords are (col, row) tuples,
//! octave 0 starts at (0,0), octave 1 starts at (6, 2), etc.
//!
//! 
use dioxus::prelude::*;
use palette::LinSrgb;
use crate::components::keyboard::{coords::gen_octave_coords, key::Key, layout::Layout};


#[derive(PartialEq, Props)]
pub struct OctaveProps {
	layout: Layout,

	octave_num: u8,

	// TODO: 
	// - add key_props_for_hex(coord: Hex) -> Option<KeyProps> delegate fn to get
	//   the definition for each key on the board.
}

/// Renders an SVG `<g>` element containing one octave of a Lumatone layout
pub fn Octave(cx: Scope<OctaveProps>) -> Element {
	let coords = gen_octave_coords(cx.props.octave_num);
	
	let keys = coords.iter().map(|c| {
		rsx! {
			Key {
				fill_color: LinSrgb::new(1.0, 0.0, 0.0), // TODO: get from delegate fn in props
				layout: &cx.props.layout,
				coord: *c,
			}
		}
	});

	cx.render(rsx! {
		g {
			keys
		}
	})
}

