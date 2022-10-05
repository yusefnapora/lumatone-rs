use std::str::FromStr;
use palette::{LinSrgb, Gradient};

fn wheel_gradient() -> Gradient<LinSrgb> {
  // hard-code control points along an "RYB" color wheel
  // TODO: lerp over one of the Lab / Lch color spaces?
  let ryb_colors: Vec<LinSrgb<f32>> = vec![
    "#ff0000", "#bf0041", "#800080", "#55308d", "#2a6099", "#158466", "#00a933", "#81d41a",
    "#ffff00", "#ffbf00", "#ff8000", "#ff4000",
  ].iter().map(|s| LinSrgb::<u8>::from_str(*s).unwrap().into_format()).collect();

  Gradient::new(ryb_colors)
}

pub fn wheel_colors_hex(divisions: u16) -> Vec<String> {
  let gradient = wheel_gradient();
  let cols = gradient.take(divisions as usize)
    .map(|c| {
      let c = c.into_format::<u8>();
      format!("#{c:x}")
    })
    .collect();
    cols
}
