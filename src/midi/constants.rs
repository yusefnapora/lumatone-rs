#![allow(dead_code)]

use num_derive::FromPrimitive;

pub const MANUFACTURER_ID: [u8; 3] = [0x00, 0x21, 0x50];

pub const ECHO_FLAG: u8 = 0x5; // used to differentiate test responses from MIDI
pub const TEST_ECHO: u8 = 0x7f; // should not be returned by lumatone

#[derive(FromPrimitive, PartialEq)]
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

#[derive(FromPrimitive, PartialEq)]
pub enum LumatoneKeyType {
    NoteOnOff = 1,
    ContinuousController = 2,
    LumaTouch = 3,
    Disabled = 4,
}

impl Into<u8> for LumatoneKeyType {
  fn into(self) -> u8 {
    self as u8
  }
}

#[derive(FromPrimitive, PartialEq)]
pub enum FirmwareAnswerCode {
    Nack = 0x0,   // Not recognized
    Ack = 0x01,   // Acknowledged, OK
    Busy = 0x02,  // Controller busy
    Error = 0x03, // Error
    State = 0x04, // Not in MIDI state (demo mode, still booting, etc)
}

impl Into<u8> for FirmwareAnswerCode {
  fn into(self) -> u8 {
    self as u8
  }
}

#[derive(FromPrimitive, PartialEq)]
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
    SetCcActiveThreshold = 0x32,
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