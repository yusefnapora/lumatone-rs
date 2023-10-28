use palette::LinSrgb;

use crate::drawing::color::wheel_colors;

use lumatone_core::geometry::coordinates::{lumatone_location_for_hex, Hex};

pub struct KeyDefinition {
  pub color: LinSrgb,
  pub label: String,
  // TODO: everything else...
}

pub trait KeyMapper {
  fn key_definition_for_coordinate(&self, coord: &Hex) -> Option<KeyDefinition>;
}

pub struct DebugMapper {
  pub color: LinSrgb,
}

impl KeyMapper for DebugMapper {
  fn key_definition_for_coordinate(&self, coord: &Hex) -> Option<KeyDefinition> {
    let label = format!("{},{}", coord.q(), coord.r());
    Some(KeyDefinition {
      color: self.color.clone(),
      label,
    })
  }
}

pub struct LumatoneLocationDebugMapper {}

impl KeyMapper for LumatoneLocationDebugMapper {
  fn key_definition_for_coordinate(&self, coord: &Hex) -> Option<KeyDefinition> {
    let colors = wheel_colors(5);
    lumatone_location_for_hex(coord).map(|loc| {
      let board_index: u8 = loc.board_index().into();
      let color = colors[(board_index as usize) - 1];
      let label = format!("{}", loc.key_index());
      KeyDefinition { color, label }
    })
  }
}
