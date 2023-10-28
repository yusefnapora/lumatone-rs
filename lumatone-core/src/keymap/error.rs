use ini;

#[derive(Debug)]
pub enum LumatoneKeymapError {
  InvalidTableDefinition(String),

  ValueParseError,

  ParseError(ini::ParseError),
}

impl From<ini::ParseError> for LumatoneKeymapError {
  fn from(err: ini::ParseError) -> Self {
    LumatoneKeymapError::ParseError(err)
  }
}
