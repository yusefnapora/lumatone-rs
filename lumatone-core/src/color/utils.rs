use palette::{LinSrgb, Srgb, Xyz, IntoColor};

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

pub trait ToHexColorStr {
  fn to_hex_color(&self) -> String;
}

impl ToHexColorStr for LinSrgb {
  fn to_hex_color(&self) -> String {
    let c: LinSrgb<u8> = self.into_format();
    format!("#{c:x}")
  }
}