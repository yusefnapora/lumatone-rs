#![allow(dead_code)]

use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use super::{
  constants::{
    BoardIndex, CommandId, LumatoneKeyFunction, LumatoneKeyLocation, MidiChannel, PresetNumber,
    RGBColor, TEST_ECHO,
  },
  sysex::{
    create_extended_key_color_sysex, create_extended_macro_color_sysex,
    create_single_arg_server_sysex, create_sysex, create_sysex_toggle, create_table_sysex,
    create_zero_arg_server_sysex, create_zero_arg_sysex, reverse_table, EncodedSysex, SysexTable,
    VelocityIntervalTable,
  },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
  /// Echo the payload, 0x00-0x7f, for use in connection monitoring
  Ping(u32),
  /// Send a single key's functional configuration
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
  /// Sets whether to invert the sustain pedal
  InvertSustainPedal(bool),
  /// Set whether to light up keys on press
  SetLightOnKeystrokes(bool),
  /// Enable or disable aftertouch functionality
  SetAftertouchEnabled(bool),
  /// Enable demo mode with `true`, or exit by sending `false`
  EnableDemoMode(bool),
  /// Initiate the pitch and mod wheel calibration routine, pass in false to stop
  EnablePitchModWheelCalibrationMode(bool),
  /// Pass in true to initiate the expression pedal calibration routine, or false to stop
  EnableExpressionPedalCalibrationMode(bool),
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
  SetKeyMaximumThreshold {
    board_index: BoardIndex,
    max_threshold: u8,
    aftertouch_max: u8,
  },

  /// Set abs. distance from min value to trigger CA-004 submodule key events, ranging from 0x00 to 0xFE
  SetKeyMinimumThreshold {
    board_index: BoardIndex,
    threshold_high: u8,
    threshold_low: u8,
  },

  /// Set the bounds from the calibrated zero adc value of the pitch wheel, 0x00 to 0x7f
  SetPitchWheelZeroThreshold(u8),

  /// Set the sensitivity for CC events, ranging from 0x00 to 0xFE
  SetKeyFaderSensitivity(BoardIndex, u8),
  /// Set the target board sensitivity for aftertouch events, ranging from 0x00 to 0xFE
  SetKeyAftertouchSensitivity(BoardIndex, u8),
  /// Set the thresold from key’s min value to trigger CA - 004 submodule CC events, ranging from 0x00 to 0xFE
  SetCCActiveThreshold(BoardIndex, u8),
  /// Reset the thresholds for events and sensitivity for CC & aftertouch on the target board
  ResetBoardThresholds(BoardIndex),

  /// Set the 8-bit aftertouch trigger delay value,
  /// the time between a note on event and the initialization of aftertouch events
  /// TODO: see if units are documented / obvious in Lumatone Editor source. Assuming millisec...
  SetAftertouchTriggerDelay(BoardIndex, u8),
  /// Retrieve the aftertouch trigger delay of the given board
  GetAftertouchTriggerDelay(BoardIndex),
  /// Set the Lumatouch note-off delay value, an 11-bit integer representing the amount of 1.1ms ticks before
  /// sending a note-off event after a Lumatone-configured key is released.  
  SetLumatouchNoteOffDelay(BoardIndex, u16),
  /// Retrieve the note-off delay value of the given board
  GetLumatouchNoteOffDelay(BoardIndex),

  /// Read back the current red intensity of all the keys of the target board.
  GetRedLEDConfig(BoardIndex),
  /// Read back the current green intensity of all the keys of the target board.
  GetGreenLEDConfig(BoardIndex),
  /// Read back the current blue intensity of all the keys of the target board.
  GetBlueLEDConfig(BoardIndex),
  /// Read back the current channel configuration of all the keys of the target board.
  GetMidiChannelConfig(BoardIndex),
  /// Read back the current note configuration of all the keys of the target board.
  GetNoteConfig(BoardIndex),
  /// Read back the current key type configuration of all the keys of the target board.
  GetKeyTypeConfig(BoardIndex),
  /// Read back the maximum fader threshold of all the keys of the target board.
  GetMaxFaderThreshold(BoardIndex),
  /// Read back the minimum fader threshold of all the keys of the target board.
  GetMinFaderThreshold(BoardIndex),
  /// Read back the aftertouch maximum threshold of all the keys of the target board
  GetMaxAftertouchThreshold(BoardIndex),
  /// Get back flag whether or not each key of target board meets minimum threshold
  GetKeyValidity(BoardIndex),
  /// Read back the fader type of all keys on the targeted board.
  GetFaderTypeConfig(BoardIndex),
  /// Retrieve the threshold values of target board
  GetBoardThresholdValues(BoardIndex),
  /// Retrieve the sensitivity values of target board
  GetBoardSensitivityValues(BoardIndex),

  /// Read back the current velocity look up table of the keyboard.
  GetVelocityConfig,
  /// Read back the velocity interval table
  GetVelocityIntervalConfig,
  /// Read back the current fader look up table of the keyboard.
  GetFaderConfig,
  /// Read back the current aftertouch look up table of the keyboard.
  GetAftertouchConfig,
  /// Read back the Lumatouch table
  GetLumatouchConfig,

  /// Read back the serial identification number of the keyboard.
  GetSerialId,
  /// Read back the current Lumatone firmware revision.
  GetFirmwareRevision,

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
  /// Reset expression pedal minimum and maximum bounds to factory settings
  ResetExpressionPedalBounds,

  /// Enable/disable key sampling over SSH for the target key and board
  EnableKeySampling(BoardIndex, bool),

  /// Set the MIDI channels for peripheral controllers
  SetPeripheralChannels {
    pitch_wheel: MidiChannel,
    mod_wheel: MidiChannel,
    expression: MidiChannel,
    sustain: MidiChannel,
  },
  /// Retrieve the MIDI channels for peripheral controllers
  GetPeripheralChannels,

  /// Set expression pedal ADC threshold value, a 12-bit integer
  SetExpressionPedalADCThreshold(u16),

  /// Get the current expression pedal ADC threshold value
  GetExpressionPedalADCThreshold,
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
      InvertSustainPedal(_) => CommandId::InvertSustainPedal,

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

      GetRedLEDConfig(_) => CommandId::GetRedLedConfig,
      GetGreenLEDConfig(_) => CommandId::GetGreenLedConfig,
      GetBlueLEDConfig(_) => CommandId::GetBlueLedConfig,
      GetMidiChannelConfig(_) => CommandId::GetChannelConfig,
      GetNoteConfig(_) => CommandId::GetNoteConfig,
      GetKeyTypeConfig(_) => CommandId::GetKeytypeConfig,
      GetMaxFaderThreshold(_) => CommandId::GetMaxThreshold,
      GetMinFaderThreshold(_) => CommandId::GetMinThreshold,
      GetMaxAftertouchThreshold(_) => CommandId::GetAftertouchMax,
      GetKeyValidity(_) => CommandId::GetKeyValidity,
      GetFaderTypeConfig(_) => CommandId::GetFaderTypeConfiguration,

      GetVelocityConfig => CommandId::GetVelocityConfig,
      GetVelocityIntervalConfig => CommandId::GetVelocityIntervals,
      GetFaderConfig => CommandId::GetFaderConfig,
      GetAftertouchConfig => CommandId::GetAftertouchConfig,
      GetLumatouchConfig => CommandId::GetLumatouchConfig,

      GetSerialId => CommandId::GetSerialIdentity,
      GetFirmwareRevision => CommandId::GetFirmwareRevision,

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

      EnableExpressionPedalCalibrationMode(_) => CommandId::CalibrateExpressionPedal,
      SetPitchWheelZeroThreshold(_) => CommandId::SetPitchWheelCenterThreshold,
      GetBoardThresholdValues(_) => CommandId::GetBoardThresholdValues,
      GetBoardSensitivityValues(_) => CommandId::GetBoardSensitivityValues,
      ResetExpressionPedalBounds => CommandId::ResetExpressionPedalBounds,
      SetPeripheralChannels { .. } => CommandId::SetPeripheralChannels,
      GetPeripheralChannels => CommandId::GetPeripheralChannels,
      SetAftertouchTriggerDelay(..) => CommandId::SetAftertouchTriggerDelay,
      GetAftertouchTriggerDelay(_) => CommandId::GetAftertouchTriggerDelay,

      SetLumatouchNoteOffDelay(..) => CommandId::SetLumatouchNoteOffDelay,
      GetLumatouchNoteOffDelay(_) => CommandId::GetLumatouchNoteOffDelay,
      SetExpressionPedalADCThreshold(_) => CommandId::SetExpressionPedalThreshold,
      GetExpressionPedalADCThreshold => CommandId::GetExpressionPedalThreshold,
    }
  }

  pub fn to_sysex_message(&self) -> EncodedSysex {
    use Command::*;
    match self {
      Ping(value) => encode_ping(*value),

      SetKeyFunction { location, function } => encode_set_key_function(location, function),

      SetKeyColor { location, color } => encode_set_key_color(location, color),

      SaveProgram(preset_number) => {
        create_single_arg_server_sysex(self.command_id(), (*preset_number).into())
      }

      SetExpressionPedalSensitivity(value) => {
        create_single_arg_server_sysex(self.command_id(), *value)
      }

      SetModWheelSensitivity(value) => {
        create_single_arg_server_sysex(self.command_id(), (*value).clamp(1, 0x7f))
      }

      SetPitchWheelSensitivity(value) => {
        let val = (*value).clamp(1, 0x3fff);
        let hi = (val >> 7) as u8;
        let lo = (val & 0x7f) as u8;

        create_sysex(BoardIndex::Server, self.command_id(), vec![hi, lo])
      }

      InvertFootController(invert) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *invert)
      }

      InvertSustainPedal(invert) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *invert)
      }

      SetAftertouchEnabled(enabled) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled)
      }

      EnableDemoMode(enabled) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled)
      }

      EnablePitchModWheelCalibrationMode(enabled) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enabled)
      }

      SetMacroButtonActiveColor(color) => {
        create_extended_macro_color_sysex(self.command_id(), color)
      }

      SetMacroButtonInactiveColor(color) => {
        create_extended_macro_color_sysex(self.command_id(), color)
      }

      SetLightOnKeystrokes(active) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *active)
      }

      SetVelocityConfig(table) =>
      // the velocity config is in the reverse order (compared to how it's specified in keymap files)
      // so we reverse it before sending
      {
        create_table_sysex(self.command_id(), &reverse_table(table))
      }

      SetFaderConfig(table) => create_table_sysex(self.command_id(), table),

      SetAftertouchConfig(table) => create_table_sysex(self.command_id(), table),

      SetLumatouchConfig(table) => create_table_sysex(self.command_id(), table),

      SetVelocityIntervals(table) => encode_set_velocity_interval_table(table),

      SetKeyMaximumThreshold {
        board_index,
        max_threshold,
        aftertouch_max,
      } => encode_set_key_thresholds(
        *board_index,
        self.command_id(),
        *max_threshold,
        *aftertouch_max,
      ),

      SetKeyMinimumThreshold {
        board_index,
        threshold_high,
        threshold_low,
      } => encode_set_key_thresholds(
        *board_index,
        self.command_id(),
        *threshold_high,
        *threshold_low,
      ),

      SetKeyFaderSensitivity(board_index, value) => {
        encode_set_key_sensitivity(*board_index, self.command_id(), *value)
      }

      SetKeyAftertouchSensitivity(board_index, value) => {
        encode_set_key_sensitivity(*board_index, self.command_id(), *value)
      }

      SetCCActiveThreshold(board_index, value) => {
        encode_set_key_sensitivity(*board_index, self.command_id(), *value)
      }

      ResetBoardThresholds(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),

      GetRedLEDConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),

      GetGreenLEDConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),

      GetBlueLEDConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),

      GetMidiChannelConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetNoteConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetKeyTypeConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetMaxFaderThreshold(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetMinFaderThreshold(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetMaxAftertouchThreshold(board_index) => {
        create_zero_arg_sysex(*board_index, self.command_id())
      }
      GetKeyValidity(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),
      GetFaderTypeConfig(board_index) => create_zero_arg_sysex(*board_index, self.command_id()),

      GetVelocityConfig => create_zero_arg_server_sysex(self.command_id()),
      GetVelocityIntervalConfig => create_zero_arg_server_sysex(self.command_id()),
      GetFaderConfig => create_zero_arg_server_sysex(self.command_id()),
      GetAftertouchConfig => create_zero_arg_server_sysex(self.command_id()),
      GetLumatouchConfig => create_zero_arg_server_sysex(self.command_id()),
      GetSerialId => create_zero_arg_server_sysex(self.command_id()),
      GetFirmwareRevision => create_zero_arg_server_sysex(self.command_id()),
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
      ResetExpressionPedalBounds => create_zero_arg_server_sysex(self.command_id()),

      EnableKeySampling(board_index, enable) => {
        create_sysex_toggle(*board_index, self.command_id(), *enable)
      }

      EnableExpressionPedalCalibrationMode(enable) => {
        create_sysex_toggle(BoardIndex::Server, self.command_id(), *enable)
      }

      SetPitchWheelZeroThreshold(value) => {
        create_single_arg_server_sysex(self.command_id(), value & 0x7f)
      }

      GetBoardThresholdValues(board_index) => {
        create_zero_arg_sysex(*board_index, self.command_id())
      }

      GetBoardSensitivityValues(board_index) => {
        create_zero_arg_sysex(*board_index, self.command_id())
      }

      SetPeripheralChannels {
        pitch_wheel,
        mod_wheel,
        expression,
        sustain,
      } => create_sysex(
        BoardIndex::Server,
        self.command_id(),
        vec![
          pitch_wheel.get_as_zero_indexed(),
          mod_wheel.get_as_zero_indexed(),
          expression.get_as_zero_indexed(),
          sustain.get_as_zero_indexed(),
        ],
      ),

      GetPeripheralChannels => create_zero_arg_server_sysex(self.command_id()),

      SetAftertouchTriggerDelay(board_index, value) => create_sysex(
        *board_index,
        self.command_id(),
        vec![value >> 4, value & 0xf],
      ),

      GetAftertouchTriggerDelay(board_index) => {
        create_zero_arg_sysex(*board_index, self.command_id())
      }

      SetLumatouchNoteOffDelay(board_index, value) => create_sysex(
        *board_index,
        self.command_id(),
        vec![
          ((value >> 8) & 0xf) as u8,
          ((value >> 4) & 0xf) as u8,
          (value & 0xf) as u8,
        ],
      ),

      GetLumatouchNoteOffDelay(board_index) => {
        create_zero_arg_sysex(*board_index, self.command_id())
      }

      SetExpressionPedalADCThreshold(value) => create_sysex(
        BoardIndex::Server,
        self.command_id(),
        vec![
          ((value >> 8) & 0xf) as u8,
          ((value >> 4) & 0xf) as u8,
          (value & 0xf) as u8,
        ],
      ),

      GetExpressionPedalADCThreshold => create_zero_arg_server_sysex(self.command_id()),
    }
  }
}

