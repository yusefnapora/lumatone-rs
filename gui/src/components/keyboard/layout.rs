use crate::drawing::Point;
use super::coords::{FractionalHex, Hex};
use std::ops::Deref;
use hexagon_tiles::layout::{Layout as _Layout, LayoutTool, LAYOUT_ORIENTATION_POINTY};

#[derive(Clone, Copy)]
pub struct Layout(_Layout);

impl Deref for Layout {
	type Target = _Layout;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq for Layout {
	fn eq(&self, rhs: &Self) -> bool {
		self.size == rhs.size && self.origin == rhs.origin
	}
}

impl Layout {
	pub fn new(size: Point) -> Layout {
		let origin = Point { x: 0.0, y: 0.0 };
		Layout(_Layout { 
			orientation: LAYOUT_ORIENTATION_POINTY,
			size,
			origin
		})
	}

	pub fn with_origin(&mut self, origin: Point) -> Self {
		self.0.origin = origin;
		*self
	}

	pub fn hex_to_pixel(&self, hex: Hex) -> Point {
		LayoutTool::hex_to_pixel(**self, *hex)
	}

	pub fn pixel_to_hex(&self, point: Point) -> FractionalHex {
		LayoutTool::pixel_to_hex(**self, point)
	}

	pub fn polygon_corners(&self, hex: Hex) -> Vec<Point> {
		LayoutTool::polygon_corners(**self, *hex)
	}
}
