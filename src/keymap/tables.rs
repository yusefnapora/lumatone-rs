use super::error::LumatoneKeymapError;
use crate::midi::sysex::{SysexTable, VelocityIntervalTable};


pub enum EditingStrategy {
  FreeDrawing,
  LinearSegments,
  QuadraticCurves,
}

pub struct ConfigTableDefinition {
  pub table: SysexTable,
  pub edit_strategy: EditingStrategy,
}

impl ConfigTableDefinition {
  pub fn to_string(&self) -> String {
    let table_str = self.table.iter()
      .map(u8::to_string)
      .collect::<Vec<String>>()
      .join(" ");

    let prefix = match self.edit_strategy {
      EditingStrategy::LinearSegments => "LINEAR",
      EditingStrategy::QuadraticCurves => "Quadratic",
      _ => "" 
    }.to_string();

    String::from(prefix + table_str.as_str())
  }

  pub fn from_str(s: &str) -> Result<Self, LumatoneKeymapError> {
    use EditingStrategy::*;
    use LumatoneKeymapError::InvalidTableDefinition;

    let (edit_strategy, start_index) = 
      if s.starts_with("LINEAR") {
        (LinearSegments, "LINEAR".len())
      } else if s.starts_with("Quadratic") {
        (QuadraticCurves, "Quadratic".len())
      } else {
        (FreeDrawing, 0)
      };

    let tokens: Vec<&str> = s.split(char::is_whitespace).collect();
    if tokens.len() == 0 {
      return Err(InvalidTableDefinition("table definition is empty, cannot parse".to_string()));
    }

    let value_strings = &tokens[start_index..];
    if value_strings.len() < 128 {
      return Err(InvalidTableDefinition(format!("table requires 128 values, but definition contains {}", value_strings.len())));
    }

    let mut table: SysexTable = [0; 128];
    for (i, s) in value_strings.iter().enumerate() {
      table[i] = u8::from_str_radix(*s, 10)
        .map_err(|e| InvalidTableDefinition(format!("unable to parse int in table definition: {e}")))?;
    }
    
    Ok(ConfigTableDefinition { table, edit_strategy })
  }
}

