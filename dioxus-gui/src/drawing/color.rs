use std::str::FromStr;
use palette::{LinSrgb, Gradient, Xyz, Srgb, IntoColor};

pub struct ColorPalette {
  gradient: Gradient<LinSrgb>,
  divisions: u16,
  colors: Vec<LinSrgb>
}

impl ColorPalette {
  pub fn new(gradient: Gradient<LinSrgb>, divisions: u16) -> Self {
    let colors = gradient.take(divisions as usize).collect();
    ColorPalette { gradient, divisions, colors }
  }

  pub fn default_gradient(divisions: u16) -> Self {
    Self::new(wheel_gradient(), divisions)
  }

  pub fn get(&self, index: usize) -> LinSrgb {
    let index = index % (self.divisions as usize);
    self.colors[index]
  }


  pub fn get_text_color(&self, index: usize) -> LinSrgb {
    let c = self.get(index);
    text_color_for_bgcolor(c)
  }
}

pub trait ToHexColorStr {
  fn to_hex_color(&self) -> String;
}

impl ToHexColorStr for LinSrgb {
  fn to_hex_color(&self) -> String {
    let c: LinSrgb<u8> = self.into_format();
    format!("#{c:x}")
  }
}

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

/// Returns the color as a CSS-compatible hex string, with `#` prefix.
pub fn color_hex(col: LinSrgb) -> String {
  let col: LinSrgb<u8> = col.into_format();
  format!("#{col:x}")
}

/// Returns a legible text color for the given background color.
/// 
/// Returns white for "dark" colors (luminance < 0.5) and black for "bright" colors.
pub fn text_color_for_bgcolor(bg: LinSrgb) -> LinSrgb {
  let xyz: Xyz = Srgb::from_linear(bg).into_color();
  let luminance = xyz.y;
  if luminance < 0.5 {
    LinSrgb::new(1.0, 1.0, 1.0)
  } else {
    LinSrgb::new(0.0, 0.0, 0.0)
  }
}