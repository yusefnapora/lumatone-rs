#![allow(dead_code)]

use std::fmt::Debug;

use crate::midi::sysex::message_command_id;

use super::{
  constants::{BoardIndex, CommandId as CMD, TEST_ECHO, LumatoneKeyFunction, RGBColor, LumatoneKeyLocation },
  error::LumatoneMidiError,
  sysex::{
    create_extended_key_color_sysex, create_sysex, is_lumatone_message, message_payload,
    EncodedSysex,
  },
};

pub type BoxedKeyLocation = Box<dyn LumatoneKeyLocation + Send>;

impl std::fmt::Debug for BoxedKeyLocation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (board_index, key_index) = self.as_board_and_key_index();
    write!(f, "BoxedKeyLocation({:?}, {:?})", board_index, key_index)
  }
}

#[derive(Debug)]
pub enum Command {
  Ping { value: u32 },
  SetKeyFunction { location: BoxedKeyLocation, function: LumatoneKeyFunction },
  SetKeyColor { location: BoxedKeyLocation, color: RGBColor },
}

impl Command {

  pub fn command_id(&self) -> CMD {
    use Command::*;
    match *self { 
      Ping { .. } => CMD::LumaPing,
      SetKeyFunction { .. } => CMD::ChangeKeyNote,
      SetKeyColor { .. } => CMD::SetKeyColour,
    }
  }

  pub fn to_sysex_message(&self) -> EncodedSysex {
    use Command::*;
    match self {
      Ping { value } => encode_ping(*value),
      SetKeyFunction { location, function } => encode_set_key_function(location, function),
      SetKeyColor { location, color } => encode_set_key_color(location, color),
    }
  }
}


pub fn ping(value: u32) -> Command {
  Command::Ping { value }
}

pub fn set_key_color<L>(location: L, color: RGBColor) -> Command
where
  L: LumatoneKeyLocation + Send + 'static
{
  Command::SetKeyColor { location: Box::new(location), color }
}

pub fn set_key_function<L>(location: L, function: LumatoneKeyFunction) -> Command
where
  L: LumatoneKeyLocation + Send + 'static
{
  Command::SetKeyFunction { location: Box::new(location), function }
}


fn encode_ping(value: u32) -> EncodedSysex {
  let val = value & 0xfffffff; // limit to 28 bits
  create_sysex(
    BoardIndex::Server,
    CMD::LumaPing, 
    vec![
      TEST_ECHO,
      ((val >> 14) & 0x7f) as u8,
      ((val >> 7) & 0x7f) as u8,
      (val & 0x7f) as u8,
    ],
  )
}

fn encode_set_key_function(location: &BoxedKeyLocation, function: &LumatoneKeyFunction) -> EncodedSysex {
  let (board_index, key_index) = location.as_board_and_key_index();
  create_sysex(board_index, CMD::ChangeKeyNote, vec![
    key_index.into(),
    function.note_or_cc_num(),
    function.midi_channel_byte(),
    function.type_code(),
  ])
}

fn encode_set_key_color(location: &BoxedKeyLocation, color: &RGBColor) -> EncodedSysex {
  let (board_index, key_index) = location.as_board_and_key_index();
  create_extended_key_color_sysex(board_index, CMD::SetKeyColour, key_index.into(), color)
}

/// Attempts to decode a sysex message as a "ping" response,
/// returning the encoded payload value on success.
pub fn decode_ping(msg: &[u8]) -> Result<u32, LumatoneMidiError> {
  if !is_lumatone_message(msg) {
    return Err(LumatoneMidiError::NotLumatoneMessage(msg.to_vec()));
  }

  let cmd_id = message_command_id(msg)?;
  if cmd_id != CMD::LumaPing {
    return Err(LumatoneMidiError::UnexpectedCommandId {
      expected: CMD::LumaPing,
      actual: cmd_id,
    });
  }

  let payload = message_payload(msg)?;
  if payload.len() < 4 {
    return Err(LumatoneMidiError::MessagePayloadTooShort {
      expected: 4,
      actual: payload.len(),
    });
  }

  if payload[0] != TEST_ECHO {
    return Err(LumatoneMidiError::InvalidResponseMessage(
      "ping response has invalid echo flag value".to_string(),
    ));
  }

  let value: u32 = ((payload[1] as u32) << 14) | ((payload[2] as u32) << 7) | (payload[3] as u32);
  Ok(value)
}


// TODO: add remaining commands
