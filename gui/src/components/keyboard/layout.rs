use lumatone_core::geometry::{
  Point,
  coordinates::{FractionalHex, Hex}
};
use hexagon_tiles::layout::{
  Layout as _Layout, LayoutTool, Orientation, LAYOUT_ORIENTATION_POINTY,
};
use std::ops::Deref;

// the lumatone has what's essentially a "pointy hex" layout that's rotated by -17.42 degrees
const LUMATONE_ROTATION_DEGREES: f64 = -17.42;

fn rot_vec2d(x: f64, y: f64, r: f64) -> (f64, f64) {
  let r = r.to_radians();
  let x2 = (x * r.cos()) - (y * r.sin());
  let y2 = (x * r.sin()) + (y * r.cos());
  (x2, y2)
}

fn rotate_orientation(o: Orientation, r: f64) -> Orientation {
  let (f0, f2) = rot_vec2d(o.f0, o.f2, r);
  let (f1, f3) = rot_vec2d(o.f1, o.f3, r);
  let (b0, b2) = rot_vec2d(o.b0, o.b2, r);
  let (b1, b3) = rot_vec2d(o.b1, o.b3, r);
  let start_angle = o.start_angle + r.to_radians();

  Orientation {
    f0,
    f1,
    f2,
    f3,
    b0,
    b1,
    b2,
    b3,
    start_angle,
  }
}

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
    // translate the default origin a bit, so that the rotated tips of the hexagons
    // don't get clipped off. This constant was derived from trial & error and could
    // use more thought.
    let origin = Point {
      x: size.x,
      y: size.y * 3.0,
    };

    let orientation = rotate_orientation(LAYOUT_ORIENTATION_POINTY, LUMATONE_ROTATION_DEGREES);
    Layout(_Layout {
      orientation,
      size,
      origin,
    })
  }

  pub fn size(&self) -> Point {
    self.0.size
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

  pub fn svg_polygon_points(&self, hex: Hex) -> String {
    self
      .polygon_corners(hex)
      .iter()
      .map(|c| format!("{},{}", c.x, c.y))
      .collect::<Vec<String>>()
      .join(" ")
  }
}
