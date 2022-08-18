#![allow(dead_code)]

use std::fmt::Debug;

use crate::midi::sysex::message_command_id;

use super::{
  constants::{
    BoardIndex, CommandId, LumatoneKeyFunction, LumatoneKeyLocation, RGBColor, TEST_ECHO,
  },
  error::LumatoneMidiError,
  sysex::{
    create_extended_key_color_sysex, create_sysex, is_lumatone_message, message_payload,
    EncodedSysex,
  },
};

#[derive(Debug)]
pub enum Command {
  Ping {
    value: u32,
  },
  SetKeyFunction {
    location: LumatoneKeyLocation,
    function: LumatoneKeyFunction,
  },
  SetKeyColor {
    location: LumatoneKeyLocation,
    color: RGBColor,
  },
}

impl Command {
  pub fn command_id(&self) -> CommandId {
    use Command::*;
    match *self {
      Ping { .. } => CommandId::LumaPing,
      SetKeyFunction { .. } => CommandId::ChangeKeyNote,
      SetKeyColor { .. } => CommandId::SetKeyColour,
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

// region: Command factory fns

pub fn ping(value: u32) -> Command {
  Command::Ping { value }
}

pub fn set_key_color(location: LumatoneKeyLocation, color: RGBColor) -> Command {
  Command::SetKeyColor { location, color }
}

pub fn set_key_function(location: LumatoneKeyLocation, function: LumatoneKeyFunction) -> Command {
  Command::SetKeyFunction { location, function }
}

// endregion

// region: Sysex Encoders

fn encode_ping(value: u32) -> EncodedSysex {
  let val = value & 0xfffffff; // limit to 28 bits
  create_sysex(
    BoardIndex::Server,
    CommandId::LumaPing,
    vec![
      TEST_ECHO,
      ((val >> 14) & 0x7f) as u8,
      ((val >> 7) & 0x7f) as u8,
      (val & 0x7f) as u8,
    ],
  )
}

fn encode_set_key_function(
  location: &LumatoneKeyLocation,
  function: &LumatoneKeyFunction,
) -> EncodedSysex {
  create_sysex(
    location.board_index(),
    CommandId::ChangeKeyNote,
    vec![
      location.key_index().into(),
      function.note_or_cc_num(),
      function.midi_channel_byte(),
      function.type_code(),
    ],
  )
}

fn encode_set_key_color(location: &LumatoneKeyLocation, color: &RGBColor) -> EncodedSysex {
  create_extended_key_color_sysex(
    location.board_index(),
    CommandId::SetKeyColour,
    location.key_index().into(),
    color,
  )
}

// endregion

// region: Sysex Decoders

/// Attempts to decode a sysex message as a "ping" response,
/// returning the encoded payload value on success.
pub fn decode_ping(msg: &[u8]) -> Result<u32, LumatoneMidiError> {
  if !is_lumatone_message(msg) {
    return Err(LumatoneMidiError::NotLumatoneMessage(msg.to_vec()));
  }

  let cmd_id = message_command_id(msg)?;
  if cmd_id != CommandId::LumaPing {
    return Err(LumatoneMidiError::UnexpectedCommandId {
      expected: CommandId::LumaPing,
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

// endregion
