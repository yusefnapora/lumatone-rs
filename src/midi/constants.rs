#![allow(dead_code)]

use bounded_integer::bounded_integer;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub const MANUFACTURER_ID: [u8; 3] = [0x00, 0x21, 0x50];

pub const ECHO_FLAG: u8 = 0x5; // used to differentiate test responses from MIDI
pub const TEST_ECHO: u8 = 0x7f; // should not be returned by lumatone

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl RGBColor {
  pub fn red() -> RGBColor {
    RGBColor(0xff, 0, 0)
  }

  pub fn green() -> RGBColor {
    RGBColor(0, 0xff, 0)
  }

  pub fn blue() -> RGBColor {
    RGBColor(0, 0, 0xff)
  }

  pub fn to_hex_string(&self) -> String {
    let RGBColor(r, g, b) = self;
    format!("{r:02x}{g:02x}{b:02x}")
  }

  /// Returns the color encoded into 6 u8's with the lower 4 bits set.
  pub fn to_bytes(&self) -> Vec<u8> {
    let RGBColor(red, green, blue) = *self;
    let red_hi = red >> 4;
    let red_lo = red & 0xf;
    let green_hi = green >> 4;
    let green_lo = green & 0xf;
    let blue_hi = blue >> 4;
    let blue_lo = blue & 0xf;
    vec![red_hi, red_lo, green_hi, green_lo, blue_hi, blue_lo]
  }
}

impl From<u32> for RGBColor {
  /// Conversion from u32 ignores the "leftmost" byte.
  /// e.g. use 0x00ffffff for white.
  fn from(val: u32) -> Self {
    let red = ((val >> 16) & 0xff) as u8;
    let green = ((val >> 8) & 0xff) as u8;
    let blue = (val & 0xff) as u8;
    RGBColor(red, green, blue)
  }
}

bounded_integer! {
  /// A zero-indexed MIDI channel number, in the range 1 ..= 16.
  ///
  /// Use `MidiChannel::default()` for channel 1.
  ///
  /// When converting from untrusted / arbitrary input, use `MidiChannel::new`, which returns an `Option`.
  /// If you're sure your value is in range, use `MidiChannel::unchecked`, which will panic if the input is
  /// out of bounds.
  pub struct MidiChannel { 1..=16 }
}

bounded_integer! {
  /// A zero-indexed Lumatone key index, in the range 0 ..= 55.
  ///
  /// Covers a single "board"; combine with [`BoardIndex`] to address a physical key.
  pub struct LumatoneKeyIndex { 0..=55 }
}

bounded_integer! {
  /// A zero-indexed Lumatone preset number (identifies the macro / preset keys above the keyboard)
  pub struct PresetNumber { 0 ..= 9 }
}

impl MidiChannel {
  pub fn unchecked(val: u8) -> Self {
    Self::new(val).expect(format!("invalid midi channel number: {val}").as_str())
  }
}

impl Default for MidiChannel {
  fn default() -> Self {
    MidiChannel::MIN
  }
}

impl LumatoneKeyIndex {
  pub fn unchecked(val: u8) -> Self {
    Self::new(val).expect(format!("invalid lumatone key index: {val}").as_str())
  }
}

impl PresetNumber {
  pub fn uncheked(val: u8) -> Self {
    Self::new(val).expect(format!("invalid preset numer: {val}").as_str())
  }
}

/// Identifies which "board" a message should be routed to.
///
/// Commands that set key parameters should be targetted at one of the Octave values,
/// which control the five 56-key Terpstra boards that comprise the full Lumatone layout.
///
/// Global operations (ping, macro keys, etc) should be sent to the Server board.
#[derive(Debug, FromPrimitive, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BoardIndex {
  Server = 0,
  Octave1,
  Octave2,
  Octave3,
  Octave4,
  Octave5,
}

impl Into<u8> for BoardIndex {
  fn into(self) -> u8 {
    self as u8
  }
}

