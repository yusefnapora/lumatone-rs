use crate::drawing::{Float, Point};

use super::coords::{FractionalHex, Hex};


/// A transformation matrix for hexagon orientations (pointy or flat).
/// We only support pointy hexes, but if we ever need flat ones, we'll
/// have the data structure :)
/// see https://www.redblobgames.com/grids/hexagons/implementation.html#Layout
#[derive(Copy, Clone)]
struct Orientation {
	f0: f64,
	f1: f64,
	f2: f64,
	f3: f64,
	b0: f64,
	b1: f64,
	b2: f64,
	b3: f64,
	start_angle: f64,
}

impl Orientation {
	pub const fn new(f0: f64, f1: f64, f2: f64, f3: f64, b0: f64, b1: f64, b2: f64, b3: f64, start_angle: f64) -> Orientation {
		Orientation { f0, f1, f2, f3, b0, b1, b2, b3, start_angle }
	}
}

static SQRT_3: f64 = 1.7320508075688772935274463415058723669428052538103806280558069794;

static POINTY: Orientation = Orientation::new(
	SQRT_3,
	SQRT_3 / 2.0,
	0.0,
	3.0 / 2.0,
	SQRT_3 / 3.0,
	-1.0 / 3.0,
	0.0,
	2.0 / 3.0,
	0.5
);

static DEFAULT_HEX_SIZE: f64 = 30.0;

pub struct Layout {
	m: &'static Orientation,
	origin: Point,
	size: Point,
}

impl Layout {
	pub fn new() -> Layout {
		let origin = Point { x: 0.0, y: 0.0 };
		let size = Point { x: DEFAULT_HEX_SIZE, y: DEFAULT_HEX_SIZE };
		Layout { m: &POINTY, size, origin }
	}

	pub fn with_origin(mut self, origin: Point) -> Self {
		self.origin = origin;
		self
	}

	pub fn with_size(mut self, size: Float) -> Self {
		self.size = Point { x: size, y: size };
		self
	}


	pub fn screen_coords(&self, hex: Hex) -> Point {
		let m = self.m;
		let x = (m.f0 * (hex.q() as f64) + m.f1 * (hex.r() as f64)) * self.size.x;
		let y = (m.f2 * (hex.q() as f64) + m.f3 * (hex.r() as f64)) * self.size.y;

		let x = x + self.origin.x;
		let y = y + self.origin.y;
		Point { x, y }
	}

	pub fn hex_coords(&self, p: Point) -> FractionalHex {
		let m = self.m;
		let p = Point {
			x: (p.x - self.origin.x) / self.size.x,
			y: (p.y - self.origin.y) / self.size.y,
		};
		let q = m.b0 * p.x + m.b1 * p.y;
		let r = m.b2 * p.x + m.b3 * p.y;
		FractionalHex::new(q, r)
	}

	fn corner_offset(&self, corner: u8) -> Point {
		use std::f64::consts::PI;
		let size = self.size;
		let angle = 2.0 * PI * (self.m.start_angle + corner as f64) / 6.0;
		Point {
			x: size.x * angle.cos(),
			y: size.y * angle.sin(),
		}
	}

	pub fn polygon_corners(&self, h: Hex) -> Vec<Point> {
		let center = self.screen_coords(h);
		(0..6).map(|i| {
			let offset = self.corner_offset(i);
			Point { x: center.x + offset.x, y: center.y + offset.y }
		}).collect()
	}
}

