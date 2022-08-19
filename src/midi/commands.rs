#![allow(dead_code)]

use std::fmt::Debug;

use crate::midi::sysex::{message_command_id, create_single_arg_server_sysex, create_table_sysex, reverse_table, create_zero_arg_server_sysex, create_zero_arg_sysex};

use super::{
  constants::{
    BoardIndex, CommandId, LumatoneKeyFunction, LumatoneKeyLocation, RGBColor, TEST_ECHO,
  },
  error::LumatoneMidiError,
  sysex::{
    create_extended_key_color_sysex, create_sysex, is_lumatone_message, message_payload,
    EncodedSysex, create_sysex_toggle, create_extended_macro_color_sysex, SysexTable, VelocityIntervalTable,
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
  SetExpressionPedalSensitivity(u8),
  SetModWheelSensitivity(u8),
  SetPitchWheelSensitivity(u16),
  InvertFootController(bool),
  SetLightOnKeystrokes(bool),
  SetAftertouchEnabled(bool),
  EnableDemoMode(bool),
  EnablePitchModWheelCalibrationMode(bool),
  SetMacroButtonActiveColor(RGBColor),
  SetMacroButtonInactiveColor(RGBColor),

  SetVelocityConfig(SysexTable),
  SetFaderConfig(SysexTable),
  SetAftertouchConfig(SysexTable),
  SetVelocityIntervals(VelocityIntervalTable),

  // The RequestThing commands ask the target board to read back the values for all keys on that board.
  RequestRedLEDConfig(BoardIndex),
  RequestGreenLEDConfig(BoardIndex),
  RequestBlueLEDConfig(BoardIndex),
  RequestMidiChannelConfig(BoardIndex),
  RequestNoteConfig(BoardIndex),
  RequestKeyTypeConfig(BoardIndex),
  RequestMaxFaderThreshold(BoardIndex),
  RequestMinFaderThreshold(BoardIndex),
  RequestMaxAftertouchThreshold(BoardIndex),
  RequestKeyValidity(BoardIndex),
  RequestFaderTypeConfig(BoardIndex),
  

  RequestVelocityConfig,
  RequestVelocityIntervalConfig,
  RequestFaderConfig,
  RequestAftertouchConfig,
  RequestSerialId,

  StartAftertouchCalibration,
  StartKeyCalibration,

  // the SaveThingConfig commands save the current config to eeprom.
  // the ResetThingConfig commands restore to factory default

  SaveVelocityConfig,
  ResetVelocityConfig,
  SaveFaderConfig,
  ResetFaderConfig,
  SaveAftertouchConfig,
  ResetAftertouchConfig,
}

impl Command {
  pub fn command_id(&self) -> CommandId {
    use Command::*;
    match *self {
      Ping { .. } => CommandId::LumaPing,
      SetKeyFunction { .. } => CommandId::ChangeKeyNote,
      SetKeyColor { .. } => CommandId::SetKeyColour,
      SetExpressionPedalSensitivity(_) => CommandId::SetFootControllerSensitivity,
      SetModWheelSensitivity(_) => CommandId::SetModWheelSensitivity,
      SetPitchWheelSensitivity(_) => CommandId::SetPitchWheelSensitivity,

      InvertFootController(_) => CommandId::InvertFootController,
      SetMacroButtonActiveColor(_) => CommandId::MacrobuttonColourOn,
      SetMacroButtonInactiveColor(_) => CommandId::MacrobuttonColourOff,
      SetLightOnKeystrokes(_) => CommandId::SetLightOnKeystrokes,
      SetAftertouchEnabled(_) => CommandId::SetAftertouchFlag,

      EnableDemoMode(_) => CommandId::DemoMode,
      EnablePitchModWheelCalibrationMode(_) => CommandId::CalibratePitchModWheel,

      SetVelocityConfig(_) => CommandId::SetVelocityConfig,
      SetFaderConfig(_) => CommandId::SetFaderConfig,
      SetAftertouchConfig(_) => CommandId::SetAftertouchConfig,
      SetVelocityIntervals(_) => CommandId::SetVelocityIntervals,

      RequestRedLEDConfig(_) => CommandId::GetRedLedConfig,
      RequestGreenLEDConfig(_) => CommandId::GetGreenLedConfig,
      RequestBlueLEDConfig(_) => CommandId::GetBlueLedConfig,
      RequestMidiChannelConfig(_) => CommandId::GetChannelConfig,
      RequestNoteConfig(_) => CommandId::GetNoteConfig,
      RequestKeyTypeConfig(_) => CommandId::GetKeytypeConfig,
      RequestMaxFaderThreshold(_) => CommandId::GetMaxThreshold,
      RequestMinFaderThreshold(_) => CommandId::GetMinThreshold,
      RequestMaxAftertouchThreshold(_) => CommandId::GetAftertouchMax,
      RequestKeyValidity(_) => CommandId::GetKeyValidity,
      RequestFaderTypeConfig(_) => CommandId::GetFaderTypeConfiguration,

      RequestVelocityConfig => CommandId::GetVelocityConfig,
      RequestVelocityIntervalConfig => CommandId::GetVelocityIntervals,
      RequestFaderConfig => CommandId::GetFaderConfig,
      RequestAftertouchConfig => CommandId::GetAftertouchConfig,
      RequestSerialId => CommandId::GetSerialIdentity,

      StartAftertouchCalibration => CommandId::CalibrateAftertouch,
      StartKeyCalibration => CommandId::CalibrateKeys,

      SaveVelocityConfig => CommandId::SaveVelocityConfig,
      ResetVelocityConfig => CommandId::ResetVelocityConfig,
      SaveFaderConfig => CommandId::SaveFaderConfig,
      ResetFaderConfig => CommandId::ResetFaderConfig,
      SaveAftertouchConfig => CommandId::SaveAftertouchConfig,
      ResetAftertouchConfig => CommandId::ResetAftertouchConfig,

    }
  }

  pub fn to_sysex_message(&self) -> EncodedSysex {
    use Command::*;
    match self {
      Ping { value } => encode_ping(*value),
      
      SetKeyFunction { location, function } => 
        encode_set_key_function(location, function),

      SetKeyColor { location, color } => 
        encode_set_key_color(location, color),

      SetExpressionPedalSensitivity(value) => 
        create_single_arg_server_sysex(self.command_id(), *value),

      SetModWheelSensitivity(value) =>
        create_single_arg_server_sysex(self.command_id(), clamp_u8(*value, 1, 0x7f)),

      SetPitchWheelSensitivity(value) => {
        let val = clamp_u16(*value, 1, 0x3fff);
        let hi = (val >> 7) as u8;
        let lo = (val & 0x7f) as u8;

        create_sysex(BoardIndex::Server, self.command_id(), vec![hi, lo])
      },

      InvertFootController(invert) => 
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *invert),

      SetAftertouchEnabled(enabled) =>
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled),

      EnableDemoMode(enabled) =>
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled),

      EnablePitchModWheelCalibrationMode(enabled) =>
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled),

      SetMacroButtonActiveColor(color) => 
        create_extended_macro_color_sysex(self.command_id(), color),

      SetMacroButtonInactiveColor(color) => 
        create_extended_macro_color_sysex(self.command_id(), color),

      SetLightOnKeystrokes(active) => 
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *active),

      SetVelocityConfig(table) => 
        // the velocity config is in the reverse order (compared to how it's specified in keymap files)
        // so we reverse it before sending
        create_table_sysex(self.command_id(), &reverse_table(table)),

      SetFaderConfig(table) =>
        create_table_sysex(self.command_id(), table),

      SetAftertouchConfig(table) =>
        create_table_sysex(self.command_id(), table),

      SetVelocityIntervals(table) =>
        encode_set_velocity_interval_table(table),

      RequestRedLEDConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),

      RequestGreenLEDConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),
 
      RequestBlueLEDConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),

      RequestMidiChannelConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestNoteConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestKeyTypeConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestMaxFaderThreshold(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestMinFaderThreshold(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestMaxAftertouchThreshold(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()), 
      RequestKeyValidity(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),       
      RequestFaderTypeConfig(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),
        
      RequestVelocityConfig => create_zero_arg_server_sysex(self.command_id()),
      RequestVelocityIntervalConfig => create_zero_arg_server_sysex(self.command_id()),
      RequestFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      RequestAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),
      RequestSerialId => create_zero_arg_server_sysex(self.command_id()),
      StartAftertouchCalibration => create_zero_arg_server_sysex(self.command_id()),
      StartKeyCalibration => create_zero_arg_server_sysex(self.command_id()),
      
      SaveVelocityConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetVelocityConfig => create_zero_arg_server_sysex(self.command_id()),      
      SaveFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      SaveAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),      
      
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

fn encode_set_velocity_interval_table(table: &VelocityIntervalTable) -> EncodedSysex {
  // unpack 12-bit values from table into pairs of u8
  let split_u16 = |n: &u16| {
    let hi = ((n >> 6) & 0x3f) as u8;
    let lo = (n & 0x3f) as u8;
    vec![hi, lo]
  };
  let bytes: Vec<u8> = table.iter().flat_map(split_u16).collect();
  create_sysex(BoardIndex::Server, CommandId::SetVelocityIntervals, bytes)
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


// region: helpers

fn clamp_u8(val: u8, min: u8, max: u8) -> u8 {
  if val < min {
    min
  } else if val > max {
    max
  } else {
    val
  }
}

fn clamp_u16(val: u16, min: u16, max: u16) -> u16 {
  if val < min {
    min
  } else if val > max {
    max
  } else {
    val
  }
}

// endregion