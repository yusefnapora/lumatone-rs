#![allow(dead_code)]

use std::fmt::Debug;

use crate::midi::sysex::{message_command_id, create_single_arg_server_sysex, create_table_sysex, reverse_table, create_zero_arg_server_sysex, create_zero_arg_sysex};

use super::{
  constants::{
    BoardIndex, CommandId, LumatoneKeyFunction, LumatoneKeyLocation, PresetNumber, RGBColor, TEST_ECHO,
  },
  error::LumatoneMidiError,
  sysex::{
    create_extended_key_color_sysex, create_sysex, is_lumatone_message, message_payload,
    EncodedSysex, create_sysex_toggle, create_extended_macro_color_sysex, SysexTable, VelocityIntervalTable,
  },
};

#[derive(Debug)]
pub enum Command {
  /// Echo the payload, 0x00-0x7f, for use in connection monitoring
  Ping(u32),
  /// Send a single key's functionctional configuration
  SetKeyFunction {
    location: LumatoneKeyLocation,
    function: LumatoneKeyFunction,
  },
  /// Send a single key's LED channel intensities
  SetKeyColor {
    location: LumatoneKeyLocation,
    color: RGBColor,
  },
  /// Save current configuration to specified preset index
  SaveProgram(PresetNumber),
  /// Send expression pedal sensitivity
  SetExpressionPedalSensitivity(u8),
  /// Set mod wheel sensitivity
  SetModWheelSensitivity(u8),
  /// Set pitch wheel sensitivity
  SetPitchWheelSensitivity(u16),
  /// Set the foot controller direction to inverted (`true`), or normal (`false`)
  InvertFootController(bool),
  /// Set whether to light up keys on press
  SetLightOnKeystrokes(bool),
  /// Enable or disable aftertouch functionality
  SetAftertouchEnabled(bool),
  /// Enable demo mode with `true`, or exit by sending `false`
  EnableDemoMode(bool),
  /// Initiate the pitch and mod wheel calibration routine, pass in false to stop
  EnablePitchModWheelCalibrationMode(bool),
  /// Set color for macro button in active state
  SetMacroButtonActiveColor(RGBColor),
  /// Set color for macro button in inactive state
  SetMacroButtonInactiveColor(RGBColor),

  /// Set the velocity lookup table
  SetVelocityConfig(SysexTable),
  /// Adjust the internal fader lookup table
  SetFaderConfig(SysexTable),
  /// Adjust the internal aftertouch lookup table
  SetAftertouchConfig(SysexTable),
  /// Adjust the Lumatouch table, a 128 byte array with value of 127 being a key fully pressed
  SetLumatouchConfig(SysexTable),
  /// Set the velocity interval table, 127 12-bit values
  SetVelocityIntervals(VelocityIntervalTable),

  /// Set abs. distance from max value to trigger CA-004 submodule key events, ranging from 0x00 to 0xFE
  SetKeyMaximumThreshold { board_index: BoardIndex, max_threshold: u8, aftertouch_max: u8 },

  /// Set abs. distance from min value to trigger CA-004 submodule key events, ranging from 0x00 to 0xFE
  SetKeyMinimumThreshold { board_index: BoardIndex, threshold_high: u8, threshold_low: u8 },

  /// Set the sensitivity for CC events, ranging from 0x00 to 0xFE
  SetKeyFaderSensitivity(BoardIndex, u8),
  /// Set the target board sensitivity for aftertouch events, ranging from 0x00 to 0xFE
  SetKeyAftertouchSensitivity(BoardIndex, u8),
  /// Set the thresold from key’s min value to trigger CA - 004 submodule CC events, ranging from 0x00 to 0xFE
  SetCCActiveThreshold(BoardIndex, u8),
  /// Reset the thresholds for events and sensitivity for CC & aftertouch on the target board
  ResetBoardThresholds(BoardIndex),