impl std::fmt::Display for Command {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Command::Ping(val) => write!(f, "Ping({val})"),
      Command::SetKeyFunction { location, function } => {
        write!(f, "SetKeyFunction({location}, {function})")
      }
      Command::SetKeyColor { location, color } => write!(f, "SetKeyColor({location}, {color})"),
      Command::SaveProgram(val) => write!(f, "SaveProgram({val})"),
      Command::SetExpressionPedalSensitivity(val) => {
        write!(f, "SetExpressionPedalSensitivity({val})")
      }
      Command::SetModWheelSensitivity(val) => write!(f, "SetModWheelSensitivity({val})"),
      Command::SetPitchWheelSensitivity(val) => write!(f, "SetPitchWheelSensitivity({val})"),
      Command::InvertFootController(val) => write!(f, "InvertFootController({val})"),
      Command::InvertSustainPedal(val) => write!(f, "InvertSustainPedal({val})"),
      Command::SetLightOnKeystrokes(val) => write!(f, "SetLightOnKeystrokes({val})"),
      Command::SetAftertouchEnabled(val) => write!(f, "SetAftertouchEnabled({val})"),
      Command::EnableDemoMode(val) => write!(f, "EnableDemoMode({val})"),
      Command::EnablePitchModWheelCalibrationMode(val) => {
        write!(f, "EnablePitchModWheelCalibrationMode({val})")
      }
      Command::EnableExpressionPedalCalibrationMode(val) => {
        write!(f, "EnableExpressionPedalCalibrationMode({val})")
      }
      Command::SetMacroButtonActiveColor(val) => write!(f, "SetMacroButtonActiveColor({val})"),
      Command::SetMacroButtonInactiveColor(val) => write!(f, "SetMacroButtonInactiveColor({val})"),
      Command::SetVelocityConfig(_) => write!(f, "SetVelocityConfig(<table...>)"),
      Command::SetFaderConfig(_) => write!(f, "SetFaderConfig(<table...>)"),
      Command::SetAftertouchConfig(_) => write!(f, "SetAftertouchConfig(<table...>"),
      Command::SetLumatouchConfig(_) => write!(f, "SetLumatouchConfig(<table...>)"),
      Command::SetVelocityIntervals(_) => write!(f, "SetVelocityIntervals(<table...>"),
      Command::SetKeyMaximumThreshold {
        board_index,
        max_threshold,
        aftertouch_max,
      } => write!(
        f,
        "SetKeyMaximumThreshold {{ board_index: {}, max_threshold: {}, aftertouch_max: {} }}",
        board_index, max_threshold, aftertouch_max
      ),
      Command::SetKeyMinimumThreshold {
        board_index,
        threshold_high,
        threshold_low,
      } => write!(
        f,
        "SetKeyMinimumThreshold {{ board_index: {}, threshold_high: {}, threshold_low: {} }}",
        board_index, threshold_high, threshold_low
      ),
      Command::SetPitchWheelZeroThreshold(val) => write!(f, "SetPitchWheelZeroThreshold({val})"),
      Command::SetKeyFaderSensitivity(board, val) => {
        write!(f, "SetKeyFaderSensitivity({board}, {val})")
      }
      Command::SetKeyAftertouchSensitivity(board, val) => {
        write!(f, "SetKeyAftertouchSensitivity({board}, {val})")
      }
      Command::SetCCActiveThreshold(board, val) => {
        write!(f, "SetCCActiveThreshold({board}, {val})")
      }
      Command::ResetBoardThresholds(board) => write!(f, "ResetBoardThresholds({board})"),
      Command::SetAftertouchTriggerDelay(board, val) => {
        write!(f, "SetAftertouchTriggerDelay({board}, {val})")
      }
      Command::GetAftertouchTriggerDelay(board) => write!(f, "GetAftertouchTriggerDelay({board})"),
      Command::SetLumatouchNoteOffDelay(board, val) => {
        write!(f, "SetLumatouchNoteOffDelay({board}, {val})")
      }
      Command::GetLumatouchNoteOffDelay(board) => write!(f, "GetLumatouchNoteOffDelay({board})"),
      Command::GetRedLEDConfig(board) => write!(f, "GetRedLEDConfig({board})"),
      Command::GetGreenLEDConfig(board) => write!(f, "GetGreenLEDConfig({board})"),
      Command::GetBlueLEDConfig(board) => write!(f, "GetBlueLEDConfig({board})"),
      Command::GetMidiChannelConfig(board) => write!(f, "GetMidiChannelConfig({board})"),
      Command::GetNoteConfig(board) => write!(f, "GetNoteConfig({board})"),
      Command::GetKeyTypeConfig(board) => write!(f, "GetKeyTypeConfig({board})"),
      Command::GetMaxFaderThreshold(board) => write!(f, "GetMaxFaderThreshold({board})"),
      Command::GetMinFaderThreshold(board) => write!(f, "GetMinFaderThreshold({board})"),
      Command::GetMaxAftertouchThreshold(board) => write!(f, "GetMaxAftertouchThreshold({board})"),
      Command::GetKeyValidity(board) => write!(f, "GetKeyValidity({board})"),
      Command::GetFaderTypeConfig(board) => write!(f, "GetFaderTypeConfig({board})"),
      Command::GetBoardThresholdValues(board) => write!(f, "GetBoardThresholdValues({board})"),
      Command::GetBoardSensitivityValues(board) => write!(f, "GetBoardSensitivityValues({board})"),
      Command::GetVelocityConfig => write!(f, "GetVelocityConfig"),
      Command::GetVelocityIntervalConfig => write!(f, "GetVelocityIntervalConfig"),
      Command::GetFaderConfig => write!(f, "GetFaderConfig"),
      Command::GetAftertouchConfig => write!(f, "GetAftertouchConfig"),
      Command::GetLumatouchConfig => write!(f, "GetLumatouchConfig"),
      Command::GetSerialId => write!(f, "GetSerialId"),
      Command::GetFirmwareRevision => write!(f, "GetFirmwareRevision"),
      Command::StartAftertouchCalibration => write!(f, "StartAftertouchCalibration"),
      Command::StartKeyCalibration => write!(f, "StartKeyCalibration"),
      Command::SaveVelocityConfig => write!(f, "SaveVelocityConfig"),
      Command::ResetVelocityConfig => write!(f, "ResetVelocityConfig"),
      Command::SaveFaderConfig => write!(f, "SaveFaderConfig"),
      Command::ResetFaderConfig => write!(f, "ResetFaderConfig"),
      Command::SaveAftertouchConfig => write!(f, "SaveAftertouchConfig"),
      Command::ResetAftertouchConfig => write!(f, "ResetAftertouchConfig"),
      Command::SaveLumatouchConfig => write!(f, "SaveLumatouchConfig"),
      Command::ResetLumatouchConfig => write!(f, "ResetLumatouchConfig"),
      Command::ResetWheelThresholds => write!(f, "ResetWheelThresholds"),
      Command::ResetExpressionPedalBounds => write!(f, "ResetExpressionPedalBounds"),
      Command::EnableKeySampling(board, val) => write!(f, "EnableKeySampling({board}, {val})"),
      Command::SetPeripheralChannels {
        pitch_wheel,
        mod_wheel,
        expression,
        sustain,
      } => write!(
        f,
        "SetPeripheralChannels {{ pitch_wheel: {}, mod_wheel: {}, expression: {}, sustain: {} }}",
        pitch_wheel, mod_wheel, expression, sustain
      ),
      Command::GetPeripheralChannels => write!(f, "GetPeripheralChannels"),
      Command::SetExpressionPedalADCThreshold(val) => {
        write!(f, "SetExpressionPedalADCThreshold({val})")
      }
      Command::GetExpressionPedalADCThreshold => write!(f, "GetExpressionPedalADCThreshold"),
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

fn encode_set_key_thresholds(
  board_index: BoardIndex,
  cmd: CommandId,
  t1: u8,
  t2: u8,
) -> EncodedSysex {
  let t1 = t1 & 0xfe;
  let t2 = t2 & 0xfe;
  let data = vec![t1 >> 4, t1 & 0xf, t2 >> 4, t2 & 0xf];
  create_sysex(board_index, cmd, data)
}

fn encode_set_key_sensitivity(board_index: BoardIndex, cmd: CommandId, value: u8) -> EncodedSysex {
  let value = value & 0xfe;
  let data = vec![value >> 4, value & 0xf];
  create_sysex(board_index, cmd, data)
}

// endregion
