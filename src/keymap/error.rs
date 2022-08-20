use ini;

pub enum LumatoneKeymapError {
  InvalidTableDefinition(String),

  ParseError(ini::ParseError),
}

impl From<ini::ParseError> for LumatoneKeymapError {
  fn from(err: ini::ParseError) -> Self {
    LumatoneKeymapError::ParseError(err)
  }
}