  /// Read back the current red intensity of all the keys of the target board.
  RequestRedLEDConfig(BoardIndex),
  /// Read back the current green intensity of all the keys of the target board.
  RequestGreenLEDConfig(BoardIndex),
  /// Read back the current blue intensity of all the keys of the target board.
  RequestBlueLEDConfig(BoardIndex),
  /// Read back the current channel configuration of all the keys of the target board.
  RequestMidiChannelConfig(BoardIndex),
  /// Read back the current note configuration of all the keys of the target board.
  RequestNoteConfig(BoardIndex),
  /// Read back the current key type configuration of all the keys of the target board.
  RequestKeyTypeConfig(BoardIndex),
  /// Read back the maximum fader threshold of all the keys of the target board.
  RequestMaxFaderThreshold(BoardIndex),
  /// Read back the minimum fader threshold of all the keys of the target board.
  RequestMinFaderThreshold(BoardIndex),
  /// Read back the aftertouch maximum threshold of all the keys of the target board
  RequestMaxAftertouchThreshold(BoardIndex),
  /// Get back flag whether or not each key of target board meets minimum threshold
  RequestKeyValidity(BoardIndex),
  /// Read back the fader type of all keys on the targeted board.
  RequestFaderTypeConfig(BoardIndex),
  
  /// Read back the current velocity look up table of the keyboard.
  RequestVelocityConfig,
  /// Read back the velocity interval table
  RequestVelocityIntervalConfig,
  /// Read back the current fader look up table of the keyboard.
  RequestFaderConfig,
  /// Read back the current aftertouch look up table of the keyboard.
  RequestAftertouchConfig,
  /// Read back the Lumatouch table
  RequestLumatouchConfig,

  /// Read back the serial identification number of the keyboard.
  RequestSerialId,
  /// Read back the current Lumatone firmware revision.
  RequestFirmwareRevision,

  /// Initiate aftertouch calibration routine
  StartAftertouchCalibration,
  /// Initiate the key calibration routine. Each pair of macro buttons
  /// on each octave must be pressed to return to normal state
  StartKeyCalibration,

  /// Save velocity config to EEPROM
  SaveVelocityConfig,
  /// Reset the velocity config to value from EEPROM
  ResetVelocityConfig,
  /// Save the changes made to the fader look-up table
  SaveFaderConfig,
  /// Reset the fader lookup table back to its factory fader settings.
  ResetFaderConfig,
  /// Save the changes made to the aftertouch look-up table
  SaveAftertouchConfig,
  /// Reset the aftertouch lookup table back to its factory aftertouch settings.
  ResetAftertouchConfig,
  /// Save Lumatouch table changes
  SaveLumatouchConfig,
  /// Reset the Lumatouch table back to factory settings
  ResetLumatouchConfig,
  /// Set thresholds for the pitch and modulation wheel to factory settings
  ResetWheelThresholds,

  /// Enable/disable key sampling over SSH for the target key and board
  EnableKeySampling(BoardIndex, bool),
}

impl Command {
  pub fn command_id(&self) -> CommandId {
    use Command::*;
    match *self {
      Ping(_) => CommandId::LumaPing,
      SetKeyFunction { .. } => CommandId::ChangeKeyNote,
      SetKeyColor { .. } => CommandId::SetKeyColour,
      SaveProgram(_) => CommandId::SaveProgram,
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
      SetLumatouchConfig(_) => CommandId::SetLumatouchConfig,
      SetVelocityIntervals(_) => CommandId::SetVelocityIntervals,

      SetKeyMaximumThreshold { .. } => CommandId::SetKeyMaxThreshold,
      SetKeyMinimumThreshold { .. } => CommandId::SetKeyMinThreshold,
      SetKeyFaderSensitivity(..) => CommandId::SetKeyFaderSensitivity,
      SetKeyAftertouchSensitivity(..) => CommandId::SetKeyAftertouchSensitivity,
      SetCCActiveThreshold(..) => CommandId::SetCCActiveThreshold,
      ResetBoardThresholds(_) => CommandId::ResetBoardThresholds,

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
      RequestLumatouchConfig => CommandId::GetLumatouchConfig,

      RequestSerialId => CommandId::GetSerialIdentity,
      RequestFirmwareRevision => CommandId::GetFirmwareRevision,

      StartAftertouchCalibration => CommandId::CalibrateAftertouch,
      StartKeyCalibration => CommandId::CalibrateKeys,

      SaveVelocityConfig => CommandId::SaveVelocityConfig,
      ResetVelocityConfig => CommandId::ResetVelocityConfig,
      SaveFaderConfig => CommandId::SaveFaderConfig,
      ResetFaderConfig => CommandId::ResetFaderConfig,
      SaveAftertouchConfig => CommandId::SaveAftertouchConfig,
      ResetAftertouchConfig => CommandId::ResetAftertouchConfig,
      SaveLumatouchConfig => CommandId::SaveLumatouchConfig,
      ResetLumatouchConfig => CommandId::ResetLumatouchConfig,
      ResetWheelThresholds => CommandId::ResetWheelsThreshold,

      EnableKeySampling(..) => CommandId::SetKeySampling,
    }
  }

