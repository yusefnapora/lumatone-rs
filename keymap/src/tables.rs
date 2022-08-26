#![allow(unused)]
use super::{error::LumatoneKeymapError, table_defaults::*};
use lumatone_midi::sysex::{SysexTable, VelocityIntervalTable};

use ini::Ini;

pub enum EditingStrategy {
  FreeDrawing,
  LinearSegments,
  QuadraticCurves,
}

pub struct ConfigurationTables {
  pub on_off_velocity: Option<ConfigTableDefinition>,
  pub fader_velocity: Option<ConfigTableDefinition>,
  pub aftertouch_velocity: Option<ConfigTableDefinition>,
  pub lumatouch_velocity: Option<ConfigTableDefinition>,
  pub velocity_intervals: Option<VelocityIntervalTable>,
}

impl Default for ConfigurationTables {
  fn default() -> Self {
    ConfigurationTables {
      on_off_velocity: None,
      fader_velocity: None,
      aftertouch_velocity: None,
      lumatouch_velocity: None,
      velocity_intervals: None,
    }
  }
}

pub struct ConfigTableDefinition {
  pub table: SysexTable,
  pub edit_strategy: EditingStrategy,
}

impl ConfigTableDefinition {
  pub fn new(table: SysexTable) -> Self {
    ConfigTableDefinition {
      table,
      edit_strategy: EditingStrategy::FreeDrawing,
    }
  }

  pub fn new_with_edit_strategy(table: SysexTable, edit_strategy: EditingStrategy) -> Self {
    ConfigTableDefinition {
      table: table,
      edit_strategy: edit_strategy,
    }
  }

  pub fn to_string(&self) -> String {
    let table_str = self
      .table
      .iter()
      .map(u8::to_string)
      .collect::<Vec<String>>()
      .join(" ");

    let prefix = match self.edit_strategy {
      EditingStrategy::LinearSegments => "LINEAR",
      EditingStrategy::QuadraticCurves => "Quadratic",
      _ => "",
    }
    .to_string();

    String::from(prefix + table_str.as_str())
  }

  pub fn from_str(s: &str) -> Result<Self, LumatoneKeymapError> {
    use EditingStrategy::*;
    use LumatoneKeymapError::InvalidTableDefinition;

    let (edit_strategy, start_index) = if s.starts_with("LINEAR") {
      (LinearSegments, "LINEAR".len())
    } else if s.starts_with("Quadratic") {
      (QuadraticCurves, "Quadratic".len())
    } else {
      (FreeDrawing, 0)
    };

    let s = &s[start_index..];

    let tokens: Vec<&str> = s.split(char::is_whitespace).collect();
    if tokens.len() < 128 {
      return Err(InvalidTableDefinition(format!(
        "table requires 128 values, but definition contains {}",
        tokens.len()
      )));
    }

    let mut table: SysexTable = [0; 128];
    for (i, s) in tokens.iter().enumerate() {
      table[i] = u8::from_str_radix(*s, 10).map_err(|e| {
        InvalidTableDefinition(format!("unable to parse int in table definition: {e}"))
      })?;
    }

    Ok(ConfigTableDefinition {
      table,
      edit_strategy,
    })
  }
}

pub fn parse_velocity_intervals(s: &str) -> Result<VelocityIntervalTable, LumatoneKeymapError> {
  use LumatoneKeymapError::InvalidTableDefinition;
  let tokens: Vec<&str> = s.split(char::is_whitespace).collect();

  if tokens.len() < 127 {
    return Err(InvalidTableDefinition(format!(
      "velocity interval table is 127 elements long, but string has {}",
      tokens.len()
    )));
  }

  let mut table: VelocityIntervalTable = [0; 127];
  for (i, s) in tokens.iter().enumerate() {
    let val = u16::from_str_radix(s, 10).map_err(|e| {
      InvalidTableDefinition(format!("unable to parse in in table definition: {e}"))
    })?;
    table[i] = val;
  }
  Ok(table)
}

pub fn velocity_intervals_to_string(table: &VelocityIntervalTable) -> String {
  table
    .iter()
    .map(u16::to_string)
    .collect::<Vec<String>>()
    .join(" ")
}
