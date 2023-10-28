use palette::{Gradient, LinSrgb};
use std::str::FromStr;
use super::utils::text_color_for_bgcolor;

#[derive(PartialEq)]
pub struct ColorPalette {
  // gradient: Gradient<LinSrgb>,
  divisions: usize,
  colors: Vec<LinSrgb>,
}

impl ColorPalette {
  pub fn new(gradient: Gradient<LinSrgb>, divisions: usize) -> Self {
    let colors = gradient.take(divisions).collect();
    ColorPalette { divisions, colors }
  }

  pub fn default_gradient(divisions: usize) -> Self {
    Self::new(wheel_gradient(), divisions)
  }

  pub fn get(&self, index: usize) -> LinSrgb {
    let index = index % self.divisions;
    self.colors[index]
  }

  pub fn get_text_color(&self, index: usize) -> LinSrgb {
    let c = self.get(index);
    text_color_for_bgcolor(c)
  }
}

fn wheel_gradient() -> Gradient<LinSrgb> {
  // hard-code control points along an "RYB" color wheel
  // TODO: lerp over one of the Lab / Lch color spaces?
  let ryb_colors: Vec<LinSrgb<f32>> = vec![
    "#ff0000", "#bf0041", "#800080", "#55308d", "#2a6099", "#158466", "#00a933", "#81d41a",
    "#ffff00", "#ffbf00", "#ff8000", "#ff4000",
  ]
    .iter()
    .map(|s| LinSrgb::<u8>::from_str(*s).unwrap().into_format())
    .collect();

  Gradient::new(ryb_colors)
}

pub fn wheel_colors(divisions: usize) -> Vec<LinSrgb> {
  wheel_gradient().take(divisions).collect()
}