  pub fn to_sysex_message(&self) -> EncodedSysex {
    use Command::*;
    match self {
      Ping(value) => encode_ping(*value),
      
      SetKeyFunction { location, function } => 
        encode_set_key_function(location, function),

      SetKeyColor { location, color } => 
        encode_set_key_color(location, color),

      SaveProgram(preset_number) =>
        create_single_arg_server_sysex(self.command_id(), (*preset_number).into()),

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
      
      SetLumatouchConfig(table) =>
        create_table_sysex(self.command_id(), table),
      
      SetVelocityIntervals(table) =>
        encode_set_velocity_interval_table(table),

      SetKeyMaximumThreshold { board_index, max_threshold, aftertouch_max } =>
        encode_set_key_thresholds(*board_index, self.command_id(), *max_threshold, *aftertouch_max),
      
      SetKeyMinimumThreshold { board_index, threshold_high, threshold_low } =>
        encode_set_key_thresholds(*board_index, self.command_id(), *threshold_high, *threshold_low),
      
      SetKeyFaderSensitivity(board_index, value) =>
        encode_set_key_sensitivity(*board_index, self.command_id(), *value),
      
      SetKeyAftertouchSensitivity(board_index, value) =>
        encode_set_key_sensitivity(*board_index, self.command_id(), *value),

      SetCCActiveThreshold(board_index, value) =>
        encode_set_key_sensitivity(*board_index, self.command_id(), *value),

      ResetBoardThresholds(board_index) =>
        create_zero_arg_sysex(*board_index, self.command_id()),

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
      RequestLumatouchConfig => create_zero_arg_server_sysex(self.command_id()),
      RequestSerialId => create_zero_arg_server_sysex(self.command_id()),
      RequestFirmwareRevision => create_zero_arg_server_sysex(self.command_id()),
      StartAftertouchCalibration => create_zero_arg_server_sysex(self.command_id()),
      StartKeyCalibration => create_zero_arg_server_sysex(self.command_id()),
      
      SaveVelocityConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetVelocityConfig => create_zero_arg_server_sysex(self.command_id()),      
      SaveFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      SaveAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),      
      SaveLumatouchConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetLumatouchConfig => create_zero_arg_server_sysex(self.command_id()),
      ResetWheelThresholds => create_zero_arg_server_sysex(self.command_id()),

      EnableKeySampling(board_index, enable) =>
        create_sysex_toggle(*board_index, self.command_id(), *enable)
    }
  }
}

// region: Command factory fns

pub fn ping(value: u32) -> Command {
  Command::Ping(value)
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

fn encode_set_key_thresholds(board_index: BoardIndex, cmd: CommandId, t1: u8, t2: u8) -> EncodedSysex {
  let t1 = t1 & 0xfe;
  let t2 = t2 & 0xfe;
  let data = vec![
    t1 >> 4,
    t1 & 0xf,
    t2 >> 4,
    t2 & 0xf,
  ];
  create_sysex(board_index, cmd, data)
}

fn encode_set_key_sensitivity(board_index: BoardIndex, cmd: CommandId, value: u8) -> EncodedSysex {
  let value = value & 0xfe;
  let data = vec![
    value >> 4,
    value & 0xf,
  ];
  create_sysex(board_index, cmd, data)
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