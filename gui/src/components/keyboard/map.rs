use super::coords::Hex;


pub struct KeyDefinition {
  pub color: String, // TODO: use proper color type
  pub label: String,
  // TODO: everything else...
}

pub trait KeyMapper {
  fn key_definition_for_coordinate(&self, coord: &Hex) -> Option<KeyDefinition>;
}

pub struct DebugMapper {
  pub color: String,
}

impl KeyMapper for DebugMapper {
    fn key_definition_for_coordinate(&self, coord: &Hex) -> Option<KeyDefinition> {
        let label = format!("{},{}", coord.q(), coord.r());
        Some(KeyDefinition { color: self.color.clone(), label })
    }
}