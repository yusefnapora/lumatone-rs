pub mod coordinates;

pub use hexagon_tiles::point::Point;

/// Just a typedef for the floating point type used for coordinates, etc.
/// This only exists to make it a bit easier to change to f64 if that's ever
/// needed.
pub type Float = f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
  Degrees(Float),
  Radians(Float),
}

impl From<Float> for Angle {
  fn from(f: Float) -> Self {
    Angle::Degrees(f)
  }
}

impl Angle {
  pub fn as_degrees(&self) -> Float {
    match self {
      Angle::Degrees(d) => *d,
      Angle::Radians(r) => r.to_degrees(),
    }
  }

  pub fn as_radians(&self) -> Float {
    match self {
      Angle::Degrees(d) => d.to_radians(),
      Angle::Radians(r) => *r,
    }
  }
}

/// Convert polar coordinates in the form of (center, radius, angle) to
/// Cartesian (x,y) coordinates.
pub fn polar_to_cartesian(center: Point, radius: Float, angle: Angle) -> Point {
  let a = (angle.as_degrees() - 90.0).to_radians();
  Point {
    x: center.x + (radius * a.cos()),
    y: center.y + (radius * a.sin()),
  }
}

/// Return a String describing an SVG path for an arc segment.
/// The arc will follow the circumference of an imaginary circle of
/// the given `center` and `radius`, and will be filled from `start`
/// to `end` angles.
pub fn arc_svg_path(center: Point, radius: Float, start: Angle, end: Angle) -> String {
  let large_arc_flag = if end.as_degrees() - start.as_degrees() <= 180.0 {
    "0"
  } else {
    "1"
  };

  let Point {
    x: start_x,
    y: start_y,
  } = polar_to_cartesian(center, radius, end);
  let Point { x: end_x, y: end_y } = polar_to_cartesian(center, radius, start);
  format!("M {start_x} {start_y} A {radius} {radius} 0 {large_arc_flag} 0 {end_x} {end_y}")
}

/// Return a String describing an SVG line from the current point to the given point `p`.
pub fn line_to(p: Point) -> String {
  format!("L {}, {}", p.x, p.y)
}

/// Given a center point and the size (indiameter) of a hexagon, return
/// the x,y position of a single corner, identfied by an index from 0-5.
pub fn hex_corner(center: Point, size: Float, corner_index: u8) -> Point {
  assert!(corner_index < 6, "invalid hex corner index {corner_index}");

  let angle = Angle::Degrees((60.0 * (corner_index as Float)) - 30.0);
  let radians = angle.as_radians();
  Point {
    x: center.x + size * radians.cos(),
    y: center.y + size * radians.sin(),
  }
}

/// Given a center point and the size (indiameter) of a hexagon,
/// return a String describing the points needed to render an SVG
/// <polygon> element.
pub fn hexagon_svg_points(center: Point, size: Float) -> String {
  (0..6)
    .map(|i| hex_corner(center, size, i))
    .map(|pt| format!("{},{}", pt.x, pt.y))
    .collect::<Vec<String>>()
    .join(" ")
}
