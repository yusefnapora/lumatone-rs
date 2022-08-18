#![allow(dead_code)]

use std::fmt::Debug;

use crate::midi::sysex::message_command_id;

use super::{
  constants::{BoardIndex, CommandId as CMD, TEST_ECHO, LumatoneKeyIndex, LumatoneKeyFunction, MidiChannel },
  error::LumatoneMidiError,
  sysex::{
    create_extended_key_color_sysex, create_sysex, is_lumatone_message, message_payload,
    EncodedSysex,
  },
};

pub trait LumatoneCommand {
  fn command_id(&self) -> CMD;

  fn to_sysex_message(&self) -> EncodedSysex;
}

impl Debug for (dyn LumatoneCommand + 'static) {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "LumatoneCommand: {:?}", self.command_id())
  }
}

impl Debug for (dyn LumatoneCommand + Send) {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "LumatoneCommand: {:?}", self.command_id())
  } 
}

pub struct SetKeyFunction {
  pub board_index: BoardIndex,
  pub key_index: LumatoneKeyIndex,
  pub midi_channel: MidiChannel,
  pub function: LumatoneKeyFunction,
}

impl LumatoneCommand for SetKeyFunction {
  fn command_id(&self) -> CMD {
    CMD::ChangeKeyNote
  }

  fn to_sysex_message(&self) -> EncodedSysex {
    use LumatoneKeyFunction::*;
    let note_or_cc_num = match self.function {
      NoteOnOff { note_num } => note_num,
      ContinuousController { cc_num, fader_up_is_null: _ } => cc_num,
      _ => 0
    };

    create_sysex(self.board_index, self.command_id(), vec![
      self.key_index.into(),
      note_or_cc_num,
      self.midi_channel.into(),
      self.function.type_code(),
    ])
  }
}

pub struct SetKeyColor {
  board_index: BoardIndex,
  key_index: LumatoneKeyIndex,
  red: u8,
  green: u8,
  blue: u8,
}

impl SetKeyColor {
  pub fn new(
    board_index: BoardIndex,
    key_index: LumatoneKeyIndex,
    red: u8,
    green: u8,
    blue: u8
  ) -> Self {
    Self { board_index, key_index, red, green, blue }
  }
} 

impl LumatoneCommand for SetKeyColor {
  fn command_id(&self) -> CMD {
    CMD::SetKeyColour
  }

  fn to_sysex_message(&self) -> EncodedSysex {
    create_extended_key_color_sysex(self.board_index, self.command_id(), self.key_index.into(), self.red, self.green, self.blue)
  }
}

pub struct Ping {
  pub value: u32
}

impl Ping {
  pub fn new(value: u32) -> Ping {
    Ping { value }
  }

  /// Attempts to decode a sysex message as a "ping" response,
  /// returning the encoded payload value on success.
  pub fn decode(msg: &[u8]) -> Result<u32, LumatoneMidiError> {
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
}

impl LumatoneCommand for Ping {
  fn command_id(&self) -> CMD {
    CMD::LumaPing
  }

  fn to_sysex_message(&self) -> EncodedSysex {
    let val = self.value & 0xfffffff; // limit to 28 bits
    create_sysex(
      BoardIndex::Server,
      self.command_id(),
      vec![
        TEST_ECHO,
        ((val >> 14) & 0x7f) as u8,
        ((val >> 7) & 0x7f) as u8,
        (val & 0x7f) as u8,
      ],
    )
  }
}



// TODO: add remaining commands
