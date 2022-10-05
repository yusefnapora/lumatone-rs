use std::str::FromStr;
use palette::{LinSrgb, Gradient, Xyz, Srgb, IntoColor};

fn wheel_gradient() -> Gradient<LinSrgb> {
  // hard-code control points along an "RYB" color wheel
  // TODO: lerp over one of the Lab / Lch color spaces?
  let ryb_colors: Vec<LinSrgb<f32>> = vec![
    "#ff0000", "#bf0041", "#800080", "#55308d", "#2a6099", "#158466", "#00a933", "#81d41a",
    "#ffff00", "#ffbf00", "#ff8000", "#ff4000",
  ].iter().map(|s| LinSrgb::<u8>::from_str(*s).unwrap().into_format()).collect();

  Gradient::new(ryb_colors)
}

pub fn wheel_colors(divisions: usize) -> Vec<LinSrgb> {
  wheel_gradient().take(divisions).collect()
}

pub fn color_hex(col: &LinSrgb) -> String {
  let col: LinSrgb<u8> = col.into_format();
  format!("#{col:x}")
}

/// Returns a legible text color for the given background color.
/// 
/// Returns white for "dark" colors (luminance < 0.5) and black for "bright" colors.
pub fn text_color_for_bgcolor(bg: &LinSrgb) -> LinSrgb {
  let xyz: Xyz = Srgb::from_linear(*bg).into_color();
  let luminance = xyz.y;
  if luminance < 0.5 {
    LinSrgb::new(1.0, 1.0, 1.0)
  } else {
    LinSrgb::new(0.0, 0.0, 0.0)
  }
}