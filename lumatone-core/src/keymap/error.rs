use ini;

#[derive(Debug)]
pub enum LumatoneKeymapError {
  InvalidTableDefinition(String),

  ValueParseError,

  ParseError(ini::ParseError),
  IoError(std::io::Error),
  EncodingError(std::str::Utf8Error),
}

impl From<ini::ParseError> for LumatoneKeymapError {
  fn from(err: ini::ParseError) -> Self {
    LumatoneKeymapError::ParseError(err)
  }
}

impl From<std::io::Error> for LumatoneKeymapError {
  fn from(value: std::io::Error) -> Self {
    LumatoneKeymapError::IoError(value)
  }
}

impl From<std::str::Utf8Error> for LumatoneKeymapError {
  fn from(value: std::str::Utf8Error) -> Self {
    LumatoneKeymapError::EncodingError(value)
  }
}