/// Uniquely identifies one of the keys on the Lumatone keyboard.
///
/// To convert from another coordinate system, add an `impl Into<LumatoneKeyLocation>` to your coordinate type.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LumatoneKeyLocation(pub BoardIndex, pub LumatoneKeyIndex);

impl LumatoneKeyLocation {
  pub fn board_index(&self) -> BoardIndex {
    self.0
  }

  pub fn key_index(&self) -> LumatoneKeyIndex {
    self.1
  }
}

impl Into<LumatoneKeyLocation> for (BoardIndex, LumatoneKeyIndex) {
  fn into(self) -> LumatoneKeyLocation {
    LumatoneKeyLocation(self.0, self.1)
  }
}

/// Returns a (BoardIndex, LumatoneKeyIndex) tuple that identifies a Lumatone key.
/// Will panic if input is out of range - use only on static / trusted input.
pub fn key_loc_unchecked(board_index: u8, key_index: u8) -> LumatoneKeyLocation {
  let board_index = FromPrimitive::from_u8(board_index)
    .expect(format!("invalid board index: {board_index}").as_str());
  let key_index = LumatoneKeyIndex::unchecked(key_index);
  LumatoneKeyLocation(board_index, key_index)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LumatoneKeyFunction {
  /// Key sends note on/off messages
  NoteOnOff {
    channel: MidiChannel,
    note_num: u8,
  },
  ContinuousController {
    channel: MidiChannel,
    cc_num: u8,
    fader_up_is_null: bool,
  },
  LumaTouch {
    channel: MidiChannel,
    note_num: u8,
    fader_up_is_null: bool,
  },
  Disabled,
}

impl LumatoneKeyFunction {
  pub fn type_code(&self) -> u8 {
    use LumatoneKeyFunction::*;
    match *self {
      NoteOnOff { .. } => 1,
      ContinuousController {
        fader_up_is_null: false,
        ..
      } => 2,
      ContinuousController {
        fader_up_is_null: true,
        ..
      } => (1 << 4) | 2,
      LumaTouch {
        fader_up_is_null: false,
        ..
      } => 3,
      LumaTouch {
        fader_up_is_null: true,
        ..
      } => (1 << 4) | 3,
      Disabled => 4,
    }
  }

  pub fn key_type_code(&self) -> u8 {
    use LumatoneKeyFunction::*;
    match *self {
      NoteOnOff { .. } => 1,
      ContinuousController { .. } => 2,
      LumaTouch { .. } => 3,
      Disabled { .. } => 4,
    }
  }

  pub fn note_or_cc_num(&self) -> u8 {
    use LumatoneKeyFunction::*;
    match *self {
      NoteOnOff { note_num, .. } => note_num,
      ContinuousController { cc_num, .. } => cc_num,
      LumaTouch { note_num, .. } => note_num,
      Disabled => 0,
    }
  }

  pub fn midi_channel_byte(&self) -> u8 {
    use LumatoneKeyFunction::*;
    match *self {
      NoteOnOff { channel, .. } => channel.into(),
      ContinuousController { channel, .. } => channel.into(),
      LumaTouch { channel, .. } => channel.into(),
      Disabled => 0,
    }
  }
}

/// A status code included in response messages sent by the Lumatone device.
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum ResponseStatusCode {
  /// NACK - Command not recognized
  Nack = 0x0,
  /// ACK - Command successful
  Ack = 0x01,
  /// BUSY - Device busy, try again later
  Busy = 0x02,
  /// ERROR - Command failed
  Error = 0x03,
  /// STATE - Device is not in MIDI mode. Usually indicates device is in demo mode.
  State = 0x04,

  /// Unknown - Not returned by Lumatone device - indicates that the device sent a code we don't understand
  Unknown = 0xff,
}

impl Into<u8> for ResponseStatusCode {
  fn into(self) -> u8 {
    self as u8
  }
}

/// Identifies a Lumatone command.
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum CommandId {
  // Start support at 55-keys firmware version, Developmental versions
  ChangeKeyNote = 0x00,
  SetKeyColour = 0x01,

  SaveProgram = 0x02,

  SetFootControllerSensitivity = 0x03,
  InvertFootController = 0x04,

  MacrobuttonColourOn = 0x05,
  MacrobuttonColourOff = 0x06,

  SetLightOnKeystrokes = 0x07,

  SetVelocityConfig = 0x08,
  SaveVelocityConfig = 0x09,
  ResetVelocityConfig = 0x0a,

  SetFaderConfig = 0x0b,
  SaveFaderConfig = 0x0c,
  ResetFaderConfig = 0x0d,

  SetAftertouchFlag = 0x0e,
  CalibrateAftertouch = 0x0f,
  SetAftertouchConfig = 0x10,
  SaveAftertouchConfig = 0x11,
  ResetAftertouchConfig = 0x12,

  GetRedLedConfig = 0x13,
  GetGreenLedConfig = 0x14,
  GetBlueLedConfig = 0x15,
  GetChannelConfig = 0x16,
  GetNoteConfig = 0x17,
  GetKeytypeConfig = 0x18,

  GetMaxThreshold = 0x19,
  GetMinThreshold = 0x1a,
  GetAftertouchMax = 0x1b,
  GetKeyValidity = 0x1c,

  GetVelocityConfig = 0x1d,
  GetFaderConfig = 0x1e,
  GetAftertouchConfig = 0x1f,

  // Firmware Version 1.0.3
  SetVelocityIntervals = 0x20,
  GetVelocityIntervals = 0x21,

  // Firmware Version 1.0.4
  GetFaderTypeConfiguration = 0x22,

  // Start 56-keys, Firmware Version 1.0.5
  GetSerialIdentity = 0x23,
  // 0x23 will acknowledge in 55-keys but will not return serial number
  CalibrateKeys = 0x24,

  DemoMode = 0x25,

  // Firmware Version 1.0.6
  CalibratePitchModWheel = 0x26,
  SetModWheelSensitivity = 0x27,
  SetPitchWheelSensitivity = 0x28,

  // Firmware Version 1.0.7, Start shipping backers and batches
  SetKeyMaxThreshold = 0x29,
  SetKeyMinThreshold = 0x2a,
  SetKeyFaderSensitivity = 0x2b,
  SetKeyAftertouchSensitivity = 0x2c,

  SetLumatouchConfig = 0x2d,
  SaveLumatouchConfig = 0x2e,
  ResetLumatouchConfig = 0x2f,
  GetLumatouchConfig = 0x30,

  // Firmware Version 1.0.8
  GetFirmwareRevision = 0x31,

  // Firmware Version 1.0.9
  SetCCActiveThreshold = 0x32,
  LumaPing = 0x33,

  // Firmware Version 1.0.10
  ResetBoardThresholds = 0x34,
  SetKeySampling = 0x35,

  // Firmware Version 1.0.11
  ResetWheelsThreshold = 0x36,
  SetPitchWheelCenterThreshold = 0x37,
  CallibrateExpressionPedal = 0x38,
  ResetExpressionPedalBounds = 0x39,

  // Firmware Version 1.0.12
  GetBoardThresholdValues = 0x3a,
  GetBoardSensitivityValues = 0x3b,

  // Firmware Version 1.0.13
  SetPeripheralChannels = 0x3c,
  GetPeripheralChannels = 0x3d,
  PeripheralCalbrationData = 0x3e,

  // Firmware Version 1.0.14
  SetAftertouchTriggerDelay = 0x3f,
  GetAftertouchTriggerDelay = 0x40,

  // Firmware Version 1.0.15
  SetLumatouchNoteOffDelay = 0x41,
  GetLumatouchNoteOffDelay = 0x42,
  SetExpressionPedalThreshold = 0x43,
  GetExpressionPedalThreshold = 0x44,
  InvertSustainPedal = 0x45,
}

impl Into<u8> for CommandId {
  fn into(self) -> u8 {
    self as u8
  }
}

#[cfg(test)]
mod tests {
  use super::RGBColor;

  #[test]
  fn test_rgb_color() {
    assert_eq!(RGBColor::from(0x00aabbcc), RGBColor(0xaa, 0xbb, 0xcc));
  }
}
